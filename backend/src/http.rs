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

use futures::{stream::SplitSink, SinkExt, StreamExt};
use futures_concurrency::prelude::*;

use serde_derive::{Deserialize, Serialize};
use tower_http::services::ServeDir;
use tracing::{debug, error};

use crate::{
    bus::BUS,
    device::{Device, SOURCES},
};

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
    if let Err(e) = handle_socket_err(socket).await {
        let err = e.to_string();
        if err != "Connection closed normally" {
            error!("{}", err);
        }
    }
}

#[derive(Deserialize)]
struct Publish {
    topic: String,
    payload: serde_json::Value,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum Response<'a> {
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "device")]
    Device(&'a Device),
    #[serde(rename = "value")]
    Value {
        device: &'a str,
        property: &'a str,
        value: &'a Result<serde_json::Value, String>,
    },
}

pub enum Update {
    Socket(Result<Message, axum::Error>),
    Device(Arc<Device>),
    Value((String, String, Result<serde_json::Value, String>)),
}

async fn handle_socket_err(mut socket: WebSocket) -> anyhow::Result<()> {
    // Send current state down the pipe first
    for device in Device::all()? {
        let data = serde_json::to_string(&Response::Device(&device))?;
        socket.send(Message::Text(data)).await?;
    }

    // Send the values we know about
    for r in SOURCES.all() {
        let key = r.key();

        let data = serde_json::to_string(&Response::Value {
            device: &key.0,
            property: &key.1,
            value: r.value(),
        })?;

        socket.send(Message::Text(data)).await?;
    }

    let mut devices = BUS.device.add.subscribe().map(Update::Device);
    let mut values = BUS.device.value.subscribe().map(Update::Value);

    let (mut socket, incoming) = socket.split();
    let mut incoming = incoming.map(Update::Socket);

    loop {
        let next = (incoming.next(), devices.next(), values.next());

        let Some(output) = next.race().await else {
            // The only time we get a None is when the socket closes,
            // Then we just break out of the loop
            break;
        };

        if let Err(err) = handle_update(&mut socket, output).await {
            let data = serde_json::to_string(&Response::Error {
                message: format!("{err:?}"),
            })?;

            socket.send(Message::Text(data)).await?;
        }
    }

    debug!("client disconnected");

    Ok(())
}

async fn handle_update(
    socket: &mut SplitSink<WebSocket, Message>,
    update: Update,
) -> anyhow::Result<()> {
    match update {
        Update::Socket(data) => {
            let message = data;
            let Message::Text(payload) = message? else {
                anyhow::bail!("Payload must be of Text type");
            };

            let message: Publish = serde_json::from_str(&payload)?;

            let Publish { topic, payload } = message;
            BUS.publish_dynamic(&topic, payload)?;
        }
        Update::Device(device) => {
            let data = serde_json::to_string(&Response::Device(&device))?;
            socket.send(Message::Text(data)).await?;
        }
        Update::Value(value) => {
            let data = serde_json::to_string(&Response::Value {
                device: &value.0,
                property: &value.1,
                value: &value.2,
            })?;
            socket.send(Message::Text(data)).await?;
        }
    }

    Ok(())
}
