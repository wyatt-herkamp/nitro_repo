pub mod api_response;
pub mod authentication;
pub mod cli;
pub mod constants;
pub mod error;
pub mod frontend;
pub mod install;
pub mod misc;
pub mod repository;
pub mod settings;
pub mod storage;
pub mod system;
pub mod updater;
pub mod utils;

use crate::settings::models::{GeneralSettings, Settings};
use actix_web::web::Data;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct NitroRepo {
    pub settings: RwLock<Settings>,
    pub core: GeneralSettings,
    pub current_version: semver::Version,
}

pub type NitroRepoData = Data<NitroRepo>;
