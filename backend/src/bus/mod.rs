mod subscriber;
mod topic;

use once_cell::sync::Lazy;

use subscriber::*;
pub use topic::*;

use crate::{device::DeviceBus, io::mqtt::MqttBus};

#[derive(Default)]
pub struct GlobalBus {
    pub device: DeviceBus,
    pub mqtt: MqttBus,
}

pub static BUS: Lazy<GlobalBus> = Lazy::new(GlobalBus::default);
