use actix_web::web as ActixWeb;

pub mod deploy;
pub mod frontend;
pub mod maven;
pub mod models;
pub mod nitro;
pub mod npm;
pub mod public;
pub mod settings;
pub mod types;
pub mod utils;
pub mod web;

pub static REPOSITORY_CONF: &str = "repository.nitro_repo";
pub static REPOSITORY_CONF_BAK: &str = "repository.nitro_repo.bak";


