use actix_web::web;

use crate::system::web::public;

pub mod admin;
pub mod repository_handler;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource([
            "/storages/{storage}/{repository}",
            "/storages/{storage}/{repository}/{file:.*}",
            "/storages/{storage}/{repository}/",
        ])
        .route(web::get().to(repository_handler::get_repository))
        .route(web::put().to(repository_handler::put_repository))
        .route(web::head().to(repository_handler::head_repository))
        .route(web::patch().to(repository_handler::patch_repository))
        .route(web::post().to(repository_handler::post_repository)),
    );
}
