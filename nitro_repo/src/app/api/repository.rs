use actix_web::{
    get, put,
    web::{self, Data, ServiceConfig},
    HttpResponse, Scope,
};
use nr_core::{
    database::repository::{DBRepository, DBRepositoryWithStorageName, GenericDBRepositoryConfig},
    user::permissions::HasPermissions,
};
use uuid::Uuid;

use crate::{
    app::{authentication::Authentication, DatabaseConnection, NitroRepo},
    error::internal_error::InternalError,
    repository::Repository,
};
mod config;

pub fn init(service: &mut ServiceConfig) {
    service
        .service(repository_types)
        .service(update_config)
        .service(Scope::new("/config").configure(config::init))
        .service(update_config)
        .service(list_repositories);
}
#[get("/types")]
pub async fn repository_types(site: Data<NitroRepo>) -> HttpResponse {
    // TODO: Add Client side caching

    let types: Vec<_> = site
        .repository_types
        .iter()
        .map(|v| v.get_description())
        .collect();
    HttpResponse::Ok().json(types)
}
#[put("/{repository}/{config_key}")]
pub async fn update_config(
    site: Data<NitroRepo>,
    auth: Authentication,
    params: web::Path<(Uuid, String)>,
    config: web::Json<serde_json::Value>,
    database: DatabaseConnection,
) -> Result<HttpResponse, InternalError> {
    let (repository, config_key) = params.into_inner();
    if !auth.can_edit_repository(repository) {
        return Ok(HttpResponse::Forbidden().finish());
    }
    let Some(config_type) = site.get_repository_config_type(&config_key) else {
        return Ok(HttpResponse::BadRequest().finish());
    };
    let Some(db_repository) = DBRepository::get_by_id(repository, &database).await? else {
        return Ok(HttpResponse::NotFound().finish());
    };
    let Some(repository) = site.get_repository(db_repository.id) else {
        return Ok(HttpResponse::NotFound().finish());
    };
    if !repository.config_types().contains(&config_key) {
        return Ok(HttpResponse::BadRequest().finish());
    }
    let config = config.into_inner();
    if let Err(error) = config_type.validate_config(config.clone()) {
        return Ok(HttpResponse::BadRequest().body(error.to_string()));
    }
    GenericDBRepositoryConfig::add_or_update(db_repository.id, config_key, config, &database)
        .await?;
    //TODO: Update the instance of the repository with the new config
    Ok(HttpResponse::NoContent().finish())
}
#[get("/list")]
pub async fn list_repositories(
    auth: Authentication,
    database: DatabaseConnection,
) -> Result<HttpResponse, InternalError> {
    if !auth.can_view_repositories() {
        return Ok(HttpResponse::Forbidden().finish());
    }
    let repositories = DBRepositoryWithStorageName::get_all(&database).await?;
    Ok(HttpResponse::Ok().json(repositories))
}
