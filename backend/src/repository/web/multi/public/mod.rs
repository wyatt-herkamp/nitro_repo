use actix_web::web;

pub mod project;
pub mod repositories;

pub fn init_public(cfg: &mut web::ServiceConfig) {
    cfg.service(repositories::get_repositories)
        .service(repositories::get_repository);
}
