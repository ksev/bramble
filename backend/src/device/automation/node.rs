use crate::{
    program::{Inputs, Outputs, ProgramNode},
    strings::IString,
    value::ValueId,
};

use anyhow::Result;
use serde_json::{json, Value as Json};
use tracing::debug;

type NodeFn0 = fn(&Inputs, &mut Outputs) -> Result<()>;
type NodeFn1<T> = fn(&mut T, &Inputs, &mut Outputs) -> Result<()>;

struct AutomationNode0(NodeFn0);

impl ProgramNode for AutomationNode0 {
    fn run(&mut self, inputs: &Inputs, outputs: &mut Outputs) -> Result<()> {
        self.0(inputs, outputs)
    }
}

struct AutomationNode1<T>(T, NodeFn1<T>);

impl<T> ProgramNode for AutomationNode1<T>
where
    T: Send,
{
    fn run(&mut self, inputs: &Inputs, outputs: &mut Outputs) -> Result<()> {
        self.1(&mut self.0, inputs, outputs)
    }
}

pub fn node0(f: NodeFn0) -> Box<dyn ProgramNode> {
    Box::new(AutomationNode0(f))
}

pub fn node1<T>(data: T, f: NodeFn1<T>) -> Box<dyn ProgramNode>
where
    T: Send + 'static,
{
    Box::new(AutomationNode1(data, f))
}

pub fn device(id: &mut IString, input: &Inputs, output: &mut Outputs) -> Result<()> {
    for name in output.slots() {
        let id = ValueId::new(*id, name);
        let v = input.program(&id)?.clone();

        output.slot(name, v);
    }

    Ok(())
}

pub fn target(id: &mut ValueId, input: &Inputs, output: &mut Outputs) -> Result<()> {
    let v = input.slot_one(id.feature)?.unwrap_or(&Json::Null);

    output.program(id.clone(), v.clone());

    debug!("{:?} target with {:?}", id, v);

    Ok(())
}

pub fn is_null(input: &Inputs, output: &mut Outputs) -> Result<()> {
    let v = input.slot_one("input")?.unwrap_or(&Json::Null);

    output.slot("result", json!(v.is_null()));

    Ok(())
}

pub fn or(input: &Inputs, output: &mut Outputs) -> Result<()> {
    let out = input
        .slot("input")?
        .into_iter()
        .any(|v| matches!(v, Json::Bool(true)));

    output.slot("result", json!(out));

    Ok(())
}

pub fn and(input: &Inputs, output: &mut Outputs) -> Result<()> {
    let out = input
        .slot("input")?
        .into_iter()
        .all(|v| matches!(v, Json::Bool(true)));

    output.slot("result", json!(out));

    Ok(())
}

pub fn xor(input: &Inputs, output: &mut Outputs) -> Result<()> {
    let ones = input
        .slot("input")?
        .into_iter()
        .filter(|v| matches!(v, Json::Bool(true)))
        .count();

    if ones == 1 {
        output.slot("result", json!(true));
    } else {
        output.slot("result", json!(false));
    }

    Ok(())
}

pub fn not(input: &Inputs, output: &mut Outputs) -> Result<()> {
    let val = input.slot_one("input")?;

    let inverse = match val {
        Some(Json::Bool(true)) => json!(false),
        Some(Json::Bool(false)) => json!(true),
        _ => Json::Null,
    };

    output.slot("result", inverse);

    Ok(())
}

pub fn latch(high: &mut bool, input: &Inputs, output: &mut Outputs) -> Result<()> {
    let low = json!(false);

    let signal = input.slot_or("input", &low);
    let reset = input.slot_or("reset", &low);

    if reset == &json!(true) {
        *high = false;
    }

    let is_high = signal == &json!(true);
    let out = json!(is_high || *high);

    output.slot("result", out);

    *high = is_high;

    Ok(())
}

pub fn toggle(high: &mut bool, input: &Inputs, output: &mut Outputs) -> Result<()> {
    let signal = input.slot_one("input")?.unwrap_or(&Json::Bool(false));
    let is_true = matches!(signal, Json::Bool(true));

    if is_true {
        *high = !*high;
    }

    output.slot("result", json!(high));

    Ok(())
}

pub fn equals(input: &Inputs, output: &mut Outputs) -> Result<()> {
    let this = input.slot_or("input", &Json::Null);
    let other = input.slot_or("other", &Json::Null);

    output.slot("result", json!(this == other));

    Ok(())
}

pub fn static_value(value: &mut Json, _: &Inputs, output: &mut Outputs) -> Result<()> {
    output.slot("value", value.clone());

    Ok(())
}
