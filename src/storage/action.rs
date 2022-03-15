use diesel::prelude::*;
use diesel::MysqlConnection;

use crate::storage;
use crate::storage::models::Storage;

pub fn get_storage_by_name(
    d: &str,
    conn: &MysqlConnection,
) -> Result<Option<storage::models::Storage>, diesel::result::Error> {
    use crate::schema::storages::dsl::*;

    let found_mod = storages
        .filter(name.like(d))
        .first::<storage::models::Storage>(conn)
        .optional()?;

    Ok(found_mod)
}
pub fn get_storage_by_public_name(
    d: &str,
    conn: &MysqlConnection,
) -> Result<Option<storage::models::Storage>, diesel::result::Error> {
    use crate::schema::storages::dsl::*;

    let found_mod = storages
        .filter(public_name.like(d))
        .first::<storage::models::Storage>(conn)
        .optional()?;

    Ok(found_mod)
}

pub fn get_storage_name_by_id(
    d: &i64,
    conn: &MysqlConnection,
) -> Result<Option<String>, diesel::result::Error> {
    use crate::schema::storages::dsl::*;

    let found_mod = storages
        .select(name)
        .filter(id.eq(d))
        .first(conn)
        .optional()?;

    Ok(found_mod)
}

pub fn get_storage_by_id(
    d: &i64,
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
pub fn delete_storage_by_id(d: &i64, conn: &MysqlConnection) -> Result<bool, diesel::result::Error> {
    use crate::schema::storages::dsl::*;
    let x = diesel::delete(storages).filter(id.eq(d)).execute(conn)?;
    Ok(x >= 1)
}
pub fn get_storages(
    conn: &MysqlConnection,
) -> Result<Vec<storage::models::Storage>, diesel::result::Error> {
    use crate::schema::storages::dsl::*;
    storages.load::<storage::models::Storage>(conn)
}
