use std::{
    any::Any,
    pin::Pin,
    sync::{Arc, Weak},
};

use futures::{Future, FutureExt};
use tokio::sync::{
    mpsc::{UnboundedSender},
    oneshot::channel,
};

use super::{
    task::{HandlerTask, Task},
    Actor, Context, Handler, ActorDead,
};


/// An actor address that wont keep the actor from stopping,
/// You need to convert the WeakAddr to a full Addr to send a message
pub struct WeakAddr<A: Actor> {
    sender: Weak<UnboundedSender<Box<dyn Task<A>>>>,
}

impl<A: Actor> WeakAddr<A> {
    /// Try and upgrade the WeakAddr to a full fledged Addr that can send messages
    pub fn to_addr(&self) -> Option<Addr<A>> {
        let arc = self.sender.upgrade()?;
        Some(Addr::new(arc))
    }

    pub fn send<M: 'static + Send>(
        &self,
        message: M,
    ) -> Result<impl Future<Output = A::Result>, ActorDead>
    where
        A: Handler<M>,
    {
        match self.to_addr() {
            None => Err(ActorDead),
            Some(addr) => addr.send(message)
        }
    }
}

/// Use [`Addr`] to send messages to [`Actor`] and get the handling result.
///
/// Clones of [`Addr`] send messages to the same [`Actor`] instance.
pub struct Addr<A: Actor> {
    sender: Arc<UnboundedSender<Box<dyn Task<A>>>>,
}

impl<A: Actor> Addr<A> {
    pub(crate) fn new(sender: Arc<UnboundedSender<Box<dyn Task<A>>>>) -> Self {
        Addr { sender }
    }

    /// Sends a message to [`Actor`] and returns the handling result, if the [`Actor`] is a [`Handler`]`<M>`.
    pub fn send<M: 'static + Send>(
        &self,
        message: M,
    ) -> Result<impl Future<Output = A::Result>, ActorDead>
    where
        A: Handler<M>,
    {
        let (sender, receiver) = channel();
        let task = Box::new(HandlerTask::new(message, handle, sender));
        self.sender.send(task).map_err(|_| ActorDead)?;
        Ok(receiver
            .map(|maybe_result| maybe_result.expect("Worker thread has stopped unexpectedly")))
    }

    /// Downgrade the Addr to a WeakAddr
    pub fn to_weak(&self) -> WeakAddr<A> {
        let sender = Arc::downgrade(&self.sender);
        WeakAddr { sender }
    }

    /// Turn the address into an untyped reference, useful if you dont want to send anything to the actor anymore
    /// but want to keep it alive
    pub fn untyped_reference(self) -> UntypedAddr {
        UntypedAddr { _addr: self.sender }
    }
}

fn handle<'a, A: Actor + Handler<M>, M: 'static + Send>(
    actor: &'a mut A,
    message: M,
    context: &'a mut Context<A>,
) -> Pin<Box<dyn Future<Output = A::Result> + Send + 'a>> {
    A::handle(actor, message, context)
}

impl<A: Actor> Clone for Addr<A> {
    fn clone(&self) -> Self {
        Addr {
            sender: self.sender.clone(),
        }
    }
}

impl<A: Actor> std::fmt::Debug for Addr<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Addr")
            .field("sender", &self.sender)
            .finish()
    }
}

pub struct UntypedAddr {
    _addr: Arc<dyn Any + Send + Sync>,
}
