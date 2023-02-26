mod device;
mod feature;
mod source;
mod task_spec;

use std::{collections::BTreeSet, sync::Arc};

use anyhow::Result;
use futures::{StreamExt, TryStreamExt};

use crate::{
    automation::{Automation, Program},
    bus::Topic,
    task::Task,
};
pub use device::*;
pub use feature::*;
pub use source::{SourceId, Sources};
pub use task_spec::*;

#[derive(Default)]
pub struct DeviceBus {
    /// Publish on this topic to subscribe to a MQTT topic on the specified server
    pub add: Topic<Arc<Device>>,
    pub value: Topic<(SourceId, FeatureValue)>,
}

/// Restore all devices tasks on restart
pub async fn restore(task: Task) -> Result<()> {
    let mut conn = task.db.acquire().await?;

    {
        let mut devices = Device::all(&mut conn);
        while let Some(device) = devices.try_next().await? {
            spawn_device_tasks(&task, &device);
        }
    }

    {
        let mut programs = Feature::load_automations(&mut conn);

        while let Some((device_id, feature_id, automation)) = programs.try_next().await? {
            let target = SourceId::new(&device_id, &feature_id);
            spawn_automation_task(&task, target, &automation)?;
        }
    }

    Ok(())
}

pub async fn automation_task(
    (mut program, deps): (Program, Vec<SourceId>),
    task: Task,
) -> Result<()> {
    let mut vals = task.bus.device.value.subscribe();

    let chan = &task.bus.device.value;
    let sources = &task.sources;
    let deps: BTreeSet<_> = deps.into_iter().collect();

    while let Some((key, _)) = vals.next().await {
        if deps.contains(&key) {
            program.execute(chan, sources)?;
        }
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
                    (&device.id).into(),
                    crate::integration::zigbee2mqtt::zigbee2mqtt_device,
                )
            }
        }
    }
}

pub fn spawn_automation_task(task: &Task, target: SourceId, automation: &Automation) -> Result<()> {
    let package = automation.compile(target)?;
    let label = format!("{:?}/{:?}/automate", target.device, target.feature);

    task.spawn_with_argument(label, package, automation_task);

    Ok(())
}
