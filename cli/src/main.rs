mod protoc;
mod crc16;

use std::error::Error;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize install in the $PWD
    Init,
    /// Generate new protocol buffer files (dev only)
    Protoc,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Init => {
            println!("init");
        },
        Commands::Protoc => {
            protoc::generate()?;
            println!("Done");
        }
    };

    Ok(())
}
