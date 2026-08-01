//! S3 storage backend, built on the official AWS SDK.
//!
//! # Layout
//!
//! Objects live under `{repository_uuid}/{storage path}`, mirroring the directory layout the local
//! backends use. Two kinds of extra object make that layout navigable:
//!
//! - **Metadata sidecars.** S3 has nowhere to put the data [crate::meta::RepositoryMeta] carries —
//!   object tags cap out at 10 entries of 256 characters, and `x-amz-meta-*` at 2 KB in total,
//!   neither of which fits arbitrary repository metadata. So each object gets a sibling
//!   `{key}.nr-meta` holding a postcard-encoded [LocationMeta], exactly as the local backend
//!   writes beside a file on disk. One codec serves every backend.
//! - **Directory markers.** S3 has no directories, only key prefixes. An empty object with
//!   `Content-Type: application/x-directory` marks one, which is the convention MinIO and the AWS
//!   console both understand. Listing does not depend on these markers — it works off
//!   `CommonPrefixes` — but writing them keeps a bucket legible to other tools.
//!
//! # Credentials
//!
//! An explicit access/secret key pair is optional. With none configured the SDK's default provider
//! chain applies (environment, shared config, IMDS, web identity), which is what makes an IAM role
//! or an assumed role work without putting long-lived keys in the database.
use std::{borrow::Cow, ops::Deref, str::FromStr, sync::Arc};

use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{
    Client,
    config::Credentials as AwsCredentials,
    error::SdkError,
    primitives::ByteStream,
    types::{Delete, ObjectIdentifier},
};
use aws_smithy_runtime_api::client::orchestrator::HttpResponse;
use chrono::{DateTime, FixedOffset, Local};
use futures::future::BoxFuture;
use mime::Mime;
use nr_core::storage::{FileHashes, SerdeMime, StoragePath};
use regions::{CustomRegion, S3StorageRegion};

pub mod regions;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument, warn};
use utoipa::ToSchema;
pub mod tags;
use uuid::Uuid;

use crate::{
    BorrowedStorageConfig, BorrowedStorageTypeConfig, DirectoryFileType, DynStorage, FileContent,
    FileContentBytes, FileFileType, FileType, InvalidConfigType, LocationMeta, PathCollisionError,
    StaticStorageFactory, Storage, StorageConfig, StorageConfigInner, StorageError, StorageFactory,
    StorageFile, StorageFileMeta, StorageTypeConfig, StorageTypeConfigTrait,
    fs::NITRO_REPO_META_EXTENSION, meta::RepositoryMeta, streaming::VecDirectoryListStream,
    utils::new_type_arc_type,
};

/// Content type S3 tooling conventionally uses to mark a key as a directory.
const DIRECTORY_CONTENT_TYPE: &str = "application/x-directory";
/// Suffix of the sidecar object holding an object's [LocationMeta].
const META_SUFFIX: &str = ".nr-meta";
/// `DeleteObjects` accepts at most this many keys per call.
const DELETE_BATCH_SIZE: usize = 1000;

#[derive(Debug, thiserror::Error)]
pub enum S3StorageError {
    #[error("No Region Provided")]
    NoRegionSpecified,
    #[error("S3 storage is missing its {0}")]
    MissingCredential(&'static str),
    #[error("Bucket Does Not Exist {0}")]
    BucketDoesNotExist(String),
    #[error("IO Error: {0}")]
    IOError(#[from] std::io::Error),
    #[error(transparent)]
    InvalidConfigType(#[from] InvalidConfigType),
    #[error("Missing Tag: {0}")]
    MissingTag(Cow<'static, str>),
    #[error(transparent)]
    PathCollision(#[from] PathCollisionError),
    #[error("Could not decode stored metadata: {0}")]
    Meta(#[from] postcard::Error),
    #[error("S3 error: {0}")]
    S3(String),
}

impl S3StorageError {
    pub fn static_missing_tag(tag: &'static str) -> Self {
        S3StorageError::MissingTag(tag.into())
    }
}

/// The SDK's error type is generic over the operation, so each call site would otherwise need its
/// own `From`. Collapsing to the rendered message keeps one variant while preserving the source
/// chain the SDK builds up (which is where the useful detail lives).
impl<E, R> From<SdkError<E, R>> for S3StorageError
where
    E: std::error::Error + 'static,
    R: std::fmt::Debug,
{
    fn from(value: SdkError<E, R>) -> Self {
        let mut message = value.to_string();
        let mut source = std::error::Error::source(&value);
        while let Some(cause) = source {
            message.push_str(": ");
            message.push_str(&cause.to_string());
            source = cause.source();
        }
        S3StorageError::S3(message)
    }
}
impl From<aws_smithy_types::byte_stream::error::Error> for S3StorageError {
    fn from(value: aws_smithy_types::byte_stream::error::Error) -> Self {
        S3StorageError::S3(value.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct S3Credentials {
    pub access_key: Option<String>,
    /// AWS secret key.
    pub secret_key: Option<String>,
}
impl S3Credentials {
    pub fn new_access_key(access_key: impl Into<String>, secret_key: impl Into<String>) -> Self {
        S3Credentials {
            access_key: Some(access_key.into()),
            secret_key: Some(secret_key.into()),
        }
    }
    /// Whether any key material was configured at all.
    ///
    /// Neither key set means "use the environment", which is the supported way to run against an
    /// IAM role. One key set without the other is a misconfiguration, not a request for that.
    pub fn is_empty(&self) -> bool {
        self.access_key.is_none() && self.secret_key.is_none()
    }
    pub fn credentials(&self) -> Result<Option<AwsCredentials>, S3StorageError> {
        if self.is_empty() {
            return Ok(None);
        }
        let access_key = self
            .access_key
            .clone()
            .ok_or(S3StorageError::MissingCredential("access key"))?;
        let secret_key = self
            .secret_key
            .clone()
            .ok_or(S3StorageError::MissingCredential("secret key"))?;
        Ok(Some(AwsCredentials::new(
            access_key,
            secret_key,
            None,
            None,
            "nitro-repo-storage-config",
        )))
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
            return Ok(custom.region());
        }
        if let Some(region) = self.region {
            return Ok(region.into());
        }
        Err(S3StorageError::NoRegionSpecified)
    }
}

#[derive(Debug)]
pub struct S3StorageInner {
    pub config: S3Config,
    pub storage_config: StorageConfigInner,
    pub client: Client,
}

impl S3StorageInner {
    #[instrument(skip(config), fields(bucket = %config.bucket_name))]
    pub async fn build_client(config: &S3Config) -> Result<Client, S3StorageError> {
        let region = config.region()?;
        debug!(?region, "Connecting to S3 Bucket");

        let mut loader = aws_config::defaults(BehaviorVersion::latest()).region(region);
        if let Some(credentials) = config.credentials.credentials()? {
            loader = loader.credentials_provider(credentials);
        } else {
            debug!("No credentials configured; using the default AWS provider chain");
        }
        let shared_config = loader.load().await;

        let mut builder = aws_sdk_s3::config::Builder::from(&shared_config);
        if let Some(custom) = &config.custom_region {
            // A self-hosted endpoint (MinIO, Ceph, Garage) almost always needs path-style
            // addressing, because `bucket.localhost` does not resolve.
            builder = builder.endpoint_url(custom.endpoint.to_string());
        }
        builder = builder.force_path_style(config.path_style);

        Ok(Client::from_conf(builder.build()))
    }

    /// Confirms the bucket exists and is reachable with the configured credentials.
    #[instrument(skip(self))]
    pub async fn check_bucket(&self) -> Result<(), S3StorageError> {
        match self
            .client
            .head_bucket()
            .bucket(&self.config.bucket_name)
            .send()
            .await
        {
            Ok(_) => Ok(()),
            Err(err) if is_not_found(&err) => Err(S3StorageError::BucketDoesNotExist(
                self.config.bucket_name.clone(),
            )),
            Err(err) => Err(err.into()),
        }
    }

    /// The object key an artifact lives at.
    pub fn s3_path(&self, repository: &Uuid, path: &StoragePath) -> String {
        format!("{}/{}", repository, path.to_string().trim_end_matches('/'))
    }
    /// The prefix a directory's children live under. Always ends in `/`.
    fn directory_prefix(&self, repository: &Uuid, path: &StoragePath) -> String {
        let path = path.to_string();
        let path = path.trim_matches('/');
        if path.is_empty() {
            format!("{}/", repository)
        } else {
            format!("{}/{}/", repository, path)
        }
    }
    /// The sidecar key holding the metadata for `key`.
    fn meta_key(key: &str) -> String {
        format!("{}{}", key, META_SUFFIX)
    }
    /// Whether a key is one of our own bookkeeping objects rather than a stored artifact.
    fn is_hidden_key(key: &str) -> bool {
        key.ends_with(META_SUFFIX) || key.ends_with(NITRO_REPO_META_EXTENSION)
    }

    async fn head(
        &self,
        key: &str,
    ) -> Result<Option<aws_sdk_s3::operation::head_object::HeadObjectOutput>, S3StorageError> {
        match self
            .client
            .head_object()
            .bucket(&self.config.bucket_name)
            .key(key)
            .send()
            .await
        {
            Ok(output) => Ok(Some(output)),
            Err(err) if is_not_found(&err) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    #[instrument(skip(self))]
    async fn does_path_exist(&self, key: &str) -> Result<bool, S3StorageError> {
        Ok(self.head(key).await?.is_some())
    }

    /// Whether anything is stored under this key's prefix, which is what makes it a directory.
    #[instrument(skip(self))]
    async fn is_directory(&self, prefix: &str) -> Result<bool, S3StorageError> {
        let listing = self
            .client
            .list_objects_v2()
            .bucket(&self.config.bucket_name)
            .prefix(prefix)
            .max_keys(1)
            .send()
            .await?;
        Ok(listing.key_count().unwrap_or_default() > 0)
    }

    /// Rejects writing under a path whose ancestor is already a file.
    ///
    /// `/a/b` being a file makes `/a/b/c` impossible on a real filesystem, and the local backends
    /// enforce that. S3 would happily store both, so enforce it here too — otherwise the same
    /// repository behaves differently depending on where it is stored.
    async fn check_for_collisions(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<(), S3StorageError> {
        let mut key = repository.to_string();
        let mut conflicting_path = StoragePath::default();
        let mut components = location.clone().into_iter().peekable();
        while let Some(component) = components.next() {
            key.push('/');
            key.push_str(component.as_ref());
            conflicting_path.push_mut(component.as_ref());
            // The last component is the file being written; it is allowed to already exist.
            if components.peek().is_none() {
                break;
            }
            if let Some(head) = self.head(&key).await?
                && head.content_type() != Some(DIRECTORY_CONTENT_TYPE)
            {
                return Err(PathCollisionError {
                    path: location.clone(),
                    conflicts_with: conflicting_path,
                }
                .into());
            }
        }
        Ok(())
    }

    /// Reads the metadata sidecar for an object, if one was written.
    #[instrument(skip(self))]
    async fn read_meta(&self, key: &str) -> Result<Option<LocationMeta>, S3StorageError> {
        let meta_key = Self::meta_key(key);
        let output = match self
            .client
            .get_object()
            .bucket(&self.config.bucket_name)
            .key(&meta_key)
            .send()
            .await
        {
            Ok(output) => output,
            Err(err) if is_not_found(&err) => return Ok(None),
            Err(err) => return Err(err.into()),
        };
        let bytes = output.body.collect().await?.into_bytes();
        match LocationMeta::from_postcard(&bytes) {
            Ok(meta) => Ok(Some(meta)),
            Err(err) => {
                // Same call as the local backend's: a sidecar we cannot read is treated as absent
                // and rebuilt, rather than making the artifact itself unreadable.
                warn!(?err, ?meta_key, "Metadata sidecar is unreadable; ignoring");
                Ok(None)
            }
        }
    }

    #[instrument(skip(self, meta))]
    async fn write_meta(&self, key: &str, meta: &LocationMeta) -> Result<(), S3StorageError> {
        let encoded = meta.to_postcard()?;
        self.client
            .put_object()
            .bucket(&self.config.bucket_name)
            .key(Self::meta_key(key))
            .content_type(mime::APPLICATION_OCTET_STREAM.as_ref())
            .body(ByteStream::from(encoded))
            .send()
            .await?;
        Ok(())
    }

    /// Lists one directory level, returning its files and sub-directories.
    ///
    /// Paginated: a repository can hold far more than one `ListObjectsV2` page, and the previous
    /// implementation silently returned only the first.
    #[instrument(skip(self))]
    async fn list_directory(
        &self,
        prefix: &str,
    ) -> Result<Vec<StorageFileMeta<FileType>>, S3StorageError> {
        let mut files = Vec::new();
        let mut continuation: Option<String> = None;

        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&self.config.bucket_name)
                .prefix(prefix)
                .delimiter("/");
            if let Some(token) = &continuation {
                request = request.continuation_token(token);
            }
            let listing = request.send().await?;

            for object in listing.contents() {
                let Some(key) = object.key() else {
                    continue;
                };
                // The marker object for the directory being listed shows up in its own listing.
                if key == prefix || Self::is_hidden_key(key) {
                    continue;
                }
                let name = key.trim_start_matches(prefix).trim_end_matches('/');
                if name.is_empty() {
                    continue;
                }
                let meta = self.read_meta(key).await?;
                let modified = object
                    .last_modified()
                    .and_then(to_chrono)
                    .or_else(|| meta.as_ref().map(|m| m.modified))
                    .unwrap_or_else(|| Local::now().fixed_offset());
                let created = meta.as_ref().map(|m| m.created).unwrap_or(modified);
                files.push(StorageFileMeta {
                    name: name.to_owned(),
                    file_type: FileType::File(FileFileType {
                        file_size: object.size().unwrap_or_default() as u64,
                        mime_type: mime_for_name(name),
                        file_hash: meta.map(|m| m.hashes()).unwrap_or_default(),
                    }),
                    modified,
                    created,
                });
            }

            for sub_directory in listing.common_prefixes() {
                let Some(sub_prefix) = sub_directory.prefix() else {
                    continue;
                };
                let name = sub_prefix.trim_start_matches(prefix).trim_end_matches('/');
                if name.is_empty() {
                    continue;
                }
                let meta = self.read_meta(sub_prefix.trim_end_matches('/')).await?;
                let modified = meta
                    .as_ref()
                    .map(|m| m.modified)
                    .unwrap_or_else(|| Local::now().fixed_offset());
                files.push(StorageFileMeta {
                    name: name.to_owned(),
                    file_type: FileType::Directory(DirectoryFileType {
                        file_count: meta
                            .as_ref()
                            .and_then(|m| m.dir_meta_or_err().ok().map(|d| d.number_of_files))
                            .unwrap_or_default(),
                    }),
                    modified,
                    created: meta.map(|m| m.created).unwrap_or(modified),
                });
            }

            if listing.is_truncated().unwrap_or_default() {
                continuation = listing.next_continuation_token().map(str::to_owned);
                if continuation.is_some() {
                    continue;
                }
            }
            break;
        }

        Ok(files)
    }

    /// Deletes every object under a prefix, in `DeleteObjects` batches.
    #[instrument(skip(self))]
    async fn delete_prefix(&self, prefix: &str) -> Result<bool, S3StorageError> {
        let mut deleted_anything = false;
        let mut continuation: Option<String> = None;

        loop {
            let mut request = self
                .client
                .list_objects_v2()
                .bucket(&self.config.bucket_name)
                .prefix(prefix)
                .max_keys(DELETE_BATCH_SIZE as i32);
            if let Some(token) = &continuation {
                request = request.continuation_token(token);
            }
            let listing = request.send().await?;

            let keys: Vec<ObjectIdentifier> = listing
                .contents()
                .iter()
                .filter_map(|object| object.key())
                .map(|key| ObjectIdentifier::builder().key(key).build())
                .collect::<Result<Vec<_>, _>>()
                .map_err(|err| S3StorageError::S3(err.to_string()))?;

            if !keys.is_empty() {
                deleted_anything = true;
                let delete = Delete::builder()
                    .set_objects(Some(keys))
                    .quiet(true)
                    .build()
                    .map_err(|err| S3StorageError::S3(err.to_string()))?;
                self.client
                    .delete_objects()
                    .bucket(&self.config.bucket_name)
                    .delete(delete)
                    .send()
                    .await?;
            }

            // Deleting as we go invalidates the continuation token, so re-list from the start
            // until a pass finds nothing left.
            if listing.is_truncated().unwrap_or_default() {
                continuation = None;
                continue;
            }
            break;
        }

        Ok(deleted_anything)
    }

    /// Writes an empty directory marker object for a prefix, if one is not already there.
    async fn ensure_directory_marker(&self, prefix: &str) -> Result<(), S3StorageError> {
        if self.does_path_exist(prefix).await? {
            return Ok(());
        }
        self.client
            .put_object()
            .bucket(&self.config.bucket_name)
            .key(prefix)
            .content_type(DIRECTORY_CONTENT_TYPE)
            .body(ByteStream::from_static(b""))
            .send()
            .await?;
        Ok(())
    }

    /// Refreshes bookkeeping for the directory a write or delete just changed.
    ///
    /// Only the immediate parent gets its entry count recounted. Ancestors get a marker object if
    /// they are missing one, but are not re-listed: counting every ancestor on every upload would
    /// make a Maven deploy cost O(depth x objects) in `ListObjectsV2` calls. Readers
    /// ([Storage::get_file_information], [Storage::stream_directory]) count from a live listing
    /// anyway, so the stored count only backs the entry-count shown for a *sub*-directory in a
    /// listing, where being slightly stale is acceptable.
    async fn update_directory_metadata(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<(), S3StorageError> {
        let now = Local::now().fixed_offset();
        let mut directory = location.clone().parent();

        let prefix = self.directory_prefix(&repository, &directory);
        let key = prefix.trim_end_matches('/').to_owned();
        let entries = self.list_directory(&prefix).await?;
        let created = self
            .read_meta(&key)
            .await?
            .map(|existing| existing.created)
            .unwrap_or(now);
        self.ensure_directory_marker(&prefix).await?;
        self.write_meta(
            &key,
            &LocationMeta::for_directory(created, now, entries.len() as u64),
        )
        .await?;

        while directory.number_of_components() > 0 {
            directory = directory.parent();
            let prefix = self.directory_prefix(&repository, &directory);
            self.ensure_directory_marker(&prefix).await?;
        }
        Ok(())
    }
}

/// The SDK models "no such key" per operation (`HeadObjectError` has no `NoSuchKey` variant at
/// all, it just 404s), so go by the HTTP status instead of matching each operation's error enum.
fn is_not_found<E>(err: &SdkError<E, HttpResponse>) -> bool {
    match err {
        SdkError::ServiceError(service_error) => service_error.raw().status().as_u16() == 404,
        _ => false,
    }
}

fn to_chrono(timestamp: &aws_smithy_types::DateTime) -> Option<DateTime<FixedOffset>> {
    DateTime::from_timestamp(timestamp.secs(), timestamp.subsec_nanos())
        .map(|value| value.fixed_offset())
}

fn mime_for_name(name: &str) -> Option<SerdeMime> {
    mime_guess::from_path(name).first().map(SerdeMime)
}

#[derive(Debug, Clone)]
pub struct S3Storage(Arc<S3StorageInner>);
new_type_arc_type!(S3Storage(S3StorageInner));

impl Storage for S3Storage {
    type Error = S3StorageError;
    type DirectoryStream = VecDirectoryListStream;

    fn storage_type_name(&self) -> &'static str {
        S3StorageFactory::STORAGE_TYPE_NAME
    }

    #[instrument(name = "Storage::unload", skip(self), fields(storage_type = "S3"))]
    async fn unload(&self) -> Result<(), S3StorageError> {
        info!("Unloading S3 Storage");
        Ok(())
    }

    fn storage_config(&self) -> BorrowedStorageConfig<'_> {
        BorrowedStorageConfig {
            storage_config: &self.storage_config,
            config: BorrowedStorageTypeConfig::S3(&self.config),
        }
    }

    #[instrument(
        name = "Storage::save_file",
        skip(self, file),
        fields(storage_type = "S3")
    )]
    async fn save_file(
        &self,
        repository: Uuid,
        file: FileContent,
        location: &StoragePath,
    ) -> Result<(usize, bool), S3StorageError> {
        self.check_for_collisions(repository, location).await?;

        let key = self.s3_path(&repository, location);
        let existing = self.head(&key).await?;
        let already_exists = existing.is_some();
        if already_exists {
            debug!("File already exists, overwriting");
        }

        let hashes = file.generate_hashes()?;
        let bytes: FileContentBytes = file.try_into()?;
        let size = bytes.len();
        let content_type = if location.is_directory() {
            DIRECTORY_CONTENT_TYPE.to_owned()
        } else {
            mime_for_name(&key)
                .map(|mime| mime.0.to_string())
                .unwrap_or_else(|| mime::APPLICATION_OCTET_STREAM.to_string())
        };

        self.client
            .put_object()
            .bucket(&self.config.bucket_name)
            .key(&key)
            .content_type(content_type)
            .body(ByteStream::from(bytes::Bytes::from(bytes)))
            .send()
            .await?;

        let now = Local::now().fixed_offset();
        // Preserve the original creation time across an overwrite, and any repository metadata
        // already attached to the path — Maven stamps project ids onto it.
        let previous = self.read_meta(&key).await?;
        let created = previous.as_ref().map(|meta| meta.created).unwrap_or(now);
        let mut meta = LocationMeta::for_file(created, now, hashes);
        if let Some(previous) = previous {
            meta.repository_meta = previous.repository_meta;
        }
        self.write_meta(&key, &meta).await?;

        self.update_directory_metadata(repository, location).await?;

        Ok((size, !already_exists))
    }

    #[instrument(
        name = "Storage::put_repository_meta",
        skip(self),
        fields(storage_type = "S3")
    )]
    async fn put_repository_meta(
        &self,
        repository: Uuid,
        location: &StoragePath,
        value: RepositoryMeta,
    ) -> Result<(), S3StorageError> {
        let key = self.s3_path(&repository, location);
        let now = Local::now().fixed_offset();
        let mut meta = match self.read_meta(&key).await? {
            Some(meta) => meta,
            None => {
                // The path may be a directory that only exists as a prefix, so fall back to
                // directory-shaped metadata rather than refusing to record anything.
                let prefix = self.directory_prefix(&repository, location);
                if self.is_directory(&prefix).await? {
                    LocationMeta::for_directory(now, now, 0)
                } else {
                    LocationMeta::for_file(now, now, FileHashes::default())
                }
            }
        };
        meta.repository_meta = value;
        meta.modified = now;
        self.write_meta(&key, &meta).await
    }

    #[instrument(
        name = "Storage::get_repository_meta",
        skip(self),
        fields(storage_type = "S3")
    )]
    async fn get_repository_meta(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<RepositoryMeta>, S3StorageError> {
        let key = self.s3_path(&repository, location);
        Ok(self.read_meta(&key).await?.map(|meta| meta.repository_meta))
    }

    #[instrument(name = "Storage::delete_file", skip(self), fields(storage_type = "S3"))]
    async fn delete_file(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<bool, S3StorageError> {
        let key = self.s3_path(&repository, location);
        let prefix = self.directory_prefix(&repository, location);

        // A directory has to take its children and their sidecars with it, or the bucket keeps
        // serving files under a path that no longer exists.
        if self.is_directory(&prefix).await? {
            let deleted = self.delete_prefix(&prefix).await?;
            self.delete_prefix(&S3StorageInner::meta_key(&key)).await?;
            self.update_directory_metadata(repository, location).await?;
            return Ok(deleted);
        }

        if !self.does_path_exist(&key).await? {
            return Ok(false);
        }
        self.client
            .delete_object()
            .bucket(&self.config.bucket_name)
            .key(&key)
            .send()
            .await?;
        self.client
            .delete_object()
            .bucket(&self.config.bucket_name)
            .key(S3StorageInner::meta_key(&key))
            .send()
            .await?;
        self.update_directory_metadata(repository, location).await?;
        Ok(true)
    }

    #[instrument(
        name = "Storage::get_file_information",
        skip(self),
        fields(storage_type = "S3")
    )]
    async fn get_file_information(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<StorageFileMeta<FileType>>, S3StorageError> {
        let key = self.s3_path(&repository, location);
        let name = location
            .clone()
            .into_iter()
            .next_back()
            .map(|component| component.to_string())
            .unwrap_or_default();
        let stored_meta = self.read_meta(&key).await?;

        if let Some(head) = self.head(&key).await?
            && head.content_type() != Some(DIRECTORY_CONTENT_TYPE)
        {
            let modified = head
                .last_modified()
                .and_then(to_chrono)
                .or_else(|| stored_meta.as_ref().map(|meta| meta.modified))
                .unwrap_or_else(|| Local::now().fixed_offset());
            return Ok(Some(StorageFileMeta {
                name,
                file_type: FileType::File(FileFileType {
                    file_size: head.content_length().unwrap_or_default() as u64,
                    mime_type: head
                        .content_type()
                        .and_then(|value| match Mime::from_str(value) {
                            Ok(mime) => Some(SerdeMime(mime)),
                            Err(err) => {
                                warn!(
                                    ?err,
                                    content_type = value,
                                    "Ignoring unparsable content type"
                                );
                                None
                            }
                        })
                        .or_else(|| mime_for_name(&key)),
                    file_hash: stored_meta
                        .as_ref()
                        .map(|meta| meta.hashes())
                        .unwrap_or_default(),
                }),
                modified,
                created: stored_meta.map(|meta| meta.created).unwrap_or(modified),
            }));
        }

        let prefix = self.directory_prefix(&repository, location);
        if !self.is_directory(&prefix).await? {
            return Ok(None);
        }
        let entries = self.list_directory(&prefix).await?;
        let modified = stored_meta
            .as_ref()
            .map(|meta| meta.modified)
            .unwrap_or_else(|| Local::now().fixed_offset());
        Ok(Some(StorageFileMeta {
            name,
            file_type: FileType::Directory(DirectoryFileType {
                file_count: entries.len() as u64,
            }),
            modified,
            created: stored_meta.map(|meta| meta.created).unwrap_or(modified),
        }))
    }

    #[instrument(name = "Storage::open_file", skip(self), fields(storage_type = "S3"))]
    async fn open_file(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<StorageFile>, S3StorageError> {
        let Some(meta) = self.get_file_information(repository, location).await? else {
            return Ok(None);
        };

        match meta.file_type {
            FileType::Directory(file_type) => {
                let prefix = self.directory_prefix(&repository, location);
                let files = self.list_directory(&prefix).await?;
                Ok(Some(StorageFile::Directory {
                    meta: StorageFileMeta {
                        name: meta.name,
                        file_type,
                        modified: meta.modified,
                        created: meta.created,
                    },
                    files,
                }))
            }
            FileType::File(file_type) => {
                let key = self.s3_path(&repository, location);
                let output = match self
                    .client
                    .get_object()
                    .bucket(&self.config.bucket_name)
                    .key(&key)
                    .send()
                    .await
                {
                    Ok(output) => output,
                    Err(err) if is_not_found(&err) => return Ok(None),
                    Err(err) => return Err(err.into()),
                };
                // Streamed rather than buffered: an artifact can be far larger than we want to
                // hold in memory, and the response body is happy to pull from the socket.
                let reader =
                    crate::StorageFileReader::AsyncReader(Box::pin(output.body.into_async_read()));
                Ok(Some(StorageFile::File {
                    meta: StorageFileMeta {
                        name: meta.name,
                        file_type,
                        modified: meta.modified,
                        created: meta.created,
                    },
                    content: reader,
                }))
            }
        }
    }

    #[instrument(
        name = "Storage::validate_config_change",
        skip(self),
        fields(storage_type = "S3")
    )]
    async fn validate_config_change(
        &self,
        config: StorageTypeConfig,
    ) -> Result<(), S3StorageError> {
        S3StorageFactory::test_config(S3Config::from_type_config(config)?).await
    }

    #[instrument(name = "Storage::file_exists", skip(self), fields(storage_type = "S3"))]
    async fn file_exists(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<bool, S3StorageError> {
        let key = self.s3_path(&repository, location);
        if self.does_path_exist(&key).await? {
            return Ok(true);
        }
        let prefix = self.directory_prefix(&repository, location);
        self.is_directory(&prefix).await
    }

    #[instrument(
        name = "Storage::stream_directory",
        skip(self),
        fields(storage_type = "S3")
    )]
    async fn stream_directory(
        &self,
        repository: Uuid,
        location: &StoragePath,
    ) -> Result<Option<Self::DirectoryStream>, S3StorageError> {
        let prefix = self.directory_prefix(&repository, location);
        if !self.is_directory(&prefix).await? {
            return Ok(None);
        }
        let files = self.list_directory(&prefix).await?;
        let name = location
            .clone()
            .into_iter()
            .next_back()
            .map(|component| component.to_string())
            .unwrap_or_default();
        let now = Local::now().fixed_offset();
        let stored_meta = self.read_meta(prefix.trim_end_matches('/')).await?;
        let directory_meta = StorageFileMeta {
            name,
            file_type: DirectoryFileType {
                file_count: files.len() as u64,
            },
            modified: stored_meta
                .as_ref()
                .map(|meta| meta.modified)
                .unwrap_or(now),
            created: stored_meta.map(|meta| meta.created).unwrap_or(now),
        };
        Ok(Some(VecDirectoryListStream::new(files, directory_meta)))
    }
}

#[derive(Debug, Default)]
pub struct S3StorageFactory;

impl S3StorageFactory {
    pub const STORAGE_TYPE_NAME: &'static str = "S3";

    async fn test_config(config: S3Config) -> Result<(), S3StorageError> {
        let client = S3StorageInner::build_client(&config).await?;
        let inner = S3StorageInner {
            storage_config: StorageConfigInner::test_config(),
            config,
            client,
        };
        inner.check_bucket().await?;
        info!(bucket = %inner.config.bucket_name, "Successfully connected to S3 Bucket");
        Ok(())
    }
}

impl StaticStorageFactory for S3StorageFactory {
    type StorageType = S3Storage;
    type ConfigType = S3Config;
    type Error = S3StorageError;

    fn storage_type_name() -> &'static str {
        Self::STORAGE_TYPE_NAME
    }

    async fn test_storage_config(config: StorageTypeConfig) -> Result<(), S3StorageError> {
        Self::test_config(S3Config::from_type_config(config)?).await
    }

    async fn create_storage(
        inner: StorageConfigInner,
        type_config: Self::ConfigType,
    ) -> Result<Self::StorageType, S3StorageError> {
        let client = S3StorageInner::build_client(&type_config).await?;
        Ok(S3Storage::from(S3StorageInner {
            config: type_config,
            storage_config: inner,
            client,
        }))
    }
}

impl StorageFactory for S3StorageFactory {
    fn storage_name(&self) -> &'static str {
        Self::STORAGE_TYPE_NAME
    }

    fn test_storage_config(
        &self,
        config: StorageTypeConfig,
    ) -> BoxFuture<'static, Result<(), StorageError>> {
        Box::pin(async move {
            <Self as StaticStorageFactory>::test_storage_config(config).await?;
            Ok(())
        })
    }

    fn create_storage(
        &self,
        config: StorageConfig,
    ) -> BoxFuture<'static, Result<DynStorage, StorageError>> {
        Box::pin(async move {
            let storage =
                <Self as StaticStorageFactory>::create_storage_from_config(config).await?;
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
        let Some(config) = crate::testing::start_storage_test("S3").await? else {
            warn!("S3 Storage Test Skipped");
            return Ok(());
        };
        let storage =
            <S3StorageFactory as StaticStorageFactory>::create_storage_from_config(config).await?;
        let testing_storage = TestingStorage::new(storage);
        crate::testing::tests::full_test(testing_storage).await?;

        Ok(())
    }
}
