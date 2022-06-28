mod actor;
//mod database;
//mod device;
mod http;
//mod integration;
mod io;

use actor::{Context, System, Trap, Task};
use anyhow::Result;
use tracing::error;

use crate::actor::Signal;

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_BACKTRACE", "1");

    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "backend=debug,tokio=error,runtime=error")
    }

    tracing_subscriber::fmt::init();

    loop {
        let sys = System::new();

        sys.spawn_linked(start);

        if let Err(e) = sys.join().await {
            error!("Main loop: {e:?}");
        }
    }
}

async fn start(ctx: Task) -> Result<()> {
    // Start web server
    let address = "0.0.0.0:8080".parse().unwrap();
    ctx.spawn_link_with_argument(http::listen, address);

    // Start mqtt connection supervisor
    ctx.spawn_link(io::mqtt);


    // We will never receive anything to just block
    ctx.receive().await;

    Ok(())
}

/*
fn chain(ctx: Receive<String>, count: usize) -> BoxFuture<'static, Result<()>> {
    Box::pin(async move {
        if count < 100_000 {
            let next = ctx.spawn_link_with_argument(chain, count + 1);

            loop {
                let msg = ctx.receive().await;
                next.send(msg);
            }
        } else {
            loop {
                let msg = ctx.receive().await;
                println!("msg: {msg}");
            }
        }
    })
}
 */
