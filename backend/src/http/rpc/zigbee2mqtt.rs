use anyhow::Result;
use async_trait::async_trait;
use bonsaidb::core::schema::SerializedCollection;

use crate::actor::{Task, Context, Pid};

use super::protocol::{
    Void, Zigbee2MqttConfig, Zigbee2MqttServer, Zigbee2MqttServers, Zigbee2MqttService,
};
//use crate::{database, integration::zigbee2mqtt::*, };

#[derive(Default)]
pub struct Service;

#[async_trait]
impl Zigbee2MqttService for Service {
    async fn config(ctx: Task, input: Zigbee2MqttConfig) -> Result<Zigbee2MqttServer> {
    
        /*
        let info = MqttServerInfo::new(input.url, 1883);

        if Server::by_server_info(&info).await?.is_some() {
            anyhow::bail!(
                "Zigbee2Mqtt integration on {}:{} already exists",
                info.host,
                info.port
            );
        }

        let Server { info, devices } = Server::connect(info).await?;
        let MqttServerInfo { host, port } = info;

        Ok(Zigbee2MqttServer {
            host,
            port: port as u32,
            devices: devices as u32,
        })
         */
        todo!();
    }

    async fn status(ctx: Task, _: Void) -> Result<Zigbee2MqttServers> {
        /* 
        let db = database::connection().await;

        let servers = Server::all_async(db)
            .await?
            .into_iter()
            .map(|document| Zigbee2MqttServer {
                host: document.contents.info.host,
                port: document.contents.info.port as u32,
                devices: document.contents.devices as u32,
            })
            .collect();

        Ok(Zigbee2MqttServers { servers })
         */
        todo!();
    }
}
