use crate::schema::settings::dsl::settings;
use crate::storage::models::Storage;
use crate::system::models::{AuthToken, ForgotPassword, User};
use crate::utils::get_current_time;
use crate::{storage, system, utils};
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
pub fn add_new_storage(s: &Storage, conn: &MysqlConnection) -> Result<(), diesel::result::Error> {
    use crate::schema::storages::dsl::*;
    diesel::insert_into(storages)
        .values(s)
        .execute(conn)
        .unwrap();
    Ok(())
}
pub fn get_storages(
    conn: &MysqlConnection,
) -> Result<Vec<storage::models::Storage>, diesel::result::Error> {
    use crate::schema::storages::dsl::*;
    Ok(storages.load::<storage::models::Storage>(conn)?)
}
