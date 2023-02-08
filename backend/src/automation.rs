use serde::{Deserialize, Serialize};

type Slot = (u64, String);

#[derive(Serialize, Deserialize, Debug)]
pub struct Automation {
    counter: u64,
    nodes: Vec<Node>,
    connections: Vec<(Slot, Slot)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Device {
    id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Prop {
    Target,

    And,

    Device(Device),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    id: u64,
    position: (i64, i64),
    properties: Prop,
}
