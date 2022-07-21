use super::super::maven::staging::StagingRepository;
use super::{ProcessingStage, StageHandler};
use crate::error::internal_error::InternalError;
use crate::repository::settings::{RepositoryConfig, RepositoryType};
use crate::storage::models::Storage;
use crate::storage::multi::MultiStorageController;
use std::sync::Arc;
use tokio::sync::RwLockReadGuard;

macro_rules! gen_dynamic_stage {
    ($($name: ident, $ty:tt),*) => {

        pub enum DynamicStageHandler<'a, StorageType: Storage> {
            $(
                $name($ty<'a, StorageType>),
            )*
        }
        #[inline]
        pub async fn get_stage_handler<StorageType: Storage>(
            storage: RwLockReadGuard<'_, StorageType>,
            repository_config: RepositoryConfig,
        ) -> Result<StageHandlerResult<StorageType>, InternalError> {
            match repository_config.repository_type {
                $(
                    RepositoryType::$name => {
                        let handler = $ty::create(repository_config, storage).await?;
                        Ok(StageHandlerResult::Supported(DynamicStageHandler::$name(handler)))
                    },
                )*
                _ => Ok(StageHandlerResult::Unsupported(repository_config)),

            }
        }
        #[async_trait::async_trait]
        impl<'a, StorageType: Storage> StageHandler<'a, StorageType>
            for DynamicStageHandler<'a, StorageType>{
    async fn push(
        &self,
        directory: String,
        process: ProcessingStage,
        storages: Arc<MultiStorageController>,
    ) -> Result<(), InternalError>{
        match self {
            $(
                DynamicStageHandler::$name(handler) => handler.push(directory, process, storages).await,
            )*
        }
    }
        }
    };
}
pub enum StageHandlerResult<'a, StorageType: Storage> {
    Supported(DynamicStageHandler<'a, StorageType>),
    /// it is a teapot! [Teapot](https://http.cat/418)
    Unsupported(RepositoryConfig),
}

gen_dynamic_stage!(Maven, StagingRepository);
