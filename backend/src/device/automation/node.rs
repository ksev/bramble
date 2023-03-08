use crate::{
    program::{ProgramNode, Slots},
    strings::IString,
    value::{self, ValueId},
};

use anyhow::Result;
use serde_json::{json, Value as Json};
use tracing::debug;

#[derive(Debug)]
pub struct Device {
    id: IString,
}

impl Device {
    pub fn new(id: IString) -> Device {
        Device { id }
    }
}

impl ProgramNode for Device {
    fn run(&mut self, slots: &mut Slots) -> Result<()> {
        for mut slot in slots.outputs() {
            let id = ValueId::new(self.id, slot.id());
            let current = value::current(id);

            match current.value() {
                Ok(js) => slot.write(js.clone()),
                Err(e) => anyhow::bail!("{e}"),
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct IsNull;

impl ProgramNode for IsNull {
    fn run(&mut self, slots: &mut Slots) -> Result<()> {
        let v = slots.input_one("input")?.unwrap_or(&Json::Null);

        slots.output("result", json!(v.is_null()));

        Ok(())
    }
}

#[derive(Debug)]
pub struct Target {
    id: ValueId,
}

impl Target {
    pub fn new(id: ValueId) -> Target {
        Target { id }
    }
}

impl ProgramNode for Target {
    fn run(&mut self, slots: &mut Slots) -> Result<()> {
        let v = slots.input_one(self.id.feature)?.unwrap_or(&Json::Null);

        value::push(self.id, v.clone());

        debug!("{:?} target with {:?}", self.id, v);

        Ok(())
    }
}

#[derive(Debug)]
pub struct Or;

impl ProgramNode for Or {
    fn run(&mut self, slots: &mut Slots) -> Result<()> {
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
    fn run(&mut self, slots: &mut Slots) -> Result<()> {
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
    fn run(&mut self, slots: &mut Slots) -> Result<()> {
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
    fn run(&mut self, slots: &mut Slots) -> Result<()> {
        let val = slots.input_one("input")?;

        let inverse = match val {
            Some(Json::Bool(true)) => json!(false),
            Some(Json::Bool(false)) => json!(true),
            _ => Json::Null,
        };

        slots.output("result", inverse);

        Ok(())
    }
}

#[derive(Debug)]
pub struct Latch {
    high: bool,
}

impl Latch {
    pub fn new() -> Latch {
        Latch { high: false }
    }
}

impl ProgramNode for Latch {
    fn run(&mut self, slots: &mut Slots) -> Result<()> {
        let input = slots.input_or("input", json!(false));
        let reset = slots.input_or("reset", json!(false));

        if reset == json!(true) {
            self.high = false;
        }

        let is_high = input == json!(true);
        let out = json!(is_high || self.high);

        slots.output("result", out);

        self.high = is_high;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Toggle {
    high: bool,
}

impl Toggle {
    pub fn new() -> Toggle {
        Toggle { high: false }
    }
}

impl ProgramNode for Toggle {
    fn run(&mut self, slots: &mut Slots) -> Result<()> {
        let input = slots.input_one("input")?.unwrap_or(&Json::Bool(false));
        let is_true = matches!(input, Json::Bool(true));

        if is_true {
            self.high = !self.high;
        }

        slots.output("result", json!(self.high));

        Ok(())
    }
}

#[derive(Debug)]
pub struct Equals;

impl ProgramNode for Equals {
    fn run(&mut self, slots: &mut Slots) -> Result<()> {
        let input = slots.input_or("input", Json::Null);
        let other = slots.input_or("other", Json::Null);

        slots.output("result", json!(input == other));

        Ok(())
    }
}

#[derive(Debug)]
pub struct Value(pub Json);

impl ProgramNode for Value {
    fn run(&mut self, slots: &mut Slots) -> Result<()> {
        slots.output("value", self.0.clone());

        Ok(())
    }
}
