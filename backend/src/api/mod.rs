use std::{collections::BTreeMap, sync::Arc};

use futures::TryStreamExt;

use anyhow::Result;
use async_graphql::{Context, Object, Schema, SimpleObject, Subscription};
use futures::{Stream, StreamExt};

use crate::{
    automation::Automation, integration::zigbee2mqtt, io::mqtt::MqttServerInfo, task::Task,
};

pub struct Query;

#[Object]
impl Query {
    /// Get all or a specific device
    async fn device<'c>(&self, ctx: &Context<'c>, id: Option<String>) -> Result<Vec<Device>> {
        let task = ctx.data_unchecked::<Task>();
        let mut conn = task.db.acquire().await?;

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
    async fn features<'c>(&self, ctx: &Context<'c>) -> Result<Vec<Feature>> {
        let task = ctx.data_unchecked::<Task>();
        let mut conn = task.db.acquire().await?;
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
    async fn value<'c>(&self, ctx: &Context<'c>) -> Value {
        let task = ctx.data_unchecked::<Task>();
        task.sources
            .get((self.device_id.into(), self.inner.id.clone()))
            .clone()
            .into()
    }
    /// Automation associated with this feature, ONLY sink features can have automations
    async fn automate(&self) -> Option<serde_json::Value> {
        self.inner
            .automate
            .as_ref()
            .and_then(|a| serde_json::to_value(a).ok())
    }
}

pub struct Mutation;

#[Object]
impl Mutation {
    /// Create a new generic virtual device, this is just a recepticle to
    /// attach value buffers to
    async fn generic_device<'c>(&self, ctx: &Context<'c>, name: String) -> Result<String> {
        let task = ctx.data_unchecked::<Task>();
        let mut conn = task.db.acquire().await?;
        let dev = crate::device::Device::create_generic(name, &mut conn).await?;

        let id = dev.id.clone();

        let shared = Arc::new(dev);
        task.bus.device.add.publish(shared);

        Ok(id)
    }
    /// Create a value buffer on the target device
    /// this device must exist
    async fn value_buffer<'c>(
        &self,
        ctx: &Context<'c>,
        device_id: String,
        name: String,
        kind: crate::device::ValueKind,
        meta: Option<BTreeMap<String, serde_json::Value>>,
    ) -> Result<String> {
        let task = ctx.data_unchecked::<Task>();
        let mut conn = task.db.acquire().await?;

        let feature = crate::device::Feature::attach_virtual(
            &device_id,
            name,
            kind,
            meta.unwrap_or_default(),
            &mut conn,
        )
        .await?;

        notify_device_changed(&device_id, task).await?;

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
        let task = ctx.data_unchecked::<Task>();

        let server_info = MqttServerInfo::new(host, port.unwrap_or(1883), username, password);
        let mut conn = task.db.acquire().await?;

        let device = zigbee2mqtt::create_integration_device(server_info, task, &mut conn).await?;

        Ok(device.into())
    }
    /// Add or change automation for a feature
    async fn automate<'c>(
        &self,
        ctx: &Context<'c>,
        device_id: String,
        feature_id: String,
        program: serde_json::Value,
    ) -> Result<usize> {
        let program: Automation = serde_json::from_value(program)?;

        program.compile(&device_id, &feature_id)?;

        let task = ctx.data_unchecked::<Task>();

        let mut conn = task.db.acquire().await?;
        let mut feature = crate::device::Feature::load(&device_id, &feature_id, &mut conn).await?;

        feature.automate = Some(program);
        feature.save(&device_id, &mut conn).await?;

        notify_device_changed(&device_id, task).await?;

        Ok(0)
    }
}

// Helper fn to load a device and notify on the bus that it has changed
async fn notify_device_changed(id: &str, task: &Task) -> Result<()> {
    let mut conn = task.db.acquire().await?;

    let device = crate::device::Device::load_by_id(&id, &mut conn).await?;
    task.bus.device.add.publish(Arc::new(device));

    Ok(())
}

pub struct Subscription;

#[Subscription]
impl Subscription {
    /// Listen for updates to feature values on devices
    /// This will print out all updates on all devices
    async fn values<'c>(&'c self, ctx: &Context<'c>) -> impl Stream<Item = ValueUpdate> + 'c {
        let task = ctx.data_unchecked::<Task>();

        tracing::debug!("GraphQL subscribe values");
        task.bus
            .device
            .value
            .subscribe()
            .map(|(d, p, v)| ValueUpdate {
                device: d,
                feature: p,
                value: v.into(),
            })
    }

    /// Listen for changes in devices
    async fn device<'c>(&'c self, ctx: &Context<'c>) -> impl Stream<Item = Device> + 'c {
        let task = ctx.data_unchecked::<Task>();

        tracing::debug!("GraphQL subscribe device updates");
        task.bus.device.add.subscribe().map(|d| d.into())
    }
}

pub type ApiSchema = Schema<Query, Mutation, Subscription>;
