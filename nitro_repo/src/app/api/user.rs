use axum::{
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::{
    app::{
        authentication::{verify_login, Authentication, MeWithSession},
        NitroRepo, NitroRepoState,
    },
    error::internal_error::InternalError,
};
pub fn user_routes() -> axum::Router<NitroRepo> {
    axum::Router::new()
        .route("/me", axum::routing::get(me))
        .route("/login", axum::routing::post(login))
}
#[instrument]
pub async fn me(auth: Authentication) -> Response {
    match auth {
        Authentication::AuthToken(_, user) => {
            let response = Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(serde_json::to_string(&user).unwrap()))
                .unwrap();
            response
        }
        Authentication::Session(session, user) => {
            let response = Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(
                    serde_json::to_string(&MeWithSession::from((session, user))).unwrap(),
                ))
                .unwrap();
            response
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginRequest {
    pub email_or_username: String,
    pub password: String,
}
#[instrument]
pub async fn login(
    State(site): State<NitroRepo>,
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

    let session = site.session_manager.create_session(user.id, duration)?;

    let user_with_session = MeWithSession::from((session.clone(), user));

    let response = Response::builder()
        .status(StatusCode::OK)
        .body(Body::new(
            serde_json::to_string(&user_with_session).unwrap(),
        ))
        .unwrap();
    Ok(response)
}
