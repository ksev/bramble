use std::sync::Arc;

use anyhow::Result;
use flume::{Receiver, Sender};
use futures::Future;
use tokio::task::JoinHandle;

use super::{
    context::{ActorId, ActorStart, ArgTaskFn, Context, TaskFn},
    system::{ExitReason, Pid, SystemShared},
};

pub enum Signal<T> {
    Exit((ActorId, ExitReason)),
    Message(T),
}
pub struct Trap<M> {
    id: ActorId,
    shared: Arc<SystemShared>,
    sender: Sender<M>,
    receiver: Receiver<M>,
    exit_receiver: Receiver<(ActorId, ExitReason)>,
}

impl<M> ActorStart<M> for Trap<M> {
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
        let ctx = Trap {
            id,
            shared,
            sender,
            receiver,
            exit_receiver,
        };

        let future = func(ctx);
        tokio::spawn(future)
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

    fn shared_clone(&self) -> Arc<SystemShared> {
        self.shared.clone()
    }

    fn shared(&self) -> &SystemShared {
        &self.shared
    }
}

impl<M> Context<M> for Trap<M> {
    fn pid(&self) -> Pid<M> {
        Pid::new(self.id, self.sender.clone())
    }

    fn id(&self) -> ActorId {
        self.id
    }
}

impl<M> Trap<M> {
    pub async fn trap(&self) -> Signal<M> {
        tokio::select! {
            Ok(pair) = self.exit_receiver.recv_async() => Signal::Exit(pair),
            Ok(message) = self.receiver.recv_async() => Signal::Message(message),
        }
    }
}
