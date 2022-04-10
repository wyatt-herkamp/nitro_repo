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
pub mod npm;
pub mod public;
pub mod types;
pub mod utils;

pub static REPOSITORY_CONF: &str = "repository.nitro_repo";
pub static REPOSITORY_CONF_BAK: &str = "repository.nitro_repo.bak";

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(controller::browse_storage)
        .service(web::resource(["/storages/", "/storages"]).to(controller::browse))
        .service(
            web::resource([
                "/storages/{storage}/{repository}",
                "/storages/{storage}/{repository}/{file:.*}",
                "/storages/{storage}/{repository}/",
            ])
            .route(web::get().to(controller::get_repository))
            .route(web::put().to(controller::put_repository))
            .route(web::head().to(controller::head_repository))
            .route(web::patch().to(controller::patch_repository))
            .route(web::post().to(controller::post_repository)),
        )
        .service(api::get_versions)
        .service(api::get_version)
        .service(api::get_project)
        .service(badge::badge)
        .service(public::get_repo);
}
