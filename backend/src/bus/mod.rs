mod subscriber;
mod topic;

use std::sync::Arc;

use once_cell::sync::Lazy;
use anyhow::Result;

use subscriber::*;
pub use topic::*;

use crate::{device::DeviceBus, io::mqtt::MqttBus};

#[derive(Default)]
pub struct GlobalBus {
    pub device: DeviceBus,
    pub mqtt: MqttBus,
}

impl GlobalBus {
    /// Publish on the bus using a named topic and a JSON (in string form) payload,
    /// Will fail if topic is unknown and JSON parse fails
    pub fn publish_dynamic(&self, topic: &str, payload: serde_json::Value) -> Result<()> {
        match topic {
            "device.add" => {
                let payload = serde_json::from_value(payload)?;
                BUS.device.add.publish(Arc::new(payload))
            },
            _ => anyhow::bail!("No such topic {topic}"),
        }

        Ok(())
    }

}

pub static BUS: Lazy<GlobalBus> = Lazy::new(GlobalBus::default);
