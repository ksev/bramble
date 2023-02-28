mod node;

use std::collections::{BTreeSet, HashMap};

use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};

use crate::{
    program::{Program, ProgramNode},
    strings::IString,
    value::ValueId,
};

type Slot = (u64, String);
type Connection = (Slot, Slot);

fn prop_to_node(target: ValueId, prop: &Properties) -> Box<dyn ProgramNode> {
    use Properties::*;

    match prop {
        Target => Box::new(node::Target::new(target)),
        Device(id) => Box::new(node::Device::new(id.into())),
        And => Box::new(node::And),
        Or => Box::new(node::Or),
        Not => Box::new(node::Not),
        Xor => Box::new(node::Xor),
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Automation {
    counter: u64,
    nodes: Vec<Node>,
    connections: Vec<Connection>,
}

impl Automation {
    pub fn compile(&self, target: ValueId) -> Result<(Program, Vec<ValueId>)> {
        let target_count = self
            .nodes
            .iter()
            .filter(|n| matches!(n.properties, Properties::Target))
            .count();

        ensure!(target_count == 1, "Programs requires exactly one Target");

        let node: Vec<&Node> = self.nodes.iter().collect();

        // Optimisation steps
        let (node, connections) = filter_unconnected(&node, &self.connections);
        let (node, connections) = merge_device_nodes(&node, &connections);
        let connections = unique_connections(&connections);

        // We might optimise away the program
        if self.connections.is_empty() {
            // There are no connections, program is useless but valid
            return Ok((Program::new(vec![], vec![])?, vec![]));
        }

        let dependencies = find_dependencies(&node, &connections);

        // The is a program but it does not react to any data, so again useless
        if dependencies.is_empty() {
            // There are no connections, program is useless but valid
            return Ok((Program::new(vec![], vec![])?, vec![]));
        }

        // Program only works on idecies and not the id field
        let idx: HashMap<_, _> = node.iter().enumerate().map(|(i, n)| (n.id, i)).collect();

        let connections = connections
            .iter()
            // Rewrite Automation node id to Program node index
            .map(|(fr, to)| {
                let (fid, fslot) = fr;
                let (tid, tslot) = to;

                ((idx[fid], fslot.into()), (idx[tid], tslot.into()))
            })
            .collect();

        let steps = node
            .iter()
            .map(|n| &n.properties)
            .map(|prop| prop_to_node(target, prop))
            .collect();

        Ok((Program::new(steps, connections)?, dependencies))
    }
}

fn filter_unconnected<'a>(
    nodes: &'a [&'a Node],
    connections: &'a [Connection],
) -> (Vec<&'a Node>, Vec<Connection>) {
    let mut incoming: HashMap<u64, BTreeSet<u64>> = HashMap::new();

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
    let mut outgoing: HashMap<u64, BTreeSet<&str>> = HashMap::new();

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

fn unique_connections(connections: &[(Slot, Slot)]) -> Vec<(Slot, Slot)> {
    BTreeSet::from_iter(connections.iter().cloned())
        .into_iter()
        .collect()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "tag", content = "content")]
pub enum Properties {
    Target,
    Device(String),

    // Logic
    And,
    Or,
    Not,
    Xor,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    id: u64,
    position: (i64, i64),
    properties: Properties,
}

#[cfg(test)]
mod test {
    use crate::value;

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
        };

        let target = ValueId::new("", "state");
        let (mut program, _) = auto.compile(target).unwrap();

        value::set_current(ValueId::new("amk", "state"), Ok(json!(true)));
        value::set_current(ValueId::new("amk", "state_two"), Ok(json!(false)));
        value::set_current(ValueId::new("two", "state"), Ok(json!(false)));

        program.execute().unwrap();
    }

    #[test]
    fn uniqe_device_node() {
        let p = r#"{"counter":6,"nodes":[{"id":0,"position":[6355,6027],"properties":{"tag":"Target"}},{"id":1,"position":[5415,5651],"properties":{"tag":"Device","content":"0x003c84fffe164750"}},{"id":2,"position":[5418,5902],"properties":{"tag":"Device","content":"0x003c84fffe164750"}},{"id":3,"position":[5433,6145],"properties":{"tag":"Device","content":"0x00158d00075a4e9c"}},{"id":4,"position":[5769,6041],"properties":{"tag":"Or"}},{"id":5,"position":[6055,5905],"properties":{"tag":"And"}}],"connections":[[[3,"occupancy"],[4,"input"]],[[2,"occupancy"],[4,"input"]],[[2,"occupancy"],[5,"input"]],[[4,"result"],[5,"input"]],[[5,"result"],[0,"state"]],[[1,"occupancy"],[5,"input"]]]}"#;
        let auto: Automation = serde_json::de::from_str(&p).unwrap();

        let target = ValueId::new("", "state");
        let (_program, _) = auto.compile(target).unwrap();
    }

    #[test]
    fn dependency_list() {
        let p = r#"{"counter":6,"nodes":[{"id":0,"position":[6113,5917],"properties":{"tag":"Target"}},{"id":3,"position":[5840,5917],"properties":{"tag":"Or"}},{"id":4,"position":[5439,5761],"properties":{"tag":"Device","content":"0x003c84fffe164750"}},{"id":5,"position":[5432,5995],"properties":{"tag":"Device","content":"0x00158d00075a4e9c"}}],"connections":[[[3,"result"],[0,"state"]],[[4,"occupancy"],[3,"input"]],[[5,"occupancy"],[3,"input"]],[[4,"illuminance_above_threshold"],[3,"input"]]]}"#;

        let auto: Automation = serde_json::de::from_str(&p).unwrap();

        let target = ValueId::new("", "state");
        let (_, deps) = auto.compile(target).unwrap();

        assert_eq!(
            deps.into_iter().collect::<Vec<_>>(),
            vec![
                ValueId::new("0x003c84fffe164750", "illuminance_above_threshold"),
                ValueId::new("0x003c84fffe164750", "occupancy"),
                ValueId::new("0x00158d00075a4e9c", "occupancy")
            ]
        );
    }
}
