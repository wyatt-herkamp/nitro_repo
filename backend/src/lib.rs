#![allow(clippy::from_over_into)]

use actix_web::web::Data;
use tokio::sync::RwLock;

use crate::settings::models::{GeneralSettings, Settings};

pub mod authentication;
pub mod cli;
pub mod constants;
pub mod error;
pub mod frontend;
pub mod generators;
pub mod helpers;
pub mod install;
pub mod repository;
pub mod settings;
pub mod storage;
pub mod system;
pub mod updater;
pub mod utils;

#[derive(Debug)]
pub struct NitroRepo {
    pub settings: RwLock<Settings>,
    pub core: GeneralSettings,
    pub current_version: semver::Version,
}

pub type NitroRepoData = Data<NitroRepo>;
