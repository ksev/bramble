use crate::actor::prelude::*;
use crate::io::mqtt::MqttServerInfo;
use crate::io::mqtt::{Mqtt, Subscribe};
use anyhow::Result;
use async_trait::async_trait;
use rumqttc::Publish;
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum TaskSpec {
    Mqtt {
        server_info: MqttServerInfo,
        topic: String,
        value_spec: ValueSpec,
    },
}

impl TaskSpec {
    pub fn start(self) -> Result<()> {
        match self {
            TaskSpec::Mqtt {
                server_info,
                topic,
                value_spec: ValueSpec::Json(value_spec),
            } => Ok(SourceMqttSubscribeJson::start(
                server_info.clone(),
                topic.clone(),
                value_spec,
            )?),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum ValueSpec {
    Json(Vec<JsonSpec>),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct JsonSpec {
    pub pointer: String,
    pub name: String,
}

////// Actor

struct SourceMqttSubscribeJson {
    value_spec: Vec<JsonSpec>,
}

impl SourceMqttSubscribeJson {
    pub fn start(
        server_info: MqttServerInfo,
        topic: String,
        value_spec: Vec<JsonSpec>,
    ) -> Result<()> {
        let mqtt = Mqtt::connect(server_info);
        let addr = SourceMqttSubscribeJson { value_spec }.start();

        let saddr = addr.clone();
        mqtt.subscribe(topic, saddr.into());

        Ok(())
    }
}

impl Actor for SourceMqttSubscribeJson {
    type Context = Context<Self>;
}

#[async_trait]
impl Handler<Publish> for SourceMqttSubscribeJson {
    type Result = ();

    async fn handle(&mut self, message: Publish, _context: &mut Context<Self>) -> Self::Result {
        let json: serde_json::Value = serde_json::from_slice(&message.payload).unwrap();

        for spec in self.value_spec.iter() {
            match json.pointer(spec.pointer) {
                Some(data) => println!("{data:?}"),
                None => warn!("json pointer {} not found", spec.pointer),
            }
        }
    }
}
