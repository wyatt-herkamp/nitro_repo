use crate::cli;
use clap::{Parser, Subcommand};

pub mod install;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct UtilCommand {
    #[clap(subcommand)]
    pub subcommand: UtilCommands,
}

#[derive(Subcommand, Debug)]
pub enum UtilCommands {
    Install(cli::install::InstallCommand),
    Update,
    DockerPreRun(cli::install::InstallCommand),
}
