use actix_web::web;

pub mod admin;
pub mod configs;
pub mod public;
pub mod repository_handler;
use crate::repository::settings::badge;
use crate::repository::settings::frontend;
use crate::repository::settings::repository_page;
pub fn init_repository_handlers(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource([
            "/repositories/{storage}/{repository}",
            "/repositories/{storage}/{repository}/{file:.*}",
            "/repositories/{storage}/{repository}/",
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

pub fn init_admin(cfg: &mut web::ServiceConfig) {
    cfg.configure(repository_page::init_repository_page);
    cfg.configure(frontend::init_frontend);
    cfg.configure(badge::init_badge);
    cfg.service(admin::get_repository)
        .service(admin::get_repositories)
        .service(admin::create_repository)
        .service(admin::delete_repository)
        .service(admin::update_repository_active)
        .service(admin::update_repository_policy)
        .service(admin::update_repository_visibility);
}
