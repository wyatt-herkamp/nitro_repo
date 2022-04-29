pub mod api_response;
pub mod constants;
pub mod error;
pub mod frontend;
pub mod install;
pub mod misc;
pub mod repository;
pub mod authentication;
pub mod settings;
pub mod storage;
pub mod system;
pub mod utils;
pub mod webhook;
pub mod updater;
pub mod cli;

use actix_web::web::Data;
use tokio::sync::RwLock;
use crate::settings::models::{GeneralSettings, Settings};

#[derive(Debug)]
pub struct NitroRepo {
    pub settings: RwLock<Settings>,
    pub  core: GeneralSettings,
    pub current_version: semver::Version,
}

pub type NitroRepoData = Data<NitroRepo>;
