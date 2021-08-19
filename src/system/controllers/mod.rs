use actix_web::web;

pub mod public;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(public::login);
}
https://www.youtube.com/watch?v=CH96Id_rMz8