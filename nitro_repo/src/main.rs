use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use clap::{Parser, Subcommand};
pub mod app;
pub mod error;
mod exporter;
pub mod repository;
pub mod utils;
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ExportOptions {
    RepositoryConfigTypes,
    RepositoryTypes,
    OpenAPI,
}
#[derive(Parser)]
struct Command {
    #[clap(subcommand)]
    sub_command: SubCommands,
}
#[derive(Subcommand, Clone, Debug)]
enum SubCommands {
    Start {
        // Comments will be destroyed by TOML
        #[clap(long, default_value = "false")]
        add_defaults_to_config: bool,
        /// The thd-helper config file
        #[clap(short, long, default_value = "nitro_repo.toml")]
        config: PathBuf,
    },
    Export {
        export: ExportOptions,
        location: PathBuf,
    },
}
fn main() -> anyhow::Result<()> {
    // For Some Reason Lettre fails if this is not installed
    if rustls::crypto::ring::default_provider()
        .install_default()
        .is_err()
    {
        eprintln!("Default Crypto Provider already installed. This is not an error. But it should be reported.");
    }

    let command = Command::parse();

    return match command.sub_command {
        SubCommands::Start {
            add_defaults_to_config,
            config,
        } => web_start(add_defaults_to_config, config),
        SubCommands::Export { export, location } => match export {
            ExportOptions::RepositoryConfigTypes => exporter::export_repository_configs(location),
            ExportOptions::RepositoryTypes => exporter::export_repository_types(location),
            ExportOptions::OpenAPI => exporter::export_openapi(location),
        },
    };
}

fn web_start(add_defaults: bool, config_path: PathBuf) -> anyhow::Result<()> {
    let tokio = tokio::runtime::Builder::new_current_thread()
        .thread_name_fn(thread_name)
        .enable_all()
        .build()?;
    tokio.block_on(app::web::start(config_path, add_defaults))
}
fn thread_name() -> String {
    static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
    let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
    format!("nitro-repo-{}", id)
}
