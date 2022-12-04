use std::sync::{Arc, Mutex};

use smallvec::SmallVec;
use tokio::sync::Notify;
//use slotmap::{DefaultKey, SlotMap};
use tracing::warn;

pub struct Subscriber<T> {
    pub mutex: Arc<Mutex<SmallVec<[T; 1]>>>,
}

pub struct Subscription<T> {
    mutex: Arc<Mutex<SmallVec<[T; 1]>>>,
    notify: Arc<Notify>,
    /*
       subs: Arc<Mutex<SlotMap<DefaultKey, Subscriber<T>>>>,
       key: DefaultKey,
    */
}

impl<T> Subscription<T> {
    pub fn new(shared: Arc<Mutex<SmallVec<[T; 1]>>>, notify: Arc<Notify>) -> Subscription<T> {
        Subscription {
            mutex: shared,
            notify,
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

impl<T> Drop for Subscription<T> {
    fn drop(&mut self) {
        //let mut subs = self.subs.lock().expect("This is a bad idea");
        //subs.remove(self.key);
        // TODO: Fix this
        warn!("Topic dropped, this need to clean")
    }
}
