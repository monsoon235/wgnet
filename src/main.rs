extern crate core;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod client;
mod server;
mod config;
mod wg;
mod utils;
mod api;

use tonic;


#[derive(Parser)]
#[command(name = "wgnet")]
#[command(author = "monsoon")]
#[command(version = "0.1.0")]
#[command(about = "A simple wireguard network manager")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Verbose output, use -vv or -vvv for higher verbositude
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Run client daemon")]
    Client {
        #[arg(short, long, default_value = "/etc/wgnet/client.yaml")]
        config: PathBuf,

        #[arg(short, long)]
        init: Option<String>,
    },
    #[command(about = "Run server daemon")]
    Server {
        #[arg(short, long, default_value = "/etc/wgnet/server.yaml")]
        config: PathBuf,

        #[arg(short, long, default_value = "/var/lib/wgnet")]
        data: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Client { config, init } => {
            if let Some(reginfo) = init {
                // client::init(&config, &reginfo);
            } else {
                // client::run(&config);
            }
        }
        Command::Server { config, data } => {
            // server::run(&config, &data);
        }
    }
}
