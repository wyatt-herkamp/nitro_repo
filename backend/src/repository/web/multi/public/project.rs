use actix_web::{get, web, HttpResponse};
use comrak::Arena;
use log::warn;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

use crate::authentication::Authentication;
use crate::error::internal_error::InternalError;
use crate::repository::handler::Repository;

use crate::repository::settings::repository_page::{PageType, RepositoryPage};
use crate::repository::settings::{RepositoryConfig, Visibility};
use crate::storage::models::Storage;
use crate::storage::multi::MultiStorageController;
use crate::storage::DynamicStorage;
use crate::system::permissions::options::{CanIDo, MissingPermission};
use crate::system::user::UserModel;

use crate::repository::nitro::nitro_repository::NitroRepositoryHandler;
use crate::repository::nitro::{ProjectRequest, VersionData};
use crate::repository::settings::RepositoryType;

pub async fn get_project(
    storage_handler: web::Data<MultiStorageController<DynamicStorage>>,
    database: web::Data<DatabaseConnection>,
    authentication: Authentication,
    path: web::Path<ProjectRequest>,
) -> actix_web::Result<HttpResponse> {
    let (storage_name, repository_name, project, version) = path.into_inner().into_inner();
    let storage = crate::helpers::get_storage!(storage_handler, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name);
    if !repository.supports_nitro() {
        return Ok(HttpResponse::BadRequest().finish());
    }
    if !repository
        .get_repository()
        .visibility
        .eq(&Visibility::Public)
    {
        let caller: UserModel = authentication.get_user(database.as_ref()).await??;
        if let Some(value) = caller.can_read_from(repository.get_repository())? {
            return Err(value.into());
        }
    }
    let value = if let Some(version) = version {
        repository
            .get_project_specific_version(project.as_str(), version.as_ref())
            .await?
    } else {
        repository.get_project_latest(project.as_str()).await?
    };
    if let Some(value) = value {
        return Ok(HttpResponse::Ok().json(value));
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}
