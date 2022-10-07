use api::cli::{UtilCommand, UtilCommands};
use api::cli::install::install_task;
use clap::Parser;
#[tokio::main]
async fn main() {
    let command: UtilCommand = UtilCommand::parse();
    match command.subcommand {
        UtilCommands::Install(install) => {
            install_task(install).await;
        }
        UtilCommands::Update => {}
        UtilCommands::DockerPreRun(_) => {}
    }
}
