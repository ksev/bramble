mod api;
mod db;
mod device;
mod http;
mod integration;
mod io;
mod program;
mod strings;
mod task;
mod topic;
mod value;

use std::str::FromStr;

use anyhow::Result;

use task::Task;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let mut connection = db::connection().await?;
    sqlx::migrate!().run(&mut connection).await?;

    task::create_group(init).complete().await?;

    Ok(())
}

async fn init(task: Task) -> Result<()> {
    task.spawn("http", http);
    task.spawn("mqtt_connections", io::mqtt::manage_connections);
    task.spawn("device_restore", device::restore_task);
    task.spawn("catch_virtual", value::catch_virtual_push);

    Ok(())
}

async fn http(t: Task) -> Result<()> {
    let addr = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let addr = std::net::SocketAddr::from_str(&addr)?;
    http::listen(t, addr).await?;

    Ok(())
}
