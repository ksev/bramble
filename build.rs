use std::{
    error::Error,
    process::{Command, ExitStatus},
};

use prost_build::ServiceGenerator;

struct Service {}

impl ServiceGenerator for Service {
    fn generate(&mut self, _service: prost_build::Service, _buf: &mut String) {
        //buf.push_str(&format!("{:#?}", service));
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    /* 
    let cmd = "npx protoc --ts_out src/net --proto_path ../ ../protocol.proto";

    let output = if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/C", cmd])
            .current_dir("./ui/")
            .output()
            .expect("failed to execute process")
    } else {
        Command::new("sh")
            .arg("-c")
            .current_dir("./ui/")
            .arg(cmd)
            .output()
            .expect("failed to execute process")
    };

    if !ExitStatus::success(&output.status) {
        return Err(String::from_utf8_lossy(&output.stderr).into());
    }

    prost_build::Config::new()
        .service_generator(Box::new(Service {}))
        .compile_protos(&["./protocol.proto"], &["./"])?;
*/
    Ok(())
}
