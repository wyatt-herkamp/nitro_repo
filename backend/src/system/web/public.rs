use actix_web::cookie::Cookie;
use actix_web::web;
use actix_web::{post, HttpRequest, HttpResponse};
use actix_web::http::StatusCode;
use log::error;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::authentication::session::SessionManager;
use crate::authentication::session::SessionManagerType;
use crate::authentication::verify_login;

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
    let user = verify_login(nc.username.clone(), nc.password.clone(), &connection).await??;
    let cookie = r.cookie("session");
    if let Some(cookie) = cookie {
        actix_web::rt::spawn(async move {
            if (session_manager.set_user(cookie.value(), user.id).await).is_err() {
                error!(
                "Unable to save user {} to cookie {}",
                user.id,
                cookie.value()
            );
            }
        });
        Ok(HttpResponse::Ok().json(user))
    } else {
        log::warn!("No cookie found for {r:?}");
        Ok(HttpResponse::Ok().status(StatusCode::BAD_GATEWAY).finish())
    }
}
