use std::path::PathBuf;

use actix_web::HttpRequest;
use actix_web::web::Bytes;
use diesel::MysqlConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::internal_error::InternalError;
use crate::storage::models::Storage;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FrontendResponse {
    pub content: String,
}
