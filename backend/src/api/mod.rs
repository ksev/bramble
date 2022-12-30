use std::{collections::BTreeMap, sync::Arc};

use futures::TryStreamExt;

use anyhow::Result;
use async_graphql::{Context, Object, Schema, SimpleObject, Subscription};
use futures::{Stream, StreamExt};

use crate::{
    bus::BUS, db, device::SOURCES, integration::zigbee2mqtt, io::mqtt::MqttServerInfo, task::Task,
};

pub struct Query;

#[Object]
impl Query {
    /// Get all or a specific device
    async fn device(&self, id: Option<String>) -> Result<Vec<Device>> {
        let mut conn = db::connection().await?;

        if let Some(id) = id {
            let device = crate::device::Device::load_by_id(&id, &mut conn).await?;

            Ok(vec![device.into()])
        } else {
            let vec = crate::device::Device::all(&mut conn)
                .map_ok(|d| d.into())
                .try_collect()
                .await?;

            Ok(vec)
        }
    }
}

// This is just not to poulte this namespace with a bunch of short super generic symbols
mod inner_value {
    use async_graphql::{Object, Union};

    use crate::device::FeatureValue;

    pub struct Ok {
        value: serde_json::Value,
    }

    #[Object]
    impl Ok {
        async fn value(&self) -> &'_ serde_json::Value {
            &self.value
        }
    }

    pub struct Err {
        value: String,
    }

    #[Object]
    impl Err {
        async fn message(&self) -> &'_ str {
            &self.value
        }
    }

    #[derive(Union)]
    pub enum Value {
        Ok(Ok),
        Err(Err),
    }

    impl From<FeatureValue> for Value {
        fn from(fv: FeatureValue) -> Self {
            match fv {
                Ok(value) => Value::Ok(Ok { value }),
                Err(value) => Value::Err(Err { value }),
            }
        }
    }
}

use inner_value::Value;

#[derive(SimpleObject)]
/// A value of a device that has been reported to the system
struct ValueUpdate {
    /// The id of the device the value is for
    device: String,
    /// The feature's name on the device the value is for
    feature: String,
    /// The value of the device, note can be error
    value: Value,
}

/// A device added to the system
struct Device {
    inner: DeviceInner,
}

enum DeviceInner {
    Arc(Arc<crate::device::Device>),
    Owned(crate::device::Device),
}

impl From<crate::device::Device> for Device {
    fn from(d: crate::device::Device) -> Self {
        Device {
            inner: DeviceInner::Owned(d),
        }
    }
}

impl From<Arc<crate::device::Device>> for Device {
    fn from(d: Arc<crate::device::Device>) -> Self {
        Device {
            inner: DeviceInner::Arc(d),
        }
    }
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
        let mut conn = db::connection().await?;
        let device_id = &self.borrow().id;

        let vec = crate::device::Feature::load_by_device(device_id, &mut conn)
            .map_ok(|inner| Feature { device_id, inner })
            .try_collect()
            .await?;

        Ok(vec)
    }
}

struct Feature<'a> {
    device_id: &'a str,
    inner: crate::device::Feature,
}

#[Object]
impl<'a> Feature<'a> {
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
    /// The current value of the feature, ONLY source features will have a value
    async fn value(&self) -> Option<Value> {
        SOURCES
            .get(&(self.device_id.into(), self.inner.id.clone()))
            .map(|r| r.clone().into())
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    /// Create a new generic virtual device, this is just a recepticle to
    /// attach value buffers to
    async fn generic_device(&self, name: String) -> Result<String> {
        let mut conn = db::connection().await?;
        let dev = crate::device::Device::create_generic(name, &mut conn).await?;

        Ok(dev.id.clone())
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
        let mut conn = db::connection().await?;

        let feature = crate::device::Feature::attach_virtual(
            &device_id,
            name,
            kind,
            meta.unwrap_or_default(),
            &mut conn,
        )
        .await?;

        Ok(feature.id)
    }
    /// Add Zigbee2Mqtt integration
    async fn zigbee_2_mqtt<'c>(
        &self,
        ctx: &Context<'c>,
        host: String,
        port: Option<u16>,
        username: Option<String>,
        password: Option<String>,
    ) -> Result<Device> {
        let server_info = MqttServerInfo::new(host, port.unwrap_or(1883), username, password);
        let mut conn = db::connection().await?;

        let task = ctx.data_unchecked::<Task>();
        let device = zigbee2mqtt::create_integration_device(server_info, task, &mut conn).await?;

        Ok(device.into())
    }
}

pub struct Subscription;

#[Subscription]
impl Subscription {
    /// Listen for updates to feature values on devices
    /// This will print out all updates on all devices
    async fn values(&self) -> impl Stream<Item = ValueUpdate> {
        tracing::debug!("GraphQL subscribe values");
        BUS.device.value.subscribe().map(|(d, p, v)| ValueUpdate {
            device: d,
            feature: p,
            value: v.into(),
        })
    }

    /// Listen for changes in devices
    async fn device(&self) -> impl Stream<Item = Device> {
        tracing::debug!("GraphQL subscribe device updates");
        BUS.device.add.subscribe().map(|d| d.into())
    }
}

pub type ApiSchema = Schema<Query, Mutation, Subscription>;
