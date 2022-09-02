use actix_web::{web, HttpResponse};

use sea_orm::DatabaseConnection;

use crate::authentication::Authentication;
use crate::generators::GeneratorCache;

use crate::repository::handler::Repository;

use crate::storage::models::Storage;
use crate::storage::multi::MultiStorageController;
use crate::storage::DynamicStorage;
use crate::system::permissions::permissions_checker::CanIDo;

use crate::repository::nitro::nitro_repository::NitroRepositoryHandler;
use crate::repository::nitro::ProjectRequest;

pub async fn get_project(
    storage_handler: web::Data<MultiStorageController<DynamicStorage>>,
    conn: web::Data<DatabaseConnection>,
    authentication: Authentication,
    path: web::Path<ProjectRequest>,
    cache: web::Data<GeneratorCache>,
) -> actix_web::Result<HttpResponse> {
    let (storage_name, repository_name, project, version) = path.into_inner().into_inner();
    let storage = crate::helpers::get_storage!(storage_handler, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name);
    if !repository.supports_nitro() {
        return Ok(HttpResponse::BadRequest().finish());
    }
    crate::helpers::read_check_web!(authentication, conn.as_ref(), repository.get_repository());

    let value = if let Some(version) = version {
        repository
            .get_project_specific_version(project.as_str(), version.as_ref(), cache.into_inner())
            .await?
    } else {
        repository
            .get_project_latest(project.as_str(), cache.into_inner())
            .await?
    };
    if let Some(value) = value {
        Ok(HttpResponse::Ok().json(value))
    } else {
        Ok(HttpResponse::NotFound().finish())
    }
}
