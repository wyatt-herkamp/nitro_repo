use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use app::config::NitroRepoConfig;
use clap::{Parser, Subcommand};
use config_editor::ConfigSection;
pub mod app;
mod config_editor;
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
        /// The thd-helper config file
        #[clap(short, long)]
        config: Option<PathBuf>,
    },
    SaveConfig {
        /// The thd-helper config file
        #[clap(short, long, default_value = "nitro_repo.toml")]
        config: PathBuf,
        /// If it should add defaults if the file already exists.
        #[clap(short, long, default_value = "false")]
        add_defaults: bool,
    },
    Config {
        /// The thd-helper config file
        #[clap(short, long, default_value = "nitro_repo.toml")]
        config: PathBuf,
        section: ConfigSection,
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

    match command.sub_command {
        SubCommands::Start { config } => web_start(config),
        SubCommands::SaveConfig {
            config,
            add_defaults,
        } => save_config(config, add_defaults),
        SubCommands::Export { export, location } => match export {
            ExportOptions::RepositoryConfigTypes => exporter::export_repository_configs(location),
            ExportOptions::RepositoryTypes => exporter::export_repository_types(location),
            ExportOptions::OpenAPI => exporter::export_openapi(location),
        },
        SubCommands::Config { config, section } => {
            let tokio = tokio::runtime::Builder::new_current_thread()
                .thread_name_fn(thread_name)
                .enable_all()
                .build()?;
            tokio.block_on(config_editor::editor(section, config))
        }
    }
}

fn web_start(config_path: Option<PathBuf>) -> anyhow::Result<()> {
    let tokio = tokio::runtime::Builder::new_current_thread()
        .thread_name_fn(thread_name)
        .enable_all()
        .build()?;
    tokio.block_on(app::web::start(config_path))
}
fn save_config(config_path: PathBuf, add_defaults: bool) -> anyhow::Result<()> {
    if config_path.exists() && !add_defaults {
        anyhow::bail!("Config file already exists. Please remove it first. or use the --add-defaults flag to overwrite it.");
    }
    if config_path.is_dir() {
        anyhow::bail!("Config file is a directory. Please pass a file path.");
    }
    let config = if config_path.exists() {
        let config = std::fs::read_to_string(&config_path)?;
        toml::from_str(&config)?
    } else {
        NitroRepoConfig::default()
    };
    let contents = toml::to_string_pretty(&config)?;
    std::fs::write(config_path, contents)?;
    Ok(())
}
fn thread_name() -> String {
    static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
    let id = ATOMIC_ID.fetch_add(1, Ordering::SeqCst);
    format!("nitro-repo-{}", id)
}
