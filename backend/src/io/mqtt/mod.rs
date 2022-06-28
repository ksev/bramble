mod server_info;

use std::collections::HashMap;

use crate::actor::{Context, Receive, Trap};
use anyhow::Result;
use rumqttc::Publish;

use self::server_info::MqttServerInfo;

pub enum Mqtt {
    Subscribe(MqttServerInfo, String, Box<dyn From<Publish>>),
    Unsubscribe(MqttServerInfo, String),
}

async fn mqtt(ctx: Trap<Mqtt>) -> Result<()> {
    use crate::actor::Signal::*;

    let mut connections = HashMap::new();
    let mut pidmap = HashMap::new();

    loop {
        match ctx.trap().await {
            Exit(_) => todo!(),
            Message(message) => match message {
                Mqtt::Unsubscribe(info, subject) => {}
                Mqtt::Subscribe(info, subject) => {
                    let conn = connections
                        .entry(info.clone())
                        .or_insert_with(|| ctx.spawn_link_with_argument(mqtt_connection, info));

                    conn.send(MqttConnection::Subscribe(subject, Pid<>));

                    if let Some(conn) = connections.get(&info) {}
                }
            },
        }
    }
}

pub enum MqttConnection {}

async fn mqtt_connection(ctx: Receive<MqttConnection>, info: MqttServerInfo) -> Result<()> {
    Ok(())
}
