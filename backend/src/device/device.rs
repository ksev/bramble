use anyhow::Result;
use futures::Stream;
use serde_derive::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, types::Json, Row, SqliteConnection};

use super::TaskSpec;

#[derive(Debug)]
pub struct Device {
    pub id: String,
    /// Name of the device
    pub name: String,
    /// Define what type of device this is
    pub device_type: DeviceType,
    /// Defined if some other device is controlling this device
    pub parent: Option<String>,
    /// What do we need to create in order for data to flow into sources and out of sinks
    /// This needs to be plain data so we can recreate the tasks on restart
    pub task_spec: TaskSpec,
}

impl Device {
    /// Save the device to storage
    pub async fn save(&self, conn: &mut SqliteConnection) -> Result<()> {
        sqlx::query(include_str!("../../sql/device_insert.sql"))
            .bind(&self.id)
            .bind(&self.name)
            .bind(Json(&self.device_type))
            .bind(&self.parent)
            .bind(Json(&self.task_spec))
            .execute(conn)
            .await?;

        Ok(())
    }

    pub fn all(
        conn: &mut SqliteConnection,
    ) -> impl Stream<Item = Result<Device, sqlx::Error>> + '_ {
        sqlx::query(include_str!("../../sql/device_all.sql"))
            .try_map(|row: SqliteRow| {
                let Json(task_spec): Json<TaskSpec> = row.try_get("task_spec")?;
                let Json(device_type): Json<DeviceType> = row.try_get("type")?;

                Ok(Device {
                    id: row.try_get("id")?,
                    name: row.try_get("name")?,
                    parent: row.try_get("parent")?,
                    device_type,
                    task_spec,
                })
            })
            .fetch(conn)
    }

    pub async fn load_by_id(device_id: &str, conn: &mut SqliteConnection) -> Result<Device> {
        let dev = sqlx::query(include_str!("../../sql/device_by_id.sql"))
            .bind(device_id)
            .try_map(|row: SqliteRow| {
                let Json(task_spec): Json<TaskSpec> = row.try_get("task_spec")?;
                let Json(device_type): Json<DeviceType> = row.try_get("type")?;

                Ok(Device {
                    id: row.try_get("id")?,
                    name: row.try_get("name")?,
                    parent: row.try_get("parent")?,
                    device_type,
                    task_spec,
                })
            })
            .fetch_one(conn)
            .await?;

        Ok(dev)
    }

    /// High level call to create a generic device save to database and notify on the device bus that a device was added
    pub async fn create_generic(name: String, conn: &mut SqliteConnection) -> Result<Device> {
        let id = super::random_id("virtual");

        let device = Device {
            id,
            name,
            device_type: DeviceType::Virtual {
                vty: VirtualType::Generic,
            },
            parent: None,
            task_spec: TaskSpec::NoOp,
        };

        device.save(conn).await?;

        Ok(device)
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum DeviceType {
    #[serde(rename = "integration")]
    Integration { name: String },
    #[serde(rename = "hardware")]
    Hardware,
    #[serde(rename = "virtual")]
    Virtual { vty: VirtualType },
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum VirtualType {
    #[serde(rename = "generic")]
    Generic,
    #[serde(rename = "button")]
    Button,
    #[serde(rename = "slider")]
    Slider,
    #[serde(rename = "toggle")]
    Toggle,
}
