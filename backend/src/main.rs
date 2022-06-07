mod actor;
mod database;
mod device;
mod http;
mod integration;
mod io;

use futures::future::join_all;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");

    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "backend=debug,tower_http=debug")
    }

    tracing_subscriber::fmt::init();

    // Load all the integrations we have from the database
    integration::zigbee2mqtt::Server::hydrate().await?;

    let all = join_all(vec![http::listen(SocketAddr::from(([0, 0, 0, 0], 8080)))]);

    all.await;

    Ok(())
}
