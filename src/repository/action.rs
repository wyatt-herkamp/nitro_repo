use crate::schema::settings::dsl::settings;
use crate::system::models::{AuthToken, ForgotPassword, User};
use crate::utils::get_current_time;
use crate::{system, utils, storage, repository};
use diesel::prelude::*;
use diesel::MysqlConnection;
pub fn get_repo_by_name_and_storage(
    d: String, storage: i64,
    conn: &MysqlConnection,
) -> Result<Option<repository::models::Repository>, diesel::result::Error> {
    use crate::schema::repositories::dsl::*;

    let found_mod = repositories
        .filter(name.like(d).and(storage.eq(storage)))
        .first::<repository::models::Repository>(conn)
        .optional()?;

    Ok(found_mod)
}