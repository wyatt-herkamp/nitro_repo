use clap::SubCommand;
use api::cli::UtilCommand;

#[tokio::main]
async fn main() {
    let command: UtilCommand = UtilCommand::parse();

}
