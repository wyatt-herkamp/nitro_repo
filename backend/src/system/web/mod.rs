use actix_web::web::ServiceConfig;

pub mod admin;
pub mod public;
pub mod user;

pub fn init_public_routes(cfg: &mut ServiceConfig) {
    cfg.service(public::login);
}

pub fn user_routes(cfg: &mut ServiceConfig) {
    cfg.service(user::me).service(user::update_password);
}

pub fn init_user_manager_routes(cfg: &mut ServiceConfig) {
    cfg.service(admin::list_users).service(admin::get_user);
}
