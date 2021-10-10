use actix_web::web;

pub mod controllers;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(controllers::index);
        //.service(controllers::admin)
        //.service(controllers::browse)
        //.service(controllers::browse_extend)
        //.service(controllers::login);
}
