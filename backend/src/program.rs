use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    ops::{Index, Range},
};

use anyhow::{ensure, Result};
use serde_json::Value as Json;

use crate::strings::IString;

pub trait ProgramNode: Send + std::fmt::Debug {
    fn run(&mut self, slots: &mut Slots) -> Result<()>;
}

#[derive(Debug)]
pub struct Program {
    nodes: Vec<Box<dyn ProgramNode>>,

    outputs: MVec<IString>,
    inputs: MVec<(IString, usize)>,
    input_value_index: MVec<usize>,

    values: Vec<Json>,
}

impl Program {
    pub fn new(
        nodes: Vec<Box<dyn ProgramNode>>,
        connections: Vec<((usize, IString), (usize, IString))>,
    ) -> Result<Program> {
        // Make sure we execute the nodes in the right order
        let (nodes, connections) = topological_sort(nodes, connections)?;

        let mut incoming: HashMap<usize, BTreeMap<IString, Vec<(usize, IString)>>> = HashMap::new();
        let mut outgoing: HashMap<usize, BTreeSet<IString>> = HashMap::new();

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

        let mut outputs = MVec::new();
        let mut inputs = MVec::new();
        let mut input_value_index = MVec::new();

        // Build indecies for node outputs
        for i in 0..nodes.len() {
            let n = outgoing.get(&i);
            let slots = n.iter().flat_map(|&out| out.iter()).copied();

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
                (slot, input_value_index.push(vs))
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

        Ok(p)
    }

    pub fn execute(&mut self) -> Result<()> {
        // Reset data from the last run
        for v in self.values.iter_mut() {
            *v = Json::Null;
        }

        for (index, node) in self.nodes.iter_mut().enumerate() {
            let mut slots = Slots {
                index,

                outputs: &self.outputs,
                inputs: &self.inputs,

                values: &mut self.values[..],
                input_value_index: &self.input_value_index,
            };

            node.run(&mut slots)?;
        }

        Ok(())
    }
}

fn topological_sort(
    nodes: Vec<Box<dyn ProgramNode>>,
    connections: Vec<((usize, IString), (usize, IString))>,
) -> Result<(
    Vec<Box<dyn ProgramNode>>,
    Vec<((usize, IString), (usize, IString))>,
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
        .filter(|n| !incoming.contains_key(n))
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

    outputs: &'a MVec<IString>,
    inputs: &'a MVec<(IString, usize)>,
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
            .map(|(n, v)| Output { id: *n, value: v })
    }

    pub fn output<T>(&mut self, id: T, value: Json)
    where
        T: Into<IString>,
    {
        let id = id.into();
        let start = self.outputs.range(self.index).start;
        let outputs = &self.outputs[self.index];

        let idx = outputs.iter().position(|s| *s == id);

        if let Some(idx) = idx {
            self.values[start + idx] = value;
        }
    }

    pub fn input<T>(&self, id: T) -> Result<impl Iterator<Item = &Json>>
    where
        T: Into<IString>,
    {
        let id = id.into();
        let inputs = &self.inputs[self.index];

        let Some((_, value_index)) = inputs.iter().find(|(s, _)| *s == id) else {
            anyhow::bail!("no input named {:?}", id);  
        };

        let iter = self.input_value_index[*value_index]
            .iter()
            .map(|&i| &self.values[i]);

        Ok(iter)
    }

    pub fn input_one<T>(&self, id: T) -> Result<Option<&Json>>
    where
        T: Into<IString>,
    {
        Ok(self.input(id)?.next())
    }

    pub fn input_or<T>(&self, id: T, def: Json) -> Json
    where
        T: Into<IString>,
    {
        let Ok(mut it) = self.input(id) else {
            return def;
        };

        it.next().cloned().unwrap_or(def)
    }

    pub fn inputs(&self) -> impl Iterator<Item = IString> + '_ {
        self.inputs[self.index].iter().map(|(id, _)| *id)
    }
}

pub struct Output<'a> {
    id: IString,
    value: &'a mut Json,
}

impl Output<'_> {
    pub fn id(&self) -> IString {
        self.id
    }

    pub fn write(&mut self, data: Json) {
        *self.value = data;
    }
}

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
