use actix_web::{post, web, HttpRequest};

use crate::api_response::{APIResponse, SiteResponse};

use crate::error::response::unauthorized;
use crate::system::action::{add_new_session_token, get_user_by_email, get_user_by_username};
use crate::system::models::SessionToken;
use crate::system::utils::generate_session_token;
use crate::utils::{default_expiration, get_current_time};
use crate::DbPool;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[post("/api/login")]
pub async fn login(pool: web::Data<DbPool>, r: HttpRequest, nc: web::Json<Login>) -> SiteResponse {
    let connection = pool.get()?;

    let username = nc.username.clone();
    let user = if username.contains("@") {
        get_user_by_email(&username, &connection)?
    } else {
        get_user_by_username(&username, &connection)?
    };
    if user.is_none() {
        return unauthorized();
    }
    let user = user.unwrap();
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(user.password.as_str())?;
    let x = argon2.verify_password(nc.password.clone().as_bytes(), &parsed_hash);
    if x.is_err() {
        return unauthorized();
    }
    let token = SessionToken {
        id: 0,
        user: user.id.clone(),
        token: generate_session_token(&connection)?,
        expiration: default_expiration(),
        created: get_current_time(),
    };
    add_new_session_token(&token, &connection)?;

    return APIResponse::respond_new(Some(token), &r);
}
