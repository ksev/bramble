use std::collections::BTreeMap;

use itertools::Itertools;
use serde_derive::Deserialize;
use tracing::error;

use crate::{
    device::{TaskSpec, ValueDirection, ValueKind, ValueSpec},
    io::mqtt::{MqttServerInfo, MqttSubscribe},
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
    pub fn into_device(self, server: MqttServerInfo) -> Option<crate::device::Device> {
        let def = self.definition?;
        let topic = format!("zigbee2mqtt/{}", self.friendly_name);
        let subscribe = MqttSubscribe { server, topic };

        // Filter out duplicate properties
        let features = def
            .to_value_specs()
            .into_iter()
            .unique_by(|f| f.id.clone())
            .collect();

        let out = crate::device::Device {
            id: self.ieee_address,
            name: self.friendly_name,
            task_spec: vec![TaskSpec::Zigbee2MqttDevice(subscribe)],
            features,
        };

        Some(out)
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
    pub fn to_value_specs(&self) -> Vec<ValueSpec> {
        let mut stack = self.exposes.iter().collect::<Vec<_>>();
        let mut out = vec![];

        while let Some(feature) = stack.pop() {
            match feature {
                Feature::Binary {
                    name,
                    property,
                    value_on,
                    value_off,
                    access,
                    ..
                } => {
                    let direction = match access.try_into() {
                        Ok(dir) => dir,
                        Err(e) => {
                            error!("{e:?}");
                            continue;
                        }
                    };

                    out.push(ValueSpec {
                        name: clean_up_name(name),
                        id: property.clone(),
                        kind: ValueKind::Bool,
                        direction,
                        meta: BTreeMap::from([
                            ("value_on".into(), value_on.clone()),
                            ("value_off".into(), value_off.clone()),
                        ]),
                    })
                }

                Feature::Numeric {
                    name,
                    property,
                    unit,
                    access,
                    ..
                } => {
                    let direction = match access.try_into() {
                        Ok(dir) => dir,
                        Err(e) => {
                            error!("{e:?}");
                            continue;
                        }
                    };

                    out.push(ValueSpec {
                        name: clean_up_name(name),
                        id: property.clone(),
                        direction,
                        kind: ValueKind::Number { unit: unit.clone() },
                        meta: BTreeMap::new(),
                    })
                }

                Feature::Enum {
                    name,
                    property,
                    values,
                    access,
                } => {
                    let direction = match access.try_into() {
                        Ok(dir) => dir,
                        Err(e) => {
                            error!("{e:?}");
                            continue;
                        }
                    };

                    out.push(ValueSpec {
                        name: clean_up_name(name),
                        id: property.clone(),
                        direction,
                        kind: ValueKind::State {
                            possible: values.clone(),
                        },
                        meta: BTreeMap::new(),
                    })
                }

                Feature::Text {
                    name,
                    property,
                    access,
                } => {
                    let direction = match access.try_into() {
                        Ok(dir) => dir,
                        Err(e) => {
                            error!("{e:?}");
                            continue;
                        }
                    };

                    out.push(ValueSpec {
                        name: clean_up_name(name),
                        id: property.clone(),
                        direction,
                        kind: ValueKind::String,
                        meta: BTreeMap::new(),
                    })
                }

                Feature::List {
                    name,
                    property,
                    access,
                    ..
                } => {
                    let direction = match access.try_into() {
                        Ok(dir) => dir,
                        Err(e) => {
                            error!("{e:?}");
                            continue;
                        }
                    };

                    out.push(ValueSpec {
                        name: clean_up_name(name),
                        id: property.clone(),
                        direction,
                        kind: ValueKind::Number { unit: None },
                        meta: BTreeMap::new(),
                    })
                }

                Feature::Composite { features, .. } => stack.extend(features),
                Feature::Light { features } => stack.extend(features),
                Feature::Switch { features } => stack.extend(features),
                Feature::Fan { features } => stack.extend(features),
                Feature::Cover { features } => stack.extend(features),
                Feature::Lock { features } => stack.extend(features),
                Feature::Climate { features } => stack.extend(features),
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
    #[allow(unused)]
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

    #[allow(unused)]
    /// Can you read the steaful state of this feature
    pub fn read(&self) -> bool {
        let mask = 0b100;
        self.0 & mask == 0b100
    }
}

impl TryFrom<&Access> for ValueDirection {
    type Error = anyhow::Error;

    fn try_from(acc: &Access) -> Result<Self, Self::Error> {
        Ok(match (acc.published(), acc.write()) {
            (true, true) => ValueDirection::SourceSink,
            (true, false) => ValueDirection::Source,
            (false, true) => ValueDirection::Sink,
            (false, false) => anyhow::bail!("Can't read nor write feature"),
        })
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

fn clean_up_name(name: &str) -> String {
    let replaced = name.replace("_", " ");
    let mut iter = replaced.chars();

    let Some(first) = iter.next() else {
        return "".into();
    };

    first.to_uppercase().chain(iter).collect()
}

#[cfg(test)]
mod test {
    #[test]
    fn clean_up_name() {
        let test1 = super::clean_up_name("requested_brightness_percent");
        let test2 = super::clean_up_name("battery");
        let test3 = super::clean_up_name("");
        let test4 = super::clean_up_name("a");

        assert_eq!(test1, "Requested brightness percent");
        assert_eq!(test2, "Battery");
        assert_eq!(test3, "");
        assert_eq!(test4, "A");
    }

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
