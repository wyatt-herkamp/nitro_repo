use crate::utils::base64_utils;
use rand::distr::Alphanumeric;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
/// Creates a new token checking if it already exists
///
/// Returns a tuple with the token and the hashed token
pub async fn create_token(database: &PgPool) -> Result<(String, String), sqlx::Error> {
    let (token, hashed) = loop {
        let token = generate_token();
        let hashed_token = hash_token(&token);
        let exists: i64 =
            sqlx::query_scalar(r#"SELECT COUNT(id) FROM user_auth_tokens WHERE token = $1"#)
                .bind(&hashed_token)
                .fetch_one(database)
                .await?;
        if exists == 0 {
            break (token, hashed_token);
        }
    };
    Ok((token, hashed))
}
/// Generates a new token for the user
pub fn generate_token() -> String {
    // TODO: Secure this
    StdRng::from_os_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect()
}
/// Hashes the token using SHA256 and encodes it in base64
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token);
    base64_utils::encode(hasher.finalize())
}
