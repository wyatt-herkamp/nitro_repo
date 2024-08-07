use axum::{
    body::Body,
    response::{IntoResponse, Response},
};
use derive_more::derive::From;
use http::StatusCode;

use super::RepositoryStorageName;
#[derive(Debug, From)]
pub enum RepositoryNotFound {
    RepositoryAndNameLookup(RepositoryStorageName),
    Uuid(uuid::Uuid),
}
impl IntoResponse for RepositoryNotFound {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::RepositoryAndNameLookup(lookup) => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(format!(
                    "Repository {}/{} not found",
                    lookup.storage_name, lookup.repository_name
                )))
                .unwrap(),
            Self::Uuid(uuid) => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from(format!("Repository not found: {:?}", uuid)))
                .unwrap(),
        }
    }
}
