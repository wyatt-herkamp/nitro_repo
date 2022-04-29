use actix_web::{get, patch, post, web, HttpRequest};
use sea_orm::ActiveValue::Set;
use sea_orm::{DatabaseConnection, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::api_response::{APIResponse, SiteResponse};
use crate::error::response::{already_exists_what, not_found, unauthorized};

use crate::system::models::UserListResponse;
use crate::system::permissions::options::CanIDo;
use crate::system::permissions::UserPermissions;
use crate::system::user;
use crate::system::utils::{get_user_by_header, hash, NewPassword, NewUser};
use crate::utils::get_current_time;
use sea_orm::ActiveModelTrait;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::IntoActiveModel;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListUsers {
    pub users: Vec<UserListResponse>,
}

#[get("/api/admin/user/list")]
pub async fn list_users(connection: web::Data<DatabaseConnection>, r: HttpRequest) -> SiteResponse {
    if get_user_by_header(r.headers(), &connection)
        .await?
        .can_i_edit_users()
        .is_err()
    {
        return unauthorized();
    }
    let vec = user::Entity::find()
        .into_model::<UserListResponse>()
        .all(connection.as_ref())
        .await?;

    let response = ListUsers { users: vec };
    APIResponse::respond_new(Some(response), &r)
}

#[get("/api/admin/user/get/{user}")]
pub async fn get_user(
    connection: web::Data<DatabaseConnection>,
    r: HttpRequest,
    path: web::Path<i64>,
) -> SiteResponse {
    if get_user_by_header(r.headers(), &connection)
        .await?
        .can_i_edit_users()
        .is_err()
    {
        return unauthorized();
    }
    let user = user::Entity::find_by_id(path.into_inner())
        .one(connection.as_ref())
        .await?;

    APIResponse::respond_new(user, &r)
}

#[post("/api/admin/user/add")]
pub async fn add_user(
    connection: web::Data<DatabaseConnection>,
    r: HttpRequest,
    nc: web::Json<NewUser>,
) -> SiteResponse {
    if get_user_by_header(r.headers(), &connection)
        .await?
        .can_i_edit_users()
        .is_err()
    {
        return unauthorized();
    }
    if user::get_by_username(&nc.0.username, &connection)
        .await?
        .is_some()
    {
        return already_exists_what("username");
    }
    if user::Entity::find()
        .filter(user::Column::Email.eq(nc.email.clone()))
        .one(connection.as_ref())
        .await?
        .is_some()
    {
        return already_exists_what("email");
    }
    let user = user::ActiveModel {
        id: Default::default(),
        name: Set(nc.0.name),
        username: Set(nc.0.username.clone()),
        email: Set(nc.0.email),
        password: Set(hash(nc.0.password)?),
        permissions: Set(UserPermissions::default().try_into()?),
        created: Set(get_current_time()),
    };
    user::Entity::insert(user).exec(connection.as_ref()).await?;
    APIResponse::from(user::get_by_username(&nc.0.username, &connection).await?).respond(&r)
}

#[patch("/api/admin/user/{user}/modify")]
pub async fn modify_user(
    connection: web::Data<DatabaseConnection>,
    r: HttpRequest,
    path: web::Path<i64>,
    nc: web::Json<user::ModifyUser>,
) -> SiteResponse {
    if get_user_by_header(r.headers(), &connection)
        .await?
        .can_i_edit_users()
        .is_err()
    {
        return unauthorized();
    }

    let user = user::Entity::find_by_id(path.into_inner())
        .one(connection.as_ref())
        .await?;
    if user.is_none() {
        return not_found();
    }
    let model = nc.0.into_active_model();
    let user = model.update(connection.as_ref()).await?;
    //update_user(user.unwrap().id, &nc.email, &nc.name, &connection)?;
    APIResponse::from(Some(user)).respond(&r)
}

#[patch("/api/admin/user/{user}/modify/permissions")]
pub async fn update_permission(
    connection: web::Data<DatabaseConnection>,
    r: HttpRequest,
    permissions: web::Json<UserPermissions>,
    path: web::Path<i64>,
) -> SiteResponse {
    if get_user_by_header(r.headers(), &connection)
        .await?
        .can_i_edit_users()
        .is_err()
    {
        return unauthorized();
    }

    let user = user::Entity::find_by_id(path.into_inner())
        .one(connection.as_ref())
        .await?;
    if user.is_none() {
        return not_found();
    }
    let user: user::Model = user.unwrap();
    let mut user_active: user::ActiveModel = user.into_active_model();

    user_active.permissions = Set(permissions.0.try_into()?);

    let user = user_active.update(connection.as_ref()).await?;
    APIResponse::from(Some(user)).respond(&r)
}

#[post("/api/admin/user/{user}/password")]
pub async fn change_password(
    connection: web::Data<DatabaseConnection>,
    r: HttpRequest,
    path: web::Path<i64>,
    nc: web::Json<NewPassword>,
) -> SiteResponse {
    if get_user_by_header(r.headers(), &connection)
        .await?
        .can_i_edit_users()
        .is_err()
    {
        return unauthorized();
    }

    let user = user::Entity::find_by_id(path.into_inner())
        .one(connection.as_ref())
        .await?;
    if user.is_none() {
        return not_found();
    }
    let user: user::Model = user.unwrap();
    let hashed_password: String = hash(nc.0.password)?;
    let mut user_active: user::ActiveModel = user.into_active_model();

    user_active.password = Set(hashed_password);

    let user = user_active.update(connection.as_ref()).await?;
    APIResponse::from(Some(user)).respond(&r)
}

#[get("/api/admin/user/{user}/delete")]
pub async fn delete_user(
    connection: web::Data<DatabaseConnection>,
    r: HttpRequest,
    user: web::Path<i64>,
) -> SiteResponse {
    if get_user_by_header(r.headers(), &connection)
        .await?
        .can_i_edit_users()
        .is_err()
    {
        return unauthorized();
    }
    let user = user.into_inner();

    user::Entity::delete_by_id(user)
        .exec(connection.as_ref())
        .await?;
    APIResponse::new(true, Some(true)).respond(&r)
}
