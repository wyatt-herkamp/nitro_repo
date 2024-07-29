use actix_web::{
    get, post,
    web::{self, Data, ServiceConfig},
    HttpResponse, Scope,
};
use nr_core::{database::user::NewUserRequest, user::permissions::UserPermissions};
use serde::{Deserialize, Serialize};
use tracing::error;
pub mod repository;
pub mod user;
use crate::{
    app::NitroRepo, error::internal_error::InternalError, utils::password::encrypt_password,
};

use super::DatabaseConnection;

pub fn init(service: &mut ServiceConfig) {
    service
        .service(info)
        .service(install)
        .service(Scope::new("/user").configure(user::init))
        .service(Scope::new("/repository").configure(repository::init));
}
#[get("/info")]
pub async fn info(site: Data<NitroRepo>) -> HttpResponse {
    let site = site.instance.lock().clone();
    HttpResponse::Ok().json(site)
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstallRequest {
    pub user: NewUserRequest,
}
#[post("/install")]
pub async fn install(
    site: Data<NitroRepo>,
    request: web::Json<InstallRequest>,
    database: DatabaseConnection,
) -> Result<HttpResponse, InternalError> {
    let mut site = site.instance.lock();
    if site.is_installed {
        return Ok(HttpResponse::NotFound().finish());
    }
    let InstallRequest { mut user } = request.into_inner();
    let password = user
        .password
        .as_ref()
        .and_then(|password| encrypt_password(password));
    if password.is_none() {
        error!("A Password must exist for the first user.");
        return Ok(HttpResponse::BadRequest().finish());
    }
    user.password = password;
    user.insert(UserPermissions::admin(), &database).await?;
    site.is_installed = true;
    Ok(HttpResponse::Ok().finish())
}
