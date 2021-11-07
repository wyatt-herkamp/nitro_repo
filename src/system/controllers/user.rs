use crate::api_response::{APIResponse, SiteResponse};


use crate::system::action::{delete_user_db, get_user_by_username, get_users, update_user};
use crate::system::models::User;
use crate::system::utils::{get_user_by_header, new_user, ModifyUser, NewPassword, NewUser, NewUserError};

use crate::DbPool;
use actix_web::{get, post, web, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::error::response::{bad_request, mismatching_passwords, not_found, unauthorized};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListUsers {
    pub users: Vec<User>,
}

#[get("/api/admin/user/list")]
pub async fn list_users(
    pool: web::Data<DbPool>,
    r: HttpRequest,
) -> SiteResponse {
    let connection = pool.get()?;

    let admin = get_user_by_header(r.headers(), &connection)?;
    if admin.is_none() || !admin.unwrap().permissions.admin {
        return unauthorized();
    }
    let vec = get_users(&connection)?;

    let response = ListUsers { users: vec };
    return APIResponse::respond_new(Some(response), &r);
}

#[post("/api/admin/user/add")]
pub async fn add_user(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    nc: web::Json<NewUser>,
) -> SiteResponse {
    let connection = pool.get()?;

    let admin = get_user_by_header(r.headers(), &connection)?;
    if admin.is_none() || !admin.unwrap().permissions.admin {
        return unauthorized();
    }
    let user = new_user(nc.0, &connection)?;
    if let Err(e) = user {
        return match e {
            NewUserError::UsernameAlreadyExists => {
                bad_request("Username already exists")
            }
            NewUserError::UsernameMissing => {
                bad_request("Username Missing")
            }
            NewUserError::EmailAlreadyExists => {
                bad_request("Email already exists")
            }
            NewUserError::EmailMissing => {
                bad_request("Email Missing")
            }
            NewUserError::PasswordDoesNotMatch => {
                mismatching_passwords()
            }
            NewUserError::PasswordMissing => {
                bad_request("Password Missing")
            }
        };
    }
    return APIResponse::from(user.unwrap()).respond(&r);
}

#[post("/api/admin/user/{user}/modify")]
pub async fn modify_user(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    web::Path(user): web::Path<String>,
    nc: web::Json<ModifyUser>,
) -> SiteResponse {
    let connection = pool.get()?;

    let admin = get_user_by_header(r.headers(), &connection)?;
    if admin.is_none() || !admin.unwrap().permissions.admin {
        return unauthorized();
    }
    let user = get_user_by_username(&user, &connection)?;
    if user.is_none(){
        return not_found();
    }
    let mut user = user.unwrap();
    user.update(nc.0);
    update_user(&user, &connection)?;
    return APIResponse::from(Some(user)).respond(&r);
}

#[post("/api/admin/user/{user}/password")]
pub async fn change_password(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    web::Path(user): web::Path<String>,
    nc: web::Json<NewPassword>,
) -> SiteResponse {
    let connection = pool.get()?;

    let admin = get_user_by_header(r.headers(), &connection)?;
    if admin.is_none() || !admin.unwrap().permissions.admin {
        return unauthorized();
    }
    let user = get_user_by_username(&user, &connection)?;
    if user.is_none(){
        return not_found();
    }
    let mut user = user.unwrap();
    let string = nc.0.hash()?;
    if string.is_none() {
        return mismatching_passwords();
    }
    user.set_password(string.unwrap());
    update_user(&user, &connection)?;
    return APIResponse::from(Some(user)).respond(&r);
}

#[get("/api/admin/user/{user}/delete")]
pub async fn delete_user(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    web::Path(user): web::Path<String>,
) -> SiteResponse {
    let connection = pool.get()?;


    let admin = get_user_by_header(r.headers(), &connection)?;
    if admin.is_none() || !admin.unwrap().permissions.admin {
        return unauthorized();
    }
    let option = get_user_by_username(&user, &connection)?;
    if option.is_none() {
        return not_found();
    }
    delete_user_db(&option.unwrap().id, &connection)?;
    return APIResponse::new(true, Some(true)).respond(&r);
}
