use actix_web::web;
use log::debug;

pub mod dev_controllers;
pub mod release_controllers;

pub fn init(cfg: &mut web::ServiceConfig) {
    debug!("Calling Frontend init!");
    if cfg!(feature = "dev-frontend") {
        debug!("Initialing Dev Controllers");
        dev_controllers::init(cfg);
    } else if cfg!(feature = "frontend") {
        debug!("Initialing Release Controllers");
        release_controllers::init(cfg);
    }
}
