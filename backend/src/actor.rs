use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use anyhow::Result;
use dashmap::DashMap;
use flume::{unbounded, Receiver, Sender};
use futures::{stream::FuturesUnordered, Future, FutureExt, StreamExt};
use tokio::task::{JoinError, JoinHandle};
use tracing::error;

type TaskFn<S, A, F> = fn(S, A) -> F;

pub struct System {
    shared: Arc<SystemShared>,
    joiner: JoinHandle<()>,
}

pub struct SystemShared {
    total: AtomicUsize,
    current: AtomicUsize,
    actors: DashMap<usize, ActorState>,
    register: Sender<ActorHandle>,
}

#[derive(Default)]
struct ActorState {
    links: Vec<usize>,
    exit: Option<Sender<(usize, ExitReason)>>,
}

struct ActorHandle {
    id: usize,
    handle: JoinHandle<Result<()>>,
}

impl Future for ActorHandle {
    type Output = (usize, Result<()>);

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

impl SystemShared {
    pub fn spawn<M, S, A, F>(
        self: &Arc<Self>,
        from: Option<usize>,
        func: TaskFn<S, A, F>,
        args: A,
    ) -> Pid<M>
    where
        S: Context<M>,
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

            let mut state = self.actors.entry(from).or_default();
            state.links.reserve(1);
            state.links.push(id);
            state.links.dedup();

            drop(state);
        }

        let handle = S::start(
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
}

impl System {
    pub fn new() -> System {
        let (sender, receiver) = unbounded();

        let shared = Arc::new(SystemShared {
            total: AtomicUsize::new(1),
            current: AtomicUsize::new(0),
            actors: DashMap::new(),
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
                    futures.push(handle);
                },
                result = futures.next() => {
                    match result {
                        Some((id, _)) => {
                            // Notify sibling that they need to die
                            if let Some((_, state)) = shared.actors.remove(&id) {
                                let siblings = state
                                    .links
                                    .into_iter()
                                    .filter_map(|id| shared.actors.get_mut(&id));
                    
                                for mut sibling in siblings {
                                    // Remove back references
                                    sibling.links.retain(|&n| n != id);
                    
                                    if let Some(exit) = sibling.exit.as_ref() {
                                        let _ = exit.send((id, ExitReason::Normal));
                                    }
                                }
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

    pub fn spawn<S, M, A, F>(&self, func: TaskFn<S, A, F>, args: A) -> Pid<M>
    where
        S: Context<M>,
        M: Send + 'static,
        F: Future<Output = Result<()>> + Send + 'static,
    {
        self.shared.spawn(None, func, args)
    }

    pub async fn join(self) -> Result<(), JoinError> {
        self.joiner.await
    }
}

pub struct Pid<M> {
    id: usize,
    sender: Sender<M>,
}

impl<M> std::fmt::Debug for Pid<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Pid<0x{:x}>", self.id))
    }
}

impl<M: Send + 'static> Pid<M> {
    pub fn send(&self, message: M) {
        if let Err(_) = self.sender.send(message) {
            error!("{:?} failed to send ", self);
        }
    }

    pub fn has_id(&self, other: usize) -> bool {
        self.id == other
    }
}

#[derive(Debug)]
pub enum ExitReason {
    Normal,
    Error,
}

pub trait Context<M> {
    fn start<F, A>(
        id: usize,
        shared: Arc<SystemShared>,
        receiver: Receiver<M>,
        sender: Sender<M>,
        exit_receiver: Receiver<(usize, ExitReason)>,
        args: A,
        func: TaskFn<Self, A, F>,
    ) -> JoinHandle<Result<()>>
    where
        F: Future<Output = Result<()>> + Send + 'static;
}

pub struct Receive<M> {
    id: usize,
    shared: Arc<SystemShared>,
    sender: Sender<M>,
    receiver: Receiver<M>,
}

impl<M> Context<M> for Receive<M> {
    fn start<F, A>(
        id: usize,
        shared: Arc<SystemShared>,
        receiver: Receiver<M>,
        sender: Sender<M>,
        exit_receiver: Receiver<(usize, ExitReason)>,
        args: A,
        func: TaskFn<Self, A, F>,
    ) -> JoinHandle<Result<()>>
    where
        F: Future<Output = Result<()>> + Send + 'static,
    {
        let ctx = Receive {
            id,
            shared,
            sender,
            receiver,
        };

        let future = func(ctx, args);

        tokio::spawn(async move {
            tokio::select! {
                res = future => res,
                _ = exit_receiver.recv_async() => anyhow::bail!("exit"),
            }
        })
    }
}

impl<M> Receive<M> {
    pub fn pid(&self) -> Pid<M> {
        Pid {
            id: self.id,
            sender: self.sender.clone(),
        }
    }

    pub async fn receive(&self) -> M {
        self.receiver.recv_async().await.unwrap()
    }

    pub fn spawn<S, MN, A, F>(&self, func: TaskFn<S, A, F>, args: A) -> Pid<MN>
    where
        S: Context<MN>,
        MN: Send + 'static,
        F: Future<Output = Result<()>> + Send + 'static,
    {
        self.shared.spawn(None, func, args)
    }

    pub fn spawn_link<S, MN, A, F>(&self, func: TaskFn<S, A, F>, args: A) -> Pid<MN>
    where
        S: Context<MN>,
        MN: Send + 'static,
        F: Future<Output = Result<()>> + Send + 'static,
    {
        self.shared.spawn(Some(self.id), func, args)
    }
}

pub enum Signal<T> {
    Exit((usize, ExitReason)),
    Message(T),
}

pub struct Trap<M> {
    id: usize,
    shared: Arc<SystemShared>,
    sender: Sender<M>,
    receiver: Receiver<M>,
    exit_receiver: Receiver<(usize, ExitReason)>,
}

impl<M> Context<M> for Trap<M> {
    fn start<F, A>(
        id: usize,
        shared: Arc<SystemShared>,
        receiver: Receiver<M>,
        sender: Sender<M>,
        exit_receiver: Receiver<(usize, ExitReason)>,
        args: A,
        func: TaskFn<Self, A, F>,
    ) -> JoinHandle<Result<()>>
    where
        F: Future<Output = Result<()>> + Send + 'static,
    {
        let ctx = Trap {
            id,
            shared,
            sender,
            receiver,
            exit_receiver,
        };

        let future = func(ctx, args);
        tokio::spawn(future)
    }
}

impl<M> Trap<M> {
    pub fn pid(&self) -> Pid<M> {
        Pid {
            id: self.id,
            sender: self.sender.clone(),
        }
    }

    pub async fn trap(&self) -> Signal<M> {
        tokio::select! {
            Ok(pair) = self.exit_receiver.recv_async() => Signal::Exit(pair),
            Ok(message) = self.receiver.recv_async() => Signal::Message(message),
        }
    }

    pub fn spawn<S, MN, A, F>(&self, func: TaskFn<S, A, F>, args: A) -> Pid<MN>
    where
        S: Context<MN>,
        MN: Send + 'static,
        F: Future<Output = Result<()>> + Send + 'static,
    {
        self.shared.spawn(None, func, args)
    }

    pub fn spawn_link<S, MN, A, F>(&self, func: TaskFn<S, A, F>, args: A) -> Pid<MN>
    where
        S: Context<MN>,
        MN: Send + 'static,
        F: Future<Output = Result<()>> + Send + 'static,
    {
        self.shared.spawn(Some(self.id), func, args)
    }
}
