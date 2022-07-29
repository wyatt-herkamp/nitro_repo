pub mod database;
pub mod web;

use crate::authentication::auth_token::database::TokenProperties;
use crate::error::internal_error::InternalError;
use crate::utils::get_current_time;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
pub use database::ActiveModel as ActiveAuthTokenModel;
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
/// 0 = no user. i64 is cheaper than Option<i64>
pub async fn get_token(
    user_id: i64,
    token: impl AsRef<str>,
    connection: &DatabaseConnection,
) -> Result<Option<database::Model>, InternalError> {
    let (_, end) = token.as_ref().split_at(token.as_ref().len() - 8);
    let mut expr = database::Column::TokenLastEight.eq(end);
    if user_id != 0 {
        expr = expr.and(database::Column::UserId.eq(user_id))
    }
    let result = database::Entity::find()
        .filter(expr)
        .all(connection)
        .await?;
    let option = result.into_iter().find(|v| {
        let argon2 = Argon2::default();
        if let Ok(hash) = PasswordHash::new(v.token_hash.as_str()) {
            if argon2
                .verify_password(token.as_ref().as_bytes(), &hash)
                .is_err()
            {
                return false;
            } else {
                true
            }
        } else {
            false
        }
    });
    Ok(option)
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
