use crate::schema::settings::dsl::settings;
use crate::system::models::{AuthToken, ForgotPassword, User};
use crate::utils::get_current_time;
use crate::{system, utils, storage};
use diesel::prelude::*;
use diesel::MysqlConnection;
pub fn get_storage_by_name(
    d: String,
    conn: &MysqlConnection,
) -> Result<Option<storage::models::Storage>, diesel::result::Error> {
    use crate::schema::storages::dsl::*;

    let found_mod = storages
        .filter(name.like(d))
        .first::<storage::models::Storage>(conn)
        .optional()?;

    Ok(found_mod)
}