
/*

let mut interner = StringInterner::default();

    let mut auto = Automation {
        nodes: vec![
            Box::new(Rand),
            Box::new(Output),

            Box::new(Rand),
            Box::new(Output),

            Box::new(Rand),
            Box::new(Output),

            Box::new(Rand),
            Box::new(Output),

            Box::new(Rand),
            Box::new(Output),

            Box::new(Max),
            Box::new(Output), 
        ],

        slots: vec![
            BTreeMap::from([(interner.get_or_intern("value"), smallvec![0])]),
            BTreeMap::from([(interner.get_or_intern("value"), smallvec![0])]),

            BTreeMap::from([(interner.get_or_intern("value"), smallvec![1])]),
            BTreeMap::from([(interner.get_or_intern("value"), smallvec![1])]),

            BTreeMap::from([(interner.get_or_intern("value"), smallvec![2])]),
            BTreeMap::from([(interner.get_or_intern("value"), smallvec![2])]),

            BTreeMap::from([(interner.get_or_intern("value"), smallvec![3])]),
            BTreeMap::from([(interner.get_or_intern("value"), smallvec![3])]),

            BTreeMap::from([(interner.get_or_intern("value"), smallvec![4])]),
            BTreeMap::from([(interner.get_or_intern("value"), smallvec![4])]),

            BTreeMap::from([
                (interner.get_or_intern("numbers"), smallvec![0,1,2,3,4]),
                (interner.get_or_intern("max"), smallvec![5]),
            ]),
            BTreeMap::from([(interner.get_or_intern("value"), smallvec![5])]),
        ],

        values: HashMap::new(),
    };

    loop {

        auto.run(&mut interner)?;

        println!("=======================");
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

i
*/


#[derive(Debug)]
enum Value {
    Number(f32),
    Bool(bool),
}

impl TryFrom<&Value> for f32 {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Number(v) => Ok(*v),
            _ => Err(anyhow::anyhow!("Slot value is of wrong type")),
        }
    }
}

trait Node {
    fn eval(&mut self, context: &mut NodeContext) -> Result<()>;
}

struct Rand;

impl Node for Rand {
    fn eval(&mut self, context: &mut NodeContext) -> Result<()> {
        context.set_output("value", Value::Number(random()))
    }
}

struct Output;

impl Node for Output {
    fn eval(&mut self, context: &mut NodeContext) -> Result<()> {
        let val = context.input("value")?;

        println!("{val:?}");

        Ok(())
    }
}

struct IsOverHalf;

impl Node for IsOverHalf {
    fn eval(&mut self, context: &mut NodeContext) -> Result<()> {
        if let Some(val) = context.input("number")? {
            let num: f32 = val.try_into()?;
            context.set_output("value", Value::Bool(num > 0.5))?;
        }

        Ok(())
    }
}

struct Max;

impl Node for Max {
    fn eval(&mut self, context: &mut NodeContext) -> Result<()> {
        let max = context
            .all_input("numbers")?
            .filter_map(|v| v.try_into().ok())
            .reduce(f32::max);

        if let Some(max) = max {
            context.set_output("max", Value::Number(max))?;
        }

        Ok(())
    }
}

struct NodeContext<'a> {
    interner: &'a mut StringInterner,
    slots: &'a BTreeMap<SymbolU32, SmallVec<[usize; 1]>>,
    values: &'a mut HashMap<usize, Value>,
}

impl NodeContext<'_> {
    fn set_output(&mut self, name: &str, value: Value) -> Result<()> {
        let target = self
            .slots
            .get(&self.interner.get_or_intern(name))
            .ok_or_else(|| anyhow::anyhow!("Node does not have output slot name {name}"))?[0];

        self.values.insert(target, value);

        Ok(())
    }

    fn input(&mut self, name: &str) -> Result<Option<&Value>> {
        let from = self
            .slots
            .get(&self.interner.get_or_intern(name))
            .ok_or_else(|| anyhow::anyhow!("Node does not have input slot name {name}"))?[0];

        Ok(self.values.get(&from))
    }

    fn all_input<'a>(&'a mut self, name: &str) -> Result<ValuesIterator<'a>> {
        let svec = self
            .slots
            .get(&self.interner.get_or_intern(name))
            .ok_or_else(|| anyhow::anyhow!("Node does not have input slot name {name}"))?;

        Ok(ValuesIterator {
            indexes: svec,
            values: self.values,
            idx: 0,
        })
    }
}

struct ValuesIterator<'a> {
    indexes: &'a SmallVec<[usize; 1]>,
    values: &'a HashMap<usize, Value>,
    idx: usize,
}

impl<'a> Iterator for ValuesIterator<'a> {
    type Item = &'a Value;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let idx = self.indexes.get(self.idx)?;
            self.idx += 1;

            if let Some(val) = self.values.get(&idx) {
                return Some(val);
            }
        }
    }
}
 
struct Automation {
    /// All the nodes that are in the Automation
    /// These need to be topologically sorted for execution
    nodes: Vec<Box<dyn Node>>,
    /// A list of all the slot mappings per node we only ever reference the value on an output slot
    slots: Vec<BTreeMap<SymbolU32, SmallVec<[usize; 1]>>>,
    /// Output slot values, id is monotonic based on the entire automation
    values: HashMap<usize, Value>,
}

impl Automation {
    fn run(&mut self, interner: &mut StringInterner) -> Result<()> {
        // Run assumes the nodes are topologically sorted
        for idx in 0..self.nodes.len() {
            let node = &mut self.nodes[idx];
            let slots = &self.slots[idx];

            let mut ctx = NodeContext {
                interner,
                slots,
                values: &mut self.values,
            };

            node.eval(&mut ctx)?;
        }

        Ok(())
    }
}
