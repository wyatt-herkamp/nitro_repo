use crate::authentication::auth_token::database::TokenProperties;
use crate::authentication::auth_token::utils::hash_token;
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
use sea_orm::{ColumnTrait, EntityTrait, FromQueryResult, QueryFilter};
use sea_orm::{DatabaseConnection, IntoActiveModel};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::digest::FixedOutput;
use sha2::{Digest, Sha512};
use uuid::Uuid;

pub fn authentication_router(cfg: &mut web::ServiceConfig) {
    cfg.service(create_token)
        .service(list_tokens)
        .service(delete_token);
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewTokenResponse {
    pub token: String,
    pub token_id: Uuid,
}
#[derive(Serialize, Deserialize, FromQueryResult, Clone, Debug)]
pub struct TokenResponse {
    pub id: Uuid,
    pub properties: TokenProperties,
    pub created: i64,
}
#[get("/token/list")]
pub async fn list_tokens(
    database: web::Data<DatabaseConnection>,
    authentication: Authentication,
) -> actix_web::Result<HttpResponse> {
    let login = authentication.get_user(database.as_ref()).await??;
    let tokens = AuthTokenEntity::find()
        .filter(super::database::Column::UserId.eq(login.id))
        .into_model::<TokenResponse>()
        .all(database.as_ref())
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().json(tokens))
}

#[post("/token/create")]
pub async fn create_token(
    connection: web::Data<DatabaseConnection>,
    login: web::Json<SecureAction<Option<String>>>,
) -> actix_web::Result<HttpResponse> {
    let login = login.into_inner();
    let user = login.verify(connection.get_ref()).await??;
    let token = generate_token();
    let hash = hash_token(&token);
    let uuid = Uuid::new_v4();
    let value = ActiveAuthTokenModel {
        id: Set(uuid.clone()),
        token_hash: Set(hash),
        properties: Set(TokenProperties {
            description: login
                .into_inner()
                .and_then(|x| if x.is_empty() { None } else { Some(x) }),
        }),
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
    Ok(HttpResponse::Created().json(response))
}

#[delete("/token/{id}")]
pub async fn delete_token(
    database: web::Data<DatabaseConnection>,
    delete_token: web::Path<Uuid>,
    authentication: Authentication,
) -> actix_web::Result<HttpResponse> {
    let user = authentication.get_user(database.get_ref()).await??;
    let result = AuthTokenEntity::delete_many()
        .filter(
            super::database::Column::Id
                .eq(delete_token.into_inner())
                .and(super::database::Column::UserId.eq(user.id)),
        )
        .exec(database.as_ref())
        .await
        .map_err(ErrorInternalServerError)?;
    if result.rows_affected == 0 {
        Ok(HttpResponse::NotFound().finish())
    } else {
        Ok(HttpResponse::NoContent().finish())
    }
}
