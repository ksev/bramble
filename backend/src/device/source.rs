use dashmap::DashMap;
use once_cell::sync::Lazy;
use tracing::debug;

use crate::bus::BUS;

pub static SOURCES: Lazy<Sources> = Lazy::new(|| Sources::default());

/// Sources a struct that keeps all the current values from all the sources the application know about
#[derive(Default)]
pub struct Sources {
    storage: DashMap<(String, String), Result<serde_json::Value, String>>,
}

impl Sources {
    pub fn set(&self, key: (String, String), value: Result<serde_json::Value, String>) {
        let same = if let Some(current) = self.storage.get(&key) {
            *current == value
        } else {
            false
        };

        if !same {
            debug!("{:?} update source {:?}", key, value);

            BUS.device.value.publish((key.0.clone(), key.1.clone(), value.clone()));

            self.storage.insert(key, value);
            


        }
    }
}
