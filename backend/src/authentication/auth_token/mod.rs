pub mod database;
pub mod web;

use crate::authentication::auth_token::database::TokenProperties;
use crate::error::internal_error::InternalError;
use crate::utils::get_current_time;
pub use database::Entity as AuthTokenEntity;
pub use database::Model as AuthTokenModel;
use log::error;
use rand::distributions::Alphanumeric;
use rand::Rng;
use sea_orm::FromQueryResult;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
pub struct TokenResponse {
    pub id: i64,
    pub expiration: i64,
    pub properties: TokenProperties,
    pub created: i64,
    pub user_id: i64,
}

pub async fn get_tokens_by_user(
    user: i64,
    database: &DatabaseConnection,
) -> Result<Vec<TokenResponse>, InternalError> {
    AuthTokenEntity::find()
        .filter(database::Column::UserId.eq(user))
        .into_model::<TokenResponse>()
        .all(database)
        .await
        .map_err(InternalError::DBError)
}

pub async fn get_by_token(
    token: &str,
    connection: &DatabaseConnection,
) -> Result<Option<database::Model>, InternalError> {
    let result = database::Entity::find()
        .filter(database::Column::Token.eq(token))
        .one(connection)
        .await?;
    if let Some(token) = result {
        // Delete Token if Expired
        if token.expiration <= get_current_time() {
            let database = connection.clone();
            actix_web::rt::spawn(async move {
                let database = database;
                if let Err(error) = AuthTokenEntity::delete_by_id(token.id)
                    .exec(&database)
                    .await
                {
                    error!("Unable to delete Auth Token Error: {}", error);
                }
            });
        }
        return Ok(Some(token));
    }
    Ok(None)
}

pub async fn delete_by_token(
    token: &str,
    connection: &DatabaseConnection,
) -> Result<(), InternalError> {
    database::Entity::delete_many()
        .filter(database::Column::Token.eq(token))
        .exec(connection)
        .await?;
    Ok(())
}

pub fn generate_token() -> String {
    let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(24)
        .map(char::from)
        .collect();
    format!("nrp_{}", token)
}

pub fn token_expiration(add: i64) -> i64 {
    get_current_time() + add
}
