use std::{
    error::Error,
};

use prost_build::ServiceGenerator;

struct Service {}

impl ServiceGenerator for Service {
    fn generate(&mut self, service: prost_build::Service, buf: &mut String) {
        generate_typescript_service(&service);
        generate_rust_service(&service, buf);
    }
}

fn generate_rust_service(service: &prost_build::Service, buffer: &mut String) {
    /* Generate trait */
    buffer.push_str("#[async_trait::async_trait]\n");
    buffer.push_str(&format!("pub trait {} {{\n", service.name));

    for method in service.methods.iter() {
        if method.server_streaming {
            buffer.push_str(&format!("    async fn {}(input: {}) -> anyhow::Result<tokio::sync::mpsc::Receiver<anyhow::Result<{}>>>;\n", method.name, method.input_type, method.output_type));
        } else {
            buffer.push_str(&format!("    async fn {}(input: {}) -> anyhow::Result<{}>;\n", method.name, method.input_type, method.output_type));
        }   
    }

    buffer.push_str(&format!("}}\n"));

    /* Generate Router */

    buffer.push_str(r"#[derive(Copy, Clone)]
struct ConfigServiceRouter<T> {
    _marker: std::marker::PhantomData<T>,
}
");

    buffer.push_str(&format!("impl<T> {}Router<T> where T: {} + Copy {{\n", service.name, service.name));

    buffer.push_str(&format!(r"
    pub fn new() -> {}Router<T> {{
        {}Router {{
            _marker: Default::default(),
        }}
    }}
", service.name, service.name));

        buffer.push_str("    pub async fn route(&self, channel_id: u16, tx: &mut tokio::sync::mpsc::Sender<Vec<u8>>, data: Vec<u8>) -> anyhow::Result<()> {\n");

            buffer.push_str(r"
        let call_id = u16::from_be_bytes([
            *data.get(3).unwrap_or(&u8::MAX),
            *data.get(4).unwrap_or(&u8::MAX),
        ]);
    
        let [c1, c2] = channel_id.to_be_bytes();
");

        buffer.push_str("        match call_id {\n");
            buffer.push_str(r##"            _ =>  anyhow::bail!("invalid call id {call_id}"),"##);
        buffer.push_str("\n        }\n");

        buffer.push_str("    }\n");

    buffer.push_str("}\n");
}

fn generate_typescript_service(service: &prost_build::Service) {
    match std::fs::read_to_string("ui/src/net/protocol.ts") {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => { /* Do nothing */ }
        Err(e) => panic!("{:?}", e),
        Ok(mut buffer) if !buffer.contains("typescript-service-gen") => {
            buffer.push_str("\n\n// ========================== typescript-service-gen ==========================\n\n");

            buffer.push_str(r##"import { derived, type Readable } from "svelte/store";"##);
            buffer.push_str("\n");

            buffer.push_str(r##"export interface RpcTransport {
    call(callId: number, input: Uint8Array): Readable<Uint8Array>;
}

"##);

            buffer.push_str(&format!("export class {}Client {{\n", service.name));
            buffer.push_str(&format!("    constructor(private transport: RpcTransport) {{}} \n"));

            for (i, method) in service.methods.iter().enumerate() {
                let fnname = lower_case_first_char(method.proto_name.split('.').last().unwrap());
                let iname =  method.input_proto_type.split('.').last().unwrap();
                let oname = method.output_proto_type.split('.').last().unwrap();

                let fnbody = format!("
    {fnname}(input: {iname}): Readable<{oname}> {{
        return derived(
            this.transport.call(
                0x{i:X}, 
                {iname}.toBinary(input)
            ), 
            b => b ? {oname}.fromBinary(b) : null
        );
    }}
");
          
                buffer.push_str(&fnbody);
            }

                buffer.push_str(&"}\n");

                std::fs::write("ui/src/net/protocol.ts", buffer).unwrap();
            
        },
        _ => {}
    }
}

fn lower_case_first_char(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    prost_build::Config::new()
        .service_generator(Box::new(Service {}))
        .compile_protos(&["./protocol.proto"], &["./"])?;

    Ok(())
}

/*
Service {
    name: "ConfigService",
    proto_name: "ConfigService",
    package: "protocol",
    comments: Comments {
        leading_detached: [],
        leading: [],
        trailing: [],
    },
    methods: [
        Method {
            name: "zigbee2_mqtt",
            proto_name: "Zigbee2MQTT",
            comments: Comments {
                leading_detached: [],
                leading: [],
                trailing: [],
            },
            input_type: "Zigbee2MqttConfig",
            output_type: "ConfigResult",
            input_proto_type: ".protocol.Zigbee2MQTTConfig",
            output_proto_type: ".protocol.ConfigResult",
            options: MethodOptions {
                deprecated: None,
                idempotency_level: None,
                uninterpreted_option: [],
            },
            client_streaming: false,
            server_streaming: false,
        },
        Method {
            name: "listen",
            proto_name: "Listen",
            comments: Comments {
                leading_detached: [],
                leading: [],
                trailing: [],
            },
            input_type: "Sensor",
            output_type: "Sensor",
            input_proto_type: ".protocol.Sensor",
            output_proto_type: ".protocol.Sensor",
            options: MethodOptions {
                deprecated: None,
                idempotency_level: None,
                uninterpreted_option: [],
            },
            client_streaming: false,
            server_streaming: true,
        },
    ],
    options: ServiceOptions {
        deprecated: None,
        uninterpreted_option: [],
    },
}

*/
