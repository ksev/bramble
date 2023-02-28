mod device;

use std::{future, sync::Arc};

use anyhow::Result;

use bytes::Buf;
pub use device::*;
use futures::{StreamExt, TryStreamExt};
use futures_concurrency::future::Join;
use serde_json::Value as Json;
use sqlx::SqliteConnection;
use tracing::{error, warn};

use crate::{
    db,
    device::{spawn_device_tasks, TaskSpec, ValueKind},
    io::mqtt::{self, MqttServerInfo, MqttTopic},
    strings::IString,
    task::Task,
    value::{self, ValueId},
};

pub async fn create_integration_device(
    server_info: MqttServerInfo,
    task: &Task,
    conn: &mut SqliteConnection,
) -> Result<Arc<crate::device::Device>> {
    let device = crate::device::Device {
        id: format!("z2mqtt:{}:{}", server_info.host, server_info.port),
        name: format!("Zigbee2Mqtt ({}:{})", server_info.host, server_info.port),
        device_type: crate::device::DeviceType::Integration {
            name: "zigbee2mqtt".into(),
        },
        parent: None,
        task_spec: TaskSpec::Zigbee2Mqtt(server_info),
    };

    device.save(conn).await?;

    spawn_device_tasks(task, &device);

    let device = Arc::new(device);

    crate::device::notify_changed(device.clone());

    Ok(device)
}

pub async fn zigbee2mqtt_update(
    (parent, server): (String, MqttServerInfo),
    task: Task,
) -> Result<()> {
    let mut channel = mqtt::incoming();

    mqtt::subscribe(MqttTopic {
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

            for (device, features) in iter {
                let mut tx = db::begin().await?;

                // Persist the device
                device.save(&mut tx).await?;

                for feature in features {
                    feature.save(&device.id, &mut tx).await?;
                }

                tx.commit().await?;

                spawn_device_tasks(&task, &device);

                crate::device::notify_changed(device);
            }
        }
    }

    Ok(())
}

pub async fn zigbee2mqtt_device(device_id: IString, _: Task) -> Result<()> {
    // Get the devices and their features from the database
    let (device, features) = {
        use crate::device::{Device, Feature};

        let mut conn = db::connection().await?;
        let device = Device::load_by_id(device_id.into(), &mut conn).await?;
        let features: Vec<Feature> = Feature::load_by_device_readable(&device.id, &mut conn)
            .try_collect()
            .await?;

        (device, features)
    };

    use TaskSpec::Zigbee2MqttDevice;

    let Zigbee2MqttDevice(subscribe) = device.task_spec else {
        anyhow::bail!("zigbee2mqtt_device did not get a task spec it expected, this is a bug");
    };

    // Subscribe to all messages we receive from MQTT servers
    let incoming = async {
        // Tell the MQTT worker that we want to subscribe to a server and topic
        mqtt::subscribe(subscribe.clone());

        let mut incoming = mqtt::incoming();

        while let Some((topic, data)) = incoming.next().await {
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
                let key = ValueId::new(device_id, &spec.id);

                let Some(mut value) = json.pointer(ptr) else {
                        // Set error in Sensor map
                        let v = Err(format!("Invalid JSON pointer {}\n{:#?}", ptr, json));
                        value::set_current(key, v);
                        continue;
                    };

                // Rewrite Zigbee2Mqtt Binary to a boolean before we validate
                // if the base value is not already a boolean
                if spec.kind == ValueKind::Bool && !value.is_boolean() {
                    let Some(on) = spec.meta.get("value_on") else {
                            let v = Err("meta value 'value_on' required for binary device".into());
                        value::set_current(key, v);
                            continue;
                        };

                    let Some(off) = spec.meta.get("value_off") else {
                            let v = Err("meta value 'value_off' required for binary device".into());
                        value::set_current(key, v);
                            continue;
                        };

                    if value == on {
                        value = &Json::Bool(true)
                    } else if value == off {
                        value = &Json::Bool(false)
                    }
                }

                let output = spec.validate(value);
                value::set_current(key, output);
            }
        }

        Ok(())
    };

    let outgoing = async {
        // Subscribe to value push
        let mut outgoing =
            value::push_subscribe().filter(|(id, _)| future::ready(id.device == device_id));

        while let Some((id, mut value)) = outgoing.next().await {
            let fid: &str = id.feature.into();

            let feature = features.iter().find(|f| f.id == fid);

            if let Some(spec) = feature {
                // Rewrite Zigbee2Mqtt Binary to a boolean before we validate
                // if the base value is not already a boolean
                if spec.kind == ValueKind::Bool {
                    let Some(on) = spec.meta.get("value_on") else {
                            // let v = FeatureValue::Err("meta value 'value_on' required for binary device".into());
                            continue;
                        };

                    let Some(off) = spec.meta.get("value_off") else {
                            // let v = FeatureValue::Err("meta value 'value_off' required for binary device".into());
                            continue;
                        };

                    if value == Json::Bool(true) {
                        value = on.clone();
                    } else {
                        value = off.clone();
                    }
                }

                let mut o = serde_json::Map::new();
                o.insert(spec.id.clone(), value);

                let payload = serde_json::Value::Object(o);
                let bytes = serde_json::ser::to_vec(&payload)?;

                let sub = MqttTopic {
                    topic: format!("zigbee2mqtt/{}/set", device.id),
                    server: subscribe.server.clone(),
                };

                mqtt::publish(sub, bytes);
            } else {
                warn!("Coulid not find feature on mqtt push");
            }
        }

        Ok(())
    };

    let _: (Result<()>, Result<()>) = (incoming, outgoing).join().await;

    Ok(())
}
