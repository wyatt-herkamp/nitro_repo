use std::sync::Arc;

use ahash::HashSet;
use nr_core::storage::StoragePath;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{meta::RepositoryMeta, FileType, Storage};
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct CreatedFiles {
    pub repository: Uuid,
    pub path: StoragePath,
}
#[derive(Debug, Clone, Default)]
pub struct TestingInternalStorage {
    pub created_files: HashSet<CreatedFiles>,
}
impl TestingInternalStorage {
    pub fn add_created_file(&mut self, repository: Uuid, path: StoragePath) -> bool {
        self.created_files.insert(CreatedFiles { repository, path })
    }
    pub fn remove_created_file(&mut self, repository: Uuid, path: StoragePath) -> bool {
        self.created_files
            .remove(&CreatedFiles { repository, path })
    }
}
pub struct TestingStorage<ST: Storage> {
    storage: ST,
    testing_storage: Arc<Mutex<TestingInternalStorage>>,
    delete_files: bool,
}

impl<ST: Storage> TestingStorage<ST> {
    pub fn new(storage: ST) -> Self {
        Self {
            storage,
            testing_storage: Arc::new(Mutex::new(TestingInternalStorage::default())),
            delete_files: false,
        }
    }
    async fn add_created_file(&self, repository: Uuid, path: StoragePath) -> bool {
        self.testing_storage
            .lock()
            .await
            .add_created_file(repository, path)
    }
    async fn remove_created_file(&self, repository: Uuid, path: StoragePath) -> bool {
        let mut files = self.testing_storage.lock().await;

        files.remove_created_file(repository, path)
    }
    async fn does_file_exist(&self, repository: Uuid, path: StoragePath) -> bool {
        self.testing_storage
            .lock()
            .await
            .created_files
            .contains(&CreatedFiles { repository, path })
    }
    async fn take_testing_storage(&self) -> TestingInternalStorage {
        let mut locked = self.testing_storage.lock().await;
        std::mem::take(&mut *locked)
    }
}
impl<ST: Storage> Storage for TestingStorage<ST> {
    type Error = ST::Error;
    type DirectoryStream = ST::DirectoryStream;
    fn storage_type_name(&self) -> &'static str {
        self.storage.storage_type_name()
    }
    async fn unload(&self) -> Result<(), Self::Error> {
        let files = self.take_testing_storage().await;

        for file in files.created_files {
            if self.delete_files {
                let existed = self
                    .storage
                    .delete_file(file.repository, &file.path)
                    .await?;
                if !existed {
                    tracing::warn!(
                        "File {} in repository {} did not exist in internal storage",
                        file.path,
                        file.repository
                    );
                }
            }
        }
        self.storage.unload().await
    }

    fn storage_config(&self) -> crate::BorrowedStorageConfig<'_> {
        self.storage.storage_config()
    }

    async fn save_file(
        &self,
        repository: Uuid,
        file: crate::FileContent,
        location: &StoragePath,
    ) -> Result<(usize, bool), Self::Error> {
        let result = self.storage.save_file(repository, file, location).await?;
        let already_exists = self.add_created_file(repository, location.clone()).await;
        assert_eq!(
            already_exists, result.1,
            "File already exists however internal storage does not know about it"
        );
        Ok(result)
    }

    async fn put_repository_meta(
        &self,
        _repository: Uuid,
        _location: &StoragePath,
        _value: RepositoryMeta,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    async fn get_repository_meta(
        &self,
        _repository: Uuid,
        _location: &StoragePath,
    ) -> Result<Option<RepositoryMeta>, Self::Error> {
        todo!()
    }

    async fn delete_file(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<bool, Self::Error> {
        let result = self.storage.delete_file(repository, location).await?;
        let already_exists = self.remove_created_file(repository, location.clone()).await;
        assert_eq!(
            already_exists, result,
            "Internal Storage did not return the correct value"
        );
        Ok(result)
    }

    async fn get_file_information(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<crate::StorageFileMeta<FileType>>, Self::Error> {
        self.storage
            .get_file_information(repository, location)
            .await
    }

    async fn open_file(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<crate::StorageFile>, Self::Error> {
        let result = self.storage.open_file(repository, location).await?;

        return Ok(result);
    }

    async fn validate_config_change(
        &self,
        config: crate::StorageTypeConfig,
    ) -> Result<(), Self::Error> {
        return self.storage.validate_config_change(config).await;
    }
    async fn file_exists(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<bool, Self::Error> {
        let result = self.storage.file_exists(repository, location).await?;
        let already_exists = self.does_file_exist(repository, location.clone()).await;
        assert_eq!(
            already_exists, result,
            "Internal Storage did not return the correct value"
        );
        Ok(result)
    }
    async fn stream_directory(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<Self::DirectoryStream>, Self::Error> {
        self.storage.stream_directory(repository, location).await
    }
}
