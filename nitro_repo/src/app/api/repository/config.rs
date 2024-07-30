use actix_web::{
    get,
    web::{self, Data, ServiceConfig},
    HttpResponse,
};

use crate::{
    app::{authentication::Authentication, NitroRepo},
    error::internal_error::InternalError,
};

pub fn init(service: &mut ServiceConfig) {
    service
        .service(config_schema)
        .service(config_validate)
        .service(config_default);
}
#[get("/{key}/schema")]
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

#[get("/{key}/validate")]
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

#[get("/{key}/default")]
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
