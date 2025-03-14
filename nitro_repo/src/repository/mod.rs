#![allow(unused_variables)]

use std::{fmt::Debug, future::Future};

use nr_core::{
    repository::{Visibility, project::ProjectResolution},
    storage::StoragePath,
};

pub mod prelude {
    pub use super::{DynRepositoryHandlerError, RepositoryFactoryError, RepositoryHandlerError};
    pub use super::{RepoResponse, Repository, RepositoryRequest};
    pub use crate::app::NitroRepo;
    pub use axum::response::{IntoResponse, Response};
    pub use http::StatusCode;
    pub use nr_core::{repository::project::*, repository::*, storage::*};
}
use nr_macros::DynRepositoryHandler;
use nr_storage::DynStorage;
mod staging;
pub use staging::*;
mod repo_http;
pub use repo_http::*;
pub mod commands;
pub mod maven;
pub mod npm;
mod repo_type;
pub use repo_type::*;
use uuid::Uuid;

mod error;
pub mod utils;
use crate::{
    app::{NitroRepo, authentication::AuthenticationError},
    utils::IntoErrorResponse,
};
pub use error::*;
pub trait Repository: Send + Sync + Clone + Debug {
    type Error: IntoErrorResponse + 'static;
    fn get_storage(&self) -> DynStorage;
    /// The Repository type. This is used to identify the Repository type in the database
    fn get_type(&self) -> &'static str;

    fn full_type(&self) -> &'static str {
        self.get_type()
    }
    /// Config types that this Repository type has.
    fn config_types(&self) -> Vec<&str>;
    fn name(&self) -> String;
    fn id(&self) -> Uuid;
    fn visibility(&self) -> Visibility;
    fn is_active(&self) -> bool;
    /// Returns a copy of the site that this Repository is associated with
    fn site(&self) -> NitroRepo;
    fn resolve_project_and_version_for_path(
        &self,
        path: &StoragePath,
    ) -> impl Future<Output = Result<ProjectResolution, Self::Error>> + Send {
        async { Ok(ProjectResolution::default()) }
    }

    async fn reload(&self) -> Result<(), RepositoryFactoryError> {
        Ok(())
    }
    /// Handles a get request to a Repo
    fn handle_get(
        &self,
        request: RepositoryRequest,
    ) -> impl Future<Output = Result<RepoResponse, Self::Error>> + Send {
        async {
            Ok(RepoResponse::unsupported_method_response(
                request.parts.method,
                self.get_type(),
            ))
        }
    }
    /// Handles a Post Request to a Repo
    fn handle_post(
        &self,
        request: RepositoryRequest,
    ) -> impl Future<Output = Result<RepoResponse, Self::Error>> + Send {
        async {
            Ok(RepoResponse::unsupported_method_response(
                request.parts.method,
                self.get_type(),
            ))
        }
    }
    /// Handles a PUT Request to a Repo
    fn handle_put(
        &self,
        request: RepositoryRequest,
    ) -> impl Future<Output = Result<RepoResponse, Self::Error>> + Send {
        async {
            Ok(RepoResponse::unsupported_method_response(
                request.parts.method,
                self.get_type(),
            ))
        }
    }
    /// Handles a PATCH Request to a Repo
    fn handle_patch(
        &self,
        request: RepositoryRequest,
    ) -> impl Future<Output = Result<RepoResponse, Self::Error>> + Send {
        async {
            Ok(RepoResponse::unsupported_method_response(
                request.parts.method,
                self.get_type(),
            ))
        }
    }
    fn handle_delete(
        &self,
        request: RepositoryRequest,
    ) -> impl Future<Output = Result<RepoResponse, Self::Error>> + Send {
        async {
            Ok(RepoResponse::unsupported_method_response(
                request.parts.method,
                self.get_type(),
            ))
        }
    }
    /// Handles a HAPIResponseAD Request to a Repo
    fn handle_head(
        &self,
        request: RepositoryRequest,
    ) -> impl Future<Output = Result<RepoResponse, Self::Error>> + Send {
        async {
            Ok(RepoResponse::unsupported_method_response(
                request.parts.method,
                self.get_type(),
            ))
        }
    }
    fn handle_other(
        &self,
        request: RepositoryRequest,
    ) -> impl Future<Output = Result<RepoResponse, Self::Error>> + Send {
        async {
            Ok(RepoResponse::unsupported_method_response(
                request.parts.method,
                self.get_type(),
            ))
        }
    }
}
#[derive(Debug, Clone, DynRepositoryHandler)]
#[repository_handler(error = DynRepositoryHandlerError)]
pub enum DynRepository {
    Maven(maven::MavenRepository),
    NPM(npm::NPMRegistry),
}
