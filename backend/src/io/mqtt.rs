use std::{collections::BTreeMap, time::Duration};

use anyhow::Result;
use async_trait::async_trait;
use dashmap::DashMap;
use once_cell::sync::Lazy;

use rumqttc::{
    AsyncClient, ConnectionError, Event, MqttOptions, Packet, Publish, QoS,
    SubscribeFilter, ConnectReturnCode,
};

use serde::{Deserialize, Serialize};

use tracing::{debug, error};

use crate::actor::{prelude::*, WeakMessageAddr};

const MAX_PACKET_SIZE: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, Eq)]
pub struct MqttServerInfo {
    pub host: String,
    pub port: u16,
}

impl MqttServerInfo {
    pub fn new(host: impl Into<String>, port: u16) -> MqttServerInfo {
        MqttServerInfo {
            host: host.into(),
            port,
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

static CONNECTIONS: Lazy<DashMap<MqttServerInfo, Addr<Mqtt>>> = Lazy::new(DashMap::new);

#[derive(Debug)]
pub struct Subscribe(pub String, pub WeakMessageAddr<Publish>);

pub struct Mqtt {
    client: AsyncClient,
    server_info: MqttServerInfo,
    subscribers: BTreeMap<String, Vec<WeakMessageAddr<Publish>>>,
    waiting_subscribes: Vec<String>,
    can_subscribe: bool,
    connected: bool,
}

impl Mqtt {
    /// Connecto to mqtt server, connection actors are unique per host:port
    pub fn connect(server_info: MqttServerInfo) -> Addr<Mqtt> {
        CONNECTIONS
            .entry(server_info.clone())
            .or_insert_with(|| {
                let client_id = &format!("rome-{}", env!("CARGO_PKG_VERSION"));

                let mut mqttoptions =
                    MqttOptions::new(client_id, &server_info.host, server_info.port);
                mqttoptions.set_keep_alive(Duration::from_secs(30));
                mqttoptions.set_max_packet_size(MAX_PACKET_SIZE, MAX_PACKET_SIZE);

                let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

                let addr = Mqtt {
                    client,
                    server_info: server_info.clone(),
                    subscribers: Default::default(),
                    waiting_subscribes: vec![],
                    can_subscribe: true,
                    connected: false,
                }
                .start();

                // Make sure our little event loop task does not pin the task alive
                let adapter_addr = addr.to_weak();
                let backend_info = server_info.clone();

                tokio::spawn(async move {
                    'main: loop {
                        let event = eventloop.poll().await;
                        match adapter_addr.send(event) {
                            Ok(event) => event.await,
                            Err(e) => {
                                error!("event loop error {e:?}");
                                break 'main;
                            }
                        }
                    }

                    debug!("{:?} event loop stopped", backend_info);
                });

                debug!("{:?} started", server_info);

                addr
            })
            .clone()
    }
}

impl Actor for Mqtt {
    type Context = Context<Self>;
}

#[async_trait]
impl Handler<Result<Event, ConnectionError>> for Mqtt {
    type Result = ();

    async fn handle(
        &mut self,
        message: Result<Event, ConnectionError>,
        _context: &mut Context<Self>,
    ) -> Self::Result {
        use Event::*;

        match message {
            Err(e) => {
                debug!("{:?} error {e:?}", self.server_info);
                CONNECTIONS.remove(&self.server_info);
                // TODO: Maybe ignore some errors
            }
            Ok(event) => {
                debug!("{event:?}");

                match event {
                    Incoming(Packet::Publish(publish)) => {
                        if let Some(subscribers) = self.subscribers.get_mut(&publish.topic) {
                            subscribers.retain(|addr| addr.send(publish.clone()).is_ok());

                            if subscribers.is_empty() {
                                // Best effort unsubscribe
                                debug!("mqtt unsubscribe on client");
                                self.client.unsubscribe(&publish.topic).await.ok();
                            }
                        }
                    }
                    Incoming(Packet::ConnAck(ack)) if ack.code == ConnectReturnCode::Success => {
                        self.connected = true;

                        if !self.waiting_subscribes.is_empty() {
                            self.client
                                .subscribe_many(self.waiting_subscribes.drain(..).map(|s| {
                                    SubscribeFilter {
                                        path: s,
                                        qos: QoS::AtLeastOnce,
                                    }
                                }))
                                .await
                                .ok();
                        }
                    }
                    Incoming(Packet::SubAck { .. }) => {
                        if !self.waiting_subscribes.is_empty() {
                            self.client
                                .subscribe_many(self.waiting_subscribes.drain(..).map(|s| {
                                    SubscribeFilter {
                                        path: s,
                                        qos: QoS::AtLeastOnce,
                                    }
                                }))
                                .await
                                .ok();
                        } else {
                            self.can_subscribe = true;
                        }
                    }

                    _ => {}
                }
            }
        }
    }
}

#[async_trait]
impl Handler<Subscribe> for Mqtt {
    type Result = Result<()>;

    async fn handle(
        &mut self,
        Subscribe(topic, addr): Subscribe,
        _context: &mut Context<Self>,
    ) -> Self::Result {
        let list = self.subscribers.entry(topic.clone()).or_insert(vec![]);
        list.push(addr);

        if self.can_subscribe && self.connected {
            // Call subscribe every time to pump data from the topic
            self.client.subscribe(topic, QoS::AtLeastOnce).await?;
            self.can_subscribe = false;
        } else {
            self.waiting_subscribes.push(topic);
        }

        Ok(())
    }
}
