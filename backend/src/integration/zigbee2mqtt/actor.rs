use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use rumqttc::Publish;
use tracing::error;

use crate::io::mqtt::{Mqtt, MqttServerInfo, Subscribe};

use crate::actor::prelude::*;

use super::Device;

#[derive(Debug)]
pub struct Zigbee2Mqtt {
    server_info: MqttServerInfo,
}

impl Actor for Zigbee2Mqtt {
    type Context = Context<Self>;
}

impl Zigbee2Mqtt {
    pub fn start(server_info: MqttServerInfo) -> Result<Addr<Zigbee2Mqtt>> {
        let mqtt = Mqtt::connect(server_info.clone());
        let addr = Zigbee2Mqtt { server_info }.start();

        let _ = mqtt.send(Subscribe(
            "zigbee2mqtt/bridge/devices".into(),
            addr.to_weak().into(),
        ))?;

        Ok(addr)
    }

    pub async fn sync_devices(&mut self, data: Bytes) -> Result<()> {
        let zigbee_devices: Vec<Device> = serde_json::from_slice(&data)?;

        crate::device::Device::integration_sync(
            &format!("zigbee2mqtt/{}:{}", self.server_info.host, self.server_info.port),
            zigbee_devices
                .into_iter()
                .filter_map(|d| d.into_device(&self.server_info)),
        ).await?;

        Ok(())
    }
}

#[async_trait]
impl Handler<Publish> for Zigbee2Mqtt {
    type Result = ();

    async fn handle(&mut self, message: Publish, _context: &mut Context<Self>) -> Self::Result {
        if let Err(e) = self.sync_devices(message.payload).await {
            error!("Zigbee2Mqtt sync failed with: {e:?}");
        }
    }
}
