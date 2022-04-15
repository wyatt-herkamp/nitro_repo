use actix_web::web;

pub mod controllers;

pub fn init(cfg: &mut web::ServiceConfig) {
    controllers::init(cfg);
}
