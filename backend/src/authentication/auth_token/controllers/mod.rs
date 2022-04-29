use actix_web::web;

pub mod admin;
pub mod user;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(user::create_token)
        .service(user::list_tokens)
        .service(user::delete_token);
}
