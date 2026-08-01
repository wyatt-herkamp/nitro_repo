use rand::{RngExt, distr::Alphanumeric};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::utils::base64_utils;
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
/// Generates a new token for the user.
///
/// 32 alphanumeric characters from the OS entropy source, stored only as a SHA-256 hash. The
/// `// TODO: Secure this` that sat here predated that and had been true of an earlier
/// implementation; leaving it read as an open hole that was not one.
///
/// `rand::rng()` replaced `StdRng::from_os_rng()` when rand 0.10 removed the latter. It is a
/// ChaCha12 CSPRNG seeded from the OS and periodically reseeded — no weaker than seeding a fresh
/// `StdRng` per call, and it does not pay for that seeding every time.
pub fn generate_token() -> String {
    rand::rng()
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
