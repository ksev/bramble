mod device;
mod feature;
mod task_spec;

use std::{collections::BTreeSet, sync::Arc};

use anyhow::Result;
use futures::{Stream, StreamExt, TryStreamExt};

use crate::{
    automation::{Automation, Program},
    db,
    task::Task,
    topic::static_topic,
    value::{self, ValueId},
};
pub use device::*;
pub use feature::*;
pub use task_spec::*;

static_topic!(CHANGED, Arc<Device>);

pub fn notify_changed<T>(device: T)
where
    T: Into<Arc<Device>>,
{
    CHANGED.publish(device.into());
}

pub fn changed() -> impl Stream<Item = Arc<Device>> {
    CHANGED.subscribe()
}

/// Restore all devices tasks on restart
pub async fn restore(task: Task) -> Result<()> {
    let mut conn = db::connection().await?;

    {
        let mut devices = Device::all(&mut conn);
        while let Some(device) = devices.try_next().await? {
            spawn_device_tasks(&task, &device);
        }
    }

    {
        let mut programs = Feature::load_automations(&mut conn);

        while let Some((device_id, feature_id, automation)) = programs.try_next().await? {
            let target = ValueId::new(&device_id, &feature_id);
            spawn_automation_task(&task, target, &automation)?;
        }
    }

    Ok(())
}

pub async fn automation_task((mut program, deps): (Program, Vec<ValueId>), _: Task) -> Result<()> {
    let mut vals = value::subscribe();
    let deps: BTreeSet<_> = deps.into_iter().collect();

    while let Some((key, _)) = vals.next().await {
        if deps.contains(&key) {
            program.execute()?;
        }
    }

    Ok(())
}

pub fn spawn_device_tasks(task: &Task, device: &Device) {
    match &device.task_spec {
        TaskSpec::Zigbee2Mqtt(server) => {
            let label = format!("zigbee2mqtt:{}:{}", server.host, server.port);

            if task.has_task(&label) {
                // There is no need to reboot the task just ignore
                return;
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
            );
        }
        TaskSpec::NoOp => {}
    }
}

pub fn spawn_automation_task(task: &Task, target: ValueId, automation: &Automation) -> Result<()> {
    let package = automation.compile(target)?;
    let label = format!("{:?}/{:?}/automate", target.device, target.feature);

    task.spawn_with_argument(label, package, automation_task);

    Ok(())
}
