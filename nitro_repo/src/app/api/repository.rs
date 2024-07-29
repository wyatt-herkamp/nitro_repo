use actix_web::{
    get, put,
    web::{self, Data, ServiceConfig},
    HttpResponse,
};
use nr_core::{
    database::repository::{DBRepository, GenericDBRepositoryConfig},
    repository::config,
    user::permissions::HasPermissions,
};
use uuid::Uuid;

use crate::{
    app::{authentication::Authentication, DatabaseConnection, NitroRepo},
    error::internal_error::InternalError,
    repository::Repository,
};

pub fn init(service: &mut ServiceConfig) {
    service
        .service(repository_types)
        .service(update_config)
        .service(config_schema)
        .service(config_validate)
        .service(config_default)
        .service(update_config);
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

#[get("/config/{key}/schema")]
pub async fn config_schema(
    site: Data<NitroRepo>,
    key: web::Path<String>,
) -> Result<HttpResponse, InternalError> {
    // TODO: Add Client side caching

    let key = key.into_inner();
    let Some(config_type) = site.get_repository_config_type(&key) else {
        return Ok(HttpResponse::NotFound().finish());
    };

    let schema = config_type
        .schema()
        .map(|schema| HttpResponse::Ok().json(schema))
        .unwrap_or_else(|| HttpResponse::NotFound().finish());
    Ok(schema)
}

#[get("/config/{key}/validate")]
pub async fn config_validate(
    site: Data<NitroRepo>,
    key: web::Path<String>,
    auth: Authentication,
    config: web::Json<serde_json::Value>,
) -> Result<HttpResponse, InternalError> {
    //TODO: Check permissions
    let key = key.into_inner();
    let Some(config_type) = site.get_repository_config_type(&key) else {
        return Ok(HttpResponse::NotFound().finish());
    };

    let response = match config_type.validate_config(config.into_inner()) {
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(err) => HttpResponse::BadRequest().body(err.to_string()),
    };
    Ok(response)
}

#[get("/config/{key}/default")]
pub async fn config_default(
    site: Data<NitroRepo>,
    key: web::Path<String>,
) -> Result<HttpResponse, InternalError> {
    // TODO: Add Client side caching
    let key = key.into_inner();
    let Some(config_type) = site.get_repository_config_type(&key) else {
        return Ok(HttpResponse::NotFound().finish());
    };

    let default = match config_type.default() {
        Ok(ok) => HttpResponse::Ok().json(ok),
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    };
    Ok(default)
}
