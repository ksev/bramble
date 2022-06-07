use bonsaidb::{
    core::schema::Schema,
    local::{
        config::{Builder, StorageConfiguration},
        AsyncDatabase,
    },
};
use tokio::sync::OnceCell;

use crate::device::Device;
use crate::integration::zigbee2mqtt;

#[derive(Debug, Schema)]
#[schema(name = "rome", collections = [Device, zigbee2mqtt::Server])]
struct RomeSchema;

/// Get a connection to the database store
pub async fn connection() -> &'static AsyncDatabase {
    static DB: OnceCell<AsyncDatabase> = OnceCell::const_new();
    DB.get_or_init(|| async {
        AsyncDatabase::open::<RomeSchema>(StorageConfiguration::new("rome.db"))
            .await
            .unwrap()
    })
    .await
}
