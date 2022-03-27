pub mod json_error;

use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.app_data(json_error::json_config());
}
