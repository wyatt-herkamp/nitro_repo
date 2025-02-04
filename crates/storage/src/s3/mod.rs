use std::{borrow::Cow, ops::Deref, str::FromStr, sync::Arc};

use chrono::Local;
use futures::future::BoxFuture;
use mime::Mime;
use nr_core::storage::{FileHashes, SerdeMime, StoragePath};
use regions::{CustomRegion, S3StorageRegion};
use s3::{
    Bucket, Region, Tag,
    creds::{Credentials, Rfc3339OffsetDateTime},
    error::S3Error,
    serde_types::ListBucketResult,
};

pub mod regions;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument, warn};
use utoipa::ToSchema;
pub mod tags;
use uuid::Uuid;
#[derive(Debug, thiserror::Error)]
pub enum S3StorageError {
    #[error("Invalid Credentials: {0}")]
    Credentials(s3::creds::error::CredentialsError),
    #[error("No Region Provided")]
    NoRegionSpecified,
    #[error("S3 Error: {0}")]
    S3Error(#[from] s3::error::S3Error),

    #[error("Bucket Does Not Exist {0}")]
    BucketDoesNotExist(String),
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    InvalidConfigType(#[from] InvalidConfigType),
    #[error("Unexpected Status Code: Expected {expected}, Got {got}")]
    UnexpectedStatusCode { expected: u16, got: u16 },

    #[error("Missing Tag: {0}")]
    MissingTag(Cow<'static, str>),

    #[error(transparent)]
    PathCollision(#[from] PathCollisionError),
}
impl S3StorageError {
    pub fn static_missing_tag(tag: &'static str) -> Self {
        S3StorageError::MissingTag(tag.into())
    }
}
use crate::{
    BorrowedStorageConfig, BorrowedStorageTypeConfig, DirectoryFileType, DynStorage, FileContent,
    FileContentBytes, FileFileType, FileType, InvalidConfigType, PathCollisionError,
    StaticStorageFactory, Storage, StorageConfig, StorageConfigInner, StorageError, StorageFactory,
    StorageFile, StorageFileMeta, StorageTypeConfig, StorageTypeConfigTrait, meta::RepositoryMeta,
    streaming::VecDirectoryListStream, utils::new_type_arc_type,
};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct S3Credentials {
    pub access_key: Option<String>,
    /// AWS secret key.
    pub secret_key: Option<String>,
    pub security_token: Option<String>,
    pub session_token: Option<String>,
    #[schema(value_type = chrono::DateTime<chrono::FixedOffset>, format = DateTime)]
    pub expiration: Option<Rfc3339OffsetDateTime>,
}
impl S3Credentials {
    pub fn new_access_key(access_key: impl Into<String>, secret_key: impl Into<String>) -> Self {
        S3Credentials {
            access_key: Some(access_key.into()),
            secret_key: Some(secret_key.into()),
            security_token: None,
            session_token: None,
            expiration: None,
        }
    }
    pub fn credentials(&self) -> Result<Credentials, S3StorageError> {
        Credentials::new(
            self.access_key.as_deref(),
            self.secret_key.as_deref(),
            self.security_token.as_deref(),
            self.session_token.as_deref(),
            None,
        )
        .map_err(S3StorageError::Credentials)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct S3Config {
    pub bucket_name: String,
    pub region: Option<S3StorageRegion>,
    /// Custom region takes precedence over the region field
    #[serde(flatten)]
    pub custom_region: Option<CustomRegion>,
    pub credentials: S3Credentials,
    #[serde(default = "default_true")]
    #[schema(default = true)]
    pub path_style: bool,
}
fn default_true() -> bool {
    true
}
impl S3Config {
    pub fn region(&self) -> Result<Region, S3StorageError> {
        if let Some(custom) = &self.custom_region {
            if self.region.is_some() {
                warn!("Region set with custom region, custom region will take precedence");
            }
            return Ok(custom.clone().into());
        }
        if let Some(region) = &self.region {
            return Ok(region.clone().into());
        }
        Err(S3StorageError::NoRegionSpecified)
    }
}
#[derive(Debug, Clone)]
pub struct S3MetaTags {
    pub name: String,
    pub mime_type: Option<Mime>,
    pub is_directory: bool,
}
#[derive(Debug)]
pub struct S3StorageInner {
    pub config: S3Config,
    pub storage_config: StorageConfigInner,
    pub bucket: Box<Bucket>,
}
impl S3StorageInner {
    pub async fn load_bucket(config: &S3Config) -> Result<Box<Bucket>, S3StorageError> {
        let credentials = config.credentials.credentials()?;
        let region = config.region()?;
        debug!(?region, "Connecting to S3 Bucket");
        let mut bucket = Bucket::new(&config.bucket_name, region, credentials)?;
        if config.path_style {
            bucket.set_path_style();
        }
        if !bucket.exists().await? {
            return Err(S3StorageError::BucketDoesNotExist(
                config.bucket_name.clone(),
            ));
        }

        Ok(bucket)
    }
    pub fn s3_path(&self, repository: &Uuid, path: &StoragePath) -> String {
        format!("{}/{}", repository, path)
    }
    pub async fn get_path_for_creation(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<String, S3StorageError> {
        let mut path = repository.to_string();
        let mut conflicting_path = StoragePath::default();
        for part in location.clone().into_iter() {
            path.push('/');
            path.push_str(part.as_ref());
            conflicting_path.push_mut(part.as_ref());
            if !self.is_directory(&path).await? {
                return Err(PathCollisionError {
                    path: location.clone(),
                    conflicts_with: conflicting_path,
                }
                .into());
            }
        }
        Ok(path)
    }
    #[instrument]
    async fn does_path_exist(&self, path: &str) -> Result<bool, S3StorageError> {
        match self.bucket.head_object(path).await.map(|(_, code)| code) {
            Ok(code) => Ok(code == 200),
            Err(S3Error::HttpFailWithBody(code, _)) => {
                if code == 404 {
                    Ok(false)
                } else {
                    Err(S3StorageError::UnexpectedStatusCode {
                        expected: 200,
                        got: code,
                    })
                }
            }
            Err(e) => Err(e.into()),
        }
    }
    #[instrument]
    fn is_directory_from_result<'result>(
        &self,
        result: &'result ListBucketResult,
        path: &str,
    ) -> (bool, Option<&'result str>) {
        if result.contents.is_empty() && result.common_prefixes.is_none() {
            return (true, None);
        }
        if path.ends_with("/") && !result.contents.is_empty() {
            return (true, None);
        }
        let path_with_slash = format!("{}/", path);

        if let Some(common) = &result.common_prefixes {
            if let Some(directory) = common
                .iter()
                .find(|prefix| prefix.prefix == path_with_slash)
            {
                return (true, Some(directory.prefix.as_str()));
            }
        }
        (false, None)
    }

    #[instrument]
    async fn is_directory(&self, path: &str) -> Result<bool, S3StorageError> {
        let list = self
            .bucket
            .list(path.to_owned(), Some("/".to_owned()))
            .await?;
        let Some(first) = list.first() else {
            return Ok(false);
        };

        Ok(self.is_directory_from_result(first, path).0)
    }

    /// Returns None if the path is not a directory
    #[instrument]
    async fn index_directory(&self, path: &str) -> Result<Option<StorageFile>, S3StorageError> {
        let path = if !path.ends_with("/") {
            format!("{}/", path)
        } else {
            path.to_owned()
        };

        let mut list = self.bucket.list(path.clone(), Some("/".to_owned())).await?;

        if list.is_empty() {
            return Ok(None);
        }
        let first = list.remove(0);

        let mut files = Vec::new();
        for file in first.contents {
            let meta = self.get_meta(&file.key).await?;
            if let Some(meta) = meta {
                files.push(meta);
            }
        }
        if let Some(common_prefixes) = first.common_prefixes {
            for sub_directory in common_prefixes {
                let meta = self.get_directory_meta(&sub_directory.prefix).await?;
                if let Some(meta) = meta {
                    files.push(meta);
                }
            }
        }

        Ok(Some(StorageFile::Directory {
            meta: StorageFileMeta {
                name: path.to_owned(),
                file_type: DirectoryFileType {
                    file_count: files.len() as u64,
                },
                modified: Local::now().fixed_offset(),
                created: Local::now().fixed_offset(),
            },
            files,
        }))
    }
    async fn get_meta(
        &self,
        path: &str,
    ) -> Result<Option<StorageFileMeta<FileType>>, S3StorageError> {
        let file_file = FileType::File(FileFileType {
            file_size: 0,
            mime_type: None,
            file_hash: FileHashes::default(),
        });

        let meta = StorageFileMeta {
            name: path.to_owned(),
            file_type: file_file,
            modified: Local::now().fixed_offset(),
            created: Local::now().fixed_offset(),
        };

        Ok(Some(meta))
    }
    async fn get_directory_meta(
        &self,
        path: &str,
    ) -> Result<Option<StorageFileMeta<FileType>>, S3StorageError> {
        let file_file = FileType::Directory(DirectoryFileType { file_count: 0 });

        let meta = StorageFileMeta {
            name: path.to_owned(),
            file_type: file_file,
            modified: Local::now().fixed_offset(),
            created: Local::now().fixed_offset(),
        };

        Ok(Some(meta))
    }
    #[instrument]
    async fn get_object_tagging(&self, path: &str) -> Result<Option<Vec<Tag>>, S3StorageError> {
        let (tags, status_code) = self.bucket.get_object_tagging(path).await?;
        if status_code == 404 {
            return Ok(None);
        }
        if status_code != 200 {
            return Err(S3StorageError::UnexpectedStatusCode {
                expected: 200,
                got: status_code,
            });
        }

        Ok(Some(tags))
    }

    async fn get_meta_tags(&self, path: &str) -> Result<Option<S3MetaTags>, S3StorageError> {
        let Some(tags) = self.get_object_tagging(path).await? else {
            return Ok(None);
        };

        let name = tags
            .iter()
            .find(|tag| tag.key() == tags::NAME)
            .map(|tag| tag.value())
            .ok_or_else(|| S3StorageError::static_missing_tag(tags::NAME))?;

        let mime_type = tags
            .iter()
            .find(|tag| tag.key() == tags::MIME_TYPE)
            .map(|tag| Mime::from_str(&tag.value()))
            .transpose();
        let mime_type = match mime_type {
            Ok(ok) => ok,
            Err(e) => {
                error!(?e, ?path, "Failed to parse mime type");
                None
            }
        };

        Ok(Some(S3MetaTags {
            name,
            mime_type,
            is_directory: false,
        }))
    }
}
#[derive(Debug, Clone)]
pub struct S3Storage(Arc<S3StorageInner>);
new_type_arc_type!(S3Storage(S3StorageInner));
impl Storage for S3Storage {
    type Error = S3StorageError;
    type DirectoryStream = VecDirectoryListStream;
    fn storage_type_name(&self) -> &'static str {
        "s3"
    }
    #[instrument(name = "Storage::unload", fields(storage_type = "s3"))]
    async fn unload(&self) -> Result<(), S3StorageError> {
        info!("Unloading S3 Storage");
        Ok(())
    }
    #[instrument(fields(storage_type = "s3"))]
    fn storage_config(&self) -> BorrowedStorageConfig<'_> {
        BorrowedStorageConfig {
            storage_config: &self.storage_config,
            config: BorrowedStorageTypeConfig::S3(&self.config),
        }
    }
    #[instrument(name = "Storage::save_file", fields(storage_type = "s3"))]
    async fn save_file(
        &self,
        repository: uuid::Uuid,
        file: FileContent,
        location: &StoragePath,
    ) -> Result<(usize, bool), S3StorageError> {
        let path = self.get_path_for_creation(repository, location).await?;
        let already_exists = self.does_path_exist(&path).await?;
        if already_exists {
            debug!("File already exists, overwriting");
        }
        let content_type = if location.is_directory() {
            "application/x-directory"
        } else {
            "application/octet-stream"
        };
        let file_as_bytes: FileContentBytes = file.try_into()?;

        let response_data = self
            .bucket
            .put_object_with_content_type(path, file_as_bytes.as_ref(), content_type)
            .await?;
        debug!(?response_data, "File Saved");
        // TODO: Check if the file was created
        Ok((file_as_bytes.len(), !already_exists))
    }
    #[instrument(name = "Storage::put_repository_meta", fields(storage_type = "s3"))]
    async fn put_repository_meta(
        &self,
        repository: uuid::Uuid,
        location: &StoragePath,
        value: RepositoryMeta,
    ) -> Result<(), S3StorageError> {
        todo!()
    }
    #[instrument(name = "Storage::get_repository_meta", fields(storage_type = "s3"))]
    async fn get_repository_meta(
        &self,
        repository: uuid::Uuid,
        location: &StoragePath,
    ) -> Result<Option<RepositoryMeta>, S3StorageError> {
        todo!()
    }
    #[instrument(name = "Storage::delete_file", fields(storage_type = "s3"))]
    async fn delete_file(
        &self,
        repository: uuid::Uuid,
        location: &StoragePath,
    ) -> Result<bool, S3StorageError> {
        let path = self.s3_path(&repository, location);
        let exists = self.does_path_exist(&path).await?;
        if !exists {
            return Ok(false);
        }
        let response_data = self.bucket.delete_object(path).await?;
        debug!(?response_data, "File Deleted");
        if response_data.status_code() != 204 {
            return Err(S3StorageError::UnexpectedStatusCode {
                expected: 204,
                got: response_data.status_code(),
            });
        }
        Ok(true)
    }
    #[instrument(name = "Storage::get_file_information", fields(storage_type = "s3"))]
    async fn get_file_information(
        &self,
        repository: uuid::Uuid,
        location: &StoragePath,
    ) -> Result<Option<crate::StorageFileMeta<FileType>>, S3StorageError> {
        todo!()
    }
    #[instrument(name = "Storage::open_file", fields(storage_type = "s3"))]
    async fn open_file(
        &self,
        repository: uuid::Uuid,
        location: &StoragePath,
    ) -> Result<Option<crate::StorageFile>, S3StorageError> {
        let path = self.s3_path(&repository, location);
        let response_data = match self.bucket.get_object(&path).await {
            Ok(response_data) => {
                if response_data.status_code() == 404 {
                    // Attempt to index the directory
                    debug!("File not found, attempting to index directory");
                    return self.index_directory(&path).await;
                }

                if response_data.status_code() != 200 {
                    return Err(S3StorageError::UnexpectedStatusCode {
                        expected: 200,
                        got: response_data.status_code(),
                    });
                }
                response_data
            }
            Err(S3Error::HttpFailWithBody(code, _)) => {
                if code == 404 {
                    return self.index_directory(&path).await;
                }
                return Err(S3StorageError::UnexpectedStatusCode {
                    expected: 200,
                    got: code,
                });
            }
            Err(e) => return Err(e.into()),
        };

        let headers = response_data.headers();
        if let Some(content_type) = headers.get("content-type") {
            if content_type == "application/x-directory" {
                return self.index_directory(&path).await;
            }
        }
        let meta = StorageFileMeta::<FileFileType> {
            name: location.to_string(),
            file_type: FileFileType {
                file_size: headers
                    .get("content-length")
                    .and_then(|len| len.parse().ok())
                    .unwrap_or(0),
                mime_type: headers
                    .get("content-type")
                    .and_then(|mime| Mime::from_str(mime.as_str()).ok().map(SerdeMime::from)),
                file_hash: FileHashes::default(),
            },
            modified: Local::now().fixed_offset(),
            created: Local::now().fixed_offset(),
        };
        let result = StorageFile::File {
            meta,
            content: crate::StorageFileReader::Bytes(response_data.into_bytes().into()),
        };

        Ok(Some(result))
    }
    #[instrument(name = "Storage::validate_config_change", fields(storage_type = "s3"))]
    async fn validate_config_change(
        &self,
        config: StorageTypeConfig,
    ) -> Result<(), S3StorageError> {
        let s3_config = S3Config::from_type_config(config)?;
        let bucket = S3StorageInner::load_bucket(&s3_config).await?;
        info!(?bucket, "Successfully connected to S3 Bucket");
        Ok(())
    }
    #[instrument(name = "Storage::file_exists", fields(storage_type = "s3"))]
    async fn file_exists(
        &self,
        repository: uuid::Uuid,
        location: &StoragePath,
    ) -> Result<bool, S3StorageError> {
        let path = self.s3_path(&repository, location);
        self.does_path_exist(&path).await
    }

    async fn stream_directory(
        &self,
        _repository: Uuid,
        _location: &StoragePath,
    ) -> Result<Option<Self::DirectoryStream>, Self::Error> {
        todo!()
    }
}
#[derive(Debug, Default)]
pub struct S3StorageFactory;
impl StaticStorageFactory for S3StorageFactory {
    type StorageType = S3Storage;
    type ConfigType = S3Config;
    type Error = S3StorageError;

    fn storage_type_name() -> &'static str {
        "s3"
    }

    async fn test_storage_config(config: StorageTypeConfig) -> Result<(), S3StorageError> {
        let s3_config = S3Config::from_type_config(config)?;
        let bucket = S3StorageInner::load_bucket(&s3_config).await?;
        info!(?bucket, "Successfully connected to S3 Bucket");
        Ok(())
    }

    async fn create_storage(
        inner: StorageConfigInner,
        type_config: Self::ConfigType,
    ) -> Result<Self::StorageType, S3StorageError> {
        let bucket = S3StorageInner::load_bucket(&type_config).await?;
        let inner = S3StorageInner {
            config: type_config,
            storage_config: inner,
            bucket,
        };
        let storage = S3Storage::from(inner);
        Ok(storage)
    }
}
impl StorageFactory for S3StorageFactory {
    fn storage_name(&self) -> &'static str {
        "s3"
    }

    fn test_storage_config(
        &self,
        config: StorageTypeConfig,
    ) -> BoxFuture<'static, Result<(), StorageError>> {
        Box::pin(async move {
            let s3_config = S3Config::from_type_config(config)?;

            let bucket = S3StorageInner::load_bucket(&s3_config).await?;
            info!(?bucket, "Successfully connected to S3 Bucket");

            Ok(())
        })
    }

    fn create_storage(
        &self,
        config: StorageConfig,
    ) -> BoxFuture<'static, Result<DynStorage, StorageError>> {
        Box::pin(async move {
            let s3_config = S3Config::from_type_config(config.type_config)?;
            let storage_config = config.storage_config;
            let bucket = S3StorageInner::load_bucket(&s3_config).await?;
            let inner = S3StorageInner {
                config: s3_config,
                storage_config,
                bucket,
            };
            let storage = S3Storage::from(inner);
            Ok(DynStorage::S3(storage))
        })
    }
}
#[cfg(test)]
mod tests {
    use tracing::warn;

    use crate::{StaticStorageFactory, s3::S3StorageFactory, testing::storage::TestingStorage};

    #[tokio::test]
    pub async fn generic_test() -> anyhow::Result<()> {
        let Some(config) = crate::testing::start_storage_test("s3")? else {
            warn!("S3 Storage Test Skipped");
            return Ok(());
        };
        let local_storage =
            <S3StorageFactory as StaticStorageFactory>::create_storage_from_config(config).await?;
        let testing_storage = TestingStorage::new(local_storage);
        crate::testing::tests::full_test(testing_storage).await?;

        Ok(())
    }
}
