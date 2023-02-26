use dashmap::{mapref::one::Ref, DashMap};
use tracing::debug;

use crate::bus::Topic;
use crate::strings::IString;

use super::FeatureValue;

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug, Ord, PartialOrd)]
pub struct SourceId {
    pub device: IString,
    pub feature: IString,
}

impl SourceId {
    pub fn new<D, F>(device_id: D, feature_id: F) -> SourceId
    where
        D: Into<IString>,
        F: Into<IString>,
    {
        SourceId {
            device: device_id.into(),
            feature: feature_id.into(),
        }
    }
}

/// Sources a struct that keeps all the current values from all the sources the application know about
pub struct Sources {
    topic: Topic<(SourceId, FeatureValue)>,
    storage: DashMap<SourceId, FeatureValue>,
}

impl Sources {
    pub fn new(topic: Topic<(SourceId, FeatureValue)>) -> Sources {
        Sources {
            topic,
            storage: DashMap::default(),
        }
    }

    pub fn set(&self, key: SourceId, value: FeatureValue) -> bool {
        let same = if let Some(current) = self.storage.get(&key) {
            *current == value
        } else {
            false
        };

        if !same {
            debug!("{:?} update source {:?}", key, value);
            self.topic.publish((key.clone(), value.clone()));
            self.storage.insert(key, value);
        }

        !same
    }

    pub fn get(&self, key: SourceId) -> Ref<SourceId, FeatureValue> {
        self.storage
            .entry(key)
            .or_insert(Ok(serde_json::Value::Null))
            .downgrade()
    }

    /*
    pub fn all(&self) -> impl Iterator<Item = RefMulti<'_, (String, String), Value>> {
        self.storage.iter()
    }
     */
}
