use std::{net::SocketAddr, str::FromStr};

use axum::{
    body::Body,
    extract::{ConnectInfo, Path, State},
    response::Response,
    routing::{get, post},
    Json,
};
use axum_extra::{
    headers::{Origin, UserAgent},
    TypedHeader,
};
use http::StatusCode;
use lettre::Address;
use nr_core::database::user::{
    password_reset::{RequestDetails, UserPasswordReset},
    ChangePasswordNoCheck, UserModel, UserSafeData, UserType,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};
use utoipa::ToSchema;

use crate::{
    app::{
        authentication::password,
        email_service::{template, Email, EmailDebug},
        NitroRepo,
    },
    error::InternalError,
};

pub fn password_reset_routes() -> axum::Router<NitroRepo> {
    axum::Router::new()
        .route("/request", post(request_password_reset))
        .route("/check/{token}", get(does_exist))
        .route("/{token}", post(perform_password_change))
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RequestPasswordReset {
    pub email: String,
}
#[derive(Debug, Serialize)]
pub struct PasswordResetEmail {
    pub token: UserPasswordReset,
    pub panel_url: String,
    pub username: String,
    pub required: bool,
}

impl Email for PasswordResetEmail {
    template!("password_reset");

    fn subject() -> &'static str {
        "Password Reset"
    }

    fn debug_info(self) -> EmailDebug {
        EmailDebug {
            to: self.username,
            subject: Self::subject(),
        }
    }
}
#[utoipa::path(
    post,
    path = "/password-reset/request",
    request_body = RequestPasswordReset,
    responses(
        (status = 200, description = "Returns a JSON Schema for the config type")
    ),
)]
async fn request_password_reset(
    State(site): State<NitroRepo>,
    TypedHeader(origin): TypedHeader<Origin>,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(password_reset): Json<RequestPasswordReset>,
) -> Result<Response, InternalError> {
    let address = match Address::from_str(&password_reset.email) {
        Ok(ok) => ok,
        Err(err) => {
            warn!("Invalid email address: {}", err);
            return Ok(Response::builder().status(400).body(Body::empty()).unwrap());
        }
    };
    let request_details = RequestDetails {
        ip_address: addr.ip().to_string(),
        user_agent: user_agent.to_string(),
    };
    let origin = if origin.is_null() {
        return Ok(Response::builder().status(400).body(Body::empty()).unwrap());
    } else {
        origin.to_string()
    };
    debug!(?request_details, ?origin, "Requesting password reset");
    let user = UserModel::get_by_email(&password_reset.email, &site.database).await?;
    if let Some(user) = user {
        let token = UserPasswordReset::create(user.id, request_details, &site.database).await?;
        let email: PasswordResetEmail = PasswordResetEmail {
            token,
            panel_url: origin,
            username: user.username.into(),
            required: false,
        };
        site.email_access.send_one_fn(address, email)
    }
    Ok(Response::builder().status(200).body(Body::empty()).unwrap())
}
#[utoipa::path(
    get,
    path = "/password-reset/check/{token}",
    responses(
        (status = 204, description = "Token Exists"),
        (status = 404, description = "Token Does Not Exist")
    ),
)]
async fn does_exist(
    State(site): State<NitroRepo>,
    Path(token): Path<String>,
) -> Result<Response, InternalError> {
    let token = UserPasswordReset::does_token_exist_and_valid(&token, &site.database).await?;
    if token {
        Ok(Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .unwrap())
    } else {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap())
    }
}

#[utoipa::path(
    post,
    request_body = ChangePasswordNoCheck,
    path = "/password-reset/{token}",
    responses(
        (status = 204, description = "Password Changed"),
        (status = 404, description = "Token Does Not Exist")
    ),
)]
async fn perform_password_change(
    State(site): State<NitroRepo>,
    Path(token): Path<String>,
    Json(password_reset): Json<ChangePasswordNoCheck>,
) -> Result<Response, InternalError> {
    let Some(request) = UserPasswordReset::get_if_valid(&token, &site.database).await? else {
        return Ok(Response::builder().status(404).body(Body::empty()).unwrap());
    };

    let Some(encrypted_password) = password::encrypt_password(&password_reset.password) else {
        return Ok(Response::builder().status(400).body(Body::empty()).unwrap());
    };
    let Some(user) = UserSafeData::get_by_id(request.user_id, &site.database).await? else {
        return Ok(Response::builder().status(404).body(Body::empty()).unwrap());
    };
    user.update_password(Some(encrypted_password), &site.database)
        .await?;

    request.set_used(&site.database).await?;

    Ok(Response::builder()
        .status(StatusCode::NO_CONTENT)
        .body(Body::empty())
        .unwrap())
}
