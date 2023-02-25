mod api;
mod automation;
mod bus;
mod device;
mod http;
mod integration;
mod io;
mod task;

use std::str::FromStr;

use anyhow::Result;

use bus::GlobalBus;
use device::Sources;
use sqlx::sqlite::{SqliteAutoVacuum, SqliteConnectOptions, SqlitePoolOptions};
use task::Task;

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");

    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "backend=debug,tokio=error,runtime=error")
    }

    tracing_subscriber::fmt::init();

    let options = SqliteConnectOptions::from_str("sqlite:rome.sqlite3")?
        .create_if_missing(true)
        .auto_vacuum(SqliteAutoVacuum::Full);

    let pool = SqlitePoolOptions::new()
        .max_connections(3)
        .connect_with(options)
        .await?;

    let mut connection = pool.acquire().await?;
    sqlx::migrate!().run(&mut connection).await?;

    task::create_group(init, pool, Sources::default(), GlobalBus::default())
        .complete()
        .await?;

    Ok(())
}

async fn init(task: Task) -> Result<()> {
    task.spawn("http", http);
    task.spawn("mqtt_connections", io::mqtt::manage_connections);
    task.spawn("device_restore", device::restore);

    Ok(())
}

async fn http(t: Task) -> Result<()> {
    let addr = std::net::SocketAddr::from_str("127.0.0.1:8080")?;
    http::listen(t, addr).await?;

    Ok(())
}
