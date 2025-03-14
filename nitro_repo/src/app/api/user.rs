use std::net::SocketAddr;

use axum::{
    Json,
    body::Body,
    extract::{ConnectInfo, State},
    response::{IntoResponse, Response},
    routing::post,
};
use axum_extra::{
    TypedHeader,
    extract::{
        CookieJar,
        cookie::{Cookie, Expiration},
    },
    headers::UserAgent,
};
use http::{StatusCode, header::SET_COOKIE};
use nr_core::{
    database::entities::user::{
        ChangePasswordNoCheck, ChangePasswordWithCheck, User, UserSafeData, UserType,
        auth_token::{AuthTokenRepositoryScope, AuthTokenScope},
        permissions::FullUserPermissions,
    },
    user::token::{AuthTokenFullResponse, AuthTokenResponse},
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use utoipa::{OpenApi, ToSchema};
mod password_reset;
mod tokens;
use crate::{
    app::{
        NitroRepo,
        authentication::{
            Authentication, MeWithSession,
            password::{self, verify_password},
            session::{Session, SessionError},
            verify_login,
        },
    },
    error::InternalError,
};
#[derive(OpenApi)]
#[openapi(
    paths(
        me,
        whoami,
        login,
        get_sessions,
        logout,
        change_password,
        password_reset::request_password_reset,
        password_reset::does_exist,
        password_reset::perform_password_change,
        tokens::create,
        tokens::list,
        tokens::get_token
    ),
    components(schemas(
        UserSafeData,
        MeWithSession,
        Session,
        password_reset::RequestPasswordReset,
        ChangePasswordWithCheck,
        ChangePasswordNoCheck,
        AuthTokenFullResponse,
        AuthTokenResponse,
        AuthTokenRepositoryScope,
        AuthTokenScope
    ))
)]
pub struct UserAPI;
pub fn user_routes() -> axum::Router<NitroRepo> {
    axum::Router::new()
        .route("/me", axum::routing::get(me))
        .route("/me/permissions", axum::routing::get(me_permissions))
        .route("/change-password", post(change_password))
        .route("/whoami", axum::routing::get(whoami))
        .route("/login", axum::routing::post(login))
        .route("/sessions", axum::routing::get(get_sessions))
        .route("/logout", axum::routing::post(logout))
        .nest("/password-reset", password_reset::password_reset_routes())
        .nest("/token", tokens::token_routes())
}
#[utoipa::path(
    get,
    path = "/me",
    responses(
        (status = 200, description = "List Current User with Session", body = [MeWithSession])
    ),
    security(
        ("session" = [])
    )
)]
#[instrument]
pub async fn me(auth: Authentication) -> Response {
    match auth {
        Authentication::AuthToken(_, _) => Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("Use whoami instead of me for Auth Tokens".into())
            .unwrap(),
        Authentication::Session(session, user) => {
            let response = Json(MeWithSession::from((session, user)));
            response.into_response()
        }
    }
}
#[utoipa::path(
    get,
    path = "/me/permissions",
    responses(
        (status = 200, description = "Get All the permissions for the current user", body = [FullUserPermissions])
    )
)]
#[instrument]
pub async fn me_permissions(
    auth: Authentication,
    State(site): State<NitroRepo>,
) -> Result<Response, InternalError> {
    let Some(user) = FullUserPermissions::get_by_id(auth.get_id(), site.as_ref()).await? else {
        return Ok(Response::builder()
            .status(http::StatusCode::NOT_FOUND)
            .body("User not found".into())
            .unwrap());
    };
    Ok(Json(user).into_response())
}
#[instrument]
#[utoipa::path(
    get,
    path = "/whoami",
    responses(
        (status = 200, description = "Get current user data", body = UserSafeData)
    ),
    security(
        ("api_key" = []),
        ("session" = [])
    )
)]
pub async fn whoami(auth: Authentication) -> Json<UserSafeData> {
    match auth {
        Authentication::AuthToken(_, user) => Json(user),
        Authentication::Session(_, user) => Json(user),
    }
}
#[utoipa::path(
    get,
    path = "/sessions",
    responses(
        (status = 200, description = "List All Active Sessions", body = [Session])
    )
)]
#[instrument]
pub async fn get_sessions(
    auth: Authentication,
    State(site): State<NitroRepo>,
) -> Result<Response, SessionError> {
    let sessions = site
        .session_manager
        .filter_table(false, |session| session.user_id == auth.id)?;
    let response = Json(sessions).into_response();
    Ok(response)
}
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct LoginRequest {
    pub email_or_username: String,
    pub password: String,
}

#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "List All Active Sessions", body = MeWithSession),
        (status = 400, description = "Bad Request. Note: This request requires a User-Agent Header"),
        (status = 401, description = "Unauthorized"),
    )
)]
#[instrument]
pub async fn login(
    State(site): State<NitroRepo>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(login): axum::Json<LoginRequest>,
) -> Result<Response, InternalError> {
    let LoginRequest {
        email_or_username,
        password,
    } = login;
    let user = match verify_login(email_or_username, password, &site.database).await {
        Ok(ok) => ok,
        Err(err) => {
            return Ok(err.into_response());
        }
    };
    let duration = chrono::Duration::days(1);
    let user_agent = user_agent.to_string();
    let ip = addr.ip().to_string();
    let session = site
        .session_manager
        .create_session(user.id, user_agent, ip, duration)?;
    let cookie = Cookie::build(("session", session.session_id.clone()))
        .secure(true)
        .path("/")
        .expires(Expiration::Session)
        .build();
    let user_with_session = MeWithSession::from((session.clone(), user));
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/json")
        .header(SET_COOKIE, cookie.encoded().to_string())
        .body(serde_json::to_string(&user_with_session).unwrap().into())
        .unwrap();
    Ok(response)
}
#[utoipa::path(
    post,
    path = "/logout",
    responses(
        (status = 204, description = "Successfully Logged Out"),
        (status = 400, description = "Bad Request. Must be a session")
    )
)]
pub async fn logout(
    auth: Authentication,
    State(site): State<NitroRepo>,
    cookie: CookieJar,
) -> Result<Response, InternalError> {
    match auth {
        Authentication::AuthToken(_, _) => {
            let response = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("Must be a session".into())
                .unwrap();
            Ok(response)
        }
        Authentication::Session(session, _) => {
            site.session_manager.delete_session(&session.session_id)?;
            let empty_session_cookie = Cookie::build("session").removal().build();
            let cookies = cookie.add(empty_session_cookie);
            Ok((cookies, StatusCode::NO_CONTENT).into_response())
        }
    }
}

#[utoipa::path(
    post,
    path = "/change-password",
    request_body = ChangePasswordWithCheck,
    responses(
        (status = 204, description = "Successfully Changed Password"),
        (status = 400, description = "Bad Request. Must be a session")
    )
)]
pub async fn change_password(
    auth: Authentication,
    State(site): State<NitroRepo>,
    Json(change_password): Json<ChangePasswordWithCheck>,
) -> Result<Response, InternalError> {
    let Authentication::Session(_, user) = auth else {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("Must be a session".into())
            .unwrap());
    };
    let ChangePasswordWithCheck {
        old_password,
        new_password,
    } = change_password;
    let Some(user_password) = User::get_password_by_id(user.id, &site.database).await? else {
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("User password not found".into())
            .unwrap());
    };
    if let Err(err) = verify_password(&old_password, Some(user_password.as_str())) {
        return Ok(err.into_response());
    }
    let Some(new_password) = password::encrypt_password(&new_password) else {
        return Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("Failed to encrypt password".into())
            .unwrap());
    };
    user.update_password(Some(new_password), &site.database)
        .await?;
    Ok(Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(Body::empty())
        .unwrap())
}
