pub mod admin;

use crate::authentication::auth_token::database::TokenProperties;
use crate::authentication::auth_token::utils::hash_token;
use crate::authentication::auth_token::{generate_token, ActiveAuthTokenModel, AuthTokenEntity};
use crate::authentication::{Authentication, SecureAction};

use actix_web::error::ErrorInternalServerError;

use actix_web::web;
use actix_web::web::scope;
use actix_web::{delete, get, post, HttpResponse};
use chrono::Local;

use sea_orm::ActiveValue::Set;
use sea_orm::DatabaseConnection;
use sea_orm::{ColumnTrait, EntityTrait, FromQueryResult, QueryFilter};
use serde::{Deserialize, Serialize};

use uuid::Uuid;

pub fn authentication_router(cfg: &mut web::ServiceConfig) {
    cfg.service(
        scope("/token")
            .service(create_token)
            .service(list_tokens)
            .service(delete_token)
            .service(scope("admin").service(admin::delete_token_system)),
    );
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
#[get("/list")]
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

#[post("/create")]
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
        id: Set(uuid),
        token_hash: Set(hash),
        properties: Set(TokenProperties {
            description: login
                .into_inner()
                .and_then(|x| if x.is_empty() { None } else { Some(x) }),
        }),
        user_id: Set(user.id),
        created: Set(Local::now().into()),
    };
    let _result = AuthTokenEntity::insert(value)
        .exec(connection.as_ref())
        .await
        .map_err(ErrorInternalServerError)?;
    let response = NewTokenResponse {
        token,
        token_id: uuid,
    };
    Ok(HttpResponse::Created().json(response))
}

#[delete("/{id}")]
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
