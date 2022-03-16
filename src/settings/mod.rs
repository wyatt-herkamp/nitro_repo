use actix_web::web;

pub mod controller;
pub mod models;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(controller::setting_report);
}
