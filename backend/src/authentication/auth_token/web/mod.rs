use crate::authentication::auth_token::database::TokenProperties;
use crate::authentication::auth_token::{
    generate_token, token_expiration, ActiveAuthTokenModel, AuthTokenEntity, AuthTokenModel,
};
use crate::authentication::{verify_login, Authentication, SecureAction};
use crate::system::hash;
use crate::system::user::UserModel;
use crate::utils::get_current_time;
use actix_web::error::ErrorInternalServerError;
use actix_web::http::StatusCode;
use actix_web::web;
use actix_web::{delete, get, post, HttpRequest, HttpResponse};
use chrono::Duration;
use log::error;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use sea_orm::{DatabaseConnection, IntoActiveModel};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

pub fn authentication_router(cfg: &mut web::ServiceConfig) {
    cfg.service(create_token);
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewTokenResponse {
    pub token: String,
    pub token_id: Uuid,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TokenResponse {
    pub expiration: i64,
    pub properties: TokenProperties,
    pub created: i64,
}
#[get("/token/list")]
pub async fn list_tokens(
    database: web::Data<DatabaseConnection>,
    login: web::Json<SecureAction<()>>,
) -> actix_web::Result<HttpResponse> {
    Ok(HttpResponse::NoContent().finish())
}

#[post("/token/create")]
pub async fn create_token(
    connection: web::Data<DatabaseConnection>,
    login: web::Json<SecureAction<Option<String>>>,
) -> actix_web::Result<HttpResponse> {
    let login = login.into_inner();
    let user = login.verify(connection.get_ref()).await??;
    let token = generate_token();
    let hash = hash(&token)?;
    let token_last_eight = token.split_at(token.len() - 8).1.to_string();
    let uuid = Uuid::new_v4();
    let value = ActiveAuthTokenModel {
        id: Set(uuid.clone()),
        token_hash: Set(hash),
        properties: Set(TokenProperties {
            description: login.into_inner(),
        }),
        token_last_eight: Set(token_last_eight),
        user_id: Set(user.id),
        created: Set(get_current_time()),
    };
    let result = AuthTokenEntity::insert(value)
        .exec(connection.as_ref())
        .await
        .map_err(ErrorInternalServerError)?;
    let response = NewTokenResponse {
        token: token,
        token_id: uuid,
    };
    Ok(HttpResponse::Ok().json(response))
}

#[delete("/token/{id}")]
pub async fn delete_token(
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
) -> actix_web::Result<HttpResponse> {
    let user = auth.get_user(&database).await??;
    Ok(HttpResponse::NoContent().finish())
}
