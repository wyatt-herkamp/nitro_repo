use crate::repository::settings::RepositoryType;

pub mod ci;
pub mod docker;
pub mod frontend;
pub mod handler;
pub mod maven;
pub mod nitro;
pub mod npm;
pub mod raw;
pub mod response;
pub mod settings;
pub mod staging;
pub mod web;

pub static REPOSITORY_CONF: &str = "repository.nitro_repo";
pub static REPOSITORY_CONF_FOLDER: &str = ".config.nitro_repo";
pub static REPOSITORY_CONF_BAK: &str = "repository.nitro_repo.bak";
