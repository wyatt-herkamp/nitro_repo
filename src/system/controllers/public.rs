use actix_web::{get, post, HttpRequest, HttpResponse, web};
use tera::Context;

use crate::DbPool;
use crate::internal_error::InternalError;
use crate::site_response::SiteResponse;
use crate::utils::{installed, default_expiration, get_current_time};
use crate::api_response::APIResponse;
use crate::system::models::AuthToken;
use crate::error::request_error::RequestError;
use crate::system::action::{get_user_by_email, get_user_by_username, add_new_auth_token};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use serde::{Deserialize, Serialize};
use crate::system::utils::{generate_auth_token, get_user_by_header};
use crate::apierror::APIError;


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[post("/api/login")]
pub async fn login(
    pool: web::Data<DbPool>,
    _r: HttpRequest,
    nc: web::Json<Login>,
) -> Result<APIResponse<AuthToken>, RequestError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let username = nc.username.clone();
    let user = if username.contains("@") {
        get_user_by_email(username, &connection)?
    } else {
        get_user_by_username(username, &connection)?
    }
        .ok_or(RequestError::InvalidLogin)?;
    let argon2 = Argon2::default();
    let parsed_hash =
        PasswordHash::new(user.password.as_str()).map_err(|_| RequestError::from("Password Error"))?;
    argon2
        .verify_password(nc.password.clone().as_bytes(), &parsed_hash)
        .map_err(|_| RequestError::InvalidLogin)?;
    let token = AuthToken {
        id: 0,
        user: user.id.clone(),
        token: generate_auth_token(&connection)?,
        expiration: default_expiration(),
        created: get_current_time(),
    };
    add_new_auth_token(&token, &connection)?;

    return Ok(APIResponse::new(true, Some(token)));
}


