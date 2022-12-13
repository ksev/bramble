//mod server;
mod device;

use std::sync::Arc;

use anyhow::Result;

use bytes::Buf;
//pub use server::*;
pub use device::*;
use futures::StreamExt;
use serde_json::Value;
use tracing::error;

use crate::{
    bus::BUS,
    device::{TaskSpec, SOURCES},
    io::mqtt::{MqttServerInfo, MqttSubscribe},
    task::Task,
};

pub async fn zigbee2mqtt_update(server: MqttServerInfo, _: Task) -> Result<()> {
    let mut channel = BUS.mqtt.published.subscribe();

    BUS.mqtt.subscribe.publish(MqttSubscribe {
        topic: "zigbee2mqtt/bridge/devices".into(),
        server: server.clone(),
    });

    while let Some((topic, data)) = channel.next().await {
        if topic == "zigbee2mqtt/bridge/devices" {
            let reader = data.reader();

            let device_spec: Vec<Device> =
                serde_json::de::from_reader(reader)?;

            let iter = device_spec
                .into_iter()
                .filter_map(|d| d.into_device(server.clone()));
                
            for device in iter {
                BUS.device.add.publish(Arc::new(device));
            }
        }
    }

    Ok(())
}

pub async fn zigbee2mqtt_device(device: Arc<crate::device::Device>, _: Task) -> Result<()> {
    use TaskSpec::Zigbee2MqttDevice;

    let mut channel = BUS.mqtt.published.subscribe();

    let Some(Zigbee2MqttDevice(subscribe)) = device.task_spec.get(0) else {
        anyhow::bail!("zigbee2mqtt_device did not get a task spec it expected, this is a bug");
    };

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

        for spec in device.features.iter().filter(|f| f.direction.can_read()) {
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

            let output = spec.kind.validate(value);
            SOURCES.set(key, output);
        }
    }

    Ok(())
}
