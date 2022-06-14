use anyhow::Result;
use once_cell::sync::Lazy;

use crate::actor::prelude::*;

use super::{
    connection::MqttConn,
    manager::{Connect, MqttServerManager},
    MqttServerInfo,
};

static MANAGER: Lazy<Addr<MqttServerManager>> = Lazy::new(|| MqttServerManager::new().start());

/// API wrapper for [MqttServerManager]
pub struct Mqtt;

impl Mqtt {
    /// Connect to a Mqtt server, if the connection already exists return a reference to it instead
    pub async fn connect(server_info: MqttServerInfo) -> Result<MqttConn> {
        Ok(MANAGER.ask(Connect(server_info))?.await?.into())
    }
}
