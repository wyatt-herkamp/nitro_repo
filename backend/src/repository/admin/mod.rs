use actix_web::web;

pub mod controller;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg
        .service(controller::add_repo)
        .service(controller::get_repo)
        .service(controller::list_repos_by_storage)
        .service(controller::list_repos)
        .service(controller::modify_security)
        .service(controller::modify_frontend_settings)
        .service(controller::modify_deploy)
        .service(controller::add_webhook)
        .service(controller::remove_webhook)
        .service(controller::update_active_status)
        .service(controller::update_policy)
        .service(controller::modify_badge_settings)
        .service(controller::update_description)
        .service(controller::delete_repository);
}
