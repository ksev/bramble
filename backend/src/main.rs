mod http;

use std::net::SocketAddr;
use futures::future::join_all;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "backend=debug,tower_http=debug")
    }

    tracing_subscriber::fmt::init();

    let all = join_all(vec![
        http::listen(SocketAddr::from(([0,0,0,0], 8080))),
    ]);

    all.await;

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
