use std::sync::{Arc, Mutex};

use futures::{stream, Stream};
use slotmap::{DefaultKey, SlotMap};
use smallvec::{smallvec, SmallVec};
use tokio::sync::Notify;

macro_rules! static_topic {
    ($name:ident, $of:ty) => {
        static $name: once_cell::sync::Lazy<crate::topic::Topic<$of>> =
            once_cell::sync::Lazy::new(|| crate::topic::Topic::default());
    };
}

pub(crate) use static_topic;

#[derive(Clone)]
pub struct Topic<T> {
    subs: Arc<Mutex<SlotMap<DefaultKey, Subscriber<T>>>>,
    notify: Arc<Notify>,
}

impl<T> Topic<T>
where
    T: Clone,
{
    pub fn subscribe(&self) -> impl Stream<Item = T> + '_ {
        let mut subs = self.subs.lock().expect("Lock subscribers");

        let mutex = Arc::new(Mutex::new(smallvec![]));

        let key = subs.insert(Subscriber {
            mutex: mutex.clone(),
        });

        let guard = RemoveKey {
            key,
            subs: self.subs.clone(),
        };

        let subscription = Subscription::new(mutex, self.notify.clone(), guard);

        let strm = stream::unfold(subscription, |mut sub| async move {
            let val = sub.recv().await;
            Some((val, sub))
        });

        Box::pin(strm)
    }

    pub fn publish(&self, payload: T) {
        {
            let subs = self.subs.lock().expect("Lock subscribers failed");
            let mut iter = subs.iter().peekable();

            while let Some((_, sub)) = iter.next() {
                let mut buffer = sub.mutex.lock().expect("Locking subscriber buffer failed");

                if iter.peek().is_none() {
                    // We are at the last value
                    // So we can skip the last clone
                    buffer.push(payload);
                    break;
                } else {
                    buffer.push(payload.clone());
                }
            }
        }

        self.notify.notify_waiters();
    }
}

impl<T> Default for Topic<T> {
    fn default() -> Self {
        Self {
            subs: Default::default(),
            notify: Default::default(),
        }
    }
}

pub struct RemoveKey<T> {
    subs: Arc<Mutex<SlotMap<DefaultKey, Subscriber<T>>>>,
    key: DefaultKey,
}

impl<T> Drop for RemoveKey<T> {
    fn drop(&mut self) {
        let mut subs = self.subs.lock().expect("Lock subscribers failed");
        subs.remove(self.key);
    }
}

pub struct Subscriber<T> {
    pub mutex: Arc<Mutex<SmallVec<[T; 1]>>>,
}

pub struct Subscription<T> {
    mutex: Arc<Mutex<SmallVec<[T; 1]>>>,
    notify: Arc<Notify>,

    _guard: RemoveKey<T>,
}

impl<T> Subscription<T> {
    pub fn new(
        shared: Arc<Mutex<SmallVec<[T; 1]>>>,
        notify: Arc<Notify>,
        guard: RemoveKey<T>,
    ) -> Subscription<T> {
        Subscription {
            mutex: shared,
            notify,
            _guard: guard,
        }
    }

    pub async fn recv(&mut self) -> T {
        loop {
            {
                let mut buffer = self.mutex.lock().expect("Locking subscriber buffer failed");

                if !buffer.is_empty() {
                    return buffer.remove(0);
                }
            }

            self.notify.notified().await;
        }
    }
}
