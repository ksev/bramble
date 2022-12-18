use anyhow::Result;
use futures::Stream;
use serde_derive::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, types::Json, Row, SqliteExecutor};

use crate::task::Task;

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
    pub task_spec: Vec<TaskSpec>,
}

impl Device {
    pub fn spawn_tasks(&self, task: &mut Task) {
        for spec in &self.task_spec {
            match spec {
                TaskSpec::Zigbee2Mqtt(server) => {
                    let label = format!("zigbee2mqtt:{}:{}", server.host, server.port);

                    if task.has_task(&label) {
                        // There is no need to reboot the task just ignore
                        continue;
                    }

                    task.spawn_with_argument(
                        label,
                        (self.id.clone(), server.clone()),
                        crate::integration::zigbee2mqtt::zigbee2mqtt_update,
                    );
                }
                TaskSpec::Zigbee2MqttDevice(_) => {
                    let label = format!("{}/Zigbee2MqttDevice", self.id);

                    task.spawn_with_argument(
                        label,
                        self.id.clone(),
                        crate::integration::zigbee2mqtt::zigbee2mqtt_device,
                    )
                }
            }
        }
    }

    /// Save the device to storage
    pub async fn save<'a, E>(&'a self, tx: E) -> Result<()>
    where
        E: SqliteExecutor<'a>,
    {
        sqlx::query(include_str!("../../sql/device_insert.sql"))
            .bind(&self.id)
            .bind(&self.name)
            .bind(Json(&self.device_type))
            .bind(&self.parent)
            .bind(Json(&self.task_spec))
            .execute(tx)
            .await?;

        Ok(())
    }

    pub fn all<'a, E>(tx: E) -> impl Stream<Item = Result<Device, sqlx::Error>> + 'a
    where
        E: SqliteExecutor<'a> + 'a,
    {
        sqlx::query(include_str!("../../sql/device_all.sql"))
            .try_map(|row: SqliteRow| {
                let Json(task_spec): Json<Vec<TaskSpec>> = row.try_get("task_spec")?;
                let Json(device_type): Json<DeviceType> = row.try_get("type")?;

                Ok(Device {
                    id: row.try_get("id")?,
                    name: row.try_get("name")?,
                    parent: row.try_get("parent")?,
                    device_type,
                    task_spec,
                })
            })
            .fetch(tx)
    }

    pub async fn load_by_id<'a, E>(device_id: &str, tx: E) -> Result<Device>
    where
        E: SqliteExecutor<'a>,
    {
        let dev = sqlx::query(include_str!("../../sql/device_by_id.sql"))
            .bind(device_id)
            .try_map(|row: SqliteRow| {
                let Json(task_spec): Json<Vec<TaskSpec>> = row.try_get("task_spec")?;
                let Json(device_type): Json<DeviceType> = row.try_get("type")?;

                Ok(Device {
                    id: row.try_get("id")?,
                    name: row.try_get("name")?,
                    parent: row.try_get("parent")?,
                    device_type,
                    task_spec,
                })
            })
            .fetch_one(tx)
            .await?;

        Ok(dev)
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum DeviceType {
    #[serde(rename = "intergration")]
    Integration { name: String },
    #[serde(rename = "hardware")]
    Hardware,
    #[serde(rename = "virtual")]
    Virtual,
}
