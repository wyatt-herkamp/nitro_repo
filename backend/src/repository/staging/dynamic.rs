use super::super::maven::staging::StagingRepository;
use super::{ProcessingStage, StageHandler};
use crate::error::internal_error::InternalError;
use crate::repository::settings::{RepositoryConfig, RepositoryType};
use crate::storage::models::Storage;
use crate::storage::multi::MultiStorageController;
use crate::storage::DynamicStorage;
use std::sync::Arc;
use tokio::sync::RwLockReadGuard;

macro_rules! gen_dynamic_stage {
    ($($name: ident, $ty:tt),*) => {

        pub enum DynamicStageHandler<StorageType: Storage> {
            $(
                $name($ty<StorageType>),
            )*
        }
        #[inline]
        pub async fn get_stage_handler<StorageType: Storage>(
            storage: Arc<StorageType>,
            repository_config: RepositoryConfig,
        ) -> Result<StageHandlerResult<StorageType>, InternalError> {
            panic!("Not implemented");
        }
        #[async_trait::async_trait]
        impl<StorageType: Storage> StageHandler<StorageType>
            for DynamicStageHandler<StorageType>{
    async fn push(
        &self,
        directory: String,
        process: ProcessingStage,
        storages: Arc<MultiStorageController<DynamicStorage>>,
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
pub enum StageHandlerResult<StorageType: Storage> {
    Supported(DynamicStageHandler<StorageType>),
    /// it is a teapot! [Teapot](https://http.cat/418)
    Unsupported(RepositoryConfig),
}

gen_dynamic_stage!(Maven, StagingRepository);
