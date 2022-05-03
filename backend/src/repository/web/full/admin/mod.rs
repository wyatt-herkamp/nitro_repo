//mod controller;

pub mod controller;

use actix_web::web;


pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(controller::add_repo)
        .service(controller::get_repo)
        .service(controller::list_repos_by_storage)
        .service(controller::modify_security)
        .service(controller::update_active_status)
        .service(controller::update_policy)
        .service(controller::delete_repository);
}
