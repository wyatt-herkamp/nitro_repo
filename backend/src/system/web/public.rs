use actix_web::{HttpRequest, HttpResponse, HttpResponseBuilder, post};
use actix_web::cookie::Cookie;
use actix_web::web;
use log::error;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use crate::authentication::session::SessionManager;
use crate::authentication::session::SessionManagerType;
use crate::authentication::verify_login;
use crate::system::user::UserModel;
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
    let cookie: Cookie = r.cookie("session").unwrap();
    actix_web::rt::spawn(async move {
        if (session_manager.set_user(cookie.value(), user.id).await).is_err() {
            error!(
                "Unable to save user {} to cookie {}",
                user.id,
                cookie.value()
            );
        }
    });
    Ok(HttpResponse::Ok().finish())
}
