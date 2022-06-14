use std::{collections::BTreeMap, time::Duration};

use anyhow::Result;
use async_trait::async_trait;
use rumqttc::{
    AsyncClient, ConnAck, ConnectionError, Event, MqttOptions, Packet, Publish, QoS,
    SubscribeFilter,
};
use tokio::time::timeout;
use tracing::debug;

use crate::actor::prelude::*;

use super::MqttServerInfo;

const MAX_PACKET_SIZE: usize = 2 * 1024 * 1024;

/// API Wrapper for [MqttConnection]
pub struct MqttConn(Addr<MqttConnection>);

impl MqttConn {
    /// Subscribe to Publish messages on a topic on the Mqtt server
    ///
    /// This will resubscribe for every topic to try and pump data out of retained topics
    pub fn subscribe(
        &self,
        topic: impl Into<String>,
        target: impl Into<MessageAddr<Publish>>,
    ) -> Result<()> {
        Ok(self.0.tell(Subscribe(topic.into(), target.into()))?)
    }
}

impl From<Addr<MqttConnection>> for MqttConn {
    fn from(addr: Addr<MqttConnection>) -> Self {
        MqttConn(addr)
    }
}

pub struct MqttConnection {
    client: AsyncClient,
    server_info: MqttServerInfo,
    subscribers: BTreeMap<String, Vec<MessageAddr<Publish>>>,
    waiting_subscribes: Vec<String>,
    can_subscribe: bool,
}

impl MqttConnection {
    /// Open a connection to an Mqtt server.
    ///
    /// Warning: If you are reading this, odds are you wan't ['Mqtt::connect'] instead.
    pub async fn open(server_info: MqttServerInfo) -> Result<Addr<MqttConnection>> {
        let client_id = &format!("rome-{}", env!("CARGO_PKG_VERSION"));

        let mut mqttoptions = MqttOptions::new(client_id, &server_info.host, server_info.port);
        mqttoptions.set_keep_alive(Duration::from_secs(30));
        mqttoptions.set_max_packet_size(MAX_PACKET_SIZE, MAX_PACKET_SIZE);

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

        // Look for ConnAck, but only for so long
        timeout(Duration::from_secs(60), async {
            loop {
                // Make sure we are connected before we start the actor
                if let Event::Incoming(Packet::ConnAck(ConnAck { code, .. })) =
                    eventloop.poll().await?
                {
                    return anyhow::Ok(());
                }
            }
        })
        .await??;

        let addr = MqttConnection {
            client,
            server_info: server_info.clone(),
            subscribers: Default::default(),
            waiting_subscribes: vec![],
            can_subscribe: true,
        }
        .start();

        // Make sure our little event loop task does not pin the task alive
        let adapter_addr = addr.clone();
        let backend_info = server_info.clone();

        tokio::spawn(async move {
            'main: loop {
                let event = eventloop.poll().await;
                match adapter_addr.tell(event) {
                    Ok(event) => {}
                    Err(e) => {
                        break 'main;
                    }
                }
            }

            debug!("{:?} event loop stopped", backend_info);
        });

        debug!("{:?} started", server_info);

        Ok(addr)
    }
}

#[async_trait]
impl Actor for MqttConnection {
    type Context = Context<Self>;

    async fn startup(&mut self, _context: &mut Context<Self>) -> Result<()> {
        println!("new connection!");
        Ok(())
    }
}

#[async_trait]
impl Handler<Result<Event, ConnectionError>> for MqttConnection {
    type Result = ();

    async fn handle(
        &mut self,
        message: Result<Event, ConnectionError>,
        context: &mut Context<Self>,
    ) -> Self::Result {
        use Event::*;

        match message {
            Err(e) => context.kill(e),
            Ok(event) => {
                debug!("{event:?}");

                match event {
                    Incoming(Packet::Publish(publish)) => {
                        if let Some(subscribers) = self.subscribers.get_mut(&publish.topic) {
                            subscribers.retain(|addr| addr.ask(publish.clone()).is_ok());

                            if subscribers.is_empty() {
                                // Best effort unsubscribe
                                debug!("mqtt unsubscribe on client");
                                self.client.unsubscribe(&publish.topic).await.ok();
                            }
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

#[derive(Debug)]
pub struct Subscribe(pub String, pub MessageAddr<Publish>);

#[async_trait]
impl Handler<Subscribe> for MqttConnection {
    type Result = Result<()>;

    async fn handle(
        &mut self,
        Subscribe(topic, addr): Subscribe,
        _context: &mut Context<Self>,
    ) -> Self::Result {
        let list = self.subscribers.entry(topic.clone()).or_insert(vec![]);
        list.push(addr);

        if self.can_subscribe {
            // Call subscribe every time to pump data from the topic
            self.client.subscribe(topic, QoS::AtLeastOnce).await?;
            self.can_subscribe = false;
        } else {
            self.waiting_subscribes.push(topic);
        }

        Ok(())
    }
}
