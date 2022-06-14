//mod actor;
//mod database;
//mod device;
//mod http;
//mod integration;
//mod io;
mod actor;

use anyhow::Result;
use actor::{Pid, Receive, System, Trap};
use futures::future::{join_all, BoxFuture};
use tracing::error;
use std::{net::SocketAddr, sync::Arc, time::Duration, f32::consts::E};

//use actor::{prelude::*, ActorSystem};

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");

    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "backend=debug,tokio=trace,runtime=trace")
    }

    tracing_subscriber::fmt::init();

    loop {
        let sys = System::new();

        sys.spawn(start, ());

        if let Err(e) = sys.join().await {
            error!("Main loop: {e:?}");
        }
    }
}


async fn start(_ctx: Receive<()>, _arguments: ()) -> Result<()> {
    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
