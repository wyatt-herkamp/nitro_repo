use actix_web::web;

pub mod public;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(public::login);
}
