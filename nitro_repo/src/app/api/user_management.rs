use axum::{
    body::Body,
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use utoipa::{OpenApi, ToSchema};

use crate::{
    app::{
        authentication::{password, Authentication},
        responses::MissingPermission,
        NitroRepo,
    },
    error::InternalError,
};
use nr_core::{
    database::user::{
        user_utils, ChangePasswordNoCheck, NewUserRequest, UserSafeData, UserType as _,
    },
    user::{
        permissions::{HasPermissions, UpdatePermissions, UserPermissions},
        Email, Username,
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        list_users,
        get_user,
        create_user,
        is_taken,
        update_permissions,
        update_password
    ),
    components(schemas(IsTaken, UpdatePermissions))
)]
pub struct UserManagementAPI;
pub fn user_management_routes() -> axum::Router<NitroRepo> {
    axum::Router::new()
        .route("/list", axum::routing::get(list_users))
        .route("/get/:user_id", axum::routing::get(get_user))
        .route("/create", axum::routing::post(create_user))
        .route("/is-taken", axum::routing::post(is_taken))
        .route(
            "/update/:user_id/permissions",
            axum::routing::put(update_permissions),
        )
        .route(
            "/update/:user_id/password",
            axum::routing::put(update_password),
        )
}
#[utoipa::path(
    get,
    path = "/list",
    responses(
        (status = 200, description = "List All registered users", body = [UserSafeData])
    )
)]
#[instrument]
pub async fn list_users(
    auth: Authentication,
    State(site): State<NitroRepo>,
) -> Result<Response, InternalError> {
    if !auth.is_admin_or_user_manager() {
        return Ok(MissingPermission::UserManager.into_response());
    }
    let users = UserSafeData::get_all(&site.database).await?;
    Ok(Json(users).into_response())
}
#[utoipa::path(
    get,
    path = "/get/{user_id}",
    responses(
        (status = 200, description = "User Info", body = UserSafeData),
        (status = 404, description = "User not found")
    )
)]
pub async fn get_user(
    auth: Authentication,
    State(site): State<NitroRepo>,
    Path(user_id): Path<i32>,
) -> Result<Response, InternalError> {
    if !auth.is_admin_or_user_manager() {
        return Ok(MissingPermission::UserManager.into_response());
    }
    let Some(user) = UserSafeData::get_by_id(user_id, &site.database).await? else {
        return Ok(Response::builder()
            .status(http::StatusCode::NOT_FOUND)
            .body("User not found".into())
            .unwrap());
    };
    Ok(Json(user).into_response())
}
#[utoipa::path(
    post,
    request_body = NewUserRequest,
    path = "/create",
    responses(
        (status = 200, description = "User Created", body = UserSafeData),
    )
)]
pub async fn create_user(
    auth: Authentication,
    State(site): State<NitroRepo>,
    Json(user): Json<NewUserRequest>,
) -> Result<Response, InternalError> {
    if !auth.is_admin_or_user_manager() {
        return Ok(MissingPermission::UserManager.into_response());
    }
    if user_utils::is_username_taken(&user.username, &site.database).await? {
        return Ok(Response::builder()
            .status(http::StatusCode::CONFLICT)
            .body("Username already taken".into())
            .unwrap());
    }
    if user_utils::is_email_taken(&user.email, &site.database).await? {
        return Ok(Response::builder()
            .status(http::StatusCode::CONFLICT)
            .body("Email already taken".into())
            .unwrap());
    }
    let user = user
        .insert(UserPermissions::default(), site.as_ref())
        .await?;
    Ok(Json(user).into_response())
}
#[derive(Deserialize, ToSchema)]
#[serde(tag = "type", content = "value")]
pub enum IsTaken {
    Username(String),
    Email(String),
}

#[utoipa::path(
    post,
    path = "/is-taken",
    request_body = IsTaken,
    responses(
        (status = 204, description = "Value is available"),
        (status = 409, description = "Value is Taken"),
    )
)]
pub async fn is_taken(
    State(site): State<NitroRepo>,
    auth: Authentication,
    Json(is_taken): Json<IsTaken>,
) -> Result<Response, InternalError> {
    if !auth.is_admin_or_user_manager() {
        return Ok(MissingPermission::UserManager.into_response());
    }
    let (taken, what) = match is_taken {
        IsTaken::Username(username) => {
            if let Err(err) = Username::new(username.clone()) {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(err.to_string().into())
                    .unwrap());
            }
            (
                user_utils::is_username_taken(&username, &site.database).await?,
                "username",
            )
        }
        IsTaken::Email(email) => {
            if let Err(err) = Email::new(email.clone()) {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(err.to_string().into())
                    .unwrap());
            }
            (
                user_utils::is_email_taken(&email, &site.database).await?,
                "email",
            )
        }
    };
    if taken {
        Ok(Response::builder()
            .status(StatusCode::CONFLICT)
            .body(format!("{} is Taken", what).into())
            .unwrap())
    } else {
        Ok(Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body("".into())
            .unwrap())
    }
}

#[utoipa::path(
    put,
    path = "/update/{user_id}/permissions",
    request_body = UpdatePermissions,
    responses(
        (status = 204, description = "Permissions were updated"),
        (status = 404, description = "User not found"),
    )
)]
pub async fn update_permissions(
    auth: Authentication,
    State(site): State<NitroRepo>,
    Path(user_id): Path<i32>,
    Json(permissions): Json<UpdatePermissions>,
) -> Result<Response, InternalError> {
    if !auth.is_admin_or_user_manager() {
        return Ok(MissingPermission::UserManager.into_response());
    }
    let Some(user) = UserSafeData::get_by_id(user_id, &site.database).await? else {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("User not found".into())
            .unwrap());
    };
    let mut new_permissions = user.permissions.clone().0;
    permissions.apply(&mut new_permissions);
    user.update_permissions(new_permissions, &site.database)
        .await?;
    Ok(Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(Body::empty())
        .unwrap())
}

#[utoipa::path(
    put,
    request_body = ChangePasswordNoCheck,
    path = "/update/{user}/password",
    responses(
        (status = 204, description = "Password Changed"),
        (status = 404, description = "Token Does Not Exist")
    ),
)]
pub async fn update_password(
    auth: Authentication,
    State(site): State<NitroRepo>,
    Path(user_id): Path<i32>,
    Json(password_reset): Json<ChangePasswordNoCheck>,
) -> Result<Response, InternalError> {
    if !auth.is_admin_or_user_manager() {
        return Ok(MissingPermission::UserManager.into_response());
    }
    let Some(user) = UserSafeData::get_by_id(user_id, &site.database).await? else {
        return Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("User not found".into())
            .unwrap());
    };
    let Some(encrypted_password) = password::encrypt_password(&password_reset.password) else {
        return Ok(Response::builder()
            .status(400)
            .body("Failed to encrypt password".into())
            .unwrap());
    };
    user.update_password(Some(encrypted_password), &site.database)
        .await?;
    Ok(Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(Body::empty())
        .unwrap())
}
pub struct AdminUpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
}
