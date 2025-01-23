use chrono::{DateTime, FixedOffset, Local};
use derive_more::derive::From;
use digest::Digest;
use mime::Mime;
use nr_core::{storage::FileHashes, utils::base64_utils};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};
use tracing::{
    debug, event,
    field::{debug, Empty},
    instrument, trace, warn, Level, Span,
};

use crate::{
    fs::utils::MetadataUtils, local::error::LocalStorageError, meta::RepositoryMeta,
    path::PathUtils,
};
pub static HIDDEN_FILE_EXTENSIONS: &[&str] = &["nr-meta"];
pub static NITRO_REPO_META_EXTENSION: &str = "nr-meta";
pub static NITRO_REPO_META_FILE: &str = ".nr-meta";
pub fn is_hidden_file(path: &Path) -> bool {
    if let Some(extension) = path.extension().and_then(|v| v.to_str()) {
        HIDDEN_FILE_EXTENSIONS.contains(&extension)
    } else if let Some(file_name) = path.file_name().and_then(|v| v.to_str()) {
        file_name.eq(NITRO_REPO_META_FILE)
    } else {
        false
    }
}

pub fn generate_hashes_from_path(path: impl AsRef<Path>) -> Result<FileHashes, io::Error> {
    let mut buffer = Vec::new();
    {
        let mut file = std::fs::File::open(path)?;
        file.read_to_end(&mut buffer)?;
    }
    Ok(generate_from_bytes(&buffer))
}
#[instrument(skip(buffer))]
pub fn generate_from_bytes(buffer: &[u8]) -> FileHashes {
    FileHashes {
        md5: Some(generate_md5(buffer)),
        sha1: Some(generate_sha1(buffer)),
        sha2_256: Some(generate_sha2_256(buffer)),
        sha3_256: Some(generate_sha3_256(buffer)),
    }
}
fn generate_md5(buffer: &[u8]) -> String {
    use md5::Md5;

    let mut hasher = Md5::new();
    hasher.update(buffer);
    let hash = hasher.finalize();
    base64_utils::encode(hash)
}
fn generate_sha1(buffer: &[u8]) -> String {
    use sha1::Sha1;

    let mut hasher = Sha1::new();
    hasher.update(buffer);
    let hash = hasher.finalize();
    base64_utils::encode(hash)
}
fn generate_sha2_256(buffer: &[u8]) -> String {
    use sha2::Sha256;

    let mut hasher = Sha256::new();
    hasher.update(buffer);
    let hash = hasher.finalize();
    base64_utils::encode(hash)
}
fn generate_sha3_256(buffer: &[u8]) -> String {
    use sha3::Sha3_256;

    let mut hasher = Sha3_256::new();
    hasher.update(buffer);
    let hash = hasher.finalize();
    base64_utils::encode(hash)
}

pub const FILE_META_MIME: Mime = mime::APPLICATION_JSON;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocationMeta {
    /// None if the file is a directory or meta file
    pub created: DateTime<FixedOffset>,
    pub modified: DateTime<FixedOffset>,
    pub location_typed_meta: LocationTypedMeta,
    pub repository_meta: RepositoryMeta,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, From)]
pub enum LocationTypedMeta {
    Directory(DirectoryMeta),
    File(FileMeta),
}
impl LocationTypedMeta {
    pub fn update(&mut self, path: impl AsRef<Path>) -> Result<(), LocalStorageError> {
        match self {
            LocationTypedMeta::Directory(meta) => meta.recount_files(path),
            LocationTypedMeta::File(meta) => meta.update_hashes(path),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectoryMeta {
    pub number_of_files: u64,
}
impl DirectoryMeta {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, LocalStorageError> {
        let mut meta = Self { number_of_files: 0 };
        meta.recount_files(path)?;
        Ok(meta)
    }
    pub fn recount_files(&mut self, path: impl AsRef<Path>) -> Result<(), LocalStorageError> {
        self.number_of_files = path
            .as_ref()
            .read_dir()?
            .filter_map(|entry| {
                let entry = entry.ok()?;
                if is_hidden_file(&entry.path()) {
                    return None;
                }
                Some(entry)
            })
            .count() as u64;
        debug!(?self.number_of_files, path = ?path.as_ref(), "Counted Files");
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMeta {
    pub hashes: FileHashes,
}
impl FileMeta {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, LocalStorageError> {
        Ok(Self {
            hashes: generate_hashes_from_path(path)?,
        })
    }
    pub fn update_hashes(&mut self, path: impl AsRef<Path>) -> Result<(), LocalStorageError> {
        self.hashes = generate_hashes_from_path(path)?;

        Ok(())
    }
}
impl LocationMeta {
    pub fn dir_meta_or_err(&self) -> Result<&DirectoryMeta, LocalStorageError> {
        if let LocationTypedMeta::Directory(meta) = &self.location_typed_meta {
            Ok(meta)
        } else {
            Err(LocalStorageError::ExpectedDirectory)
        }
    }
    pub fn file_meta_or_err(&self) -> Result<&FileMeta, LocalStorageError> {
        if let LocationTypedMeta::File(meta) = &self.location_typed_meta {
            Ok(meta)
        } else {
            Err(LocalStorageError::ExpectedFile)
        }
    }
    #[instrument(
        level = "debug",
        skip(path),
        fields(
            path = ?path.as_ref(),
        )
    )]
    pub(crate) fn create_meta_or_update(path: impl AsRef<Path>) -> Result<(), LocalStorageError> {
        let (mut meta, was_created) = Self::get_or_default_local(&path)?;
        if !was_created {
            event!(Level::DEBUG, path = ?path.as_ref(), "Updating Meta File");
            meta.location_typed_meta.update(&path)?;
            meta.modified = Local::now().into();
            meta.save_meta(path)?;
        }

        Ok(())
    }
    #[instrument(
        level = "debug",
        skip(path),
        fields(
            path = ?path.as_ref(),
            path.meta = Empty,
            created = Empty,
        )
    )]
    pub(crate) fn get_or_default_local(
        path: impl AsRef<Path>,
    ) -> Result<(LocationMeta, bool), LocalStorageError> {
        let span = Span::current();
        let meta_path = meta_path(&path)?;
        span.record("path.meta", debug(&meta_path));
        if meta_path.exists() {
            trace!(?meta_path, "Meta File exists. Reading");
            match LocationMeta::read_meta_file(&meta_path) {
                Ok(meta) => {
                    span.record("created", &false);
                    return Ok((meta, false));
                }
                Err(LocalStorageError::Postcard(err)) => {
                    event!(
                        Level::ERROR,
                        ?meta_path,
                        ?err,
                        "Meta File is corrupted. Rebuilding"
                    );
                }
                Err(err) => {
                    return Err(err);
                }
            }
        } else if tracing::enabled!(Level::DEBUG) {
            debug!(?meta_path, "Meta File does not exist. Generating");
        }
        span.record("created", &true);
        let (created, modified) = {
            let file = File::open(&path)?;
            let metadata = file.metadata()?;
            let modified = metadata.modified_as_chrono_or_now()?;
            let created = metadata.created_as_chrono_or_now()?;
            (created, modified)
        };
        let location_meta = if path.as_ref().is_dir() {
            LocationTypedMeta::Directory(DirectoryMeta::new(&path)?)
        } else {
            LocationTypedMeta::File(FileMeta::new(&path)?)
        };
        let meta = LocationMeta {
            created,
            modified,
            repository_meta: RepositoryMeta::default(),
            location_typed_meta: location_meta,
        };
        meta.save_meta(&path)?;

        return Ok((meta, true));
    }

    #[instrument(
        level = "debug",
        skip(path),
        fields(
            path = ?path.as_ref(),
        )
    )]
    /// Assumes the path is the path to the `.nr-meta` file
    fn read_meta_file(path: impl AsRef<Path>) -> Result<LocationMeta, LocalStorageError> {
        let mut file = File::open(path)?;

        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        let meta: LocationMeta = postcard::from_bytes(&bytes)?;
        Ok(meta)
    }

    #[instrument(
        level = "debug",
        skip(path),
        fields(
            path = ?path.as_ref(),
            path.meta = Empty,
        )
    )]
    pub(crate) fn delete_local(path: impl AsRef<Path>) -> Result<(), LocalStorageError> {
        let meta_path = meta_path(&path)?;
        Span::current().record("path.meta", debug(&meta_path));
        if !meta_path.exists() {
            warn!(?meta_path, "Meta File does not exist");
            return Ok(());
        }
        debug!(?meta_path, "Deleting Meta File");
        std::fs::remove_file(meta_path)?;
        Ok(())
    }
    #[instrument(
        level = "debug",
        skip(path),
        fields(
            path = ?path.as_ref(),
            path.meta = Empty,
            created = Empty,
        )
    )]
    pub(crate) fn save_meta(&self, path: impl AsRef<Path>) -> Result<(), LocalStorageError> {
        let span = Span::current();
        let meta_path = meta_path(path)?;
        span.record("path.meta", debug(&meta_path));
        span.record("created", !meta_path.exists());
        let file = File::create(meta_path)?;
        postcard::to_io(self, file)?;
        event!(Level::DEBUG, "Saved Meta File");
        Ok(())
    }

    #[instrument(
        level = "debug",
        skip(path),
        fields(
            path = ?path.as_ref(),
        )
    )]
    pub(crate) fn set_repository_meta(
        path: impl AsRef<Path>,
        repository_meta: RepositoryMeta,
    ) -> Result<(), LocalStorageError> {
        let (mut meta, _) = Self::get_or_default_local(&path)?;
        meta.repository_meta = repository_meta;
        meta.save_meta(path)
    }
}

fn meta_path(path: impl AsRef<Path>) -> Result<PathBuf, LocalStorageError> {
    let meta_path = path.as_ref().to_path_buf();
    let meta_path = if meta_path.is_dir() {
        meta_path.join(NITRO_REPO_META_FILE)
    } else {
        meta_path.add_extension(NITRO_REPO_META_EXTENSION)?
    };
    Ok(meta_path)
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::meta::RepositoryMeta;

    use super::LocationMeta;
    fn random_repo_meta() -> RepositoryMeta {
        let mut meta = RepositoryMeta::default();
        meta.project_id = Some(Uuid::new_v4());
        meta.project_version_id = Some(1);

        meta.insert("test", "map");

        meta
    }
    #[test]
    pub fn post_card_compatible_meta_directory() {
        let meta = LocationMeta {
            created: chrono::Local::now().fixed_offset(),
            modified: chrono::Local::now().fixed_offset(),
            location_typed_meta: super::LocationTypedMeta::Directory(super::DirectoryMeta {
                number_of_files: 0,
            }),
            repository_meta: random_repo_meta(),
        };

        let bytes = postcard::to_allocvec(&meta).unwrap();

        let from_bytes: LocationMeta = postcard::from_bytes(&bytes).unwrap();

        assert_eq!(meta, from_bytes);
    }

    #[test]
    pub fn post_card_compatible_meta_file() {
        let meta = LocationMeta {
            created: chrono::Local::now().fixed_offset(),
            modified: chrono::Local::now().fixed_offset(),
            location_typed_meta: super::LocationTypedMeta::File(super::FileMeta {
                hashes: super::FileHashes {
                    md5: Some("md5".to_string()),
                    sha1: Some("sha1".to_string()),
                    sha2_256: Some("sha2_256".to_string()),
                    sha3_256: Some("sha3_256".to_string()),
                },
            }),
            repository_meta: random_repo_meta(),
        };

        let bytes = postcard::to_allocvec(&meta).unwrap();

        let from_bytes: LocationMeta = postcard::from_bytes(&bytes).unwrap();

        assert_eq!(meta, from_bytes);
    }
}
