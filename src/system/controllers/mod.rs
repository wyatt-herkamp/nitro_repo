use actix_web::web;

pub mod public;
pub mod user;
pub mod me;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(public::login)
        .service(user::add_user)
        .service(user::list_users)
        .service(user::delete_user)
        .service(user::modify_user)

        .service(me::me);
}
