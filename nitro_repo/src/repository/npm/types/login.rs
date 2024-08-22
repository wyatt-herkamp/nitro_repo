use std::fmt::Debug;

use axum::response::{IntoResponse, Response};
use chrono::{DateTime, FixedOffset};
use derive_more::derive::{AsRef, Deref, From};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::repository::RepoResponse;
pub static ISO_8601: &str = "%Y-%m-%dT%H:%M:%S.%f";
#[derive(Serialize, Deserialize)]
pub struct CouchDBLoginRequest {
    pub name: String,
    pub password: String,
    pub email: String,
    #[serde(rename = "type")]
    pub login_type: String,
    #[serde(default)]
    pub roles: Vec<Value>,
    pub date: DateTime<FixedOffset>,
}
#[derive(Debug, From, AsRef, Deref)]
pub struct CouchDBTime(DateTime<FixedOffset>);
impl Serialize for CouchDBTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.0.format(ISO_8601).to_string().serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for CouchDBTime {
    fn deserialize<D>(deserializer: D) -> Result<CouchDBTime, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        DateTime::parse_from_str(&s, ISO_8601)
            .map_err(serde::de::Error::custom)
            .map(CouchDBTime::from)
    }
}
impl Debug for CouchDBLoginRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CouchDBLogin")
            .field("name", &self.name)
            .field("password", &"********")
            .field("email", &self.email)
            .field("login_type", &self.login_type)
            .field("roles", &self.roles)
            .field("date", &self.date)
            .finish()
    }
}
#[derive(Debug, Serialize, Deserialize, From)]
pub struct CouchDBLoginResponse {
    pub token: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct NewLoginResponse {
    pub done_url: String,
    pub login_url: String,
}
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
        RepoResponse::Generic(value.into_response())
    }
}
