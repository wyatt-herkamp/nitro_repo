#![allow(unused_variables)]

use bytes::Bytes;

use ::http::StatusCode;
use http::{RepoResponse, RepositoryRequest};
use nr_macros::DynRepositoryHandler;
use nr_storage::DynStorage;

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

    fn reload(&self);
    /// Handles a get request to a Repo
    async fn handle_get(
        &self,
        request: RepositoryRequest,
    ) -> Result<RepoResponse, RepositoryHandlerError> {
        Ok(RepoResponse::new_from_string(
            format!("Get is not implemented for this type: {}", self.get_type()),
            StatusCode::IM_A_TEAPOT,
        ))
    }
    /// Handles a Post Request to a Repo
    async fn handle_post(
        &self,
        request: RepositoryRequest,
    ) -> Result<RepoResponse, RepositoryHandlerError> {
        Ok(RepoResponse::new_from_string(
            format!("Post is not implemented for this type: {}", self.get_type()),
            StatusCode::IM_A_TEAPOT,
        ))
    }
    /// Handles a PUT Request to a Repo
    async fn handle_put(
        &self,
        request: RepositoryRequest,
    ) -> Result<RepoResponse, RepositoryHandlerError> {
        Ok(RepoResponse::new_from_string(
            format!("Head is not implemented for this type: {}", self.get_type()),
            StatusCode::IM_A_TEAPOT,
        ))
    }
    /// Handles a PATCH Request to a Repo
    async fn handle_patch(
        &self,
        request: RepositoryRequest,
    ) -> Result<RepoResponse, RepositoryHandlerError> {
        Ok(RepoResponse::new_from_string(
            format!(
                "Patch is not implemented for this type: {}",
                self.get_type()
            ),
            StatusCode::IM_A_TEAPOT,
        ))
    }
    /// Handles a HAPIResponseAD Request to a Repo
    async fn handle_head(
        &self,
        request: RepositoryRequest,
    ) -> Result<RepoResponse, RepositoryHandlerError> {
        Ok(RepoResponse::new_from_string(
            format!("Head is not implemented for this type: {}", self.get_type()),
            StatusCode::IM_A_TEAPOT,
        ))
    }
}
#[derive(Debug, Clone, DynRepositoryHandler)]
pub enum DynRepository {
    Maven(maven::MavenRepository),
}
#[derive(Debug)]
pub enum RepositoryHandlerError {}
