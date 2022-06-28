use std::{
    any::Any,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use anyhow::Result;
use dashmap::DashMap;
use flume::{unbounded, Receiver, Sender};
use futures::{stream::FuturesUnordered, Future, FutureExt, StreamExt};
use tokio::task::{JoinError, JoinHandle};
use tracing::{debug, error};

use super::context::{ActorId, ArgTaskFn, Context, TaskFn};

#[derive(Debug, Clone)]
pub enum ExitReason {
    Normal,
    Error(String),
}

/// An Actor system is where Actors are spawned and live
/// This is the root state for a collection of Actors
pub struct System {
    shared: Arc<SystemShared>,
    joiner: JoinHandle<()>,
}

impl System {
    /// Create a new Actor system
    pub fn new() -> System {
        let (sender, receiver) = unbounded();

        let shared = Arc::new(SystemShared {
            // We initialize this to 1 because 0 is reserved for System
            total: AtomicUsize::new(1),
            current: AtomicUsize::new(0),
            actors: DashMap::new(),
            named: DashMap::new(),
            register: sender,
        });

        System {
            joiner: tokio::spawn(System::handle_exits(receiver, shared.clone())),
            shared,
        }
    }

    async fn handle_exits(receiver: Receiver<ActorHandle>, shared: Arc<SystemShared>) {
        let mut futures = FuturesUnordered::new();

        'main: loop {
            tokio::select! {
                Ok(handle) = receiver.recv_async() => {
                    debug!("Actor<0x{:x}> start", handle.id);
                    futures.push(handle);
                },
                result = futures.next() => {
                    match result {
                        Some((id, res)) => {
                            // Notify sibling that they need to die
                            if let Some((_, mut state)) = shared.actors.remove(&id) {
                                let siblings = state
                                    .links
                                    .into_iter()
                                    .filter_map(|id| shared.actors.get_mut(&id));

                                let reason = res
                                    .map_err(|e| ExitReason::Error(format!("{e:?}")))
                                    .err()
                                    .unwrap_or(ExitReason::Normal);

                                for mut sibling in siblings {
                                    // Remove back references
                                    sibling.links.retain(|&n| n != id);

                                    if let Some(exit) = sibling.exit.as_ref() {
                                        let _ = exit.send((id, reason.clone()));
                                    }
                                }

                                // If the Actor happened to be named, remove it from the registry
                                if let Some(named) = state.named.take() {
                                    shared.named.remove(&named);
                                }

                                debug!("Actor<0x{id:x}> exit");
                            }

                            shared.current.fetch_sub(1, Ordering::SeqCst);
                        },
                        None if shared.current.load(Ordering::SeqCst) < 1 => break 'main,
                        None => {},
                    }
                }
            }
        }
    }

    /// Spawn a new Actor with an argument and receive it's [`Pid`]
    pub fn spawn_with_argument<S, A, M, F>(&self, func: ArgTaskFn<S, A, F>, args: A) -> Pid<M>
    where
        S: Context<M>,
        M: Send + 'static,
        F: Future<Output = Result<()>> + Send + 'static,
    {
        self.shared.spawn_with_argument(None, func, args)
    }

    /// Spawn a new Actor and receive it's [`Pid`]
    pub fn spawn<S, M, F>(&self, func: TaskFn<S, F>) -> Pid<M>
    where
        S: Context<M>,
        M: Send + 'static,
        F: Future<Output = Result<()>> + Send + 'static,
    {
        self.shared.spawn(None, func)
    }

    /// Spawn a new Actor and receive it's [`Pid`]
    /// Linking the Actor to [`System`] is a little bit special since it does not cause the current thread to quit like linking two Actors together.
    /// When an Actor linked to the system exits it causes ALL actors to exit and [`System::join`] to complete.
    pub fn spawn_linked<S, M, F>(&self, func: TaskFn<S, F>) -> Pid<M>
    where
        S: Context<M>,
        M: Send + 'static,
        F: Future<Output = Result<()>> + Send + 'static,
    {
        self.shared.spawn(Some(0), func)
    }

    /// Want for all Actors to complete
    pub async fn join(self) -> Result<(), JoinError> {
        self.joiner.await
    }
}

pub struct ActorHandle {
    id: ActorId,
    handle: JoinHandle<Result<()>>,
}

impl Future for ActorHandle {
    type Output = (ActorId, Result<()>);

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let id = self.id;

        self.handle.poll_unpin(cx).map(|res| {
            let out = res.map_err(|join| join.into()).and_then(|inner| inner);
            (id, out)
        })
    }
}

#[derive(Default)]
pub struct ActorState {
    links: Vec<ActorId>,
    named: Option<String>,
    exit: Option<Sender<(ActorId, ExitReason)>>,
}

pub struct SystemShared {
    total: AtomicUsize,
    current: AtomicUsize,
    actors: DashMap<ActorId, ActorState>,
    named: DashMap<String, Box<dyn Any + Send + Sync + 'static>>,
    register: Sender<ActorHandle>,
}

impl SystemShared {
    pub fn register<S, M>(&self, name: S, pid: Pid<M>)
    where
        S: Into<String>,
        M: Send + 'static,
    {
        if let Some(mut actor) = self.actors.get_mut(&pid.id) {
            let name = name.into();
            actor.named = Some(name.clone());

            drop(actor);

            self.named.insert(name, Box::new(pid));
        }
    }

    pub fn lookup<M>(&self, name: &str) -> Option<Pid<M>>
    where
        M: Send + 'static,
    {
        let r = self.named.get(name)?;
        let pid: &Pid<M> = r.downcast_ref()?;

        Some(pid.clone())
    }

    pub fn spawn_with_argument<M, C, A, F>(
        self: &Arc<Self>,
        from: Option<ActorId>,
        func: ArgTaskFn<C, A, F>,
        args: A,
    ) -> Pid<M>
    where
        C: Context<M>,
        F: Future<Output = Result<()>> + Send + 'static,
    {
        self.current.fetch_add(1, Ordering::SeqCst);
        let id = self.total.fetch_add(1, Ordering::SeqCst);

        let (exit_sender, exit_receiver) = unbounded();
        let (sender, receiver) = unbounded::<M>();

        let mut state = self.actors.entry(id).or_default();
        state.exit = Some(exit_sender);

        if let Some(from) = from {
            state.links.reserve(1);
            state.links.push(from);
            state.links.dedup();

            drop(state);

            // Don't add state for System links, there will never be a "System" Actor
            if from != 0 {
                let mut state = self.actors.entry(from).or_default();
                state.links.reserve(1);
                state.links.push(id);
                state.links.dedup();

                drop(state);
            }
        }

        let handle = C::start_with_argument(
            id,
            self.clone(),
            receiver,
            sender.clone(),
            exit_receiver,
            args,
            func,
        );

        self.register.send(ActorHandle { id, handle }).unwrap();

        Pid { id, sender }
    }

    pub fn spawn<M, C, F>(self: &Arc<Self>, from: Option<ActorId>, func: TaskFn<C, F>) -> Pid<M>
    where
        C: Context<M>,
        F: Future<Output = Result<()>> + Send + 'static,
    {
        self.current.fetch_add(1, Ordering::SeqCst);
        let id = self.total.fetch_add(1, Ordering::SeqCst);

        let (exit_sender, exit_receiver) = unbounded();
        let (sender, receiver) = unbounded::<M>();

        let mut state = self.actors.entry(id).or_default();
        state.exit = Some(exit_sender);

        if let Some(from) = from {
            state.links.reserve(1);
            state.links.push(from);
            state.links.dedup();

            drop(state);

            // Don't add state for System links, there will never be a "System" Actor
            if from != 0 {
                let mut state = self.actors.entry(from).or_default();
                state.links.reserve(1);
                state.links.push(id);
                state.links.dedup();

                drop(state);
            }
        }

        let handle = C::start(
            id,
            self.clone(),
            receiver,
            sender.clone(),
            exit_receiver,
            func,
        );

        self.register.send(ActorHandle { id, handle }).unwrap();

        Pid { id, sender }
    }
}

pub struct Pid<M> {
    id: ActorId,
    sender: Sender<M>,
}

impl<M> Clone for Pid<M> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            sender: self.sender.clone(),
        }
    }
}

impl<M> std::fmt::Debug for Pid<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Pid<0x{:x}>", self.id))
    }
}

impl<M> Pid<M> {
    pub fn new(id: ActorId, sender: Sender<M>) -> Pid<M> {
        Pid { id, sender }
    }

    pub fn send(&self, message: M) {
        if let Err(_) = self.sender.send(message) {
            error!("{:?} failed to send ", self);
        }
    }

    pub fn actor_id(&self) -> ActorId {
        self.id
    }
}
