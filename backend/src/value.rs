use std::future;

use anyhow::Result;
use dashmap::{mapref::one::Ref, DashMap};
use futures::{Stream, StreamExt};
use once_cell::sync::Lazy;
use serde_json::Value as Json;
use tracing::debug;

use crate::strings::IString;
use crate::task::Task;
use crate::topic::static_topic;

static STORAGE: Lazy<DashMap<ValueId, Result<Json, String>>> = Lazy::new(DashMap::default);

static_topic!(INCOMING, (ValueId, Result<Json, String>));
static_topic!(OUTGOING, (ValueId, Json));

pub async fn catch_virtual_push(_: Task) -> Result<()> {
    let mut values = push_subscribe().filter(|(id, _)| {
        let s: &str = id.feature.into();
        future::ready(s.starts_with("virtual"))
    });

    while let Some((vid, value)) = values.next().await {
        set_current(vid, Ok(value));
    }

    Ok(())
}

#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug, Ord, PartialOrd)]
pub struct ValueId {
    pub device: IString,
    pub feature: IString,
}

impl ValueId {
    pub fn new<D, F>(device_id: D, feature_id: F) -> ValueId
    where
        D: Into<IString>,
        F: Into<IString>,
    {
        ValueId {
            device: device_id.into(),
            feature: feature_id.into(),
        }
    }
}

pub fn current(key: ValueId) -> Ref<'static, ValueId, Result<Json, String>> {
    STORAGE.entry(key).or_insert(Ok(Json::Null)).downgrade()
}

pub fn set_current(key: ValueId, value: Result<Json, String>) {
    let same = if let Some(current) = STORAGE.get(&key) {
        *current == value
    } else {
        false
    };

    if !same {
        debug!("{:?} update source {:?}", key, value);

        STORAGE.insert(key, value.clone());
        INCOMING.publish((key, value));
    }
}

pub fn subscribe() -> impl Stream<Item = (ValueId, Result<Json, String>)> {
    INCOMING.subscribe()
}

pub fn push(key: ValueId, value: Json) {
    OUTGOING.publish((key, value));
}

pub fn push_subscribe() -> impl Stream<Item = (ValueId, Json)> {
    OUTGOING.subscribe()
}
