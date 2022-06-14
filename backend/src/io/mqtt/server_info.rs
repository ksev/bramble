use serde::{Serialize, Deserialize};

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