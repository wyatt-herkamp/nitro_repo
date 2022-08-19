use actix_web::web::Bytes;
use actix_web::{web, HttpRequest, HttpResponse};
use actix_web::http::header::CONTENT_TYPE;
use badge_maker::{Badge, BadgeBuilder};
use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::authentication::Authentication;

use crate::repository::handler::Repository;
use crate::repository::nitro::nitro_repository::NitroRepositoryHandler;
use crate::repository::response::RepoResponse;
use crate::repository::settings::badge::BadgeSettings;
use crate::repository::settings::{RepositoryConfig, RepositoryConfigHandler};
use crate::repository::staging::StageHandler;
use crate::storage::models::Storage;
use crate::storage::multi::MultiStorageController;
use crate::storage::DynamicStorage;
use crate::system::permissions::permissions_checker::CanIDo;

#[derive(Deserialize, Clone)]
pub struct GetPath {
    pub storage: String,
    pub repository: String,
    #[serde(default)]
    pub file: String,
}

impl GetPath {
    pub fn into_inner(self) -> (String, String, String) {
        (self.storage, self.repository, self.file)
    }
}

pub async fn get_repository(
    pool: web::Data<DatabaseConnection>,
    storages: web::Data<MultiStorageController<DynamicStorage>>,
    auth: Authentication,
    r: HttpRequest,
    path: web::Path<GetPath>,
) -> actix_web::Result<RepoResponse> {
    let (storage_name, repository_name, file) = path.into_inner().into_inner();
    let storage = crate::helpers::get_storage!(storages, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name);
    repository
        .handle_get(&file, r.headers(), pool.get_ref(), auth)
        .await
}

pub async fn badge_repository(
    storages: web::Data<MultiStorageController<DynamicStorage>>,
    path: web::Path<GetPath>,
) -> actix_web::Result<HttpResponse> {
    let (storage_name, repository_name, file) = path.into_inner().into_inner();
    let storage = crate::helpers::get_storage!(storages, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name);
    if !RepositoryConfigHandler::<BadgeSettings>::supports_config(repository.as_ref()) || !NitroRepositoryHandler::supports_nitro(repository.as_ref()) {
        return Ok(HttpResponse::BadRequest().finish());
    }
    let settings = RepositoryConfigHandler::<BadgeSettings>::get(repository.as_ref());
    let badge: Badge = if file.eq("nitro_repo_badge") {
        let string = repository.get_repository().repository_type.to_string();
        BadgeBuilder::new().color_parse(&settings.color).label_color_parse(&settings.label_color).
            style(settings.style.to_badge_maker_style()).label(&repository.get_repository().name).message(&string).build().unwrap()
    } else {
        if let Some(some) = NitroRepositoryHandler::latest_version(repository.as_ref(), &file).await? {
            BadgeBuilder::new().color_parse(&settings.color).label_color_parse(&settings.label_color).
                style(settings.style.to_badge_maker_style()).label(&repository.get_repository().name).message(&some).build().unwrap()
        } else {
            return Ok(HttpResponse::NotFound().finish());
        }
    };

    Ok(HttpResponse::Ok().append_header((CONTENT_TYPE, "image/svg+xml")).body(badge.svg()))
}

pub async fn put_repository(
    pool: web::Data<DatabaseConnection>,
    storages: web::Data<MultiStorageController<DynamicStorage>>,
    auth: Authentication,
    r: HttpRequest,
    path: web::Path<GetPath>,
    bytes: Bytes,
) -> actix_web::Result<RepoResponse> {
    let (storage_name, repository_name, file) = path.into_inner().into_inner();
    let storage = crate::helpers::get_storage!(storages, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name);
    repository
        .handle_put(&file, r.headers(), pool.get_ref(), auth, bytes)
        .await
}

pub async fn stage_repository(
    pool: web::Data<DatabaseConnection>,
    storages: web::Data<MultiStorageController<DynamicStorage>>,
    authentication: Authentication,
    path: web::Path<GetPath>,
) -> actix_web::Result<RepoResponse> {
    let (storage_name, repository_name, file) = path.into_inner().into_inner();
    let storage = crate::helpers::get_storage!(storages, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name);
    if !repository.staging_repository() {
        return Ok(HttpResponse::BadRequest().finish().into());
    }
    let caller = authentication.get_user(pool.as_ref()).await??;
    if let Some(_value) = caller.can_deploy_to(repository.get_repository())? {}
    repository.push(file, storages.into_inner(), caller).await?;
    Ok(HttpResponse::NoContent().finish().into())
}

pub async fn head_repository(
    pool: web::Data<DatabaseConnection>,
    storages: web::Data<MultiStorageController<DynamicStorage>>,
    auth: Authentication,
    r: HttpRequest,
    path: web::Path<GetPath>,
) -> actix_web::Result<RepoResponse> {
    let (storage_name, repository_name, file) = path.into_inner().into_inner();
    let storage = crate::helpers::get_storage!(storages, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name);
    repository
        .handle_head(&file, r.headers(), pool.get_ref(), auth)
        .await
}

pub async fn post_repository(
    pool: web::Data<DatabaseConnection>,
    storages: web::Data<MultiStorageController<DynamicStorage>>,
    auth: Authentication,
    r: HttpRequest,
    path: web::Path<GetPath>,
    bytes: Bytes,
) -> actix_web::Result<RepoResponse> {
    let (storage_name, repository_name, file) = path.into_inner().into_inner();
    let storage = crate::helpers::get_storage!(storages, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name);
    repository
        .handle_put(&file, r.headers(), pool.get_ref(), auth, bytes)
        .await
}

pub async fn patch_repository(
    pool: web::Data<DatabaseConnection>,
    storages: web::Data<MultiStorageController<DynamicStorage>>,
    auth: Authentication,
    r: HttpRequest,
    path: web::Path<GetPath>,
    bytes: Bytes,
) -> actix_web::Result<RepoResponse> {
    let (storage_name, repository_name, file) = path.into_inner().into_inner();
    let storage = crate::helpers::get_storage!(storages, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name);
    repository
        .handle_patch(&file, r.headers(), pool.get_ref(), auth, bytes)
        .await
}
