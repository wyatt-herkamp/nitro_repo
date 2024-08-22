use crate::utils::base64_utils;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
/// Creates a new token checking if it already exists
pub async fn create_token(database: &PgPool) -> Result<String, sqlx::Error> {
    let token = loop {
        let token = generate_token();
        let exists: i64 =
            sqlx::query_scalar(r#"SELECT COUNT(id) FROM user_auth_tokens WHERE token = $1"#)
                .bind(&token)
                .fetch_one(database)
                .await?;
        if exists == 0 {
            break token;
        }
    };
    Ok(token)
}
/// Generates a new token for the user
pub fn generate_token() -> String {
    // TODO: Secure this
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}

pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token);
    base64_utils::encode(&hasher.finalize())
}
