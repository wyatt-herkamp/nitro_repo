use crate::authentication::Authentication;
use crate::system::permissions::permissions_checker::CanIDo;
use crate::system::user::UserModel;
use actix_web::{delete, get, post, put, web, HttpResponse};

use super::super::user::database::ActiveModel;
use super::super::user::database::*;
use crate::helpers::unwrap_or_not_found;
use crate::system::hash;
use crate::system::permissions::UserPermissions;
use crate::system::user::database::UserSafeData;
use crate::utils::get_current_time;
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;
use sea_orm::{ColumnTrait, DatabaseConnection, IntoActiveModel, QueryFilter};
use serde::{Deserialize, Serialize};
use this_actix_error::ActixError;
use thiserror::Error;

// struct that derives Serialize and Deserialize contains the number of active storages, number of active repositories, and the number of active users.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SystemStatus {
    pub active_storages: usize,
    pub active_repositories: usize,
    pub active_users: usize,
}

#[get("users/list")]
pub async fn list_users(
    auth: Authentication,
    database: web::Data<DatabaseConnection>,
) -> actix_web::Result<HttpResponse> {
    let user = auth.get_user(&database).await??;
    user.can_i_edit_users()?;
    let result: Vec<UserModel> = super::super::user::get_users(&database)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(result))
}

#[get("user/{id}")]
pub async fn get_user(
    auth: Authentication,
    database: web::Data<DatabaseConnection>,
    id: web::Path<i64>,
) -> actix_web::Result<HttpResponse> {
    let user = auth.get_user(&database).await??;
    user.can_i_edit_users()?;
    let result: UserSafeData =
        unwrap_or_not_found!(super::super::user::get_by_id(id.into_inner(), &database)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?);
    Ok(HttpResponse::Ok().json(result))
}

#[derive(Deserialize, Debug)]
pub struct NewUser {
    pub name: String,
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(Debug, Error, ActixError)]
pub enum NewUserResponse {
    #[error("Username already exists")]
    #[status_code(CONFLICT)]
    UsernameAlreadyExists,
    #[error("Email already exists")]
    #[status_code(CONFLICT)]
    EmailAlreadyExists,
}

#[post("/user")]
pub async fn create_user(
    auth: Authentication,
    database: web::Data<DatabaseConnection>,
    value: web::Json<NewUser>,
) -> actix_web::Result<HttpResponse> {
    let user = auth.get_user(&database).await??;
    user.can_i_edit_users()?;
    let user = value.into_inner();
    if Entity::find()
        .filter(Column::Username.eq(user.username.as_str()))
        .one(database.as_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .is_some()
    {
        return Err(NewUserResponse::UsernameAlreadyExists.into());
    }
    if Entity::find()
        .filter(Column::Email.eq(user.email.as_str()))
        .one(database.as_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?
        .is_some()
    {
        return Err(NewUserResponse::EmailAlreadyExists.into());
    }
    let user: ActiveModel = ActiveModel {
        id: Default::default(),
        name: Set(user.name),
        username: Set(user.username),
        email: Set(user.email),
        password: Set(hash(user.password).unwrap()),
        permissions: Set(UserPermissions::default()),
        created: Set(get_current_time()),
    };
    user.insert(database.as_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Created().finish())
}

#[put("/user/{id}/permissions")]
pub async fn update_permissions(
    auth: Authentication,
    database: web::Data<DatabaseConnection>,
    id: web::Path<i64>,
    permissions: web::Json<UserPermissions>,
) -> actix_web::Result<HttpResponse> {
    let user = auth.get_user(&database).await??;
    user.can_i_edit_users()?;

    let user: Model = crate::helpers::unwrap_or_not_found!(Entity::find()
        .filter(Column::Id.eq(id.into_inner()))
        .one(database.as_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?);
    let mut user: ActiveModel = user.into_active_model();
    user.permissions = Set(permissions.into_inner());
    user.update(database.as_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize, Debug)]
pub struct UpdatePassword {
    pub password: String,
}

#[put("/user/{id}/password")]
pub async fn update_password(
    auth: Authentication,
    database: web::Data<DatabaseConnection>,
    id: web::Path<i64>,
    password: web::Json<UpdatePassword>,
) -> actix_web::Result<HttpResponse> {
    let user = auth.get_user(&database).await??;
    user.can_i_edit_users()?;

    let user: Model = crate::helpers::unwrap_or_not_found!(Entity::find()
        .filter(Column::Id.eq(id.into_inner()))
        .one(database.as_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?);
    let mut user: ActiveModel = user.into_active_model();
    user.password = Set(hash(&password.password)?);
    user.update(database.as_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize, Debug)]
pub struct UpdateUser {
    pub name: String,
    pub email: String,
}

#[put("/user/{id}/data")]
pub async fn update_data(
    auth: Authentication,
    database: web::Data<DatabaseConnection>,
    id: web::Path<i64>,
    data: web::Json<UpdateUser>,
) -> actix_web::Result<HttpResponse> {
    let user = auth.get_user(&database).await??;
    user.can_i_edit_users()?;

    let user: Model = crate::helpers::unwrap_or_not_found!(Entity::find()
        .filter(Column::Id.eq(id.into_inner()))
        .one(database.as_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?);
    let mut user: ActiveModel = user.into_active_model();
    let data = data.into_inner();
    user.name = Set(data.name);
    user.email = Set(data.email);
    user.update(database.as_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::NoContent().finish())
}

#[delete("user/{id}")]
pub async fn delete_user(
    auth: Authentication,
    database: web::Data<DatabaseConnection>,
    id: web::Path<i64>,
) -> actix_web::Result<HttpResponse> {
    let user = auth.get_user(&database).await??;
    user.can_i_edit_users()?;
    let user: Model = unwrap_or_not_found!(Entity::find()
        .filter(Column::Id.eq(id.into_inner()))
        .one(database.as_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?);
    Entity::delete(user.into_active_model())
        .exec(database.as_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(HttpResponse::Ok().finish())
}
