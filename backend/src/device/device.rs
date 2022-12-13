use std::{collections::BTreeMap, sync::Arc};

use anyhow::Result;
use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

use crate::{task::Task, db};

use super::TaskSpec;

#[derive(Debug, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    /// Name of the device
    pub name: String,
    /// What do we need to create in order for data to flow into sources and out of sinks
    /// This needs to be plain data so we can recreate the tasks on restart
    #[serde(default)]
    pub task_spec: Vec<TaskSpec>,
    /// All the values this Devices exposes or reads and handles
    #[serde(default)]
    pub features: Vec<ValueSpec>,
}

impl Device {
    pub async fn spawn_tasks(self: Arc<Self>, task: &mut Task) {
        for spec in &self.task_spec {
            match spec {
                TaskSpec::Zigbee2Mqtt(server) => {
                    let label = format!("zigbee2mqtt:{}:{}", server.host, server.port);

                    if task.has_task(&label) {
                        // There is no need to reboot the task just ignore
                        continue;
                    }

                    task.spawn_with_argument(
                        label,
                        server.clone(),
                        crate::integration::zigbee2mqtt::zigbee2mqtt_update,
                    );
                }
                TaskSpec::Zigbee2MqttDevice(_) => {
                    let label = format!("{}/Zigbee2MqttDevice", self.id);

                    task.spawn_with_argument(
                        label,
                        self.clone(),
                        crate::integration::zigbee2mqtt::zigbee2mqtt_device,
                    )
                }
            }
        }
    }

    /// Save the device to storage
    pub fn save(&self) -> Result<()> {
        let tree = db::tree("device")?;
        tree.insert(&self.id, self)?;

        Ok(())
    }

    pub fn all() -> Result<impl Iterator<Item = Device>> {
        let tree = db::tree("device")?;
        Ok(tree.all::<Device>())
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag="type")]
pub enum ValueKind {
    #[serde(rename = "bool")]
    Bool,
    #[serde(rename = "number")]
    Number { unit: Option<String> },
    #[serde(rename = "state")]
    State { possible: Vec<String> },
    #[serde(rename = "string")]
    String,
}

impl ValueKind {
    // TODO: this is a wierd API
    pub fn validate(&self, value: &Value) -> Result<Value, String> {
        match (value, self) {
            (Value::Null, _) => Ok(Value::Null),
            (Value::Bool(b), ValueKind::Bool) => Ok(Value::Bool(*b)),
            (Value::Number(n), ValueKind::Number { .. }) => Ok(Value::Number(n.clone())),
            (Value::String(s), ValueKind::String) => Ok(Value::String(s.clone())),

            (Value::String(s), ValueKind::State { possible }) => {
                if s.is_empty() {
                    // Treat empty strings a null, quite a few devices go back to an "empty", state 
                    Ok(Value::Null)
                } else if possible.contains(s) {
                    Ok(Value::String(s.clone()))
                } else {
                    Err(format!("{} is not part of state set {:?}", s, possible))
                }
            }

            (Value::Array(_), _) => Err("Only descrete json values allowed, got array".into()),
            (Value::Object(_), _) => Err("Only descrete json values allowed, got array".into()),

            (a, b) => Err(format!(
                "Got value of {:?} expected value of kind {:?}",
                a, b
            )),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValueSpec {
    pub id: String,
    pub name: String,
    pub direction: ValueDirection,
    pub kind: ValueKind,
    pub meta: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum ValueDirection {
    #[serde(rename = "source")]
    Source,
    #[serde(rename = "sink")]
    Sink,
    #[serde(rename = "sourceSink")]
    SourceSink
}

impl ValueDirection {
    pub fn can_read(&self) -> bool {
        match self {
            ValueDirection::Source => true,
            ValueDirection::Sink => false,
            ValueDirection::SourceSink => true,
        }
    }
}
