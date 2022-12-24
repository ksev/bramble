mod device;

use std::sync::Arc;

use anyhow::Result;

use bytes::Buf;
pub use device::*;
use futures::{StreamExt, TryStreamExt};
use serde_json::Value;
use tracing::error;

use crate::{
    bus::BUS,
    db,
    device::{TaskSpec, SOURCES},
    io::mqtt::{MqttServerInfo, MqttSubscribe},
    task::Task,
};

pub async fn zigbee2mqtt_update(
    (parent, server): (String, MqttServerInfo),
    mut t: Task,
) -> Result<()> {
    let mut channel = BUS.mqtt.published.subscribe();

    BUS.mqtt.subscribe.publish(MqttSubscribe {
        topic: "zigbee2mqtt/bridge/devices".into(),
        server: server.clone(),
    });

    while let Some((topic, data)) = channel.next().await {
        if topic == "zigbee2mqtt/bridge/devices" {
            let reader = data.reader();

            let device_spec: Vec<Device> = serde_json::de::from_reader(reader)?;

            let iter = device_spec
                .into_iter()
                .filter_map(|d| d.into_device(&parent, server.clone()));

            let mut tx = db::pool().await.begin().await?;

            for (device, features) in iter {
                // Persist the device
                device.save(&mut tx).await?;

                for feature in features {
                    feature.save(&device.id, &mut tx).await?;
                }

                device.spawn_tasks(&mut t);

                BUS.device.add.publish(Arc::new(device));
            }

            tx.commit().await?;
        }
    }

    Ok(())
}

pub async fn zigbee2mqtt_device(device_id: String, _: Task) -> Result<()> {
    let (device, features) = {
        use crate::device::{Device, Feature};

        let pool = db::pool().await;
        let device = Device::load_by_id(&device_id, pool).await?;
        let features: Vec<Feature> = Feature::load_by_device_readable(&device.id, pool)
            .try_collect()
            .await?;

        (device, features)
    };

    use TaskSpec::Zigbee2MqttDevice;

    let Some(Zigbee2MqttDevice(subscribe)) = device.task_spec.get(0) else {
        anyhow::bail!("zigbee2mqtt_device did not get a task spec it expected, this is a bug");
    };

    let mut channel = BUS.mqtt.published.subscribe();

    BUS.mqtt.subscribe.publish(subscribe.clone());

    while let Some((topic, data)) = channel.next().await {
        if topic != subscribe.topic {
            continue;
        }

        let rdr = data.reader();
        let json: serde_json::Value = match serde_json::de::from_reader(rdr) {
            Ok(json) => json,
            Err(e) => {
                error!("Error parsing value json for {}\n{e:?}", device.name);
                continue;
            }
        };

        for spec in &features {
            let ptr = &format!("/{}", spec.id);
            let key = (device.id.clone(), spec.id.clone());

            let Some(mut value) = json.pointer(ptr) else {
                // Set error in Sensor map
                let v = Err(format!("Invalid JSON pointer {}\n{:#?}", ptr, json));
                SOURCES.set(key, v);
                continue;
            };

            // Rewrite Zigbee2Mqtt Binary to a boolean before we validate
            // if the base value is not already a boolean
            if spec.kind == crate::device::ValueKind::Bool && !value.is_boolean() {
                let Some(on) = spec.meta.get("value_on") else {
                    let v = Err("meta value 'value_on' required for binary device".into());
                    SOURCES.set(key, v);
                    continue;
                };
                let Some(off) = spec.meta.get("value_off") else {
                    let v = Err("meta value 'value_off' required for binary device".into());
                    SOURCES.set(key, v);
                    continue;
                };

                if value == on {
                    value = &Value::Bool(true)
                } else if value == off {
                    value = &Value::Bool(false)
                }
            }

            let output = spec.validate(value);
            SOURCES.set(key, output);
        }
    }

    Ok(())
}
