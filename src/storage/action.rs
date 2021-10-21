use crate::storage::models::Storage;

use crate::storage;
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
pub fn get_storage_by_id(
    d: i64,
    conn: &MysqlConnection,
) -> Result<Option<storage::models::Storage>, diesel::result::Error> {
    use crate::schema::storages::dsl::*;

    let found_mod = storages
        .filter(id.eq(d))
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
