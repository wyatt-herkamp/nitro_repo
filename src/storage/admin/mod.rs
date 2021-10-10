use actix_web::web;

pub mod controller;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(controller::add_storage)
        .service(controller::list_storages)
        .service(controller::get_by_id);
}
