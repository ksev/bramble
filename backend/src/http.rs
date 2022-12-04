use std::net::SocketAddr;

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
use serde_derive::{Deserialize, Serialize};
use tower_http::services::ServeDir;
use tracing::{debug, error};

use crate::bus::BUS;

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
        .route("/pipe", get(ws_handler));

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

#[derive(Deserialize)]
struct Publish {
    topic: String,
    payload: serde_json::Value,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum Response {
    Error { message: String },
}

async fn handle_socket_err(mut socket: WebSocket) -> anyhow::Result<()> {
    while let Some(data) = socket.recv().await {
        let message = match data {
            Ok(message) => message,
            Err(e) => {
                socket.send(error_message(e)?).await?;
                continue;
            }
        };

        let Message::Text(payload) = message else {
            continue;
        };

        let message: Publish = match serde_json::from_str(&payload) {
            Ok(message) => message,
            Err(e) => {
                socket.send(error_message(e)?).await?;
                continue;
            }
        };

        if let Err(e) = handle_action(message).await {
            socket.send(error_message(e)?).await?;
        }
    }

    debug!("client disconnected");

    Ok(())
}

fn error_message<T: std::fmt::Debug>(err: T) -> anyhow::Result<Message> {
    let data = serde_json::to_string(&Response::Error {
        message: format!("{err:?}"),
    })?;

    Ok(Message::Text(data))
}

async fn handle_action(message: Publish) -> anyhow::Result<()> {
    let Publish { topic, payload } = message;
    BUS.publish_dynamic(&topic, payload)?;

    Ok(())
}
