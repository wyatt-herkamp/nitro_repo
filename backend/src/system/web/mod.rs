use actix_web::web::ServiceConfig;

pub mod admin;
pub mod public;
pub mod user;

pub fn init_public_routes(cfg: &mut ServiceConfig) {
    cfg.service(public::login);
}

pub fn user_routes(cfg: &mut ServiceConfig) {
    cfg.service(user::me);
}

pub fn init_user_manager_routes(_cfg: &mut ServiceConfig) {}
