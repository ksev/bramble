mod bus;
mod http;
mod integration;
mod io;
mod task;
mod device;
mod db;
mod api;

use std::str::FromStr;

use anyhow::Result;

use task::Task;

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");

    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "backend=debug,tokio=error,runtime=error")
    }

    let _ = db::pool().await;

    tracing_subscriber::fmt::init();

    task::create_group(init).complete().await?;

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
