use crate::repository::models::Repository;



use crate::{repository};
use diesel::prelude::*;
use diesel::MysqlConnection;

pub fn get_repo_by_name_and_storage(
    d: String,
    _storage: i64,
    conn: &MysqlConnection,
) -> Result<Option<repository::models::Repository>, diesel::result::Error> {
    use crate::schema::repositories::dsl::*;

    let found_mod = repositories
        .filter(name.like(d).and(storage.eq(storage)))
        .first::<repository::models::Repository>(conn)
        .optional()?;

    Ok(found_mod)
}

pub fn add_new_repository(
    s: &Repository,
    conn: &MysqlConnection,
) -> Result<(), diesel::result::Error> {
    use crate::schema::repositories::dsl::*;
    diesel::insert_into(repositories)
        .values(s)
        .execute(conn)
        .unwrap();
    Ok(())
}

pub fn get_repositories(
    conn: &MysqlConnection,
) -> Result<Vec<repository::models::Repository>, diesel::result::Error> {
    use crate::schema::repositories::dsl::*;
    Ok(repositories.load::<repository::models::Repository>(conn)?)
}

pub fn get_repositories_by_storage(
    _storage: i64,
    conn: &MysqlConnection,
) -> Result<Vec<repository::models::Repository>, diesel::result::Error> {
    use crate::schema::repositories::dsl::*;
    Ok(repositories
        .filter(storage.eq(storage))
        .load::<repository::models::Repository>(conn)?)
}
