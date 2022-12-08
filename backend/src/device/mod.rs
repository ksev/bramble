mod device;
mod task_spec;
mod source;

use std::sync::Arc;

use anyhow::Result;
use futures::StreamExt;

pub use device::*;
pub use task_spec::*;
pub use source::SOURCES;
use crate::{bus::{BUS, Topic}, task::Task};

#[derive(Default)]
pub struct DeviceBus {
    /// Publish on this topic to subscribe to a MQTT topic on the specified server
    pub add: Topic<Arc<Device>>,
    pub value: Topic<(String, String, Result<serde_json::Value, String>)>,
}

/// Listen for devices to be added on the BUS, when that happens spawn the device's specified tasks
pub async fn add_device(mut t: Task) -> Result<()> {
    let mut channel = BUS.device.add.subscribe();

    // At startup we start all the tasks for saved devices
    for device in Device::all()? {
        // TODO: Maybe not allocate here?
        Arc::new(device).spawn_tasks(&mut t).await;
    }

    while let Some(device) = channel.next().await {
        // Persist the device
        device.save()?;
        // Spawn it's tasks
        device.spawn_tasks(&mut t).await;
    }

    Ok(())
}
