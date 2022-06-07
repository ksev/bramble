use futures::future::BoxFuture;

use super::{Addr, Handler, WeakAddr, ActorDead};

type MessageFn<M, R> = Box<dyn Fn(M) -> Result<BoxFuture<'static, R>, ActorDead> + Send>;

/// An Actor address that can only send one type of message, useful for pub sub like things
pub struct MessageAddr<M, R = ()>
where
    M: 'static + Send,
    R: 'static + Send,
{
    call: MessageFn<M, R>,
}

impl<M, R> MessageAddr<M, R>
where
    M: 'static + Send,
    R: 'static + Send,
{
    pub fn from_addr<A>(addr: Addr<A>) -> MessageAddr<M, R>
    where
        A: Handler<M, Result = R>,
    {
        MessageAddr {
            call: Box::new(move |message: M| {
                let fut = addr.send(message)?;
                Ok(Box::pin(fut))
            }),
        }
    }

    pub fn send(&self, message: M) -> Result<BoxFuture<'static, R>, ActorDead> {
        (self.call)(message)
    }
}

impl<A, M> From<Addr<A>> for MessageAddr<M, A::Result>
where
    A: Handler<M>,
    M: 'static + Send,
{
    fn from(addr: Addr<A>) -> Self {
        MessageAddr::from_addr(addr)
    }
}

impl<M, R> std::fmt::Debug for MessageAddr<M, R>
where
    M: 'static + Send,
    R: 'static + Send,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageAddr").finish()
    }
}

/// A Weak version of message Address
pub struct WeakMessageAddr<M, R = ()>
where
    M: 'static + Send,
    R: 'static + Send,
{
    call: MessageFn<M, R>,
}

impl<M, R> WeakMessageAddr<M, R>
where
    M: 'static + Send,
    R: 'static + Send,
{
    pub fn from_weak_addr<A>(addr: WeakAddr<A>) -> WeakMessageAddr<M, R>
    where
        A: Handler<M, Result = R>,
    {
        WeakMessageAddr {
            call: Box::new(move |message: M| {
                let fut = addr.send(message)?;
                Ok(Box::pin(fut))
            }),
        }
    }

    pub fn send(&self, message: M) -> Result<BoxFuture<'static, R>, ActorDead> {
        (self.call)(message)
    }
}

impl<A, M> From<WeakAddr<A>> for WeakMessageAddr<M, A::Result>
where
    A: Handler<M>,
    M: 'static + Send,
{
    fn from(addr: WeakAddr<A>) -> Self {
        WeakMessageAddr::from_weak_addr(addr)
    }
}

impl<M, R> std::fmt::Debug for WeakMessageAddr<M, R>
where
    M: 'static + Send,
    R: 'static + Send,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WeakMessageAddr").finish()
    }
}