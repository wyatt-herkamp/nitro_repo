//! Resumable blob uploads.
//!
//! Docker pushes a blob in three steps: `POST` to open a session, one or more `PATCH`es to stream
//! bytes into it, then `PUT ?digest=` to commit. `Storage::save_file` takes a whole
//! [`FileContent`](nr_storage::FileContent) and there is no append or streaming write on any
//! backend, so the in-progress bytes are buffered to the local staging directory — the same place
//! [`StagingManager`](nr_repository::staging::StagingManager) writes — and only handed to
//! storage once the digest has been verified.
//!
//! The consequence, which the Docker docs page states: an upload is tied to the node that started
//! it, and is bounded by that node's disk. Adding a streaming write to the `Storage` trait would
//! remove both limits, at the cost of implementing it on all three backends and giving S3 real
//! multipart-upload state.
//!
//! The session table is modelled on
//! npm's `NpmWebLoginManager` (in `nr-npm`): an
//! in-memory map with a TTL, swept whenever a new session is opened.

use std::{
    fmt::Debug,
    path::{Path, PathBuf},
    sync::Arc,
};

use ahash::HashMap;
use bytes::Bytes;
use chrono::{DateTime, Duration, FixedOffset, Local};
use parking_lot::Mutex;
use tokio::io::AsyncWriteExt;
use tracing::{debug, instrument, warn};
use uuid::Uuid;

use super::types::digest::Digest;

/// How long an upload can sit untouched before it is swept.
///
/// Docker retries a failed push rather than resuming a stale one, so this only has to outlast a
/// slow layer, not a slow human.
const SESSION_TTL: Duration = Duration::hours(1);

#[derive(Debug, thiserror::Error)]
pub enum UploadError {
    #[error("upload `{0}` is not open")]
    UnknownUpload(Uuid),
    #[error("IO error while buffering an upload: {0}")]
    Io(#[from] std::io::Error),
    #[error("the uploaded content is `{actual}`, but the request committed it as `{expected}`")]
    DigestMismatch { expected: String, actual: String },
    #[error("this upload starts at offset {expected}, but the chunk claims to start at {actual}")]
    OutOfOrderChunk { expected: u64, actual: u64 },
}

#[derive(Debug, Clone)]
struct UploadSession {
    repository: Uuid,
    image: String,
    path: PathBuf,
    /// How many bytes have been written, which is also the offset the next chunk must start at.
    offset: u64,
    touched: DateTime<FixedOffset>,
}

#[derive(Default)]
struct UploadState {
    sessions: HashMap<Uuid, UploadSession>,
}

/// Tracks in-progress blob uploads across every Docker repository on this node.
#[derive(Clone)]
pub struct BlobUploadManager {
    state: Arc<Mutex<UploadState>>,
    root: Arc<PathBuf>,
}

impl Debug for BlobUploadManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlobUploadManager")
            .field("root", &self.root)
            .field("open", &self.state.lock().sessions.len())
            .finish()
    }
}

impl BlobUploadManager {
    /// `staging_dir` is the instance's staging directory; uploads live in a `docker` subdirectory
    /// of it so they are obviously separate from staged artifacts.
    pub fn new(staging_dir: &Path) -> Self {
        Self {
            state: Arc::new(Mutex::new(UploadState::default())),
            root: Arc::new(staging_dir.join("docker")),
        }
    }

    /// Opens a session and returns its id, which becomes the last segment of the upload URL.
    #[instrument(skip(self))]
    pub async fn start(&self, repository: Uuid, image: &str) -> Result<Uuid, UploadError> {
        self.sweep_expired();
        tokio::fs::create_dir_all(self.root.as_path()).await?;

        let id = Uuid::new_v4();
        let path = self.root.join(id.to_string());
        // Created up front so a `PATCH` that arrives before any bytes have been written still has
        // a file to append to.
        tokio::fs::File::create(&path).await?;

        self.state.lock().sessions.insert(
            id,
            UploadSession {
                repository,
                image: image.to_owned(),
                path,
                offset: 0,
                touched: Local::now().fixed_offset(),
            },
        );
        debug!(%id, %repository, image, "Opened a blob upload");
        Ok(id)
    }

    /// Appends a chunk and returns the new total length.
    ///
    /// `expected_start` is the `Content-Range` a chunked client sends. A chunk that does not start
    /// where the session left off is refused rather than appended: silently accepting it would
    /// assemble a blob out of order and the digest check at the end would report a corrupt upload
    /// with no indication of why.
    #[instrument(skip(self, chunk), fields(chunk_length = chunk.len()))]
    pub async fn append(
        &self,
        id: Uuid,
        repository: Uuid,
        expected_start: Option<u64>,
        chunk: Bytes,
    ) -> Result<u64, UploadError> {
        let session = self.session(id, repository)?;
        if let Some(start) = expected_start
            && start != session.offset
        {
            return Err(UploadError::OutOfOrderChunk {
                expected: session.offset,
                actual: start,
            });
        }

        if !chunk.is_empty() {
            let mut file = tokio::fs::OpenOptions::new()
                .append(true)
                .open(&session.path)
                .await?;
            file.write_all(&chunk).await?;
            file.flush().await?;
        }

        let mut state = self.state.lock();
        let Some(session) = state.sessions.get_mut(&id) else {
            return Err(UploadError::UnknownUpload(id));
        };
        session.offset += chunk.len() as u64;
        session.touched = Local::now().fixed_offset();
        Ok(session.offset)
    }

    /// How many bytes a session has taken so far, for the `Range` header on a status request.
    pub fn offset(&self, id: Uuid, repository: Uuid) -> Result<u64, UploadError> {
        Ok(self.session(id, repository)?.offset)
    }

    /// Verifies the buffered bytes against the digest the client committed them under, and returns
    /// them.
    ///
    /// The session and its temporary file are removed either way — a mismatch is not resumable, and
    /// leaving the file behind would leak the bytes of a rejected upload onto disk.
    #[instrument(skip(self))]
    pub async fn finish(
        &self,
        id: Uuid,
        repository: Uuid,
        expected: &Digest,
    ) -> Result<Bytes, UploadError> {
        let session = self.session(id, repository)?;
        let bytes = tokio::fs::read(&session.path).await?;
        self.discard(id).await;

        let actual = Digest::of(expected.algorithm(), &bytes);
        if &actual != expected {
            return Err(UploadError::DigestMismatch {
                expected: expected.to_string(),
                actual: actual.to_string(),
            });
        }
        debug!(%id, length = bytes.len(), "Committed a blob upload");
        Ok(Bytes::from(bytes))
    }

    /// `DELETE` on an upload URL: abandon it.
    #[instrument(skip(self))]
    pub async fn cancel(&self, id: Uuid, repository: Uuid) -> Result<(), UploadError> {
        self.session(id, repository)?;
        self.discard(id).await;
        Ok(())
    }

    /// Looks a session up, refusing one that belongs to a different repository.
    ///
    /// The id alone is not authority to write: without this check an upload opened against a
    /// repository the caller can write to could be committed into one they cannot.
    fn session(&self, id: Uuid, repository: Uuid) -> Result<UploadSession, UploadError> {
        let state = self.state.lock();
        match state.sessions.get(&id) {
            Some(session) if session.repository == repository => Ok(session.clone()),
            Some(session) => {
                warn!(
                    %id,
                    expected = %session.repository,
                    actual = %repository,
                    "An upload id was presented to a repository it was not opened against"
                );
                Err(UploadError::UnknownUpload(id))
            }
            None => Err(UploadError::UnknownUpload(id)),
        }
    }

    /// The image an upload was opened against, for the `Location` a commit responds with.
    pub fn image(&self, id: Uuid, repository: Uuid) -> Result<String, UploadError> {
        Ok(self.session(id, repository)?.image)
    }

    async fn discard(&self, id: Uuid) {
        let removed = self.state.lock().sessions.remove(&id);
        if let Some(session) = removed {
            let _ = tokio::fs::remove_file(&session.path).await;
        }
    }

    /// Drops sessions nobody has touched inside the TTL.
    ///
    /// Synchronous, and the files are left for the next process start to clear: doing the removal
    /// here would mean holding the lock across IO.
    fn sweep_expired(&self) {
        let cutoff = Local::now().fixed_offset() - SESSION_TTL;
        let mut state = self.state.lock();
        let expired: Vec<PathBuf> = state
            .sessions
            .iter()
            .filter(|(_, session)| session.touched < cutoff)
            .map(|(_, session)| session.path.clone())
            .collect();
        if expired.is_empty() {
            return;
        }
        state
            .sessions
            .retain(|_, session| session.touched >= cutoff);
        drop(state);

        tokio::spawn(async move {
            for path in expired {
                let _ = tokio::fs::remove_file(path).await;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use uuid::Uuid;

    use super::{BlobUploadManager, UploadError};
    use crate::types::digest::Digest;

    fn manager() -> (BlobUploadManager, tempfile::TempDir) {
        let directory = tempfile::tempdir().unwrap();
        (BlobUploadManager::new(directory.path()), directory)
    }

    #[tokio::test]
    async fn a_chunked_upload_reassembles_in_order() {
        let (manager, _directory) = manager();
        let repository = Uuid::new_v4();

        let id = manager.start(repository, "alpine").await.unwrap();
        assert_eq!(
            manager
                .append(id, repository, Some(0), Bytes::from_static(b"hello "))
                .await
                .unwrap(),
            6
        );
        assert_eq!(
            manager
                .append(id, repository, Some(6), Bytes::from_static(b"world"))
                .await
                .unwrap(),
            11
        );

        let digest = Digest::sha256_of(b"hello world");
        let bytes = manager.finish(id, repository, &digest).await.unwrap();
        assert_eq!(&bytes[..], b"hello world");
    }

    #[tokio::test]
    async fn a_monolithic_upload_needs_no_content_range() {
        let (manager, _directory) = manager();
        let repository = Uuid::new_v4();

        let id = manager.start(repository, "alpine").await.unwrap();
        manager
            .append(id, repository, None, Bytes::from_static(b"payload"))
            .await
            .unwrap();

        let digest = Digest::sha256_of(b"payload");
        assert_eq!(
            &manager.finish(id, repository, &digest).await.unwrap()[..],
            b"payload"
        );
    }

    #[tokio::test]
    async fn a_chunk_that_does_not_continue_the_upload_is_refused() {
        let (manager, _directory) = manager();
        let repository = Uuid::new_v4();

        let id = manager.start(repository, "alpine").await.unwrap();
        manager
            .append(id, repository, Some(0), Bytes::from_static(b"abc"))
            .await
            .unwrap();

        let error = manager
            .append(id, repository, Some(99), Bytes::from_static(b"def"))
            .await
            .unwrap_err();
        assert!(matches!(error, UploadError::OutOfOrderChunk { .. }));
        // The rejected chunk must not have been written.
        assert_eq!(manager.offset(id, repository).unwrap(), 3);
    }

    #[tokio::test]
    async fn a_digest_mismatch_is_refused_and_closes_the_upload() {
        let (manager, _directory) = manager();
        let repository = Uuid::new_v4();

        let id = manager.start(repository, "alpine").await.unwrap();
        manager
            .append(id, repository, None, Bytes::from_static(b"actual"))
            .await
            .unwrap();

        let wrong = Digest::sha256_of(b"something else");
        let error = manager.finish(id, repository, &wrong).await.unwrap_err();
        assert!(matches!(error, UploadError::DigestMismatch { .. }));

        // A mismatch is not resumable, so the session is gone.
        assert!(matches!(
            manager.offset(id, repository).unwrap_err(),
            UploadError::UnknownUpload(_)
        ));
    }

    /// An upload id is not authority to write: it only works against the repository it was opened
    /// against.
    #[tokio::test]
    async fn an_upload_cannot_be_committed_into_another_repository() {
        let (manager, _directory) = manager();
        let mine = Uuid::new_v4();
        let theirs = Uuid::new_v4();

        let id = manager.start(mine, "alpine").await.unwrap();
        manager
            .append(id, mine, None, Bytes::from_static(b"payload"))
            .await
            .unwrap();

        let digest = Digest::sha256_of(b"payload");
        assert!(matches!(
            manager.finish(id, theirs, &digest).await.unwrap_err(),
            UploadError::UnknownUpload(_)
        ));
        assert!(matches!(
            manager
                .append(id, theirs, None, Bytes::from_static(b"x"))
                .await
                .unwrap_err(),
            UploadError::UnknownUpload(_)
        ));
        // Still usable by its owner.
        assert_eq!(manager.offset(id, mine).unwrap(), 7);
    }

    #[tokio::test]
    async fn a_cancelled_upload_is_forgotten() {
        let (manager, _directory) = manager();
        let repository = Uuid::new_v4();

        let id = manager.start(repository, "alpine").await.unwrap();
        manager.cancel(id, repository).await.unwrap();
        assert!(matches!(
            manager.offset(id, repository).unwrap_err(),
            UploadError::UnknownUpload(_)
        ));
    }

    #[tokio::test]
    async fn an_unknown_upload_is_refused() {
        let (manager, _directory) = manager();
        let repository = Uuid::new_v4();
        let id = Uuid::new_v4();
        assert!(matches!(
            manager.offset(id, repository).unwrap_err(),
            UploadError::UnknownUpload(_)
        ));
    }
}
