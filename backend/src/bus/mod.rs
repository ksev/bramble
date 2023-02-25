mod subscriber;
mod topic;

use subscriber::*;
pub use topic::*;

use crate::{device::DeviceBus, io::mqtt::MqttBus};

#[derive(Default)]
pub struct GlobalBus {
    pub device: DeviceBus,
    pub mqtt: MqttBus,
}
