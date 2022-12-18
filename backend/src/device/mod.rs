mod device;
mod source;
mod task_spec;
mod feature;

use std::sync::Arc;

use anyhow::Result;
use futures::TryStreamExt;

use crate::{bus::Topic, db, task::Task};
pub use device::*;
pub use source::SOURCES;
pub use task_spec::*;
pub use feature::*;

#[derive(Default)]
pub struct DeviceBus {
    /// Publish on this topic to subscribe to a MQTT topic on the specified server
    pub add: Topic<Arc<Device>>,
    pub value: Topic<(String, String, Result<serde_json::Value, String>)>,
}

/// Restore all devices tasks on restart
pub async fn restore(mut t: Task) -> Result<()> {
    let conn = db::pool().await;
    let mut devices = Device::all(conn);

    while let Some(device) = devices.try_next().await? {
        device.spawn_tasks(&mut t);
    }

    Ok(())
}
