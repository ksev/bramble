mod pipe;
mod rpc;

use std::{net::SocketAddr, collections::BTreeMap};

use anyhow::Result;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        WebSocketUpgrade,
    },
    headers,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, get_service},
    Router, TypedHeader,
};
use tokio::select;
use tower_http::services::ServeDir;
use tracing::{debug, error};

use pipe::{error_message, PipeMessage};

use crate::actor::{Context, ContextSpawner, Pid, Receive, Task, Trap, ExitReason};

use self::pipe::PipeRequest;

pub async fn listen(ctx: Receive<()>, address: SocketAddr) -> Result<()> {
    let spawner = ctx.spawner();

    let app = Router::new()
        .fallback(
            get_service(
                ServeDir::new("examples/websockets/assets").append_index_html_on_directories(true),
            )
            .handle_error(|error: std::io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                )
            }),
        )
        // routes are matched from bottom to top, so we have to put `nest` at the
        // top since it matches all routes
        .route(
            "/pipe",
            get(|ws, user_agent| ws_handler(spawner, ws, user_agent)),
        );

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    tracing::debug!("listening on {}", address);

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn ws_handler(
    spawner: ContextSpawner,
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
) -> impl IntoResponse {
    if let Some(TypedHeader(user_agent)) = user_agent {
        debug!("`{}` connected", user_agent.as_str());
    }

    ws.on_upgrade(|socket| async move {
        spawner.spawn_with_argument(connection, socket);
    })
}

async fn connection(ctx: Trap<Vec<u8>>, mut socket: WebSocket) -> Result<()> {
    use crate::actor::Signal;

    let mut procs = BTreeMap::new();

    loop {
        select! {
            Some(Ok(msg)) = socket.recv() => {
                match msg {
                    Message::Binary(v) => {
                        if let Err(e) = handle_pipe_action(&ctx, &mut procs, v).await {
                            error!("{e}");

                            let msg = error_message(0x0, &format!("{e}"));
                            ctx.pid().send(msg);
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }

            message = ctx.trap() => match message {
                Signal::Exit((id, reason)) => { 
                    if let Some(channel_id) = procs.remove(&id) {
                        // Make sure we let the client know the proc failed
                        if let ExitReason::Error(e) = reason {
                            let msg = error_message(channel_id, &format!("{e}"));
                            ctx.pid().send(msg);
                        }
                    }
                },
                Signal::Message(message) => { socket.send(Message::Binary(message)).await?; },
            },

            else => {
                break;
            }
        };
    }

    Ok(())
}

async fn handle_pipe_action(ctx: &Trap<Vec<u8>>, procs: &mut BTreeMap<usize, u16>, data: Vec<u8>) -> Result<()> {
    match PipeMessage::try_from(data)? {
        PipeMessage::Request(preq) => {
            let channel_id = preq.channel_id;
            let arg = (preq, ctx.pid());
            let proc = ctx.spawn_link_with_argument(rpc_call, arg);

            // Keep track of live procs
            procs.insert(proc.actor_id(), channel_id);
        }

        PipeMessage::Close => { /* Do nothing for now */ }
    };

    Ok(())
}

async fn rpc_call(ctx: Task, (req, receiver): (PipeRequest, Pid<Vec<u8>>)) -> Result<()> {
    rpc::ROUTER.route(ctx, req, receiver).await?;
    Ok(())
}
