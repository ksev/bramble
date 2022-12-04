mod device;
mod task_spec;
mod source;

use std::sync::Arc;

use anyhow::Result;

pub use device::*;
pub use task_spec::*;
pub use source::SOURCES;
use crate::{bus::{BUS, Topic}, task::Task};

#[derive(Default)]
pub struct DeviceBus {
    /// Publish on this topic to subscribe to a MQTT topic on the specified server
    pub add: Topic<Arc<Device>>,
}

/// Listen for devices to be added on the BUS, when that happens spawn the device's specified tasks
pub async fn add_device(mut t: Task) -> Result<()> {
    let mut channel = BUS.device.add.subscribe();

    loop {
        let device = channel.recv().await;
        device.spawn_tasks(&mut t).await;
    }
}
