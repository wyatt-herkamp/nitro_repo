use std::env::current_dir;

use clap::Parser;
use log::error;
use style_term::{DefaultColor, StyleString};

use crate::updater;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct NitroRepoCLI {
    #[clap(short, long)]
    update: Option<String>,
}

pub async fn handle_cli() -> std::io::Result<bool> {
    let _path = current_dir()?;

    let parse: NitroRepoCLI = NitroRepoCLI::parse();
    if let Some(update) = parse.update {
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
