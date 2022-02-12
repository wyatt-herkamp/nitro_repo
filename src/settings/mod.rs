use actix_web::web;

pub mod action;
pub mod controller;
pub mod models;
pub mod utils;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(controller::about_setting)
        .service(controller::setting_report)
        .service(controller::update_setting);
}
