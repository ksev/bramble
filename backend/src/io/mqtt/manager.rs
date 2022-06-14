use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

use crate::actor::prelude::*;

use super::{connection::MqttConnection, MqttServerInfo};

/// MqttServerManager is an actor that runs as a singleton
/// and manages connections going to mqtt server to make sure we only spawn on connection
/// per server and reuse that between tasks, it also re-creates the connection on failure
pub struct MqttServerManager {
    connections: HashMap<MqttServerInfo, Addr<MqttConnection>>,
}

impl MqttServerManager {
    pub (crate) fn new() -> MqttServerManager {
        MqttServerManager { connections: HashMap::new() }
    }
}

impl Actor for MqttServerManager {
    type Context = Context<Self>;
}

pub struct Connect(pub MqttServerInfo);

#[async_trait]
impl Handler<Connect> for MqttServerManager {
    type Result = Result<Addr<MqttConnection>>;

    async fn handle(&mut self, Connect(info): Connect, _context: &mut Context<Self>) -> Self::Result {
        if let Some(addr) = self.connections.get(&info) {
            return Ok(addr.clone())
        }

        let addr = MqttConnection::open(info.clone()).await?;
        self.connections.insert(info, addr.clone());

        Ok(addr)
    }
}

