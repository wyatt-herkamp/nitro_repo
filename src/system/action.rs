use diesel::MysqlConnection;
use diesel::prelude::*;

use crate::{system, utils};
use crate::system::models::{AuthToken, SessionToken, User, UserListResponse, UserPermissions, UserResponse};

pub fn get_users(conn: &MysqlConnection) -> Result<Vec<UserListResponse>, diesel::result::Error> {
    use crate::schema::users::dsl::*;
    users.select((id, name)).load::<UserListResponse>(conn)
}



pub fn update_user(user: i64, email: &String, username: &String, conn: &MysqlConnection) -> Result<(), diesel::result::Error> {
    use crate::schema::users::dsl::*;
    let _result1 = diesel::update(users.filter(id.eq(user)))
        .set((
            email.eq(email),
            name.eq(name),
        ))
        .execute(conn);
    Ok(())
}pub fn update_user_permissions(user: &i64, perms: &UserPermissions,conn: &MysqlConnection) -> Result<(), diesel::result::Error> {
    use crate::schema::users::dsl::*;
    let _result1 = diesel::update(users.filter(id.eq(user)))
        .set((
            permissions.eq(perms),
      ))
        .execute(conn);
    Ok(())
}

pub fn update_user_password(
    user: &i64,
    _password: String,
    conn: &MysqlConnection,
) -> Result<(), diesel::result::Error> {
    use crate::schema::users::dsl::*;
    let _result1 = diesel::update(users.filter(id.eq(user)))
        .set((password.eq(password), ))
        .execute(conn);
    Ok(())
}

pub fn get_user_by_id(
    d: &i64,
    conn: &MysqlConnection,
) -> Result<Option<system::models::User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    let found_mod = users
        .filter(id.eq(d))
        .first::<system::models::User>(conn)
        .optional()?;

    Ok(found_mod)
}

pub fn get_user_by_id_response(
    d: &i64,
    conn: &MysqlConnection,
) -> Result<Option<UserResponse>, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    users
        .filter(id.eq(d))
        .select((id, name, username, email, permissions, created))
        .first::<UserResponse>(conn)
        .optional()
}

pub fn get_user_by_email(
    d: &String,
    conn: &MysqlConnection,
) -> Result<Option<system::models::User>, diesel::result::Error> {
    use crate::schema::users::dsl::*;

    let found_mod = users
        .filter(email.like(d))
        .first::<system::models::User>(conn)
        .optional()?;

    Ok(found_mod)
}

pub fn delete_user_db(d: &i64, conn: &MysqlConnection) -> Result<bool, diesel::result::Error> {
    use crate::schema::users::dsl::*;
    let x = diesel::delete(users).filter(id.eq(d)).execute(conn)?;
    Ok(x >= 1)
}

pub fn get_user_by_username(
    d: &String,
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
    a_token: &String,
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
    token: &String,
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
    get_user_by_id(&result.user, conn)
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
    user_id: &i64,
    conn: &MysqlConnection,
) -> Result<Vec<system::models::AuthToken>, diesel::result::Error> {
    use crate::schema::session_tokens::dsl::*;
    let found_token = session_tokens
        .filter(user.eq(user_id))
        .load::<system::models::AuthToken>(conn)?;
    Ok(found_token)
}
