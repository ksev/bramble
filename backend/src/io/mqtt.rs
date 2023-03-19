use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use rumqttc::{AsyncClient, ConnAck, Event, EventLoop, MqttOptions, Packet};
use serde_derive::{Deserialize, Serialize};
use tokio::time::timeout;
use tracing::warn;

use crate::{task::Task, topic::static_topic};

static_topic!(CLIENTBUS, ClientAction);
static_topic!(INCOMING, (String, Bytes));

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
pub struct MqttTopic {
    pub topic: String,
    pub server: MqttServerInfo,
}

#[derive(Clone)]
enum ClientAction {
    S(MqttTopic),
    P(MqttTopic, Vec<u8>),
}

pub fn subscribe(topic: MqttTopic) {
    CLIENTBUS.publish(ClientAction::S(topic));
}

pub fn publish(topic: MqttTopic, value: Vec<u8>) {
    CLIENTBUS.publish(ClientAction::P(topic, value));
}

pub fn incoming() -> impl Stream<Item = (String, Bytes)> {
    INCOMING.subscribe()
}

/**
 * A task to manage connection going out to many different MQTT server, we connect and subscribe to the first topic at the same time
 */
pub async fn manage_connections(_: Task) -> Result<()> {
    let mut connections: HashMap<MqttServerInfo, AsyncClient> = HashMap::new();

    let mut actions = CLIENTBUS.subscribe();

    while let Some(r) = actions.next().await {
        use ClientAction::*;

        match r {
            S(sub) => {
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
                                INCOMING.publish((p.topic, p.payload));
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
            P(server, bytes) => {
                if let Some(client) = connections.get(&server.server) {
                    client
                        .publish(&server.topic, rumqttc::QoS::AtLeastOnce, false, bytes)
                        .await?;
                } else {
                    warn!("Tried to push to mqtt server we are not connected to, fix this");
                }
            }
        }
    }

    Ok(())
}

async fn connect(server_info: &MqttServerInfo) -> Result<(AsyncClient, EventLoop)> {
    let [a, b]: [u64; 2] = rand::random();
    let client_id = format!("bramble-{}-{}{}", env!("CARGO_PKG_VERSION"), a, b);

    let mut mqttoptions = MqttOptions::new(&client_id, &server_info.host, server_info.port);
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
