use crate::repository::repository::{RepositoryType, RepoResult};
use actix_web::{HttpRequest, web};
use crate::storage::models::Storage;
use crate::repository::models::Repository;

pub mod controller;
pub mod repo_error;
pub mod models;
pub mod repository;
pub mod maven;
pub mod action;
pub mod admin;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(controller::get_repository)
        .service(controller::post_repository)
        .service(controller::patch_repository)
        .service(controller::put_repository)
        .service(controller::head_repository);
}
