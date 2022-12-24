use std::collections::BTreeMap;

use anyhow::Result;
use async_graphql::Enum;
use futures::Stream;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{sqlite::SqliteRow, types::Json, Row, SqliteExecutor};

#[derive(Debug)]
pub struct Feature {
    pub id: String,
    pub name: String,
    pub direction: ValueDirection,
    pub kind: ValueKind,
    pub meta: BTreeMap<String, serde_json::Value>,
}

impl Feature {
    /// Load a value specs from database based in which device it belongs to
    pub fn load_by_device<'a, E>(
        device_id: &'a str,
        tx: E,
    ) -> impl Stream<Item = Result<Feature, sqlx::Error>> + 'a
    where
        E: SqliteExecutor<'a> + 'a,
    {
        sqlx::query(include_str!("../../sql/feature_by_device.sql"))
            .bind(device_id)
            .try_map(|row: SqliteRow| {
                let meta: Json<BTreeMap<String, serde_json::Value>> = row.try_get("meta")?;

                Ok(Feature {
                    id: row.try_get("id")?,
                    name: row.try_get("name")?,
                    direction: row.try_get("direction")?,
                    kind: row.try_get("kind")?,
                    meta: meta.0,
                })
            })
            .fetch(tx)
    }

    /// Read all readable features of a device
    /// This will also ignore an virtual feature that has been added
    pub fn load_by_device_readable<'a, E>(
        device_id: &'a str,
        tx: E,
    ) -> impl Stream<Item = Result<Feature, sqlx::Error>> + 'a
    where
        E: SqliteExecutor<'a> + 'a,
    {
        sqlx::query(include_str!(
            "../../sql/feature_by_device_readable_no_virtual.sql"
        ))
        .bind(device_id)
        .try_map(|row: SqliteRow| {
            let meta: Json<BTreeMap<String, serde_json::Value>> = row.try_get("meta")?;

            Ok(Feature {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                direction: row.try_get("direction")?,
                kind: row.try_get("kind")?,
                meta: meta.0,
            })
        })
        .fetch(tx)
    }

    /// Save a value spec
    pub async fn save<'a, E>(&'a self, device_id: &'a str, tx: E) -> Result<()>
    where
        E: SqliteExecutor<'a>,
    {
        sqlx::query(include_str!("../../sql/feature_insert.sql"))
            .bind(device_id)
            .bind(&self.id)
            .bind(&self.name)
            .bind(self.direction as u8)
            .bind(self.kind as u8)
            .bind(Json(&self.meta))
            .execute(tx)
            .await?;

        Ok(())
    }

    // TODO: this is a wierd API
    pub fn validate(&self, value: &Value) -> Result<Value, String> {
        let possible: Vec<String> = value
            .get("possible")
            .map(|v| serde_json::from_value(v.clone()))
            .and_then(|s| s.ok())
            .unwrap_or(vec![]);

        match (value, self.kind) {
            (Value::Null, _) => Ok(Value::Null),
            (Value::Bool(b), ValueKind::Bool) => Ok(Value::Bool(*b)),
            (Value::Number(n), ValueKind::Number { .. }) => Ok(Value::Number(n.clone())),
            (Value::String(s), ValueKind::String) => Ok(Value::String(s.clone())),

            (Value::String(s), ValueKind::State) => {
                if s.is_empty() {
                    // Treat empty strings a null, quite a few devices go back to an "empty", state
                    Ok(Value::Null)
                } else if possible.contains(s) {
                    Ok(Value::String(s.clone()))
                } else {
                    Err(format!("{} is not part of state set {:?}", s, possible))
                }
            }

            (Value::Array(_), _) => Err("Only descrete json values allowed, got array".into()),
            (Value::Object(_), _) => Err("Only descrete json values allowed, got array".into()),

            (a, b) => Err(format!(
                "Got value of {:?} expected value of kind {:?}",
                a, b
            )),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, sqlx::Type, Enum)]
#[repr(u8)]
pub enum ValueDirection {
    Source = 1,
    Sink = 2,
    SourceSink = 3,
}

impl TryFrom<u8> for ValueDirection {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => ValueDirection::Source,
            2 => ValueDirection::Sink,
            3 => ValueDirection::SourceSink,
            _ => anyhow::bail!("value {} is not a valid ValueDirection", value),
        })
    }
}

impl ValueDirection {
    #[allow(unused)]
    pub fn can_read(&self) -> bool {
        match self {
            ValueDirection::Source => true,
            ValueDirection::Sink => false,
            ValueDirection::SourceSink => true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, sqlx::Type, Enum, Copy, Clone)]
#[repr(u8)]
pub enum ValueKind {
    Bool = 0,
    Number = 1,
    State = 2,
    String = 3,
}

impl TryFrom<u8> for ValueKind {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => ValueKind::Bool,
            1 => ValueKind::Number,
            2 => ValueKind::State,
            3 => ValueKind::String,
            _ => anyhow::bail!("value {} is not a valid ValueDirection", value),
        })
    }
}
