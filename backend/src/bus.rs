use std::{any::Any, borrow::Cow, sync::Arc};

use anyhow::{anyhow, Result};
use dashmap::{self, DashMap};
use flume::{unbounded, Receiver, Sender};
use once_cell::sync::Lazy;

pub static BUS: Lazy<Bus> = Lazy::new(|| Bus {
    topics: Default::default(),
});

pub trait Topic {
    type Payload: Send + Sync + 'static;
    fn name(&self) -> Cow<'static, str>;
}

pub struct Subscribed;

impl Topic for Subscribed {
    type Payload = TopicName;

    fn name(&self) -> Cow<'static, str> {
        "subscribed".into()
    }
}

enum TopicSignal<T>
where
    T: Topic,
{
    Publish(T::Payload),
    Subscribe(Sender<T::Payload>),
}

struct TopicContext<T>
where
    T: Topic,
{
    sender: Sender<TopicSignal<T>>,
}

type ThreadSafeAny = Box<dyn Any + 'static + Send + Sync>;
type TopicName = Cow<'static, str>;

pub struct Bus {
    topics: Arc<DashMap<TopicName, ThreadSafeAny>>,
}

impl Bus {
    /// Publish a message on the Bus on the specified topic this is non-blocking and safe
    pub fn publish<T>(&self, topic: &T, message: T::Payload) -> Result<()>
    where
        T: Topic + 'static + Send + Sync,
    {
        let key = topic.name();

        // If no ones subscribes then we dont even need to send the message
        if let Some(ctx) = self.topics.get(&key) {
            let ctx: &TopicContext<T> = ctx
                .downcast_ref()
                .ok_or_else(|| anyhow!("downcast bus context failed"))?;

            ctx.sender.send(TopicSignal::Publish(message))?;
        }

        Ok(())
    }

    /// Subscribe to a topic, returns a channel receiver to to get messages on that topic
    /// A meta publish will be generated on the topic 'subscibed'
    pub fn subscribe<T>(&self, topic: &T) -> Result<Receiver<T::Payload>>
    where
        T: Topic + 'static + Send + Sync,
    {
        let key = topic.name();
        let ctx = self
            .topics
            .entry(key)
            .or_insert_with(create_context::<T>)
            .downgrade();

        let ctx: &TopicContext<T> = ctx
            .downcast_ref()
            .ok_or_else(|| anyhow!("downcast bus context failed"))?;

        let (sender, receiver) = unbounded();

        ctx.sender.send(TopicSignal::Subscribe(sender))?;

        self.publish(&Subscribed, topic.name())?;

        Ok(receiver)
    }
}

fn create_context<T>() -> ThreadSafeAny
where
    T: Topic + 'static + Send + Sync,
{
    let (sender, receiver) = unbounded::<TopicSignal<T>>();

    // Route the message coming in on topics
    tokio::spawn(async move {
        let mut subscribers = Vec::with_capacity(1);

        while let Ok(signal) = receiver.recv_async().await {
            match signal {
                TopicSignal::Publish(payload) => {
                    todo!();
                    //subscribers.retain(|sender: &Sender<T::Payload>| sender.send(payload).is_ok())
                }
                TopicSignal::Subscribe(sender) => subscribers.push(sender),
            }
        }
    });

    Box::new(TopicContext { sender })
}
