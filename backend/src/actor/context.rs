use std::sync::Arc;

use anyhow::Result;
use flume::{Receiver, Sender};
use futures::Future;
use tokio::task::JoinHandle;

use super::system::{ExitReason, Pid, SystemShared};

/// A type of function that could be turned into an actor
/// Takes an Argument
pub type ArgTaskFn<S, A, F> = fn(S, A) -> F;

/// A type of function that could be turned into a actor
pub type TaskFn<S, F> = fn(S) -> F;

/// A monotonic Actor id
pub type ActorId = usize;

pub trait Context<M>: ActorStart<M> {
    fn pid(&self) -> Pid<M>;
    fn id(&self) -> ActorId;

    fn spawner(&self) -> ContextSpawner {
        let id = self.id();

        ContextSpawner {
            id,
            shared: self.shared_clone(),
        }
    }

    fn spawn_with_argument<S, MN, A, F>(&self, func: ArgTaskFn<S, A, F>, args: A) -> Pid<MN>
    where
        S: Context<MN>,
        MN: Send + 'static,
        F: Future<Output = Result<()>> + Send + 'static,
    {
        self.shared_clone().spawn_with_argument(None, func, args)
    }

    fn spawn_link_with_argument<S, MN, A, F>(&self, func: ArgTaskFn<S, A, F>, args: A) -> Pid<MN>
    where
        S: Context<MN>,
        MN: Send + 'static,
        F: Future<Output = Result<()>> + Send + 'static,
    {
        let id = self.id();
        self.shared_clone()
            .spawn_with_argument(Some(id), func, args)
    }

    fn spawn<S, MN, F>(&self, func: TaskFn<S, F>) -> Pid<MN>
    where
        S: Context<MN>,
        MN: Send + 'static,
        F: Future<Output = Result<()>> + Send + 'static,
    {
        self.shared_clone().spawn(None, func)
    }

    fn spawn_link<S, MN, F>(&self, func: TaskFn<S, F>) -> Pid<MN>
    where
        S: Context<MN>,
        MN: Send + 'static,
        F: Future<Output = Result<()>> + Send + 'static,
    {
        let id = self.id();
        self.shared_clone().spawn(Some(id), func)
    }

    fn register<S, ME>(&self, name: S, pid: Pid<ME>)
    where
        S: Into<String>,
        ME: Send + Sync + 'static,
    {
        self.shared().register(name, pid)
    }

    fn lookup<ME>(&self, name: &str) -> Result<Pid<ME>>
    where
        ME: Send + Sync + 'static,
    {
        self.shared().lookup(name).ok_or_else(|| anyhow::anyhow!("Actor is not registered {}", name))
    }
}

pub trait ActorStart<M> {
    fn start_with_argument<F, A>(
        id: ActorId,
        shared: Arc<SystemShared>,
        receiver: Receiver<M>,
        sender: Sender<M>,
        exit_receiver: Receiver<(ActorId, ExitReason)>,
        args: A,
        func: ArgTaskFn<Self, A, F>,
    ) -> JoinHandle<Result<()>>
    where
        F: Future<Output = Result<()>> + Send + 'static;

    fn start<F>(
        id: ActorId,
        shared: Arc<SystemShared>,
        receiver: Receiver<M>,
        sender: Sender<M>,
        exit_receiver: Receiver<(ActorId, ExitReason)>,
        func: TaskFn<Self, F>,
    ) -> JoinHandle<Result<()>>
    where
        F: Future<Output = Result<()>> + Send + 'static;

    fn shared_clone(&self) -> Arc<SystemShared>;
    fn shared(&self) -> &SystemShared;
}

/// ContextSpawner is a way to hand out the ability to spawn and link Actors on behalf of another Actor.
///
/// This is useful when ownership rules won't allow the base actor to pass it's context using borrowing which is often the case in async with `'static` + [`Send`] requirements.
/// The ContextSpawner might outlive the Actor that it spawns on behalf of.
#[derive(Clone)]
pub struct ContextSpawner {
    id: ActorId,
    shared: Arc<SystemShared>,
}

impl ContextSpawner {
    pub fn spawn_with_argument<S, M, A, F>(&self, func: ArgTaskFn<S, A, F>, args: A) -> Pid<M>
    where
        S: Context<M>,
        M: Send + 'static,
        F: Future<Output = Result<()>> + Send + 'static,
    {
        self.shared.spawn_with_argument(None, func, args)
    }

    pub fn spawn_link_with_argument<S, M, A, F>(&self, func: ArgTaskFn<S, A, F>, args: A) -> Pid<M>
    where
        S: Context<M>,
        M: Send + 'static,
        F: Future<Output = Result<()>> + Send + 'static,
    {
        self.shared.spawn_with_argument(Some(self.id), func, args)
    }

    pub fn spawn<S, M, F>(&self, func: TaskFn<S, F>) -> Pid<M>
    where
        S: Context<M>,
        M: Send + 'static,
        F: Future<Output = Result<()>> + Send + 'static,
    {
        self.shared.spawn(None, func)
    }

    pub fn spawn_link<S, M, F>(&self, func: TaskFn<S, F>) -> Pid<M>
    where
        S: Context<M>,
        M: Send + 'static,
        F: Future<Output = Result<()>> + Send + 'static,
    {
        self.shared.spawn(Some(self.id), func)
    }
}
