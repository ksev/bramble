use std::sync::{Arc, Mutex};

use slotmap::{DefaultKey, SlotMap};
use smallvec::smallvec;
use tokio::sync::Notify;

use super::{Subscriber, Subscription};

pub struct Topic<T> {
    subs: Arc<Mutex<SlotMap<DefaultKey, Subscriber<T>>>>,
    notify: Arc<Notify>,
}

impl<T> Topic<T>
where
    T: Clone,
{
    pub fn subscribe(&self) -> Subscription<T> {
        let mut subs = self.subs.lock().expect("Lock subscribers");

        let mutex = Arc::new(Mutex::new(smallvec![]));

        let _key = subs.insert(Subscriber {
            mutex: mutex.clone(),
        });

        let subscription = Subscription::new(mutex, self.notify.clone());

        subscription
    }

    pub fn publish(&self, payload: T) {
        {
            let subs = self.subs.lock().expect("Lock subscribers");
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
