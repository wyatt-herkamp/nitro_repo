pub mod database;
pub mod utils;
pub mod web;

use crate::authentication::auth_token::database::TokenProperties;
use crate::authentication::auth_token::utils::hash_token;
use crate::error::internal_error::InternalError;
use crate::settings::models::Database;
use crate::system::user::database::{Model, UserSafeData};
use crate::system::user::{UserEntity, UserModel};
use crate::utils::get_current_time;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
pub use database::ActiveModel as ActiveAuthTokenModel;
pub use database::Entity as AuthTokenEntity;
pub use database::Model as AuthTokenModel;
use futures_util::TryStreamExt;
use log::error;
use rand::distributions::Alphanumeric;
use rand::Rng;
use sea_orm::FromQueryResult;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, FromQueryResult)]
pub struct TokenResponse {
    pub id: i64,
    pub properties: TokenProperties,
    pub user_id: i64,
    pub created: i64,
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
pub async fn get_token(
    token: impl AsRef<str>,
    connection: &DatabaseConnection,
) -> Result<Option<(database::Model, UserSafeData)>, InternalError> {
    let string = hash_token(token);
    // Look for all data that matches the filter.
    let result: Option<(database::Model, Option<UserModel>)> = database::Entity::find()
        .filter(database::Column::TokenHash.eq(string))
        .find_also_related(UserEntity)
        .one(connection)
        .await?;
    match result {
        Some((model, user)) => {
            if let Some(user) = user {
                Ok(Some((model, user.into())))
            } else {
                Err(InternalError::Error("No user found".to_string()))
            }
        }
        None => Ok(None),
    }
}

pub fn generate_token() -> String {
    let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(40)
        .map(char::from)
        .collect();
    token
}

pub fn token_expiration(add: i64) -> i64 {
    get_current_time() + add
}
