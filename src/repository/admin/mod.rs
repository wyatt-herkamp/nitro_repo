use actix_web::web;

pub mod controller;
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(controller::add_repo)
        .service(controller::list_repos)
        .service(controller::modify_security)
        .service(controller::modify_settings);
}
