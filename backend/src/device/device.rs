use std::sync::Arc;

use anyhow::Result;
use futures::Stream;
use serde_derive::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteRow, types::Json, Row, SqliteConnection};
use uuid::Uuid;

use crate::{bus::BUS, task::Task};

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
    pub fn spawn_tasks(&self, task: &Task) {
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
            .fetch(conn)
    }

    pub async fn load_by_id(device_id: &str, conn: &mut SqliteConnection) -> Result<Device> {
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
            .fetch_one(conn)
            .await?;

        Ok(dev)
    }

    /// High level call to create a generic device save to database and notify on the device bus that a device was added
    pub async fn create_generic(name: String, conn: &mut SqliteConnection) -> Result<Arc<Device>> {
        let id = Uuid::new_v4().to_string();

        let device = Device {
            id,
            name,
            device_type: DeviceType::Virtual {
                vty: VirtualType::Generic,
            },
            parent: None,
            task_spec: vec![],
        };

        device.save(conn).await?;

        let shared = Arc::new(device);

        shared.clone().notify_changed();

        Ok(shared)
    }

    /// Notify that this device has changed
    pub fn notify_changed(self: Arc<Self>) {
        BUS.device.add.publish(self)
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
