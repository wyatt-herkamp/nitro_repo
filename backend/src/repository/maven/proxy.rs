use crate::repository::handler::RepositoryHandler;
use crate::repository::maven::models::ProxySettings;
use crate::repository::nitro::nitro_repository::NitroRepositoryHandler;
use crate::repository::settings::RepositoryConfig;
use crate::storage::models::Storage;
use async_trait::async_trait;
use tokio::sync::RwLockReadGuard;

pub struct ProxyMavenRepository<'a, S: Storage> {
    pub config: RepositoryConfig,
    pub proxy: Vec<ProxySettings>,
    pub storage: RwLockReadGuard<'a, S>,
}
#[async_trait]
impl<'a, S: Storage> RepositoryHandler<'a, S> for ProxyMavenRepository<'a, S> {}
impl<StorageType: Storage> NitroRepositoryHandler<StorageType>
    for ProxyMavenRepository<'_, StorageType>
{
    fn parse_project_to_directory<S: Into<String>>(value: S) -> String {
        value.into().replace('.', "/").replace(':', "/")
    }

    fn storage(&self) -> &StorageType {
        &self.storage
    }

    fn repository(&self) -> &RepositoryConfig {
        &self.config
    }
}
