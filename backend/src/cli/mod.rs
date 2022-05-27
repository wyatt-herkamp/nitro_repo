use crate::install::load_installer;
use crate::settings::models::Mode;
use crate::updater;
use crate::utils::load_logger;
use clap::Parser;
use log::error;
use std::path::Path;
use style_term::{DefaultColor, StyleString};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short, long)]
    install: bool,
    #[clap(short, long)]
    update: Option<String>,
}

pub async fn handle_cli() -> std::io::Result<bool> {
    let path = Path::new("cfg");

    let parse: Cli = Cli::parse();
    if parse.install {
        load_logger(Mode::Install);
        if let Err(error) = load_installer(path).await {
            error!("Unable to complete Install {error}");
            println!("{}", "Unable to Complete Installation".style().text_color(DefaultColor::Red));
        }
        return Ok(true);
    } else if let Some(update) = parse.update {
        if let Err(error) = updater::update(update).await {
            error!("Unable to complete update {error}");
            println!("{}", "Unable to Complete Update".style().text_color(DefaultColor::Red));
        }
        return Ok(true);
    }
    Ok(false)
}
