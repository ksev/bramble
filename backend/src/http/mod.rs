mod pipe;
mod rpc;

use std::{net::SocketAddr, sync::Arc};

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
use tokio::{select, sync::mpsc::Sender};
use tower_http::services::ServeDir;
use tracing::{debug, error};

use pipe::{error_message, PipeMessage};

pub async fn listen(address: SocketAddr) -> anyhow::Result<()> {
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
        .route("/pipe", get(|ws, user_agent| ws_handler(ws, user_agent)));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    tracing::debug!("listening on {}", address);

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
) -> impl IntoResponse {
    if let Some(TypedHeader(user_agent)) = user_agent {
        debug!("`{}` connected", user_agent.as_str());
    }

    ws.on_upgrade(handle_socket)
}

async fn handle_socket(socket: WebSocket) {
    match handle_socket_err(socket).await {
        Ok(_) => {}
        Err(e) => error!("{}", e),
    }
}

async fn handle_socket_err(mut socket: WebSocket) -> anyhow::Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(5);

    loop {
        select! {
            Some(Ok(msg)) = socket.recv() => {
                match msg {
                    Message::Binary(v) => {
                        if let Err(e) = handle_pipe_action(v, tx.clone()).await {
                            let msg = error_message(0x0, &format!("{e}"));
                            tx.send(msg).await?;
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }

            Some(message) = rx.recv() => {
                socket.send(Message::Binary(message)).await?;
            }

            else => {
                break;
            }
        };
    }

    debug!("client disconnected");

    Ok(())
}

async fn handle_pipe_action(data: Vec<u8>, response: Sender<Vec<u8>>) -> anyhow::Result<()> {
    let rpc = Arc::new(rpc::Router::default());

    match PipeMessage::try_from(data)? {
        PipeMessage::Request(ctx) => {
            let rpc = rpc.clone();

            tokio::spawn(async move {
                let resp = response.clone();
                let channel_id = ctx.channel_id;

                if let Err(e) = rpc.route(ctx, resp).await {
                    let msg = error_message(channel_id, &format!("{e}"));
                    response.send(msg).await.expect("Can send on channel");
                }
            });
        }

        PipeMessage::Close => { /* Do nothing for now */ }
    };

    Ok(())
}
