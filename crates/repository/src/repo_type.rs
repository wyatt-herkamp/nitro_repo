use std::{fmt::Debug, sync::Arc};

use ahash::HashMap;
use auto_impl::auto_impl;
use axum::Router;
use digestible::Digestible;
use futures::future::BoxFuture;
use nr_core::database::entities::repository::{DBRepository, GenericDBRepositoryConfig};
use nr_storage::DynStorage;
use nr_web_core::error::InternalError;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

use super::DynRepository;
use crate::SiteContext;

#[derive(Debug, Clone, Serialize, ToSchema, Digestible)]
pub struct RepositoryTypeDescription {
    pub type_name: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub documentation_url: Option<&'static str>,
    pub is_stable: bool,
    pub required_configs: Vec<&'static str>,
}
#[derive(Debug)]
pub struct NewRepository {
    pub name: String,
    pub uuid: Uuid,
    pub repository_type: String,
    pub configs: HashMap<String, Value>,
}
impl NewRepository {
    /// Creates the repository and all of its configs as one unit.
    ///
    /// A repository is only usable once its required configs exist — `load_repo` reads them to
    /// decide which sub-handler to build. These used to be separate statements, so a config that
    /// failed to insert left behind a repository row that could never be loaded.
    pub async fn insert(
        self,
        storage: Uuid,
        database: &sqlx::PgPool,
    ) -> Result<DBRepository, InternalError> {
        let mut transaction = database.begin().await?;
        let repository = sqlx::query_as::<_, DBRepository>(
            r#"INSERT INTO repositories (id, storage_id, name, repository_type, active) VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
        )
        .bind(self.uuid)
        .bind(storage)
        .bind(&self.name)
        .bind(&self.repository_type)
        .bind(true)
        .fetch_one(&mut *transaction).await?;
        for (key, value) in self.configs {
            GenericDBRepositoryConfig::add_or_update(repository.id, key, value, &mut *transaction)
                .await?;
        }
        transaction.commit().await?;
        Ok(repository)
    }
}
/// This trait is invoked via dynamic dispatch for simplicity reasons.
#[auto_impl(&, Box, Arc)]
pub trait RepositoryType: Send + Debug + Sync {
    fn get_type(&self) -> &'static str;
    fn get_description(&self) -> RepositoryTypeDescription;

    /// Config types that this Repository could have.
    /// Some Repositories might not have a config type listed here.
    ///
    /// Such as Maven has hosted and proxy. The proxy type has an additional config type of "proxy"
    ///
    /// This array will contain the proxy type. But when calling Repository::config_types() on a hosted one will not contain "proxy"
    fn config_types(&self) -> Vec<&str>;
    /// Creates a new repository.
    /// Implementations of this function should validate the config and return an error if it is invalid
    /// Tell the storage any necessary information to create the repository
    fn create_new(
        &self,
        name: String,
        uuid: Uuid,
        configs: HashMap<String, Value>,
        storage: DynStorage,
    ) -> BoxFuture<'static, Result<NewRepository, RepositoryFactoryError>>;
    /// Load a repository from the database
    /// This function should load the repository from the database and return a DynRepository
    fn load_repo(
        &self,
        repo: DBRepository,
        storage: DynStorage,
        website: SiteContext,
    ) -> BoxFuture<'static, Result<DynRepository, RepositoryFactoryError>>;

    /// Routes this type serves under `/api/{get_type()}`, if it has any.
    ///
    /// Returned already carrying whatever state is private to the type — npm's browser-login
    /// manager, for instance, goes in as a request extension — so the application can mount these
    /// without being able to name any of it. That is what lets the manager live on the type
    /// instead of on the application state, which is where it used to have to live precisely
    /// because these routes needed to reach it.
    fn api_router(&self) -> Option<Router<SiteContext>> {
        None
    }
}
pub type DynRepositoryType = Box<dyn RepositoryType + Send + Sync>;

/// Every repository type this instance knows about.
///
/// Built once at startup and then read-only. A runtime list rather than the `&'static` slice it
/// replaces because a type is allowed to own state — Docker's blob-upload manager needs the
/// staging directory, which is configuration and so is not known until the config is read.
///
/// Still assembled by hand in the binary. There is no registration magic: adding a type means
/// adding a line, and `every_repository_type_is_registered` fails if that line goes missing.
#[derive(Clone, Debug, Default)]
pub struct RepositoryTypeRegistry(Arc<Vec<Arc<dyn RepositoryType>>>);

impl RepositoryTypeRegistry {
    pub fn new(types: Vec<Arc<dyn RepositoryType>>) -> Self {
        Self(Arc::new(types))
    }

    /// Case-insensitive, because the value comes from `repositories.repository_type` and from
    /// URLs, and neither is normalised on the way in.
    pub fn get(&self, name: &str) -> Option<Arc<dyn RepositoryType>> {
        self.0
            .iter()
            .find(|repository_type| repository_type.get_type().eq_ignore_ascii_case(name))
            .cloned()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Arc<dyn RepositoryType>> {
        self.0.iter()
    }

    pub fn descriptions(&self) -> Vec<RepositoryTypeDescription> {
        self.0
            .iter()
            .map(|repository_type| repository_type.get_description())
            .collect()
    }
}
#[derive(Debug, Error)]
pub enum RepositoryFactoryError {
    #[error("Invalid Config: {0}. Error: {1}")]
    InvalidConfig(&'static str, String),
    #[error("Invalid Sub Type")]
    InvalidSubType,
    #[error("Missing Config: {0}")]
    MissingConfig(&'static str),
    #[error("Database Error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Loaded Repository Not Found {0}")]
    LoadedRepositoryNotFound(Uuid),
}
