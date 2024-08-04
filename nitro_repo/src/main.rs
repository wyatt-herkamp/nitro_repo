use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use clap::{Parser, Subcommand};

pub mod app;
pub mod error;
pub mod repository;
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
    let tokio = tokio::runtime::Builder::new_current_thread()
        .thread_name_fn(thread_name)
        .enable_all()
        .build()?;
    tokio.block_on(inner_main(command.sub_command, config))
}

async fn inner_main(
    command: SubCommands,
    config: app::config::NitroRepoConfig,
) -> anyhow::Result<()> {
    return match command {
        SubCommands::Start => app::web::start(config).await,
    };
}
fn thread_name() -> String {
    static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
    let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
    format!("nitro-repo-{}", id)
}
