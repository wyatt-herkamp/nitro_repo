pub mod add;
pub mod api;
pub mod configs;
pub mod instances;

use crate::add::AddInstance;
use crate::instances::Instances;
use clap::{Parser, Subcommand};
use std::ffi::OsString;

#[derive(Debug, Parser)]
#[clap(name = "nrc")]
#[clap(about = "Nitro Repository CLI", long_about = None)]
struct NitroRepositoryCLI {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Adds a Nitro Repository Instance
    #[clap(arg_required_else_help = true)]
    Login(AddInstance),
    Instances(Instances),
    #[clap(external_subcommand)]
    External(Vec<OsString>),
}
#[tokio::main]
async fn main() {
    let args = NitroRepositoryCLI::parse();
    match args.command {
        Commands::Login(login) => {
            if let Err(error) = login.execute().await {
                println!("{:?}", error);
            }
        }
        Commands::Instances(instances) => {
            if let Err(error) = instances.execute().await {
                println!("{:?}", error);
            }
        }
        Commands::External(args) => {
            println!("{:?}", args);
        }
    }
}
