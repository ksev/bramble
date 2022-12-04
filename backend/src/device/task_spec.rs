use serde_derive::{Deserialize, Serialize};

use crate::{
    io::mqtt::{MqttServerInfo, MqttSubscribe},
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TaskSpec {
    #[serde(rename = "zigbee2Mqtt")]
    Zigbee2Mqtt(MqttServerInfo),
    #[serde(rename = "zigbee2MqttDevice")]
    Zigbee2MqttDevice(MqttSubscribe),
}
