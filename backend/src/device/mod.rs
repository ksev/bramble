mod automation;
mod device;
mod feature;
mod sun;
mod task_spec;

use std::sync::Arc;

use anyhow::Result;
use futures::{Stream, StreamExt, TryStreamExt};
use serde_json::json;
use time::{Duration, OffsetDateTime};
use tracing::debug;

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

use self::sun::SunPhase;

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
        TaskSpec::Sun { lat, lon } => task.spawn_with_argument("thesun", (*lat, *lon), the_sun),
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
    for (k, v) in program.execute(&input)? {
        // Push program outputs
        value::push(k, v);
    }

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

async fn the_sun((lat, lon): (f64, f64), _: Task) -> Result<()> {
    let state_id = ValueId::new("thesun", "state");
    let up_id = ValueId::new("thesun", "up");
    let height = 0.0;

    loop {
        let off = OffsetDateTime::now_utc();

        let set_start = sun::time_at_phase(off, SunPhase::SunsetStart, lat, lon, height);
        let set = sun::time_at_phase(off, SunPhase::Sunset, lat, lon, height);

        let rise = sun::time_at_phase(off, SunPhase::Sunrise, lat, lon, height);
        let rise_end = sun::time_at_phase(off, SunPhase::SunriseEnd, lat, lon, height);

        let up = off >= rise && off <= set;
        let in_rise = off >= rise && off <= rise_end;
        let in_set = off >= set_start && off <= set;

        let next = [set, set_start, rise, rise_end]
            .into_iter()
            .filter(|&d| d >= off)
            .min()
            .unwrap_or(rise + Duration::hours(22))
            - off;

        let state = match (up, in_rise, in_set) {
            (_, true, _) => "sunrise",
            (_, _, true) => "sunset",
            (true, _, _) => "day",
            (false, _, _) => "night",
        };

        value::set_current(state_id, Ok(json!(state)));
        value::set_current(up_id, Ok(json!(up)));

        tokio::time::sleep(next.try_into()?).await;
    }
}
