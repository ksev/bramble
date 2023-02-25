mod device;
mod feature;
mod source;
mod task_spec;

use std::sync::Arc;

use anyhow::Result;
use futures::TryStreamExt;

use crate::{bus::Topic, task::Task};
pub use device::*;
pub use feature::*;
pub use source::Sources;
pub use task_spec::*;

#[derive(Default)]
pub struct DeviceBus {
    /// Publish on this topic to subscribe to a MQTT topic on the specified server
    pub add: Topic<Arc<Device>>,
    pub value: Topic<(String, String, FeatureValue)>,
}

/// Restore all devices tasks on restart
pub async fn restore(task: Task) -> Result<()> {
    let mut conn = task.db.acquire().await?;
    let mut devices = Device::all(&mut conn);

    while let Some(device) = devices.try_next().await? {
        spawn_device_tasks(&task, &device);
    }

    Ok(())
}

pub fn spawn_device_tasks(task: &Task, device: &Device) {
    for spec in &device.task_spec {
        match spec {
            TaskSpec::Zigbee2Mqtt(server) => {
                let label = format!("zigbee2mqtt:{}:{}", server.host, server.port);

                if task.has_task(&label) {
                    // There is no need to reboot the task just ignore
                    continue;
                }

                task.spawn_with_argument(
                    label,
                    (device.id.clone(), server.clone()),
                    crate::integration::zigbee2mqtt::zigbee2mqtt_update,
                );
            }
            TaskSpec::Zigbee2MqttDevice(_) => {
                let label = format!("{}/Zigbee2MqttDevice", device.id);

                task.spawn_with_argument(
                    label,
                    device.id.clone(),
                    crate::integration::zigbee2mqtt::zigbee2mqtt_device,
                )
            }
        }
    }
}

pub fn spawn_automation_task(task: &Task, device_id: &str, feature: &Feature) -> Result<()> {
    let Some(automation) = &feature.automate else {
            anyhow::bail!("Feature ({}, {}) has no automation to spawn", device_id, feature.id);
        };

    let program = automation.compile(device_id, &feature.id)?;
    let label = format!("{}/{}/automate", device_id, feature.id);

    Ok(())
}
