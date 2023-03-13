use std::collections::{BTreeMap, BTreeSet, HashMap};

use anyhow::{ensure, Result};
use serde_json::Value as Json;

use crate::{strings::IString, value::ValueId};

pub trait ProgramNode: Send {
    fn run(&mut self, inputs: &Inputs, outputs: &mut Outputs) -> Result<()>;
}

pub struct Inputs<'a> {
    program: &'a BTreeMap<ValueId, Json>,
    slots: &'a BTreeMap<IString, Vec<(u32, IString)>>,

    values: &'a BTreeMap<(u32, IString), Json>,
}

impl Inputs<'_> {
    pub fn slot<T>(&self, name: T) -> Result<impl Iterator<Item = &Json>>
    where
        T: Into<IString>,
    {
        let n = name.into();
        let Some(list) = self.slots.get(&n) else {
            anyhow::bail!("no input named {:?}", n);
        };

        let iter = list.iter().map(|k| &self.values[k]);

        Ok(iter)
    }

    pub fn slot_one<T>(&self, name: T) -> Result<Option<&Json>>
    where
        T: Into<IString>,
    {
        Ok(self.slot(name)?.next())
    }

    pub fn slot_or<'a, T>(&'a self, name: T, value: &'a Json) -> &Json
    where
        T: Into<IString>,
    {
        if let Ok(Some(v)) = self.slot_one(name) {
            v
        } else {
            value
        }
    }

    pub fn program(&self, id: &ValueId) -> Result<&Json> {
        let Some(json) = self.program.get(&id) else {
            anyhow::bail!("No such program input {:?}", id);
        };

        Ok(json)
    }
}

pub struct Outputs<'a> {
    id: u32,
    program: &'a mut BTreeMap<ValueId, Json>,
    slots: &'a BTreeSet<IString>,

    values: &'a mut BTreeMap<(u32, IString), Json>,
}

impl<'a> Outputs<'a> {
    pub fn slot<T>(&mut self, name: T, value: Json)
    where
        T: Into<IString>,
    {
        self.values.insert((self.id, name.into()), value);
    }

    pub fn slots(&self) -> Vec<IString> {
        self.slots.iter().copied().collect()
    }

    pub fn program(&mut self, id: ValueId, value: Json) {
        self.program.insert(id, value);
    }
}

#[derive(Debug)]
struct Slots {
    inputs: BTreeMap<IString, Vec<(u32, IString)>>,
    outputs: BTreeSet<IString>,
}

#[derive(Default)]
pub struct Program {
    /// All the steps of the program in topological order
    steps: Vec<(u32, Slots, Box<dyn ProgramNode>)>,
}

impl Program {
    pub fn new(
        nodes: Vec<(u32, Box<dyn ProgramNode>)>,
        connections: Vec<((u32, IString), (u32, IString))>,
    ) -> Result<Program> {
        // Make sure we execute the nodes in the right order
        let nodes = topological_sort(nodes, &connections)?;

        // Incoming slots on every node, the map has the slot name as key and a list of output slots
        // the input is connected
        let mut incoming: HashMap<u32, BTreeMap<IString, Vec<(u32, IString)>>> = HashMap::new();
        let mut outgoing: HashMap<u32, BTreeSet<IString>> = HashMap::new();

        for ((f, fs), (t, ts)) in &connections {
            // For inputs we need to know the name of each input on the nodes
            // but also how they are connected to outputs so we can
            // setup the indexes for inputs to read the correct output data
            incoming
                .entry(*t)
                .or_default()
                .entry(*ts)
                .or_default()
                .push((*f, *fs));

            // For outputs we only need two know the name of the outputs on each node
            // to build the index
            outgoing.entry(*f).or_default().insert(*fs);
        }

        // Assemble the steps of the program
        let steps = nodes
            .into_iter()
            .map(|(id, node)| {
                let inputs = incoming.remove(&id).unwrap_or_default();
                let outputs = outgoing.remove(&id).unwrap_or_default();

                (id, Slots { inputs, outputs }, node)
            })
            .collect();

        Ok(Program { steps })
    }

    /// The number of steps to evaluate the program
    pub fn steps(&self) -> usize {
        self.steps.len()
    }

    pub fn execute(
        &mut self,
        program_input: &BTreeMap<ValueId, Json>,
    ) -> Result<BTreeMap<ValueId, Json>> {
        let mut program_output = BTreeMap::new();

        let mut slot_inputs = BTreeMap::new();
        let mut slot_outputs = BTreeMap::new();

        for (id, slots, node) in self.steps.iter_mut() {
            let inputs = Inputs {
                program: program_input,
                slots: &slots.inputs,
                values: &slot_inputs,
            };

            let mut outputs = Outputs {
                id: *id,
                program: &mut program_output,
                slots: &slots.outputs,
                values: &mut slot_outputs,
            };

            node.run(&inputs, &mut outputs)?;

            // Move output values into input values, this will leave outputs empty
            slot_inputs.append(&mut slot_outputs);
        }

        Ok(program_output)
    }
}

fn topological_sort(
    nodes: Vec<(u32, Box<dyn ProgramNode>)>,
    connections: &[((u32, IString), (u32, IString))],
) -> Result<Vec<(u32, Box<dyn ProgramNode>)>> {
    let mut incoming: HashMap<u32, BTreeSet<u32>> = HashMap::new();
    let mut outgoing: HashMap<u32, BTreeSet<u32>> = HashMap::new();

    for ((f, _), (t, _)) in connections {
        incoming.entry(*t).or_default().insert(*f);
        outgoing.entry(*f).or_default().insert(*t);
    }

    let mut start: Vec<_> = nodes
        .iter()
        .map(|(id, _)| *id)
        // Only nodes with no incoming connections to start with.
        .filter(|id| !incoming.contains_key(&id))
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

    let mut n: HashMap<_, _> = nodes.into_iter().collect();

    let mut nodes = Vec::with_capacity(n.len());

    for &i in &order {
        let Some(node) = n.remove(&i) else {
            anyhow::bail!("Could not lookup node from order, this is a bug");
        };

        nodes.push((i, node));
    }

    anyhow::ensure!(n.is_empty(), "Node lookup table is not empty");

    Ok(nodes)
}
