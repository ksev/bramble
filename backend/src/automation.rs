use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    ops::{Index, Range},
};

use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

use crate::{
    bus::Topic,
    device::{FeatureValue, Sources},
};

type Slot = (u64, String);

#[derive(Serialize, Deserialize, Debug)]
pub struct Automation {
    counter: u64,
    nodes: Vec<Node>,
    connections: Vec<(Slot, Slot)>,
}

impl Automation {
    pub fn compile(&self, device_id: &str, feature_id: &str) -> Result<Program> {
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
            return Ok(Program::new(vec![], vec![])?);
        }

        // Program only works on idecies and not the id field
        let idx: HashMap<_, _> = node.iter().enumerate().map(|(i, n)| (n.id, i)).collect();

        let connections = connections
            .iter()
            .cloned()
            // Rewrite Automation node id to Program node index
            .map(|(fr, to)| {
                let (fid, fslot) = fr;
                let (tid, tslot) = to;

                ((idx[&fid], fslot), (idx[&tid], tslot))
            })
            .collect();

        let steps = node
            .iter()
            .map(|n| &n.properties)
            .map(|prop| {
                use Properties::*;
                match prop {
                    Target => bnode(nodes::Target::new(device_id.into(), feature_id.into())),
                    Device(id) => bnode(nodes::Device::new(id.clone())),
                    And => bnode(nodes::And),
                    Or => bnode(nodes::Or),
                    Not => bnode(nodes::Not),
                    Xor => bnode(nodes::Xor),
                }
            })
            .collect();

        Ok(Program::new(steps, connections)?)
    }
}

fn filter_unconnected<'a>(
    nodes: &'a [&'a Node],
    connections: &'a [(Slot, Slot)],
) -> (Vec<&'a Node>, Vec<(Slot, Slot)>) {
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
        .map(|&n| n)
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
    connections: &'a [(Slot, Slot)],
) -> (Vec<&'a Node>, Vec<(Slot, Slot)>) {
    let mut first = HashMap::new();
    let mut re = HashMap::new();

    let device_nodes = nodes.iter().filter_map(|n| match &n.properties {
        Properties::Device(d) => Some((n.id.clone(), d)),
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
        .map(|&n| n)
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

fn unique_connections<'a>(connections: &'a [(Slot, Slot)]) -> Vec<(Slot, Slot)> {
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

////////   Program deff

struct MVec<T> {
    values: Vec<T>,
    ranges: Vec<usize>,
}

impl<T> MVec<T> {
    pub fn new() -> MVec<T> {
        MVec {
            values: vec![],
            ranges: vec![0],
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn range(&self, index: usize) -> Range<usize> {
        let from = self.ranges[index];
        let to = self.ranges[index + 1];

        from..to
    }

    pub fn push<I>(&mut self, data: I) -> usize
    where
        I: Iterator<Item = T>,
    {
        for v in data {
            self.values.push(v);
        }

        self.ranges.push(self.values.len());

        self.ranges.len() - 2
    }
}

impl<T> Index<usize> for MVec<T> {
    type Output = [T];

    fn index(&self, index: usize) -> &Self::Output {
        let from = self.ranges[index];
        let to = self.ranges[index + 1];

        &self.values[from..to]
    }
}

impl<T> std::fmt::Debug for MVec<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut lst = f.debug_list();

        for i in 0..self.ranges.len() - 1 {
            let s = &self[i];
            lst.entry(&s);
        }

        lst.finish()
    }
}

pub struct Context<'a> {
    pub set_value: &'a Topic<(String, String, FeatureValue)>,
    pub sources: &'a Sources,
}

pub trait ProgramNode: std::fmt::Debug {
    fn run(&mut self, context: &Context, slots: &mut Slots) -> Result<()>;
}

fn bnode<T>(node: T) -> Box<dyn ProgramNode>
where
    T: ProgramNode + 'static,
{
    Box::new(node)
}

#[derive(Debug)]
pub struct Program {
    nodes: Vec<Box<dyn ProgramNode>>,

    outputs: MVec<String>,
    inputs: MVec<(String, usize)>,
    input_value_index: MVec<usize>,

    values: Vec<Json>,
}

impl Program {
    pub fn new(
        nodes: Vec<Box<dyn ProgramNode>>,
        connections: Vec<((usize, String), (usize, String))>,
    ) -> Result<Program> {
        // Make sure we execute the nodes in the right order
        let (nodes, connections) = topological_sort(nodes, connections)?;

        let mut incoming: HashMap<usize, BTreeMap<&str, Vec<(usize, &str)>>> = HashMap::new();
        let mut outgoing: HashMap<usize, BTreeSet<&str>> = HashMap::new();

        // Node output slot id to value index cache
        //let mut vi = BTreeMap::new();

        for ((f, fs), (t, ts)) in &connections {
            // For inputs we need to know the name of each input on the nodes
            // but also how they are connected to outputs so we can
            // setup the indexes for inputs to read the correct output data
            incoming
                .entry(*t)
                .or_default()
                .entry(ts)
                .or_default()
                .push((*f, fs));

            // For outputs we only need two know the name of the outputs on each node
            // to build the index
            outgoing.entry(*f).or_default().insert(fs);
        }

        let mut outputs = MVec::new();
        let mut inputs = MVec::new();
        let mut input_value_index = MVec::new();

        // Build indecies for node outputs
        for i in 0..nodes.len() {
            let n = outgoing.get(&i);
            let slots = n
                .iter()
                .flat_map(|&out| out.iter())
                .map(|&slot| slot.into());

            outputs.push(slots);
        }

        // Build indecies for inputs and the set of values represented by each input
        for i in 0..nodes.len() {
            let n = incoming.get(&i);
            let slots = n.iter().flat_map(|&inc| inc.iter()).map(|(&slot, v)| {
                let vs = v.iter().filter_map(|(n, sl)| {
                    let start = outputs.range(*n).start;
                    outputs[*n].iter().position(|s| s == sl).map(|p| p + start)
                });
                (slot.into(), input_value_index.push(vs))
            });

            inputs.push(slots);
        }

        let values = vec![Json::Null; outputs.len()];

        let p = Program {
            nodes,
            values,
            outputs,
            inputs,
            input_value_index,
        };

        println!("{p:#?}");

        Ok(p)
    }

    pub fn execute(
        &mut self,
        set_value: &Topic<(String, String, FeatureValue)>,
        sources: &Sources,
    ) -> Result<()> {
        // Reset data from the last run
        for v in self.values.iter_mut() {
            *v = Json::Null;
        }

        let ctx = Context { set_value, sources };

        for (index, node) in self.nodes.iter_mut().enumerate() {
            let mut slots = Slots {
                index,

                outputs: &self.outputs,
                inputs: &self.inputs,

                values: &mut self.values[..],
                input_value_index: &self.input_value_index,
            };

            node.run(&ctx, &mut slots)?;
        }

        Ok(())
    }
}

fn topological_sort(
    nodes: Vec<Box<dyn ProgramNode>>,
    connections: Vec<((usize, String), (usize, String))>,
) -> Result<(
    Vec<Box<dyn ProgramNode>>,
    Vec<((usize, String), (usize, String))>,
)> {
    let mut incoming: HashMap<usize, BTreeSet<usize>> = HashMap::new();
    let mut outgoing: HashMap<usize, BTreeSet<usize>> = HashMap::new();

    for ((f, _), (t, _)) in &connections {
        incoming.entry(*t).or_default().insert(*f);
        outgoing.entry(*f).or_default().insert(*t);
    }

    let mut start: Vec<_> = (0..nodes.len())
        .into_iter()
        // Only nodes with no incoming connections to start with.
        .filter(|n| !incoming.contains_key(&n))
        .collect();

    ensure!(!start.is_empty(), "Progam is not acyclic");

    let mut order = vec![];

    while let Some(n) = start.pop() {
        order.push(n);

        let Some(e) = outgoing.get(&n) else {
            continue;
        };

        for &m in e.iter() {
            if let Some(mut inc) = incoming.remove(&m) {
                inc.remove(&n);

                if inc.is_empty() {
                    start.push(m);
                } else {
                    incoming.insert(m, inc);
                }
            }
        }
    }

    ensure!(incoming.is_empty(), "Program is not acyclic");

    let reverse: HashMap<_, _> = order
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, j)| (j, i))
        .collect();

    let connections = connections
        .into_iter()
        .map(|((f, fs), (t, ts))| ((reverse[&f], fs), (reverse[&t], ts)))
        .collect();

    let mut n: HashMap<_, _> = nodes.into_iter().enumerate().collect();

    let mut nodes = Vec::with_capacity(n.len());

    for i in &order {
        let Some(node) = n.remove(i) else {
            anyhow::bail!("Could not lookup node from order, this is a bug");
        };

        nodes.push(node);
    }

    anyhow::ensure!(n.is_empty(), "Node lookup table is not empty");

    Ok((nodes, connections))
}

pub struct Slots<'a> {
    index: usize,

    outputs: &'a MVec<String>,
    inputs: &'a MVec<(String, usize)>,
    input_value_index: &'a MVec<usize>,

    values: &'a mut [Json],
}

impl Slots<'_> {
    pub fn outputs(&mut self) -> impl Iterator<Item = Output> {
        let range = self.outputs.range(self.index);
        let vals = self.values[range].iter_mut();
        let outputs = &self.outputs[self.index];

        outputs
            .iter()
            .zip(vals)
            .map(|(n, v)| Output { id: n, value: v })
    }

    pub fn output(&mut self, id: &str, value: Json) {
        let start = self.outputs.range(self.index).start;
        let outputs = &self.outputs[self.index];

        let idx = outputs.iter().position(|s| s == id);

        if let Some(idx) = idx {
            self.values[start + idx] = value;
        }
    }

    pub fn input(&self, id: &str) -> Result<impl Iterator<Item = &Json>> {
        let inputs = &self.inputs[self.index];

        let Some((_, value_index)) = inputs.iter().find(|(s, _)| s == id) else {
            anyhow::bail!("no input named {}", id);  
        };

        let iter = self.input_value_index[*value_index]
            .iter()
            .map(|&i| &self.values[i]);

        Ok(iter)
    }

    pub fn inputs(&self) -> impl Iterator<Item = &str> {
        self.inputs[self.index].iter().map(|(id, _)| id.as_str())
    }
}

pub struct Output<'a> {
    id: &'a str,
    value: &'a mut Json,
}

impl Output<'_> {
    pub fn id(&self) -> &str {
        self.id
    }

    pub fn write(&mut self, data: Json) {
        *self.value = data;
    }
}

mod nodes {
    use anyhow::Result;
    use serde_json::{json, Value as Json};

    use super::{Context, ProgramNode, Slots};

    #[derive(Debug)]
    pub struct Device {
        id: String,
    }

    impl Device {
        pub fn new(id: String) -> Device {
            Device { id }
        }
    }

    impl ProgramNode for Device {
        fn run(&mut self, context: &Context, slots: &mut Slots) -> Result<()> {
            for mut slot in slots.outputs() {
                let current = context.sources.get((self.id.clone(), slot.id().into()));

                match current.value() {
                    Ok(js) => slot.write(js.clone()),
                    Err(e) => anyhow::bail!("{e}"),
                }
            }

            Ok(())
        }
    }

    #[derive(Debug)]
    pub struct Target {
        id: (String, String),
    }

    impl Target {
        pub fn new(device_id: String, feature_id: String) -> Target {
            Target {
                id: (device_id, feature_id),
            }
        }
    }

    impl ProgramNode for Target {
        fn run(&mut self, _: &Context, slots: &mut Slots) -> Result<()> {
            let v = slots.input(&self.id.1)?.next().unwrap_or(&Json::Null);

            println!("Target {v:?}");

            Ok(())
        }
    }

    #[derive(Debug)]
    pub struct Or;

    impl ProgramNode for Or {
        fn run(&mut self, _: &Context, slots: &mut Slots) -> Result<()> {
            let out = slots
                .input("input")?
                .into_iter()
                .any(|v| matches!(v, Json::Bool(true)));

            slots.output("result", json!(out));

            Ok(())
        }
    }

    #[derive(Debug)]
    pub struct And;

    impl ProgramNode for And {
        fn run(&mut self, _: &Context, slots: &mut Slots) -> Result<()> {
            let out = slots
                .input("input")?
                .into_iter()
                .all(|v| matches!(v, Json::Bool(true)));

            slots.output("result", json!(out));

            Ok(())
        }
    }

    #[derive(Debug)]
    pub struct Xor;

    impl ProgramNode for Xor {
        fn run(&mut self, _: &Context, slots: &mut Slots) -> Result<()> {
            let ones = slots
                .input("input")?
                .into_iter()
                .filter(|v| matches!(v, Json::Bool(true)))
                .count();

            if ones == 1 {
                slots.output("result", json!(true));
            } else {
                slots.output("result", json!(false));
            }

            Ok(())
        }
    }

    #[derive(Debug)]
    pub struct Not;

    impl ProgramNode for Not {
        fn run(&mut self, _: &Context, slots: &mut Slots) -> Result<()> {
            let val = slots.input("input")?.next();

            let inverse = match val {
                Some(Json::Bool(true)) => json!(false),
                Some(Json::Bool(false)) => json!(true),
                _ => Json::Null,
            };

            slots.output("result", inverse);

            Ok(())
        }
    }
}

#[cfg(test)]
mod test {
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

        let mut program = auto.compile("", "state").unwrap();

        let t = Topic::default();

        let sources = Sources::default();

        sources.set(("amk".into(), "state".into()), Ok(json!(true)));
        sources.set(("amk".into(), "state_two".into()), Ok(json!(false)));
        sources.set(("two".into(), "state".into()), Ok(json!(false)));

        program.execute(&t, &sources).unwrap();
    }

    #[test]
    fn uniqe_device_node() {
        let p = r#"{"counter":6,"nodes":[{"id":0,"position":[6355,6027],"properties":{"tag":"Target"}},{"id":1,"position":[5415,5651],"properties":{"tag":"Device","content":"0x003c84fffe164750"}},{"id":2,"position":[5418,5902],"properties":{"tag":"Device","content":"0x003c84fffe164750"}},{"id":3,"position":[5433,6145],"properties":{"tag":"Device","content":"0x00158d00075a4e9c"}},{"id":4,"position":[5769,6041],"properties":{"tag":"Or"}},{"id":5,"position":[6055,5905],"properties":{"tag":"And"}}],"connections":[[[3,"occupancy"],[4,"input"]],[[2,"occupancy"],[4,"input"]],[[2,"occupancy"],[5,"input"]],[[4,"result"],[5,"input"]],[[5,"result"],[0,"state"]],[[1,"occupancy"],[5,"input"]]]}"#;
        let auto: Automation = serde_json::de::from_str(&p).unwrap();

        let program = auto.compile("", "state").unwrap();

        // We have a duplicate device node in the Automation, the compile should merge them
        assert_eq!(program.values.len(), 4);
    }
}
