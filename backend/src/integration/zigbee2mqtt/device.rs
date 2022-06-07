use bonsaidb::core::transmog_pot::pot::de;
use serde::Deserialize;

use crate::{
    device::{JsonSpec, TaskSpec, ValueSpec},
    io::mqtt::MqttServerInfo,
};

#[derive(Deserialize, Debug)]
pub enum DeviceType {
    Router,
    EndDevice,
    Coordinator,
}

#[derive(Deserialize, Debug)]
pub struct Device {
    pub ieee_address: String,
    #[serde(rename = "type")]
    pub device_type: DeviceType,
    pub network_address: u32,
    pub supported: bool,
    pub friendly_name: String,
    pub description: Option<String>,
    // pub endpoints: ...
    pub definition: Option<Definition>,
    pub power_source: Option<String>,
    pub date_code: Option<String>,
    pub model_id: Option<String>,
    // pub scenes: Vec<...>,
    pub interviewing: bool,
    pub interview_completed: bool,
}

impl Device {
    pub fn into_device(self, server_info: &MqttServerInfo) -> Option<crate::device::Device> {
        let definition = self.definition?;
        let device_topic = format!("zigbee2mqtt/{}", self.friendly_name);

        let task_spec = vec![TaskSpec::Mqtt {
            server_info: server_info.clone(),
            topic: device_topic,
            value_spec: ValueSpec::Json(definition.sources_spec()),
        }];

        let device = crate::device::Device {
            id: format!(
                "zigbee2mqtt/{}:{}/{}",
                server_info.host, server_info.port, self.ieee_address
            ),
            name: self.friendly_name,
            last_seen: None,

            task_spec,

            sources: vec![],
            sinks: vec![],
        };

        Some(device)
    }
}

#[derive(Deserialize, Debug)]
pub struct Definition {
    pub model: String,
    pub vendor: String,
    pub description: String,
    pub options: Vec<Feature>,
    pub exposes: Vec<Feature>,
}

impl Definition {
    pub fn sources_spec(&self) -> Vec<JsonSpec> {
        let mut stack = self.exposes.iter().collect::<Vec<_>>();
        let mut out = vec![];

        while let Some(feature) = stack.pop() {
            match feature {
                Feature::Binary {
                    name,
                    property,
                    value_on,
                    value_off,
                    value_toggle,
                    access,
                } if access.published() => out.push(JsonSpec {
                    query: format!("$.{property}"),
                    name: name.into(),
                }),
                
                Feature::Numeric {
                    name,
                    property,
                    value_min,
                    value_max,
                    value_step,
                    unit,
                    access,
                } if access.published() => out.push(JsonSpec {
                    query: format!("$.{property}"),
                    name: name.into(),
                }),

                Feature::Enum {
                    name,
                    property,
                    values,
                    access,
                } if access.published() => out.push(JsonSpec {
                    query: format!("$.{property}"),
                    name: name.into(),
                }),

                Feature::Text {
                    name,
                    property,
                    access,
                } if access.published() => out.push(JsonSpec {
                    query: format!("$.{property}"),
                    name: name.into(),
                }),

                Feature::List {
                    name,
                    property,
                    item_type,
                    access,
                } if access.published() => out.push(JsonSpec {
                    query: format!("$.{property}"),
                    name: name.into(),
                }),

                Feature::Composite {features, ..} => stack.extend(features),
                Feature::Light { features } => stack.extend(features),
                Feature::Switch { features } => stack.extend(features),
                Feature::Fan { features } => stack.extend(features),
                Feature::Cover { features } => stack.extend(features),
                Feature::Lock { features } => stack.extend(features),
                Feature::Climate { features } => stack.extend(features),

                _ => { /* We dont care about Writes, yet atleast */ }
            };
        }

        out
    }
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct Access(u8);

impl std::ops::BitAnd for Access {
    type Output = Access;

    fn bitand(self, rhs: Self) -> Self::Output {
        Access(self.0 & rhs.0)
    }
}

impl Access {
    const NONE: Access = Access(0);

    /// Is the value of this feature published in a channel
    pub fn published(&self) -> bool {
        let mask = 0b001;
        self.0 & mask == 0b001
    }

    /// Can you write the value of this feature
    pub fn write(&self) -> bool {
        let mask = 0b010;
        self.0 & mask == 0b010
    }

    #[allow(dead_code)]
    /// Can you read the steaful state of this feature
    pub fn read(&self) -> bool {
        let mask = 0b100;
        self.0 & mask == 0b100
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Feature {
    #[serde(rename = "binary")]
    Binary {
        name: String,
        property: String,
        value_on: serde_json::Value,
        value_off: serde_json::Value,
        value_toggle: Option<serde_json::Value>,
        access: Access,
    },
    #[serde(rename = "numeric")]
    Numeric {
        name: String,
        property: String,
        value_min: Option<i32>,
        value_max: Option<i32>,
        value_step: Option<i32>,
        unit: Option<String>,
        access: Access,
    },
    #[serde(rename = "enum")]
    Enum {
        name: String,
        property: String,
        values: Vec<String>,
        access: Access,
    },
    #[serde(rename = "text")]
    Text {
        name: String,
        property: String,
        access: Access,
    },
    #[serde(rename = "composite")]
    Composite {
        name: String,
        property: String,
        features: Vec<Feature>,
    },
    #[serde(rename = "list")]
    List {
        name: String,
        property: String,
        item_type: String,
        access: Access,
    },
    #[serde(rename = "light")]
    Light { features: Vec<Feature> },
    #[serde(rename = "switch")]
    Switch { features: Vec<Feature> },
    #[serde(rename = "fan")]
    Fan { features: Vec<Feature> },
    #[serde(rename = "cover")]
    Cover { features: Vec<Feature> },
    #[serde(rename = "lock")]
    Lock { features: Vec<Feature> },
    #[serde(rename = "climate")]
    Climate { features: Vec<Feature> },
}

#[cfg(test)]
mod test {
    #[test]
    fn test_access() {
        let publish = super::Access(0b001);

        assert_eq!(publish.published(), true);
        assert_eq!(publish.write(), false);
        assert_eq!(publish.read(), false);

        let write = super::Access(0b010);

        assert_eq!(write.published(), false);
        assert_eq!(write.write(), true);
        assert_eq!(write.read(), false);

        let read = super::Access(0b100);

        assert_eq!(read.published(), false);
        assert_eq!(read.write(), false);
        assert_eq!(read.read(), true);

        let all = super::Access(0b111);

        assert_eq!(all.published(), true);
        assert_eq!(all.write(), true);
        assert_eq!(all.read(), true);

        let rw = super::Access(0b011);

        assert_eq!(rw.published(), true);
        assert_eq!(rw.write(), true);
        assert_eq!(rw.read(), false);
    }
}
