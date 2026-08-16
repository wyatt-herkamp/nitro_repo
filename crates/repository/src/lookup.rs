//! Turning a name, a hostname or an id into a loaded repository.
//!
//! Kept apart from [`SiteContext`](crate::SiteContext) on purpose. A repository holds a
//! context; if the context could resolve repositories it would have to hold the map that owns
//! them, and the `Arc` cycle would never be collected. So the context carries the hostname
//! *index* — ids only — and resolution lives here, on a handle the router owns.

use std::{fmt::Debug, sync::Arc};

use axum::{
    body::Body,
    extract::FromRef,
    response::{IntoResponse, Response},
};
use derive_more::derive::From;
use futures::future::BoxFuture;
use http::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::{DynRepository, SiteContext};

/// A repository addressed the way a URL addresses it: by storage name and repository name.
///
/// Case-insensitive. Both halves are lowercased on the way in, and the database lookup is
/// case-insensitive too.
#[derive(Debug, Clone, Hash, PartialEq, Eq, IntoParams, Deserialize)]
#[into_params(parameter_in = Path)]
pub struct RepositoryStorageName {
    /// The name of the storage
    pub storage_name: String,
    /// The name of the repository
    pub repository_name: String,
}

impl RepositoryStorageName {
    pub async fn query_db(&self, database: &PgPool) -> Result<Option<Uuid>, sqlx::Error> {
        let query: Option<Uuid> = sqlx::query_scalar(
            r#"SELECT repositories.id FROM repositories INNER JOIN storages
                    ON storages.id = repositories.storage_id AND storages.name = $1
                    WHERE repositories.name = $2"#,
        )
        .bind(&self.storage_name)
        .bind(&self.repository_name)
        .fetch_optional(database)
        .await?;
        Ok(query)
    }
}

impl From<(&str, &str)> for RepositoryStorageName {
    fn from((storage_name, repository_name): (&str, &str)) -> Self {
        Self {
            storage_name: storage_name.to_lowercase(),
            repository_name: repository_name.to_lowercase(),
        }
    }
}

impl From<(String, String)> for RepositoryStorageName {
    fn from((storage_name, repository_name): (String, String)) -> Self {
        Self {
            storage_name: storage_name.to_lowercase(),
            repository_name: repository_name.to_lowercase(),
        }
    }
}

#[derive(Debug, From)]
pub enum RepositoryNotFound {
    RepositoryAndNameLookup(RepositoryStorageName),
    Uuid(Uuid),
}

impl IntoResponse for RepositoryNotFound {
    fn into_response(self) -> Response {
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

/// Resolves an address to a loaded repository.
///
/// Implemented by the application, which is the thing that owns the repository map. Deliberately
/// not reachable from a [`SiteContext`] — see the module docs.
pub trait RepositoryResolver: Send + Sync + Debug {
    fn repository_by_id(&self, id: Uuid) -> Option<DynRepository>;

    /// `host` must already be normalised by
    /// [`normalize_host`](nr_web_core::utils::host::normalize_host).
    fn repository_for_hostname(&self, host: &str) -> Option<DynRepository>;

    /// Cache-then-database, unlike the other two: the name table is filled lazily, so a miss here
    /// is not authoritative and has to be checked against the database before it is believed.
    fn repository_from_names<'a>(
        &'a self,
        names: &'a RepositoryStorageName,
    ) -> BoxFuture<'a, Result<Option<DynRepository>, sqlx::Error>>;
}

/// Router state for anything that has to resolve a repository from a request.
///
/// Holds the resolver as well as the context, which is why it is the router's state and never a
/// repository's: the router is owned by the server, so the strong reference back to the
/// application closes no cycle.
#[derive(Clone, Debug)]
pub struct RepositoryRouterState {
    pub context: SiteContext,
    pub resolver: Arc<dyn RepositoryResolver>,
}

impl RepositoryRouterState {
    pub fn new(context: SiteContext, resolver: Arc<dyn RepositoryResolver>) -> Self {
        Self { context, resolver }
    }
}

// Legal despite `FromRef` being foreign: `RepositoryRouterState` is local and appears as a trait
// argument. These are what let the authentication extractors, which ask only for a `PgPool`, run
// on a router whose state is this.
impl FromRef<RepositoryRouterState> for SiteContext {
    fn from_ref(state: &RepositoryRouterState) -> SiteContext {
        state.context.clone()
    }
}

impl FromRef<RepositoryRouterState> for PgPool {
    fn from_ref(state: &RepositoryRouterState) -> PgPool {
        state.context.database.clone()
    }
}
