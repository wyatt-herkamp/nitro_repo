use crate::system::models::{SessionToken, User, AuthToken};

use crate::{system, utils};
use diesel::prelude::*;
use diesel::MysqlConnection;

pub fn get_users(
    conn: &MysqlConnection,
) -> Result<Vec<system::models::User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;
    Ok(users.load::<system::models::User>(conn)?)
}
pub fn update_user(user: &User, conn: &MysqlConnection) -> Result<(), diesel::result::Error> {
    use crate::schema::users::dsl::*;
    let _result1 = diesel::update(users.filter(id.eq(user.id)))
        .set((
            password.eq(user.password.clone()),
            email.eq(user.email.clone()),
            name.eq(user.name.clone()),
            permissions.eq(user.permissions.clone()),
        ))
        .execute(conn);
    Ok(())
}

pub fn get_user_by_id(
    d: i64,
    conn: &MysqlConnection,
) -> Result<Option<system::models::User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    let found_mod = users
        .filter(id.eq(d))
        .first::<system::models::User>(conn)
        .optional()?;

    Ok(found_mod)
}

pub fn get_user_by_email(
    d: String,
    conn: &MysqlConnection,
) -> Result<Option<system::models::User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    let found_mod = users
        .filter(email.like(d))
        .first::<system::models::User>(conn)
        .optional()?;

    Ok(found_mod)
}

pub fn delete_user_db(d: i64, conn: &MysqlConnection) -> Result<bool, diesel::result::Error> {
    use crate::schema::users::dsl::*;
    let x = diesel::delete(users).filter(id.eq(d)).execute(conn)?;
    Ok(x >= 1)
}

pub fn get_user_by_username(
    d: String,
    conn: &MysqlConnection,
) -> Result<Option<system::models::User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    let found_mod = users
        .filter(username.like(d))
        .first::<system::models::User>(conn)
        .optional()?;

    Ok(found_mod)
}

pub fn add_new_user(s: &User, conn: &MysqlConnection) -> Result<(), diesel::result::Error> {
    use crate::schema::users::dsl::*;
    diesel::insert_into(users).values(s).execute(conn).unwrap();
    Ok(())
}

//Session Token
pub fn get_session_token(
    a_token: String,
    conn: &MysqlConnection,
) -> Result<Option<system::models::SessionToken>, diesel::result::Error> {
    use crate::schema::session_tokens::dsl::*;
    let found_token = session_tokens
        .filter(token.eq(a_token))
        .first::<system::models::SessionToken>(conn)
        .optional()?;
    Ok(found_token)
}

pub fn add_new_session_token(
    t: &SessionToken,
    conn: &MysqlConnection,
) -> Result<(), diesel::result::Error> {
    use crate::schema::session_tokens::dsl::*;
    diesel::insert_into(session_tokens)
        .values(t)
        .execute(conn)
        .unwrap();
    Ok(())
}

pub fn get_user_from_session_token(
    token: String,
    conn: &MysqlConnection,
) -> Result<Option<system::models::User>, diesel::result::Error> {
    let result = get_session_token(token, conn)?;
    if result.is_none() {
        return Ok(None);
    }
    let result = result.unwrap();
    if result.expiration <= utils::get_current_time() {
        return Ok(None);
    }
    return get_user_by_id(result.user, conn);
}



//Session Token

pub fn add_new_auth_token(
    t: &AuthToken,
    conn: &MysqlConnection,
) -> Result<(), diesel::result::Error> {
    use crate::schema::auth_tokens::dsl::*;
    diesel::insert_into(auth_tokens)
        .values(t)
        .execute(conn)
        .unwrap();
    Ok(())
}

pub fn get_tokens(
    user: i64,
    conn: &MysqlConnection,
) -> Result<Vec<system::models::AuthToken>, diesel::result::Error> {
    use crate::schema::session_tokens::dsl::*;
    let found_token = session_tokens
        .filter(user.eq(user))
        .load::<system::models::AuthToken>(conn)?;
    Ok(found_token)
}
