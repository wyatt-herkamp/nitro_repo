use std::path::PathBuf;

use clap::{Parser, Subcommand};

pub mod app;
pub mod error;

pub mod request_error;
pub mod utils;
#[derive(Parser)]
struct Command {
    #[clap(subcommand)]
    sub_command: SubCommands,
    /// The thd-helper config file
    #[clap(short, long, default_value = "nitro_repo.toml")]
    config: PathBuf,
    // Comments will be destroyed by TOML
    #[clap(long, default_value = "false")]
    add_defaults_to_config: bool,
}
#[derive(Subcommand, Clone, Debug)]
enum SubCommands {
    Start,
}
fn main() -> anyhow::Result<()> {
    let command = Command::parse();
    let config =
        app::config::NitroRepoConfig::load(command.config, command.add_defaults_to_config)?;

    return match command.sub_command {
        SubCommands::Start => actix_rt::System::new().block_on(app::web::start(config)),
    };
}
