use std::{collections::BTreeMap, sync::Arc};

use futures::TryStreamExt;

use anyhow::Result;
use async_graphql::{Object, Schema, SimpleObject, Subscription};
use futures::{Stream, StreamExt};
use uuid::Uuid;

use crate::{bus::BUS, db};

pub struct Query;

#[Object]
impl Query {
    /// Get all or a specific device
    async fn device(&self, id: Option<String>) -> Result<Vec<Device>> {
        let pool = crate::db::pool().await;

        if let Some(id) = id {
            let device = crate::device::Device::load_by_id(&id, pool).await?;

            Ok(vec![Device {
                inner: DeviceInner::Owned(device),
            }])
        } else {
            let vec = crate::device::Device::all(pool)
                .map_ok(|d| Device {
                    inner: DeviceInner::Owned(d),
                })
                .try_collect()
                .await?;

            Ok(vec)
        }
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
    /// All the features a device exposes
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
    /// What type of value this feature has
    async fn kind(&self) -> crate::device::ValueKind {
        self.inner.kind
    }
    /// Json metadata about the feature
    /// Common meta data is Number unit a list of possible States for state
    async fn meta(&self) -> &BTreeMap<String, serde_json::Value> {
        &self.inner.meta
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    /// Create a new generic virtual device, this is just a recepticle to
    /// attach value buffers to
    async fn generic_device(&self, name: String) -> Result<String> {
        let id = Uuid::new_v4().to_string();

        let conn = db::pool().await;

        let device = crate::device::Device {
            id: id.clone(),
            name,
            device_type: crate::device::DeviceType::Virtual {
                vty: crate::device::VirtualType::Generic,
            },
            parent: None,
            task_spec: vec![],
        };

        device.save(conn).await?;

        Ok(id)
    }
    /// Create a value buffer on the target device
    /// this device must exist
    async fn value_buffer(
        &self,
        device_id: String,
        name: String,
        kind: crate::device::ValueKind,
        meta: Option<BTreeMap<String, serde_json::Value>>,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();

        let conn = db::pool().await;

        let feature = crate::device::Feature {
            id: id.clone(),
            name,
            direction: crate::device::ValueDirection::SourceSink,
            kind,
            meta: meta.unwrap_or_default(),
        };

        feature.save(&device_id, conn).await?;

        Ok(id)
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

pub type ApiSchema = Schema<Query, Mutation, Subscription>;
