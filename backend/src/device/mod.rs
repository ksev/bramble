mod automation;
mod device;
mod feature;
mod task_spec;

use std::sync::Arc;

use anyhow::Result;
use futures::{Stream, StreamExt, TryStreamExt};

use crate::{
    db,
    program::Program,
    task::Task,
    topic::static_topic,
    value::{self, ValueId},
};
pub use automation::Automation;
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

pub fn random_id(prefix: &str) -> String {
    let [a, b]: [u64; 2] = rand::random();
    format!("{prefix}:{a:x}{b:x}")
}

/// Restore all devices tasks on restart
pub async fn restore_task(task: Task) -> Result<()> {
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

async fn automation_task((mut program, deps): (Program, Vec<ValueId>), _: Task) -> Result<()> {
    if program.steps() == 0 {
        // Program does not do anything, no need for us to run
        return Ok(());
    }

    let mut vals = value::subscribe();

    // Fetch the current world view
    let mut input = deps
        .into_iter()
        .map(|vid| {
            let current = value::current(vid).value().clone().unwrap_or_default();

            (vid, current)
        })
        .collect();

    // Execute once on the availiable data
    program.execute(&input)?;

    while let Some((key, value)) = vals.next().await {
        // Make sure its a value we care about in this Automation
        if input.contains_key(&key) {
            // We keep track of the input values into the program away from the global value store
            // to make sure we have stable values for the entire execution and so we dont miss an intermediate value
            input.insert(key, value.unwrap_or_default());

            // Execute the program
            for (k, v) in program.execute(&input)? {
                // Push program outputs
                value::push(k, v);
            }
        }
    }

    Ok(())
}
