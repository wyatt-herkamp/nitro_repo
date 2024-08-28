use axum::response::{IntoResponse, Response};
use couch_db::CouchDBLoginResponse;
use derive_more::derive::From;
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::repository::RepoResponse;
pub mod couch_db;
pub mod web_login;
#[derive(Debug, Serialize, Deserialize)]
pub struct NewLoginResponse {
    pub done_url: String,
    pub login_url: String,
}
#[derive(Debug, From)]
pub enum LoginResponse {
    ValidCouchDBLogin(CouchDBLoginResponse),
    UnsupportedLogin,
}

impl IntoResponse for LoginResponse {
    fn into_response(self) -> axum::response::Response {
        match self {
            LoginResponse::ValidCouchDBLogin(login) => {
                return Response::builder()
                    .status(StatusCode::CREATED)
                    .body(serde_json::to_string(&login).unwrap().into())
                    .unwrap();
            }
            LoginResponse::UnsupportedLogin => {
                return Response::builder()
                    .status(StatusCode::IM_A_TEAPOT)
                    .body("Unsupported Login Type".into())
                    .unwrap();
            }
        }
    }
}
impl From<LoginResponse> for RepoResponse {
    fn from(value: LoginResponse) -> Self {
        RepoResponse::Other(value.into_response())
    }
}
