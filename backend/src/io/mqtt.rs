use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use bytes::Bytes;
use futures::StreamExt;
use rumqttc::{AsyncClient, ConnAck, Event, EventLoop, MqttOptions, Packet};
use serde_derive::{Deserialize, Serialize};
use tokio::time::timeout;

use crate::{
    bus::{Topic, BUS},
    task::Task,
};

#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
pub struct MqttServerInfo {
    pub host: String,
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

impl MqttServerInfo {
    /// Create a new MqttServerInfo struct
    pub fn new(
        host: String,
        port: u16,
        username: Option<String>,
        password: Option<String>,
    ) -> MqttServerInfo {
        MqttServerInfo {
            host,
            port,
            username,
            password,
        }
    }
}

impl std::cmp::PartialEq for MqttServerInfo {
    fn eq(&self, other: &Self) -> bool {
        self.host == other.host && self.port == other.port
    }
}

impl std::hash::Hash for MqttServerInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.host.hash(state);
        self.port.hash(state);
    }
}

const MAX_PACKET_SIZE: usize = 2 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct MqttSubscribe {
    pub topic: String,
    pub server: MqttServerInfo,
}

#[derive(Default)]
pub struct MqttBus {
    /// Publish on this topic to subscribe to a MQTT topic on the specified server
    pub subscribe: Topic<MqttSubscribe>,
    pub published: Topic<(String, Bytes)>,
}

/**
 * A task to manage connection going out to many different MQTT server, we connect and subscribe to the first topic at the same time
 */
pub async fn manage_connections(_: Task) -> Result<()> {
    let mut connections: HashMap<MqttServerInfo, AsyncClient> = HashMap::new();
    let mut channel = BUS.mqtt.subscribe.subscribe();

    while let Some(sub) = channel.next().await {
        let key = sub.server.clone();

        if let Some(client) = connections.get_mut(&key) {
            client
                .subscribe(&sub.topic, rumqttc::QoS::AtLeastOnce)
                .await?;
        } else {
            let (client, mut eventloop) = connect(&sub.server).await?;

            tokio::spawn(async move {
                while let Ok(notification) = eventloop.poll().await {
                    if let Event::Incoming(Packet::Publish(p)) = notification {
                        BUS.mqtt.published.publish((p.topic, p.payload));
                    }
                }

                println!("mqtt background died");
            });

            client
                .subscribe(&sub.topic, rumqttc::QoS::AtLeastOnce)
                .await?;

            connections.insert(key, client);
        }
    }

    Ok(())
}

async fn connect(server_info: &MqttServerInfo) -> Result<(AsyncClient, EventLoop)> {
    let client_id = &format!("rome-{}", env!("CARGO_PKG_VERSION"));

    let mut mqttoptions = MqttOptions::new(client_id, &server_info.host, server_info.port);
    mqttoptions.set_keep_alive(Duration::from_secs(30));
    mqttoptions.set_max_packet_size(MAX_PACKET_SIZE, MAX_PACKET_SIZE);

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // Look for ConnAck, but only for so long
    timeout(Duration::from_secs(60), async {
        loop {
            // Make sure we are connected before we start the actor
            if let Event::Incoming(Packet::ConnAck(ConnAck { .. })) = eventloop.poll().await? {
                return anyhow::Ok(());
            }
        }
    })
    .await??;

    Ok((client, eventloop))
}
