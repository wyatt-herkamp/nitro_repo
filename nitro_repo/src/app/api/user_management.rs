use axum::{
    Json,
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use http::StatusCode;
use nr_core::{
    database::entities::user::{
        ChangePasswordNoCheck, NewUserRequest, UserSafeData, UserType as _, auth_token::AuthToken,
        permissions::FullUserPermissions, user_utils,
    },
    user::{
        Email, Username,
        permissions::{HasPermissions, UpdatePermissions},
        scopes::NRScope,
    },
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use utoipa::{OpenApi, ToSchema};

use crate::{
    app::{
        NitroRepo,
        api::require_scope,
        authentication::{Authentication, password},
        responses::MissingPermission,
    },
    error::InternalError,
    utils::{ResponseBuilder, conflict::ConflictResponse, json::JsonBody},
};

#[derive(OpenApi)]
#[openapi(
    paths(
        list_users,
        get_user,
        create_user,
        is_taken,
        update_permissions,
        update_user,
        update_password,
        revoke_user_tokens
    ),
    components(schemas(IsTaken, UpdatePermissions, UpdateUserRequest))
)]
pub struct UserManagementAPI;
pub fn user_management_routes() -> axum::Router<NitroRepo> {
    axum::Router::new()
        .route("/list", axum::routing::get(list_users))
        .route("/get/{user_id}", axum::routing::get(get_user))
        .route(
            "/get/{user_id}/permissions",
            axum::routing::get(get_user_permissions),
        )
        .route("/create", axum::routing::post(create_user))
        .route("/is-taken", axum::routing::post(is_taken))
        .route(
            "/update/{user_id}/permissions",
            axum::routing::put(update_permissions),
        )
        .route("/update/{user_id}", axum::routing::put(update_user))
        .route(
            "/update/{user_id}/password",
            axum::routing::put(update_password),
        )
        .route(
            "/{user_id}/tokens/revoke-all",
            axum::routing::delete(revoke_user_tokens),
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
    if let Some(denied) = require_scope(&auth, NRScope::ReadUser, &site).await? {
        return Ok(denied);
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
    if let Some(denied) = require_scope(&auth, NRScope::ReadUser, &site).await? {
        return Ok(denied);
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
    get,
    path = "/get/{user_id}/permissions",
    responses(
        (status = 200, description = "User Info", body = UserSafeData),
        (status = 404, description = "User not found")
    )
)]
pub async fn get_user_permissions(
    auth: Authentication,
    State(site): State<NitroRepo>,
    Path(user_id): Path<i32>,
) -> Result<Response, InternalError> {
    if !auth.is_admin_or_user_manager() {
        return Ok(MissingPermission::UserManager.into_response());
    }
    if let Some(denied) = require_scope(&auth, NRScope::ReadUser, &site).await? {
        return Ok(denied);
    }
    let Some(user) = FullUserPermissions::get_by_id(user_id, site.as_ref()).await? else {
        return Ok(ResponseBuilder::not_found()
            .error_reason("User not found")
            .body("User not found"));
    };
    Ok(ResponseBuilder::ok().json(&user))
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
    JsonBody(user): JsonBody<NewUserRequest>,
) -> Result<Response, InternalError> {
    if !auth.is_admin_or_user_manager() {
        return Ok(MissingPermission::UserManager.into_response());
    }
    if let Some(denied) = require_scope(&auth, NRScope::CreateUser, &site).await? {
        return Ok(denied);
    }
    if user_utils::is_username_taken(&user.username, &site.database).await? {
        return Ok(ConflictResponse::from("username").into_response());
    }
    if user_utils::is_email_taken(&user.email, &site.database).await? {
        return Ok(ConflictResponse::from("email").into_response());
    }
    let user = user.insert(site.as_ref()).await?;
    Ok(ResponseBuilder::ok().json(&user))
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
        (status = 409, description = "Value is Taken", body = String, content_type = "text/plain"),
    )
)]
pub async fn is_taken(
    State(site): State<NitroRepo>,
    auth: Authentication,
    JsonBody(is_taken): JsonBody<IsTaken>,
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
        Ok(ResponseBuilder::conflict()
            .content_type(mime::TEXT_PLAIN_UTF_8)
            .body(format!("{} is Taken", what)))
    } else {
        Ok(ResponseBuilder::no_content().empty())
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
    JsonBody(permissions): JsonBody<UpdatePermissions>,
) -> Result<Response, InternalError> {
    if !auth.is_admin_or_user_manager() {
        return Ok(MissingPermission::UserManager.into_response());
    }
    if let Some(denied) = require_scope(&auth, NRScope::EditUser, &site).await? {
        return Ok(denied);
    }
    let Some(user) = UserSafeData::get_by_id(user_id, &site.database).await? else {
        return Ok(ResponseBuilder::not_found()
            .error_reason("User not found")
            .empty());
    };
    permissions
        .update_permissions(user.id, &site.database)
        .await?;
    Ok(ResponseBuilder::no_content().empty())
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
    JsonBody(password_reset): JsonBody<ChangePasswordNoCheck>,
) -> Result<Response, InternalError> {
    if !auth.is_admin_or_user_manager() {
        return Ok(MissingPermission::UserManager.into_response());
    }
    if let Some(denied) = require_scope(&auth, NRScope::EditUser, &site).await? {
        return Ok(denied);
    }
    let Some(user) = UserSafeData::get_by_id(user_id, &site.database).await? else {
        return Ok(ResponseBuilder::not_found()
            .error_reason("User not found")
            .empty());
    };
    let Some(encrypted_password) = password::encrypt_password(&password_reset.password) else {
        return Ok(ResponseBuilder::bad_request().body("Failed to encrypt password"));
    };
    user.update_password(Some(encrypted_password), &site.database)
        .await?;
    Ok(ResponseBuilder::no_content().empty())
}
pub struct AdminUpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, serde::Serialize, Deserialize, ToSchema)]
pub struct RevokedTokensResponse {
    pub revoked: u64,
}

/// Revokes every API token belonging to a user.
///
/// The admin surface had no way to do this — "revoke" appeared nowhere in the codebase — so a
/// leaked token could only be dealt with by the user who owned it, assuming they still could.
#[utoipa::path(
    delete,
    path = "/{user_id}/tokens/revoke-all",
    responses(
        (status = 200, description = "How many tokens were revoked", body = RevokedTokensResponse),
        (status = 404, description = "User not found"),
    )
)]
#[instrument]
pub async fn revoke_user_tokens(
    auth: Authentication,
    State(site): State<NitroRepo>,
    Path(user_id): Path<i32>,
) -> Result<Response, InternalError> {
    if !auth.is_admin_or_user_manager() {
        return Ok(MissingPermission::UserManager.into_response());
    }
    if let Some(denied) = require_scope(&auth, NRScope::EditUser, &site).await? {
        return Ok(denied);
    }
    let Some(user) = UserSafeData::get_by_id(user_id, &site.database).await? else {
        return Ok(ResponseBuilder::not_found()
            .error_reason("User not found")
            .empty());
    };
    let revoked = AuthToken::delete_all_for_user(user.id, site.as_ref()).await?;
    tracing::info!(?user_id, revoked, "Revoked every token for a user");
    Ok(ResponseBuilder::ok().json(&RevokedTokensResponse { revoked }))
}

#[derive(Debug, Default, Serialize, Deserialize, ToSchema)]
#[serde(default)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
}

/// Updates a user's details.
///
/// The admin user page has had a form for these three fields with no `@submit` handler and no
/// endpoint behind it, so editing a name, username or email was silently impossible. The entity
/// methods (`update_name`, `update_username`, `update_email_address`) already existed; nothing
/// exposed them.
#[utoipa::path(
    put,
    request_body = UpdateUserRequest,
    path = "/update/{user_id}",
    responses(
        (status = 204, description = "The user was updated"),
        (status = 400, description = "The username or email is not valid"),
        (status = 404, description = "User not found"),
        (status = 409, description = "That username or email is already taken"),
    ),
)]
#[instrument(skip(site))]
pub async fn update_user(
    auth: Authentication,
    State(site): State<NitroRepo>,
    Path(user_id): Path<i32>,
    JsonBody(request): JsonBody<UpdateUserRequest>,
) -> Result<Response, InternalError> {
    if !auth.is_admin_or_user_manager() {
        return Ok(MissingPermission::UserManager.into_response());
    }
    if let Some(denied) = require_scope(&auth, NRScope::EditUser, &site).await? {
        return Ok(denied);
    }
    let Some(user) = UserSafeData::get_by_id(user_id, &site.database).await? else {
        return Ok(ResponseBuilder::not_found()
            .error_reason("User not found")
            .empty());
    };

    // Validated and checked for conflicts before anything is written, so a request that changes two
    // fields cannot leave one applied and the other rejected.
    if let Some(username) = &request.username
        && username != user.username.as_ref()
    {
        if let Err(error) = Username::new(username.clone()) {
            return Ok(ResponseBuilder::bad_request().body(error.to_string()));
        }
        if user_utils::is_username_taken(username, &site.database).await? {
            return Ok(ConflictResponse::from("username").into_response());
        }
    }
    if let Some(email) = &request.email
        && email != user.email.as_ref()
    {
        if let Err(error) = Email::new(email.clone()) {
            return Ok(ResponseBuilder::bad_request().body(error.to_string()));
        }
        if user_utils::is_email_taken(email, &site.database).await? {
            return Ok(ConflictResponse::from("email").into_response());
        }
    }

    if let Some(name) = &request.name {
        user.update_name(name, &site.database).await?;
    }
    if let Some(username) = &request.username {
        user.update_username(username, &site.database).await?;
    }
    if let Some(email) = &request.email {
        user.update_email_address(email, &site.database).await?;
    }

    Ok(ResponseBuilder::no_content().empty())
}
