use anyhow::Result;
use bonsaidb::core::{
    document::CollectionDocument,
    schema::{Collection, SerializedCollection},
};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{actor::Addr, database, io::mqtt::MqttServerInfo};

use super::actor::Zigbee2Mqtt;

static SERVER_TASK: Lazy<DashMap<MqttServerInfo, Addr<Zigbee2Mqtt>>> = Lazy::new(DashMap::new);

#[derive(Debug, Serialize, Deserialize, Collection, Clone)]
#[collection(name = "zigbee2mqttServers", primary_key = (String, u16), natural_id = |s: &Server| Some((s.info.host.clone(), s.info.port)))]
pub struct Server {
    pub info: MqttServerInfo,
    pub devices: u32,
}

impl Server {
    /// Save server to database, this will handle upserts if needed
    pub async fn save(self) -> Result<CollectionDocument<Self>> {
        let conn = database::connection().await;

        if let Some(mut existing) = Server::by_server_info(&self.info).await? {
            existing.contents.devices = self.devices;
            existing.update_async(conn).await?;

            Ok(existing)
        } else {
            Ok(self.push_into_async(conn).await?)
        }
    }

    pub async fn by_server_info(
        server_info: &MqttServerInfo,
    ) -> Result<Option<CollectionDocument<Self>>> {
        let conn = database::connection().await;
        Ok(Server::get_async((server_info.host.clone(), server_info.port), conn).await?)
    }

    /// Load state from database end start the required tasks and connection to start the service
    pub async fn hydrate() -> Result<()> {
        let conn = database::connection().await;

        for document in Server::all_async(conn).await? {
            let Server { info, .. } = document.contents;
            Server::connect(info).await?;
        }

        Ok(())
    }

    /*
    /// Create a Server instance or update and update devices based on a Zigbee2Mqtt devices publish
    async fn from_zigbee2mqtt_publish(
        server_info: MqttServerInfo,
        publish: Publish,
    ) -> Result<CollectionDocument<Self>> {
        let zigbee_devices: Vec<Device> = serde_json::from_slice(&publish.payload)?;
        let integration_id = format!("zigbee2mqtt/{}:{}", server_info.host, server_info.port);

        let server = Server {
            info: server_info.clone(),
            devices: zigbee_devices.len() as u32,
        };

        let out = server.save().await?;

        device::Device::integration_add(
            &integration_id,
            zigbee_devices
                .iter()
                .filter_map(|d| d.to_device(&server_info)),
        )
        .await?;

        Ok(out)
    }
    */
}

impl Server {
    pub async fn connect(server_info: MqttServerInfo) -> Result<Server> {
        SERVER_TASK
            .entry(server_info.clone())
            .or_try_insert_with(|| Zigbee2Mqtt::start(server_info.clone()))?;

        Ok(Server {
            info: server_info,
            devices: 10,
        })

        /*
        let key = format!("zigbee2mqtt/{}:{}", server_info.host, server_info.port);
        let _task_handle = work::start_job(key, |_rx: UnboundedReceiver<()>, close_shot| async {
            let mqtt = Mqtt::open(server_info.clone()).await?;
            let mut receiver = mqtt.subscribe("zigbee2mqtt/bridge/devices").await?;

            let publish = timeout(Duration::from_secs(30), receiver.recv())
                .await?
                .ok_or_else(|| anyhow::anyhow!("closed internal pipe"))?;

            Server::from_zigbee2mqtt_publish(server_info.clone(), publish).await?;

            Ok(start_stuff(server_info.clone(), receiver, close_shot))
        })
        .await?;

        let server = Server::by_server_info(&server_info)
            .await?
            .ok_or_else(|| anyhow!("server did not exist in database even though it should"))?;

        Ok(server.contents)
         */
    }
}
