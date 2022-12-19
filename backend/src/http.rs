use std::net::SocketAddr;

use async_graphql::{http::GraphiQLSource, EmptyMutation, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::{
    extract::Extension,
    http::{header::CONTENT_TYPE, Method},
    response::{self, IntoResponse},
    routing::get,
    Router,
};

use tower_http::cors::{Any, CorsLayer};

use crate::{
    api::{ApiSchema, Mutation, Query, Subscription},
    task::Task,
};

async fn graphql_handler(schema: Extension<ApiSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

async fn graphiql() -> impl IntoResponse {
    response::Html(
        GraphiQLSource::build()
            .endpoint("http://localhost:8080/api")
            .subscription_endpoint("ws://localhost:8080/api/ws")
            .finish(),
    )
}

pub async fn listen(task: Task, address: SocketAddr) -> anyhow::Result<()> {
    let schema = Schema::build(Query, Mutation, Subscription)
        .data(task)
        .finish();

    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE])
        .allow_origin(Any);

    let app = Router::new()
        .route("/api", get(graphiql).post(graphql_handler))
        .route_service("/api/ws", GraphQLSubscription::new(schema.clone()))
        .layer(Extension(schema))
        .layer(cors);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    tracing::debug!("listening on {}", address);

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/*

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
#[serde(tag = "event")]
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
    for device in Device::all().await? {
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
 */
