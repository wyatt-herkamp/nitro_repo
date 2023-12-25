use actix_web::{get, http::StatusCode, post, web, HttpRequest, HttpResponse};
use log::error;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::{
    authentication::{session::SessionManager, verify_login},
    NitroRepoData,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[post("/login")]
pub async fn login(
    connection: web::Data<DatabaseConnection>,
    session_manager: web::Data<SessionManager>,
    r: HttpRequest,
    nc: web::Json<Login>,
) -> actix_web::Result<HttpResponse> {
    let user = verify_login(nc.username.clone(), nc.password.clone(), &connection)
        .await?
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Invalid login"))?;
    let session = session_manager
        .create_session_default(user.id)
        .map_err(|e| {
            error!("Failed to create session: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to create session")
        })?;

    Ok(HttpResponse::Ok().cookie(session.new_cookie()).finish())
}

#[get("/version")]
pub async fn get_version(data: NitroRepoData) -> HttpResponse {
    HttpResponse::Ok().json(&data.current_version)
}
