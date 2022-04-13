use actix_web::{get, patch, post, web, HttpRequest};
use serde::{Deserialize, Serialize};

use crate::api_response::{APIResponse, SiteResponse};
use crate::database::DbPool;
use crate::error::response::{already_exists_what, bad_request, not_found, unauthorized};
use crate::system::action::{
    add_new_user, delete_user_db, get_user_by_email, get_user_by_id_response, get_user_by_username,
    get_users, update_user, update_user_password, update_user_permissions,
};
use crate::system::models::{User, UserListResponse};
use crate::system::permissions::UserPermissions;
use crate::system::utils::{get_user_by_header, hash, ModifyUser, NewPassword, NewUser};
use crate::utils::get_current_time;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListUsers {
    pub users: Vec<UserListResponse>,
}

#[get("/api/admin/user/list")]
pub async fn list_users(pool: web::Data<DbPool>, r: HttpRequest) -> SiteResponse {
    let connection = pool.get()?;

    let admin = get_user_by_header(r.headers(), &connection)?;
    if admin.is_none() || !admin.unwrap().permissions.admin {
        return unauthorized();
    }
    let vec = get_users(&connection)?;

    let response = ListUsers { users: vec };
    APIResponse::respond_new(Some(response), &r)
}

#[get("/api/admin/user/get/{user}")]
pub async fn get_user(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<i64>,
) -> SiteResponse {
    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let repo = get_user_by_id_response(&path.into_inner(), &connection)?;

    APIResponse::respond_new(repo, &r)
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
    if get_user_by_username(&nc.username, &connection)?.is_some() {
        return already_exists_what("username");
    }
    if get_user_by_email(&nc.email, &connection)?.is_some() {
        return already_exists_what("email");
    }
    let user = User {
        id: 0,
        name: nc.0.name,
        username: nc.0.username,
        email: nc.0.email,
        password: hash(nc.0.password)?,
        permissions: UserPermissions::default(),
        created: get_current_time(),
    };
    add_new_user(&user, &connection)?;
    APIResponse::from(get_user_by_username(&user.username, &connection)?).respond(&r)
}

#[patch("/api/admin/user/{user}/modify")]
pub async fn modify_user(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    name: web::Path<String>,
    nc: web::Json<ModifyUser>,
) -> SiteResponse {
    let connection = pool.get()?;
    let name = name.into_inner();
    let admin = get_user_by_header(r.headers(), &connection)?;
    if admin.is_none() || !admin.unwrap().permissions.admin {
        return unauthorized();
    }
    let user = get_user_by_username(&name, &connection)?;
    if user.is_none() {
        return not_found();
    }
    update_user(user.unwrap().id, &nc.email, &nc.name, &connection)?;
    APIResponse::from(get_user_by_username(&name, &connection)?).respond(&r)
}

#[patch("/api/admin/user/{user}/modify/permissions")]
pub async fn update_permission(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    permissions: web::Json<UserPermissions>,
    path: web::Path<String>,
) -> SiteResponse {
    let connection = pool.get()?;
    let user = path.into_inner();
    let admin = get_user_by_header(r.headers(), &connection)?;
    if admin.is_none() || !admin.unwrap().permissions.admin {
        return unauthorized();
    }
    let user = get_user_by_username(&user, &connection)?;
    if user.is_none() {
        return not_found();
    }
    let user = user.unwrap();
    update_user_permissions(&user.id, &permissions.into_inner(), &connection)?;
    APIResponse::from(true).respond(&r)
}

#[post("/api/admin/user/{user}/password")]
pub async fn change_password(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    user: web::Path<String>,
    nc: web::Json<NewPassword>,
) -> SiteResponse {
    let connection = pool.get()?;
    let admin = get_user_by_header(r.headers(), &connection)?;
    if admin.is_none() || !admin.unwrap().permissions.admin {
        return unauthorized();
    }
    let user = user.into_inner();

    let user = get_user_by_username(&user, &connection)?;
    if user.is_none() {
        return not_found();
    }
    let user = user.unwrap();
    let string = hash(nc.0.password)?;

    update_user_password(&user.id, string, &connection)?;
    APIResponse::from(Some(user)).respond(&r)
}

#[get("/api/admin/user/{user}/delete")]
pub async fn delete_user(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    user: web::Path<String>,
) -> SiteResponse {
    let connection = pool.get()?;

    let admin = get_user_by_header(r.headers(), &connection)?;
    if admin.is_none() || !admin.unwrap().permissions.admin {
        return unauthorized();
    }
    let user = user.into_inner();

    let option = get_user_by_username(&user, &connection)?;
    if option.is_none() {
        return not_found();
    }
    delete_user_db(&option.unwrap().id, &connection)?;
    APIResponse::new(true, Some(true)).respond(&r)
}
