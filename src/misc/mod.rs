use actix_web::web;

mod frontend_helpers;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(frontend_helpers::site_info);
}
