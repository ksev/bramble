use std::io::Write;
use std::path::Path;
use std::{error::Error, process::Command};

use prost_build::ServiceGenerator;
use serde_derive::Serialize;
use tera::Context;
use tera::Tera;

use crate::crc16::crc16;

#[derive(Serialize)]
struct MethodSpec {
    name: String,
    hash: String,
    input_type: String,
    output_type: String,
    server_streaming: bool,
}

struct Service {
    tera: Tera,
}

impl ServiceGenerator for Service {
    fn generate(&mut self, service: prost_build::Service, buf: &mut String) {
        generate_typescript_service(&service, &mut self.tera);
        generate_rust_service(&service, buf, &mut self.tera);
    }
}

fn generate_rust_service(service: &prost_build::Service, buffer: &mut String, tera: &mut Tera) {
    let methods = service
        .methods
        .iter()
        .map(|method| MethodSpec {
            name: method.name.clone(),
            hash: format!("0x{:x}", hash_str(&method.name)),
            input_type: method.input_type.clone(),
            output_type: method.output_type.clone(),
            server_streaming: method.server_streaming,
        })
        .collect::<Vec<_>>();

    let mut ctx = Context::new();
    ctx.insert("service_name", &service.name);
    ctx.insert("service_hash", &format!("0x{:x}", hash_str(&service.name)));
    ctx.insert("methods", &methods);

    let result = tera.render("rust.tpl", &ctx).unwrap();

    buffer.push_str(&result);
}

fn generate_typescript_service(service: &prost_build::Service, tera: &mut Tera) {
    let methods = service
        .methods
        .iter()
        .map(|method| MethodSpec {
            name: lower_case_first_char(method.proto_name.split('.').last().unwrap()),
            hash: format!("0x{:x}", hash_str(&method.name)),
            input_type: method.input_proto_type.split('.').last().unwrap().into(),
            output_type: method.output_proto_type.split('.').last().unwrap().into(),
            server_streaming: method.server_streaming,
        })
        .collect::<Vec<_>>();

    let mut ctx = Context::new();
    ctx.insert("service_name", &service.name);
    ctx.insert("service_hash", &format!("0x{:x}", hash_str(&service.name)));
    ctx.insert("methods", &methods);

    let result = tera.render("typescript.tpl", &ctx).unwrap();

    append_to_file("./frontend/src/net/protocol.ts", result).unwrap();
}

fn hash_str(s: &str) -> u16 {
    crc16(s.as_bytes())
}

fn lower_case_first_char(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
    }
}

fn append_to_file<P, C>(path: P, content: C) -> std::io::Result<()>
where
    P: AsRef<Path>,
    C: AsRef<[u8]>,
{
    let mut file = std::fs::File::options()
        .read(true)
        .write(true)
        .append(true)
        .open(path.as_ref())?;

    file.write_all(content.as_ref())?;
    file.flush()?;

    Ok(())
}

fn run_npm_gen(tera: &mut Tera) -> Result<(), Box<dyn Error>> {
    let _ = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .current_dir("./frontend")
            .args(["/C", "npm run proto:generate"])
            .output()?;
    } else {
        Command::new("sh")
            .current_dir("./frontend")
            .args(["-c", "npm run proto:generate"])
            .output()?;
    };

    let ctx = Context::new();
    let result = tera.render("typescript-prelude.tpl", &ctx)?;
    append_to_file("./frontend/src/net/protocol.ts", result)?;

    Ok(())
}

pub fn generate() -> Result<(), Box<dyn Error>> {
    let mut tera = Tera::new("cli/templates/*")?;

    run_npm_gen(&mut tera)?;

    prost_build::Config::new()
        .service_generator(Box::new(Service { tera }))
        .out_dir("./backend/src/http/rpc")
        .compile_protos(&["./protocol.proto"], &["./"])?;

    Ok(())
}
