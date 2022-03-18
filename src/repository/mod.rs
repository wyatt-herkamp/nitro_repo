use actix_web::web;

pub mod admin;
mod api;
mod badge;
pub mod controller;
pub mod deploy;
pub mod frontend;
pub mod maven;
pub mod models;
pub mod nitro;
pub mod public;
pub mod types;
pub mod utils;
pub static REPOSITORY_CONF: &str = "repository.nitro_repo";
pub static REPOSITORY_CONF_BAK: &str = "repository.nitro_repo.bak";

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(controller::browse)
        .service(controller::browse_storage)
        .service(controller::get_repository)
        .service(controller::post_repository)
        .service(controller::patch_repository)
        .service(controller::put_repository)
        .service(controller::head_repository)
        .service(api::get_versions)
        .service(api::get_version)
        .service(api::get_project)
        .service(badge::badge)
        .service(public::get_repo);
}
