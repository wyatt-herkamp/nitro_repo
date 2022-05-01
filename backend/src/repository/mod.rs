use actix_web::web as ActixWeb;

pub mod deploy;
pub mod frontend;
pub mod maven;
pub mod nitro;
pub mod npm;
pub mod settings;
pub mod response;
pub mod web;
pub mod data;
pub mod handler;
pub mod error;

pub static REPOSITORY_CONF: &str = "repository.nitro_repo";
pub static REPOSITORY_CONF_BAK: &str = "repository.nitro_repo.bak";


