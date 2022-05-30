use std::env::current_dir;

use clap::Parser;
use log::error;
use style_term::{DefaultColor, StyleString};

use crate::install::load_installer;
use crate::settings::models::Mode;
use crate::updater;
use crate::utils::load_logger;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct NitroRepoCLI {
    #[clap(short, long)]
    install: bool,
    #[clap(short, long)]
    update: Option<String>,
}

pub async fn handle_cli() -> std::io::Result<bool> {
    let path = current_dir()?;

    let parse: NitroRepoCLI = NitroRepoCLI::parse();
    if parse.install {
        load_logger(Mode::Install);
        if let Err(error) = load_installer(path).await {
            error!("Unable to complete Install {error}");
            println!(
                "{}",
                "Unable to Complete Installation"
                    .style()
                    .text_color(DefaultColor::Red)
            );
        }
        return Ok(true);
    } else if let Some(update) = parse.update {
        if let Err(error) = updater::update(update).await {
            error!("Unable to complete update {error}");
            println!(
                "{}",
                "Unable to Complete Update"
                    .style()
                    .text_color(DefaultColor::Red)
            );
        }
        return Ok(true);
    }
    Ok(false)
}
