use dashmap::DashMap;
use once_cell::sync::Lazy;
use tracing::debug;

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
            self.storage.insert(key, value);
            // Report on the bus
        }
    }
}
