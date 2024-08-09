use std::fmt::Debug;

use ahash::HashMap;
use auto_impl::auto_impl;
use futures::future::{BoxFuture, LocalBoxFuture};
use nr_core::database::repository::{DBRepository, GenericDBRepositoryConfig};
use nr_storage::DynStorage;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{app::NitroRepo, error::InternalError};

use super::DynRepository;

#[derive(Debug, Clone, Serialize, ToSchema)]
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
    // TODO: Transactions
    pub async fn insert(
        self,
        storage: Uuid,
        database: &sqlx::PgPool,
    ) -> Result<DBRepository, InternalError> {
        let repository = sqlx::query_as::<_, DBRepository>(
            r#"INSERT INTO repositories (id, storage_id, name, repository_type, active) VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
        )
        .bind(&self.uuid)
        .bind(&storage)
        .bind(&self.name)
        .bind(&self.repository_type)
        .bind(true)
        .fetch_one( database).await?;
        for (key, value) in self.configs {
            GenericDBRepositoryConfig::add_or_update(repository.id, key, value, database).await?;
        }
        Ok(repository)
    }
}
/// This trait is invoked via dynamic dispatch for simplicity reasons.
#[auto_impl(&, Box)]
pub trait RepositoryType: Send + Debug {
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
        website: NitroRepo,
    ) -> BoxFuture<'static, Result<DynRepository, RepositoryFactoryError>>;
}
pub type DynRepositoryType = Box<dyn RepositoryType + Send + Sync>;
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
