use std::collections::BTreeMap;

use anyhow::Result;
use async_graphql::Enum;
use futures::Stream;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use sqlx::{sqlite::SqliteRow, types::Json as SqlJson, Row, SqliteConnection};
use uuid::Uuid;

use super::Automation;

#[derive(Debug)]
pub struct Feature {
    pub id: String,
    pub name: String,
    pub direction: ValueDirection,
    pub kind: ValueKind,
    pub meta: BTreeMap<String, serde_json::Value>,
    pub automate: Option<Automation>,
}

impl Feature {
    /// Load a value specs from database based in which device it belongs to
    pub fn load_by_device<'a>(
        device_id: &'a str,
        conn: &'a mut SqliteConnection,
    ) -> impl Stream<Item = Result<Feature, sqlx::Error>> + 'a {
        sqlx::query(include_str!("../../sql/feature_by_device.sql"))
            .bind(device_id)
            .try_map(|row: SqliteRow| {
                let meta: SqlJson<BTreeMap<String, Json>> = row.try_get("meta")?;
                let auto: Option<SqlJson<Automation>> = row.try_get("automate")?;

                Ok(Feature {
                    id: row.try_get("id")?,
                    name: row.try_get("name")?,
                    direction: row.try_get("direction")?,
                    kind: row.try_get("kind")?,
                    meta: meta.0,
                    automate: auto.map(|j| j.0),
                })
            })
            .fetch(conn)
    }

    /// Read all readable features of a device
    /// This will also ignore an virtual feature that has been added
    pub fn load_by_device_readable<'a>(
        device_id: &'a str,
        conn: &'a mut SqliteConnection,
    ) -> impl Stream<Item = Result<Feature, sqlx::Error>> + 'a {
        sqlx::query(include_str!(
            "../../sql/feature_by_device_readable_no_virtual.sql"
        ))
        .bind(device_id)
        .try_map(|row: SqliteRow| {
            let meta: SqlJson<BTreeMap<String, Json>> = row.try_get("meta")?;
            let auto: Option<SqlJson<Automation>> = row.try_get("automate")?;

            Ok(Feature {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                direction: row.try_get("direction")?,
                kind: row.try_get("kind")?,
                meta: meta.0,
                automate: auto.map(|j| j.0),
            })
        })
        .fetch(conn)
    }

    pub async fn load(
        device_id: &str,
        feature_id: &str,
        conn: &mut SqliteConnection,
    ) -> Result<Feature> {
        let fet = sqlx::query(include_str!("../../sql/feature_load.sql"))
            .bind(device_id)
            .bind(feature_id)
            .try_map(|row: SqliteRow| {
                let meta: SqlJson<BTreeMap<String, Json>> = row.try_get("meta")?;
                let auto: Option<SqlJson<Automation>> = row.try_get("automate")?;

                Ok(Feature {
                    id: row.try_get("id")?,
                    name: row.try_get("name")?,
                    direction: row.try_get("direction")?,
                    kind: row.try_get("kind")?,
                    meta: meta.0,
                    automate: auto.map(|j| j.0),
                })
            })
            .fetch_one(conn)
            .await?;

        Ok(fet)
    }

    pub fn load_automations(
        conn: &mut SqliteConnection,
    ) -> impl Stream<Item = Result<(String, String, Automation), sqlx::Error>> + '_ {
        sqlx::query(include_str!("../../sql/feature_automation.sql"))
            .try_map(|row: SqliteRow| {
                let auto: SqlJson<Automation> = row.try_get("automate")?;
                Ok((row.try_get("device")?, row.try_get("id")?, auto.0))
            })
            .fetch(conn)
    }

    /// Save a value spec
    pub async fn save(&self, device_id: &str, conn: &mut SqliteConnection) -> Result<()> {
        sqlx::query(include_str!("../../sql/feature_insert.sql"))
            .bind(device_id)
            .bind(&self.id)
            .bind(&self.name)
            .bind(self.direction as u8)
            .bind(self.kind as u8)
            .bind(SqlJson(&self.meta))
            .bind(self.automate.as_ref().map(SqlJson))
            .execute(conn)
            .await?;

        Ok(())
    }

    /// High level API to attach a Virtual feature to a device and notify that the device has changed
    pub async fn attach_virtual(
        device_id: &str,
        name: String,
        kind: ValueKind,
        meta: BTreeMap<String, Json>,
        conn: &mut SqliteConnection,
    ) -> Result<Feature> {
        let id = Uuid::new_v4().to_string();

        let feature = Feature {
            id,
            name,
            direction: ValueDirection::SourceSink,
            kind,
            meta,
            automate: None,
        };

        feature.save(device_id, conn).await?;

        Ok(feature)
    }

    /// Validate a if [`Value`] is Valid for this Feature
    pub fn validate(&self, value: &Json) -> Result<Json, String> {
        let possible: Vec<String> = self
            .meta
            .get("possible")
            .map(|v| serde_json::from_value(v.clone()))
            .and_then(|s| s.ok())
            .unwrap_or(vec![]);

        match (value, self.kind) {
            (Json::Null, _) => Ok(Json::Null),
            (Json::Bool(b), ValueKind::Bool) => Ok(Json::Bool(*b)),
            (Json::Number(n), ValueKind::Number { .. }) => Ok(Json::Number(n.clone())),
            (Json::String(s), ValueKind::String) => Ok(Json::String(s.clone())),

            (Json::String(s), ValueKind::State) => {
                if s.is_empty() {
                    // Treat empty strings a null, quite a few devices go back to an "empty", state
                    Ok(Json::Null)
                } else if possible.contains(s) {
                    Ok(Json::String(s.clone()))
                } else {
                    Err(format!("{} is not part of state set {:?}", s, possible))
                }
            }

            (Json::Array(_), _) => Err("Only descrete json values allowed, got array".into()),
            (Json::Object(_), _) => Err("Only descrete json values allowed, got array".into()),

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
