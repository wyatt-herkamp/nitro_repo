//! What a repository type implements, and the HTTP plumbing that drives it.
//!
//! [`Repository`] is the trait a type writes; [`RepositoryHandler`] is its object-safe shadow, and
//! [`DynRepository`] is what everything else holds. [`RepositoryType`] is the factory that loads
//! one from the database.
//!
//! The application state is not visible from here. A repository gets a [`SiteContext`] — the
//! database, the instance settings, the staging directory, the request metrics, and a hostname
//! index — and resolving one repository from another's address is a separate capability,
//! [`RepositoryResolver`], held by the router rather than by a repository. See [`context`] for why
//! that separation is not optional.

#![allow(unused_variables)]

use std::{fmt::Debug, future::Future, sync::Arc};

use futures::future::BoxFuture;
use http::Method;
// `pub use`, not `use`: the four repository types reach these through `use super::*`, and the
// `DynRepositoryHandler` derive emits them unqualified. Re-exporting keeps that working while the
// types are still in the binary; each will import them directly once it is its own crate.
pub use nr_core::{
    repository::{Visibility, project::ProjectResolution},
    storage::StoragePath,
};

pub mod prelude {
    pub use axum::response::{IntoResponse, Response};
    pub use http::StatusCode;
    pub use nr_core::{
        repository::{project::*, *},
        storage::*,
    };

    pub use super::{
        DynRepositoryHandlerError, IntoDynRepository, RepoResponse, Repository,
        RepositoryFactoryError, RepositoryHandlerError, RepositoryRequest,
    };
    pub use crate::SiteContext;
}
pub use nr_storage::DynStorage;
pub use nr_web_core::{authentication::AuthenticationError, utils::IntoErrorResponse};
use uuid::Uuid;

pub mod commands;
pub mod context;
mod error;
mod lookup;
mod repo_http;
mod repo_type;
pub mod staging;
pub mod utils;

pub use context::{HostnameIndex, SiteContext, SiteContextInner};
pub use error::*;
pub use lookup::*;
pub use repo_http::*;
pub use repo_type::*;
pub use staging::*;
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
    fn site(&self) -> SiteContext;
    fn resolve_project_and_version_for_path(
        &self,
        path: &StoragePath,
    ) -> impl Future<Output = Result<ProjectResolution, Self::Error>> + Send {
        async { Ok(ProjectResolution::default()) }
    }

    /// Spelled `-> impl Future + Send` rather than `async fn`, unlike everything else here only
    /// because it used to be the odd one out. An `async fn` in a trait declares a future with no
    /// `Send` bound, which cannot be boxed into the `BoxFuture` that [`RepositoryHandler`] returns.
    /// Implementations can still write `async fn`; the bound is checked at the impl.
    fn reload(&self) -> impl Future<Output = Result<(), RepositoryFactoryError>> + Send {
        async { Ok(()) }
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
/// A loaded repository, with its concrete type erased.
///
/// This is what everything outside a repository's own module holds. [`Repository`] cannot be a
/// trait object — it has an associated error type and returns `impl Future` — so this is the
/// object-safe shadow of it, with the error boxed and the futures boxed.
///
/// Never implemented by hand: [`ErasedRepository`] implements it for any [`Repository`], which
/// stays the trait a repository type actually writes. The `DynRepositoryHandler` derive,
/// `RepositoryExt` and the per-type extension traits are all built on `Repository`, and none of
/// them had to change.
///
/// This replaced a closed `enum DynRepository { Maven, NPM, Cargo, Docker }`, which named all four
/// concrete types and so had to live somewhere that could see all four.
pub trait RepositoryHandler: Send + Sync + Debug + 'static {
    fn get_storage(&self) -> DynStorage;
    fn get_type(&self) -> &'static str;
    fn full_type(&self) -> &'static str;
    fn config_types(&self) -> Vec<&str>;
    fn name(&self) -> String;
    fn id(&self) -> Uuid;
    fn visibility(&self) -> Visibility;
    fn is_active(&self) -> bool;
    fn site(&self) -> SiteContext;

    fn resolve_project_and_version_for_path<'a>(
        &'a self,
        path: &'a StoragePath,
    ) -> BoxFuture<'a, Result<ProjectResolution, DynRepositoryHandlerError>>;

    fn reload(&self) -> BoxFuture<'_, Result<(), RepositoryFactoryError>>;

    /// Runs a request, dispatching on its method.
    ///
    /// One method rather than the seven `handle_*` of [`Repository`]: every caller already
    /// dispatched on the method before calling, so the split only ever duplicated that match.
    fn handle(
        &self,
        request: RepositoryRequest,
    ) -> BoxFuture<'_, Result<RepoResponse, DynRepositoryHandlerError>>;
}

/// A loaded repository. `Arc` supplies the `Clone` the enum used to provide.
pub type DynRepository = Arc<dyn RepositoryHandler>;

/// Carries a concrete repository as a [`RepositoryHandler`].
///
/// A newtype rather than a blanket `impl<T: Repository> RepositoryHandler for T`, which would
/// compile but would give every repository two inherent-looking `get_type`, `name`, `id` and
/// `site` methods. Every one of the several hundred `self.get_type()` calls inside the four
/// repository types would then be ambiguous, for no benefit — nothing wants to call the erased
/// method on a concrete type.
pub struct ErasedRepository<T>(T);

/// Forwards, so `?repository` in a span still prints the repository rather than a wrapper around
/// one.
impl<T: Debug> Debug for ErasedRepository<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T> RepositoryHandler for ErasedRepository<T>
where
    T: Repository + 'static,
{
    fn get_storage(&self) -> DynStorage {
        self.0.get_storage()
    }
    fn get_type(&self) -> &'static str {
        self.0.get_type()
    }
    fn full_type(&self) -> &'static str {
        self.0.full_type()
    }
    fn config_types(&self) -> Vec<&str> {
        self.0.config_types()
    }
    fn name(&self) -> String {
        self.0.name()
    }
    fn id(&self) -> Uuid {
        self.0.id()
    }
    fn visibility(&self) -> Visibility {
        self.0.visibility()
    }
    fn is_active(&self) -> bool {
        self.0.is_active()
    }
    fn site(&self) -> SiteContext {
        self.0.site()
    }

    fn resolve_project_and_version_for_path<'a>(
        &'a self,
        path: &'a StoragePath,
    ) -> BoxFuture<'a, Result<ProjectResolution, DynRepositoryHandlerError>> {
        Box::pin(async move {
            self.0
                .resolve_project_and_version_for_path(path)
                .await
                .map_err(DynRepositoryHandlerError::new)
        })
    }

    fn reload(&self) -> BoxFuture<'_, Result<(), RepositoryFactoryError>> {
        Box::pin(self.0.reload())
    }

    fn handle(
        &self,
        request: RepositoryRequest,
    ) -> BoxFuture<'_, Result<RepoResponse, DynRepositoryHandlerError>> {
        Box::pin(async move {
            let result = match request.parts.method {
                Method::GET => self.0.handle_get(request).await,
                Method::POST => self.0.handle_post(request).await,
                Method::PUT => self.0.handle_put(request).await,
                Method::DELETE => self.0.handle_delete(request).await,
                Method::PATCH => self.0.handle_patch(request).await,
                Method::HEAD => self.0.handle_head(request).await,
                _ => self.0.handle_other(request).await,
            };
            result.map_err(DynRepositoryHandlerError::new)
        })
    }
}

/// Erases a concrete repository into a [`DynRepository`].
///
/// Exists so the four `load_repo` bodies stay one-liners: `.map(IntoDynRepository::into_dyn)`
/// where they used to say `.map(DynRepository::Maven)`.
pub trait IntoDynRepository: Repository + Sized + 'static {
    fn into_dyn(self) -> DynRepository {
        Arc::new(ErasedRepository(self))
    }
}
impl<T: Repository + 'static> IntoDynRepository for T {}
