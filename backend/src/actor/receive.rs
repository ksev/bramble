use std::sync::Arc;

use anyhow::Result;
use flume::{Receiver, Sender};
use futures::Future;
use tokio::task::JoinHandle;

use super::{
    context::{ActorId, ActorStart, ArgTaskFn, Context, TaskFn},
    system::{ExitReason, Pid, SystemShared},
};

/// A task is a normal Actor that does not receive and messages so
/// 
/// This is just type type alias the communicate that fact
pub type Task = Receive<()>;

/// [`Receive`] is a [`Context`] for Actors that only want to react to the messages they receive.
/// 
/// This is the "normal" context Actors should use. Messages are the way actors communicate with each other.
/// 
/// # Example
/// 
/// ```
/// // Define the messages we can receive
/// struct Message(String);
/// 
/// async fn actor(ctx: Receive<Message>) -> Result<()> {
///     loop {
///         let Message(m) = ctx.receive().await;
///         println!("{m}");
///     }
/// }
/// ```
pub struct Receive<M> {
    id: ActorId,
    shared: Arc<SystemShared>,
    sender: Sender<M>,
    receiver: Receiver<M>,
}

impl<M> Context<M> for Receive<M> {
    fn pid(&self) -> Pid<M> {
        Pid::new(self.id, self.sender.clone())
    }

    fn id(&self) -> ActorId {
        self.id
    }
}

impl<M> ActorStart<M> for Receive<M> {
    fn start<F>(
        id: ActorId,
        shared: Arc<SystemShared>,
        receiver: Receiver<M>,
        sender: Sender<M>,
        exit_receiver: Receiver<(ActorId, ExitReason)>,
        func: TaskFn<Self, F>,
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

        let future = func(ctx);

        tokio::spawn(async move {
            tokio::select! {
                res = future => res,
                _ = exit_receiver.recv_async() => anyhow::bail!("exit"),
            }
        })
    }

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

    fn shared_clone(&self) -> Arc<SystemShared> {
        self.shared.clone()
    }

    fn shared(&self) -> &SystemShared {
        &self.shared
    }
}

impl<M> Receive<M> {
    /// Receive a message from the mailbox
    pub async fn receive(&self) -> M {
        self.receiver.recv_async().await.unwrap()
    }
}
