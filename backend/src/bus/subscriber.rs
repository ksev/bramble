use std::sync::{Arc, Mutex};

use smallvec::SmallVec;
use tokio::sync::Notify;

pub struct Subscriber<T> {
    pub mutex: Arc<Mutex<SmallVec<[T; 1]>>>,
}

pub struct Subscription<T> {
    mutex: Arc<Mutex<SmallVec<[T; 1]>>>,
    notify: Arc<Notify>,

    _guard: super::RemoveKey<T>,
}

impl<T> Subscription<T> {
    pub fn new(
        shared: Arc<Mutex<SmallVec<[T; 1]>>>,
        notify: Arc<Notify>,
        guard: super::RemoveKey<T>,
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
