use crate::{
    program::{ProgramNode, Slots},
    strings::IString,
    value::{self, ValueId},
};
use anyhow::Result;
use serde_json::{json, Value as Json};

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
        let v = slots.input(self.id.feature)?.next().unwrap_or(&Json::Null);

        value::push(self.id, v.clone());

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
