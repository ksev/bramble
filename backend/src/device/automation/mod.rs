mod node;

use std::collections::{BTreeSet, HashMap};

use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use crate::{
    program::{Program, ProgramNode},
    strings::IString,
    value::ValueId,
};

type Slot = (u32, String);
type Connection = (Slot, Slot);

use node::{node0, node1, node1_mut};

fn prop_to_node(target: ValueId, prop: &Properties) -> Box<dyn ProgramNode> {
    use Properties::*;

    match prop {
        Target => node1(target, node::target),
        Device(id) => node1(id.into(), node::device),
        Value(v) => node1_mut(v.clone(), node::static_value),
        IsNull(_) => node0(node::is_null),
        If { .. } => node0(node::alt),
        Equals { .. } => node0(node::equals),
        Toggle => node1_mut(false, node::toggle),
        And => node0(node::and),
        Or => node0(node::or),
        Not => node0(node::not),
        Xor => node0(node::xor),
        Latch => node1_mut(false, node::latch),
        MathCompare { operator } => node1(*operator, node::compare),
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Automation {
    counter: u32,
    nodes: Vec<Node>,
    connections: Vec<Connection>,
    defaults: Vec<(Slot, Json)>,
}

impl Automation {
    pub fn compile(&self, target: ValueId) -> Result<(Program, Vec<ValueId>)> {
        let target_count = self
            .nodes
            .iter()
            .filter(|n| matches!(n.properties, Properties::Target))
            .count();

        ensure!(target_count == 1, "Programs requires exactly one Target");

        // Convert the default value list to static value nodes
        let (added_nodes, connections) =
            default_values(self.counter, &self.connections, &self.defaults);

        let node: Vec<&Node> = self.nodes.iter().chain(&added_nodes).collect();

        // Optimisation steps
        let (node, connections) = filter_unconnected(&node, &connections);
        let (node, connections) = merge_device_nodes(&node, &connections);
        let connections = unique_connections(&connections);

        // We might optimise away the program
        if self.connections.is_empty() {
            // There are no connections, program is useless but valid
            return Ok((Program::default(), vec![]));
        }

        let dependencies = find_dependencies(&node, &connections);

        // The is a program but it does not react to any data, so again useless
        if dependencies.is_empty() {
            // There are no connections, program is useless but valid
            return Ok((Program::default(), vec![]));
        }

        let connections = connections
            .iter()
            .map(|(fr, to)| {
                let (fid, fslot) = fr;
                let (tid, tslot) = to;

                ((*fid, fslot.into()), (*tid, tslot.into()))
            })
            .collect();

        let steps = node
            .iter()
            .map(|n| (n.id, prop_to_node(target, &n.properties)))
            .collect();

        Ok((Program::new(steps, connections)?, dependencies))
    }
}

fn default_values(
    counter: u32,
    connections: &[Connection],
    defaults: &[(Slot, Json)],
) -> (Vec<Node>, Vec<Connection>) {
    let mut cons: Vec<Connection> = connections.into();
    let mut node = vec![];

    let mut incoming: BTreeSet<Slot> = BTreeSet::new();

    for (_, inc) in connections {
        incoming.insert(inc.clone());
    }

    for (i, (slot, value)) in defaults.iter().enumerate() {
        // Dont add default values to connected inputs
        if incoming.contains(slot) {
            continue;
        }

        let id = counter + i as u32;

        // Every default value is just a static value node
        node.push(Node {
            id,
            position: (0, 0),
            properties: Properties::Value(value.clone()),
        });

        cons.push(((id, "value".into()), slot.clone()));
    }

    (node, cons)
}

fn filter_unconnected<'a>(
    nodes: &'a [&'a Node],
    connections: &'a [Connection],
) -> (Vec<&'a Node>, Vec<Connection>) {
    let mut incoming: HashMap<u32, BTreeSet<u32>> = HashMap::new();

    for ((f, _), (t, _)) in connections {
        incoming.entry(*t).or_default().insert(*f);
    }

    let mut keep = BTreeSet::new();
    let mut stack = vec![0];

    while let Some(n) = stack.pop() {
        keep.insert(n);

        if let Some(inc) = incoming.get(&n) {
            for &next in inc {
                stack.push(next);
            }
        }
    }

    let nodes = nodes
        .iter()
        .filter(|n| keep.contains(&n.id))
        .copied()
        .collect();

    let connections = connections
        .iter()
        .filter(|((f, _), (t, _))| keep.contains(f) && keep.contains(t))
        .cloned()
        .collect();

    (nodes, connections)
}

fn merge_device_nodes<'a>(
    nodes: &'a [&'a Node],
    connections: &'a [Connection],
) -> (Vec<&'a Node>, Vec<Connection>) {
    let mut first = HashMap::new();
    let mut re = HashMap::new();

    let device_nodes = nodes.iter().filter_map(|n| match &n.properties {
        Properties::Device(d) => Some((n.id, d)),
        _ => None,
    });

    for (id, dev) in device_nodes {
        if let Some(idx) = first.get(dev).cloned() {
            // This device can also be found at the idx
            re.insert(id, idx);
        } else {
            // Add a note where we saw this device, that we havent seen before
            first.insert(dev.clone(), id);
        }
    }

    let nodes = nodes
        .iter()
        .filter(|n| !re.contains_key(&n.id))
        .copied()
        .collect();

    let connections = connections
        .iter()
        // Device nodes only has outgoing connections so we only need to rewrite the start
        .map(|tup @ ((f, fs), to)| {
            if let Some(nid) = re.get(f) {
                ((*nid, fs.clone()), to.clone())
            } else {
                tup.clone()
            }
        })
        .collect();

    (nodes, connections)
}

fn find_dependencies<'a>(nodes: &'a [&'a Node], connections: &'a [Connection]) -> Vec<ValueId> {
    let mut outgoing: HashMap<u32, BTreeSet<&str>> = HashMap::new();

    for ((f, fs), _) in connections {
        outgoing.entry(*f).or_default().insert(fs);
    }

    nodes
        .iter()
        .filter_map(|n| match &n.properties {
            Properties::Device(d) => Some((n.id, d)),
            _ => None,
        })
        .filter_map(|(id, dev)| Some((outgoing.get(&id)?, dev)))
        .flat_map(|(set, dev)| {
            let dev: IString = dev.into();
            set.iter().map(move |&slot| ValueId::new(dev, slot))
        })
        .collect()
}

fn unique_connections(connections: &[Connection]) -> Vec<Connection> {
    BTreeSet::from_iter(connections.iter().cloned())
        .into_iter()
        .collect()
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub enum CompareOp {
    Eq,
    Gt,
    Lt,
    Ge,
    Le,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "tag", content = "content")]
pub enum Properties {
    Target,
    Device(String),
    Value(Json),

    // Universal
    IsNull(String),
    Equals { kind: String, meta: Option<Json> },
    If { kind: String },

    // Logic
    And,
    Or,
    Not,
    Xor,
    Latch,
    Toggle,

    // Math
    MathCompare { operator: CompareOp },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    id: u32,
    position: (i64, i64),
    properties: Properties,
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use super::*;
    use serde_json::json;

    #[test]
    fn can_compile_automation() {
        let nodes = vec![
            Node {
                id: 0,
                position: (0, 0),
                properties: Properties::Target,
            },
            Node {
                id: 2,
                position: (0, 0),
                properties: Properties::Device("amk".into()),
            },
            Node {
                id: 4,
                position: (0, 0),
                properties: Properties::Device("two".into()),
            },
            Node {
                id: 7,
                position: (0, 0),
                properties: Properties::Or,
            },
        ];

        let connections = vec![
            ((2, "state".into()), (7, "input".into())),
            ((2, "state_two".into()), (7, "input".into())),
            ((4, "state".into()), (7, "input".into())),
            ((7, "result".into()), (0, "state".into())),
        ];

        let auto = Automation {
            counter: 0, // Makes no difference
            nodes,
            connections,
            defaults: vec![],
        };

        let target = ValueId::new("", "state");
        let (mut program, _) = auto.compile(target).unwrap();

        let mut input = BTreeMap::new();

        input.insert(ValueId::new("amk", "state"), json!(true));
        input.insert(ValueId::new("amk", "state_two"), json!(false));
        input.insert(ValueId::new("two", "state"), json!(false));

        let out = program.execute(&input).unwrap();

        assert_eq!(out[&target], json!(true));
    }
}
