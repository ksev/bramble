use anyhow::Result;
use bonsaidb::core::schema::{Collection, SerializedCollection};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{actor::UntypedAddr, database};

use super::TaskSpec;

static DEVICE_TASKS: Lazy<DashMap<String, Vec<UntypedAddr>>> = Lazy::new(DashMap::new);

#[derive(Debug, Serialize, Deserialize, Collection)]
#[collection(name = "devices", primary_key = String, natural_id = |d: &Device| Some(d.id.clone()))]
pub struct Device {
    pub id: String,
    /// Name of the device
    pub name: String,
    /// The last time we saw a live update from the device
    pub last_seen: Option<DateTime<Utc>>,
    /// What do we need to create in order for data to flow into sources and out of sinks
    /// This needs to be plain data so we can recreate the tasks on restart
    pub task_spec: Vec<TaskSpec>,
    /// Which data can this device generate
    pub sources: Vec<Source>,
    /// Which data can this device receive
    pub sinks: Vec<Sink>,
}

impl Device {
    pub async fn remove(id: &str) -> Result<()> {
        //work::close_job(job_key).await?;

        let conn = database::connection().await;

        if let Some(d) = Device::get_async(id, conn).await? {
            d.delete_async(conn).await?;
        }

        Ok(())
    }

    /// Sync devices from an integration, will Add/Delete/Modify based on need
    pub async fn integration_sync(
        integration: &str,
        devices: impl Iterator<Item = Device>,
    ) -> Result<()> {
        
        for device in devices {
            let tasks = device.task_spec.into_iter().map(|t| t.start().unwrap()).collect();
            DEVICE_TASKS.insert(device.id.clone(), tasks);
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Value {
    Bool(bool),
    Number(f32),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Values {
    Single(Value),
    Composite(Vec<(String, Value)>),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Sink {
    name: String,
    value: Values,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Source {
    name: String,
    value: Values,
}
