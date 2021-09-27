


use actix_web::{web};

pub mod action;
pub mod admin;
pub mod controller;
pub mod maven;
pub mod models;
pub mod repo_error;
pub mod repository;
mod api;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(controller::browse)
        .service(controller::browse_storage)
        .service(controller::get_repository)
        .service(controller::post_repository)
        .service(controller::patch_repository)
        .service(controller::put_repository)
        .service(controller::head_repository)
        .service(api::get_versions);
}
