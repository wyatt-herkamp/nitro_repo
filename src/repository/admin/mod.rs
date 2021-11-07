use actix_web::web;

pub mod controller;
pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(controller::add_repo)
        .service(controller::list_repos)
        .service(controller::modify_security)
        .service(controller::update_deployers_readers)
        .service(controller::modify_frontend_settings)
        .service(controller::modify_general_settings);
}
