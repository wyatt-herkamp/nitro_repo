#![allow(unused_variables)]

use actix_web::http::StatusCode;
use bytes::Bytes;

use http::{RepoResponse, RepositoryRequest};
use nr_storage::DynStorage;

pub mod dyn_repository;
pub mod http;
pub mod maven;
mod repo_type;
pub use repo_type::*;
pub trait Repository: Send + Sync + Clone {
    fn get_storage(&self) -> DynStorage;
    /// The Repository type. This is used to identify the Repository type in the database
    fn get_type(&self) -> &'static str;
    /// Config types that this Repository type has.
    fn config_types(&self) -> Vec<String>;
    /// Handles a get request to a Repo
    async fn handle_get(
        &self,
        request: RepositoryRequest<'_>,
    ) -> Result<RepoResponse, actix_web::Error> {
        Ok(RepoResponse::new_from_string(
            format!("Head is not implemented for this type: {}", self.get_type()),
            StatusCode::IM_A_TEAPOT,
        ))
    }
    /// Handles a Post Request to a Repo
    async fn handle_post(
        &self,
        request: RepositoryRequest<'_>,
        bytes: Bytes,
    ) -> Result<RepoResponse, actix_web::Error> {
        Ok(RepoResponse::new_from_string(
            format!("Head is not implemented for this type: {}", self.get_type()),
            StatusCode::IM_A_TEAPOT,
        ))
    }
    /// Handles a PUT Request to a Repo
    async fn handle_put(
        &self,
        request: RepositoryRequest<'_>,
        bytes: Bytes,
    ) -> Result<RepoResponse, actix_web::Error> {
        Ok(RepoResponse::new_from_string(
            format!("Head is not implemented for this type: {}", self.get_type()),
            StatusCode::IM_A_TEAPOT,
        ))
    }
    /// Handles a PATCH Request to a Repo
    async fn handle_patch(
        &self,
        request: RepositoryRequest<'_>,
        bytes: Bytes,
    ) -> Result<RepoResponse, actix_web::Error> {
        Ok(RepoResponse::new_from_string(
            format!("Head is not implemented for this type: {}", self.get_type()),
            StatusCode::IM_A_TEAPOT,
        ))
    }
    /// Handles a HAPIResponseAD Request to a Repo
    async fn handle_head(
        &self,
        request: RepositoryRequest<'_>,
    ) -> Result<RepoResponse, actix_web::Error> {
        Ok(RepoResponse::new_from_string(
            format!("Head is not implemented for this type: {}", self.get_type()),
            StatusCode::IM_A_TEAPOT,
        ))
    }
}
