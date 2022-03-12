mod net;

use std::{collections::HashMap, future::Future, iter::once, net::SocketAddr, time::Duration};

use async_trait::async_trait;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        TypedHeader,
    },
    headers,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, get_service},
    Router,
};
use prost::Message as _;
use tokio::{
    select,
    sync::mpsc::{error::SendError, Receiver, Sender, channel},
    task::JoinHandle, time::sleep,
};
use tower_http::services::ServeDir;
use tracing::{debug, error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "rome=debug,tower_http=debug")
    }

    tracing_subscriber::fmt::init();

    // build our application with some routes
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
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
) -> impl IntoResponse {
    if let Some(TypedHeader(user_agent)) = user_agent {
        println!("`{}` connected", user_agent.as_str());
    }

    ws.on_upgrade(handle_socket)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum PipeAction {
    Error = 0x0,
    Request = 0x1,
    Response = 0x2,
    Part = 0x3,
    Close = 0x4,
}

impl TryFrom<u8> for PipeAction {
    type Error = ();

    fn try_from(num: u8) -> Result<Self, <PipeAction as TryFrom<u8>>::Error> {
        match num {
            0x0 => Ok(PipeAction::Error),
            0x1 => Ok(PipeAction::Request),
            0x2 => Ok(PipeAction::Response),
            0x3 => Ok(PipeAction::Part),
            0x4 => Ok(PipeAction::Close),
            _ => Err(()),
        }
    }
}

fn error_message(channel: u16, message: &str) -> Vec<u8> {
    once(PipeAction::Error as u8)
        .chain(channel.to_be_bytes())
        .chain(message.bytes())
        .collect::<Vec<_>>()
}



#[derive(Copy, Clone)]
struct X;

#[async_trait]
impl ConfigService for X {
    async fn zigbee2_mqtt(input: net::Zigbee2MqttConfig) -> anyhow::Result<net::ConfigResult> {

        debug!("rcp request {:?}", input);

        let out = net::ConfigResult {
            result_oneof: Some(net::config_result::ResultOneof::Success(true)),
        };

        Ok(out)
    }

    async fn listen(input: net::Sensor) -> anyhow::Result<Receiver<anyhow::Result<net::Sensor>>> {
        let (tx, rx) = channel(2);

        tokio::spawn(async move {
            loop {
                let mut s = net::Sensor::default();

                s.id = 2;
                s.value = rand::random();

                tx.send(Ok(s)).await.unwrap();

                sleep(Duration::from_secs(2)).await;
            }
        });

        Ok(rx)
    }
}

#[async_trait::async_trait]
pub trait ConfigService {
    async fn zigbee2_mqtt(input: net::Zigbee2MqttConfig) -> anyhow::Result<net::ConfigResult>;
    async fn listen(input: net::Sensor) -> anyhow::Result<tokio::sync::mpsc::Receiver<anyhow::Result<net::Sensor>>>;
}
#[derive(Copy, Clone)]
struct ConfigServiceRouter<T> {
    _marker: std::marker::PhantomData<T>,
}
impl<T> ConfigServiceRouter<T> where T: ConfigService + Copy {

    pub fn new() -> ConfigServiceRouter<T> {
        ConfigServiceRouter {
            _marker: Default::default(),
        }
    }
    pub async fn route(&self, channel_id: u16, tx: &mut tokio::sync::mpsc::Sender<Vec<u8>>, data: Vec<u8>) -> anyhow::Result<()> {
        let call_id = u16::from_be_bytes([
            *data.get(3).unwrap_or(&u8::MAX),
            *data.get(4).unwrap_or(&u8::MAX),
        ]);
    
        let [c1, c2] = channel_id.to_be_bytes();
        match call_id {
            0 => {
                let input = net::Zigbee2MqttConfig::decode(&data[5..])?;
                let data = T::zigbee2_mqtt(input).await?;
                let mut out = vec![PipeAction::Response as u8, c1, c2];
                data.encode(&mut out)?;
                tx.send(out).await?;
                Ok(())
            }
            1 => {
                let input = net::Sensor::decode(&data[5..])?;
                let mut rx = T::listen(input).await?;
                while let Some(data) = rx.recv().await {
                    let mut out = vec![PipeAction::Part as u8, c1, c2];
                    data?.encode(&mut out)?;
                    tx.send(out).await?;
                }
                Ok(())
            }
            _ =>  anyhow::bail!("invalid call id {call_id}"),
        }
    }
}

async fn handle_socket(mut socket: WebSocket) -> anyhow::Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::channel(2);
    let router: ConfigServiceRouter<X> = ConfigServiceRouter::new();

    loop {
        select! {
            Some(Ok(msg)) = socket.recv() => {
                match msg {
                    Message::Binary(v) if v.len() >= 3 => {
                        let action = match PipeAction::try_from(v[0]) {
                            Ok(action) => action,
                            Err(_) => {
                                let msg = error_message(0x0, "invalid pipe action");
                                socket.send(Message::Binary(msg)).await?;
                                continue;
                            },
                        };

                        let channel_id = u16::from_be_bytes([v[1], v[2]]);

                        match action {
                            PipeAction::Request => {
                                let mut chantx = tx.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = router.route(channel_id, &mut chantx, v).await {
                                        let msg = error_message(channel_id, &format!("{e}"));
                                        chantx.send(msg).await.expect("Can send on channel");
                                    }
                                });
                            },
                            PipeAction::Close => {
                                // Trigger closing of task
                                // And clean up tracking state
                            },
                            _ => {
                                let msg = error_message(channel_id, &format!("{action:?} can not be sent by the client"));
                                socket.send(Message::Binary(msg)).await?;
                            }
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

/*
let server_addr = matches.value_of("SERVER").unwrap();
let client_id = matches
    .value_of("CLIENT_ID")
    .map(|x| x.to_owned())
    .unwrap_or_else(generate_client_id);
let channel_filters: Vec<(TopicFilter, QualityOfService)> = matches
    .values_of("SUBSCRIBE")
    .unwrap()
    .map(|c| (TopicFilter::new(c.to_string()).unwrap(), QualityOfService::Level2))
    .collect();

let keep_alive = 10;

info!("Connecting to {:?} ... ", server_addr);
let mut stream = net::TcpStream::connect(server_addr).unwrap();
info!("Connected!");

info!("Client identifier {:?}", client_id);
let mut conn = ConnectPacket::new(client_id);
conn.set_clean_session(true);
conn.set_keep_alive(keep_alive);
let mut buf = Vec::new();
conn.encode(&mut buf).unwrap();
stream.write_all(&buf[..]).unwrap();

let connack = ConnackPacket::decode(&mut stream).unwrap();
trace!("CONNACK {:?}", connack);

if connack.connect_return_code() != ConnectReturnCode::ConnectionAccepted {
    panic!(
        "Failed to connect to server, return code {:?}",
        connack.connect_return_code()
    );
}

// const CHANNEL_FILTER: &'static str = "typing-speed-test.aoeu.eu";
info!("Applying channel filters {:?} ...", channel_filters);
let sub = SubscribePacket::new(10, channel_filters);
let mut buf = Vec::new();
sub.encode(&mut buf).unwrap();
stream.write_all(&buf[..]).unwrap();

loop {
    let packet = match VariablePacket::decode(&mut stream) {
        Ok(pk) => pk,
        Err(err) => {
            error!("Error in receiving packet {:?}", err);
            continue;
        }
    };
    trace!("PACKET {:?}", packet);

    if let VariablePacket::SubackPacket(ref ack) = packet {
        if ack.packet_identifier() != 10 {
            panic!("SUBACK packet identifier not match");
        }

        info!("Subscribed!");
        break;
    }
}

// connection made, start the async work
let stream = TcpStream::from_std(stream).unwrap();
let (mut mqtt_read, mut mqtt_write) = stream.into_split();

let ping_time = Duration::new((keep_alive / 2) as u64, 0);

let ping_sender = tokio::spawn(async move {
    loop {
        tokio::time::sleep(ping_time).await;

        info!("Sending PINGREQ to broker");

        let pingreq_packet = PingreqPacket::new();

        let mut buf = Vec::new();
        pingreq_packet.encode(&mut buf).unwrap();
        mqtt_write.write_all(&buf).await.unwrap();
    }
});

let receiver = async move {
    while let Ok(packet) = VariablePacket::parse(&mut mqtt_read).await {
        trace!("PACKET {:?}", packet);

        match packet {
            VariablePacket::PingrespPacket(..) => {
                info!("Receiving PINGRESP from broker ..");
            }
            VariablePacket::PublishPacket(ref publ) => {
                let msg = match str::from_utf8(publ.payload()) {
                    Ok(msg) => msg,
                    Err(err) => {
                        error!("Failed to decode publish message {:?}", err);
                        continue;
                    }
                };
                info!("PUBLISH ({}): {}", publ.topic_name(), msg);
            }
            _ => {}
        }
    }
};

join!(ping_sender, receiver);

*/
