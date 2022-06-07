use anyhow::Result;
use async_trait::async_trait;
use jsonpath_rust::JsonPathQuery;
use rumqttc::Publish;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::actor::prelude::*;
use crate::io::mqtt::{Mqtt, Subscribe};
use crate::{actor::UntypedAddr, io::mqtt::MqttServerInfo};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum TaskSpec {
    Mqtt {
        server_info: MqttServerInfo,
        topic: String,
        value_spec: ValueSpec,
    },
}

impl TaskSpec {
    pub fn start(self) -> Result<UntypedAddr> {
        match self {
            TaskSpec::Mqtt {
                server_info,
                topic,
                value_spec: ValueSpec::Json(value_spec),
            } => {
                Ok(
                    SourceMqttSubscribeJson::start(server_info.clone(), topic.clone(), value_spec)?
                        .untyped_reference(),
                )
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum ValueSpec {
    Json(Vec<JsonSpec>),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct JsonSpec {
    pub query: String,
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
    ) -> Result<Addr<SourceMqttSubscribeJson>> {
        let mqtt = Mqtt::connect(server_info);
        let addr = SourceMqttSubscribeJson { value_spec }.start();

        let saddr = addr.clone();
        let _ = mqtt.send(Subscribe(topic, saddr.to_weak().into()))?;

        Ok(addr)
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
            match json.clone().path(&spec.query) {
                Ok(data) => println!("{}", data),
                Err(e) => error!("jsonpath error: {e:?}"),
            }
        }
    }
}
