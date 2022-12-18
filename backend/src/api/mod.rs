use std::{collections::BTreeMap, sync::Arc};

use futures::TryStreamExt;

use anyhow::Result;
use async_graphql::{EmptyMutation, Object, Schema, SimpleObject, Subscription};
use futures::{Stream, StreamExt};

use crate::{bus::BUS, db};

pub struct Query;

#[Object]
impl Query {
    /// Get all or a specific device
    async fn device(&self, id: Option<String>) -> Result<Vec<Device>> {
        let pool = crate::db::pool().await;

        let vec = crate::device::Device::all(pool)
            .map_ok(|d| Device {
                inner: DeviceInner::Owned(d),
            })
            .try_collect()
            .await?;

        Ok(vec)
    }
}

#[derive(SimpleObject)]
/// A value of a device that has been reported to the system
struct Value {
    /// The id of the device the value is for
    device: String,
    /// The feature's name on the device the value is for
    feature: String,
    /// The value of the device, note can be error
    value: Result<serde_json::Value, String>,
}

/// A device added to the system
struct Device {
    inner: DeviceInner,
}

enum DeviceInner {
    Arc(Arc<crate::device::Device>),
    Owned(crate::device::Device),
}

impl Device {
    fn borrow(&self) -> &'_ crate::device::Device {
        match &self.inner {
            DeviceInner::Arc(a) => a,
            DeviceInner::Owned(o) => o,
        }
    }
}

#[Object]
impl Device {
    /// Device id, unique device id
    async fn id(&self) -> &'_ str {
        &self.borrow().id
    }
    /// A nicer looking name for the device
    async fn name(&self) -> &'_ str {
        &self.borrow().name
    }
    /// Some devices are spawned by other devices, this tracks that higharchy
    async fn parent(&self) -> Option<&'_ String> {
        self.borrow().parent.as_ref()
    }

    async fn features(&self) -> Result<Vec<Feature>> {
        let conn = db::pool().await;

        let vec = crate::device::Feature::load_by_device(&self.borrow().id, conn)
            .map_ok(|inner| Feature { inner })
            .try_collect()
            .await?;

        Ok(vec)
    }
}

struct Feature {
    inner: crate::device::Feature,
}

#[Object]
impl Feature {
    /// Feature id these are only device unique not global unique
    async fn id(&self) -> &'_ str {
        &self.inner.id
    }
    /// Feature name, an nice-er to look at name
    async fn name(&self) -> &'_ str {
        &self.inner.name
    }
    /// Which direction does the data flow
    async fn direction(&self) -> crate::device::ValueDirection {
        self.inner.direction
    }

    async fn meta(&self) -> &BTreeMap<String, serde_json::Value> {
        &self.inner.meta
    }
}

pub struct Subscription;

#[Subscription]
impl Subscription {
    /// Listen for updates to feature values on devices
    /// This will print out all updates on all devices
    async fn values(&self) -> impl Stream<Item = Value> {
        tracing::debug!("GraphQL subscribe values");
        BUS.device.value.subscribe().map(|(d, p, v)| Value {
            device: d,
            feature: p,
            value: v,
        })
    }

    /// Listen for changes in devices
    async fn device(&self) -> impl Stream<Item = Device> {
        tracing::debug!("GraphQL subscribe device updates");
        BUS.device.add.subscribe().map(|d| Device {
            inner: DeviceInner::Arc(d.clone()),
        })
    }
}

pub type ApiSchema = Schema<Query, EmptyMutation, Subscription>;
