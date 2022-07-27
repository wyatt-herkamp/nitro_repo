use crate::authentication::auth_token::database::TokenProperties;
use crate::authentication::auth_token::{generate_token, token_expiration};
use crate::authentication::{verify_login, Authentication};
use crate::system::user::UserModel;
use crate::utils::get_current_time;
use actix_web::http::StatusCode;
use actix_web::web;
use actix_web::{delete, get, post, HttpRequest, HttpResponse};
use chrono::Duration;
use log::error;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use sea_orm::{DatabaseConnection, IntoActiveModel};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;
pub fn authentication_router(cfg: &mut web::ServiceConfig) {
    cfg.service(create_token);
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TokenAPISecure {
    pub username: String,
    pub password: String,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewTokenRequest {
    pub token_name: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewTokenResponse {
    pub token: String,
    pub token_id: Uuid,
    pub expiration: i64,
}

#[get("/token/list")]
pub async fn list_tokens(
    database: web::Data<DatabaseConnection>,
    login: web::Json<TokenAPISecure>,
) -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::NoContent().finish())
}
#[post("/token/create")]
pub async fn create_token(
    connection: web::Data<DatabaseConnection>,
    login: web::Json<TokenAPISecure>,
    new_token: web::Query<NewTokenRequest>,
) -> actix_web::Result<HttpResponse> {
    let login = login.into_inner();
    let user: UserModel = verify_login(login.username, login.password, &connection).await??;
    let uuid = Uuid::new_v4();
    let string = generate_token();
    let i = token_expiration(Duration::days(31).num_milliseconds());
    let v = super::AuthTokenModel {
        id: uuid.clone(),
        token: string.clone(),
        expiration: i,
        properties: TokenProperties {
            description: new_token.token_name.clone(),
        },
        created: get_current_time(),
        user_id: user.id,
    };
    super::AuthTokenEntity::insert(v.into_active_model())
        .exec(connection.as_ref())
        .await;
    Ok(HttpResponse::Ok().json(NewTokenResponse {
        token: string,
        token_id: uuid,
        expiration: i,
    }))
}

#[delete("/token/{id}")]
pub async fn delete_token(
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
) -> actix_web::Result<HttpResponse> {
    let user: UserModel = auth.get_user(&database).await??;
    Ok(HttpResponse::NoContent().finish())
}
