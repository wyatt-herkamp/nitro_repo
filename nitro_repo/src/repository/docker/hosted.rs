use std::sync::{
    Arc,
    atomic::{self, AtomicBool},
};

use bytes::Bytes;
use derive_more::derive::Deref;
use http::{
    HeaderValue, StatusCode,
    header::{CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, LOCATION, RANGE},
    request::Parts,
};
use nr_core::{
    database::entities::{
        docker_manifest::DBDockerManifest,
        project::{
            DBProject, NewProject, NewProjectMember, ProjectDBType,
            versions::{DBProjectVersion, NewVersion, UpdateProjectVersion},
        },
        repository::DBRepository,
    },
    repository::{
        Visibility,
        config::{
            RepositoryConfigType, project::ProjectConfigType, repository_page::RepositoryPageType,
        },
        project::{ReleaseType, VersionData},
    },
    storage::StoragePath,
    user::permissions::RepositoryActions,
};
use nr_storage::{DynStorage, FileContent, Storage, StorageFile};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

use super::{
    DOCKER_API_VERSION_HEADER, DOCKER_API_VERSION_VALUE, DOCKER_CONTENT_DIGEST_HEADER,
    DOCKER_UPLOAD_UUID_HEADER, DockerError, DockerRegistryConfigType,
    errors::ErrorCode,
    types::{
        Digest, DockerPath, Manifest, Reference,
        manifest::{DEFAULT_MANIFEST_MEDIA_TYPE, is_index},
    },
    uploads::BlobUploadManager,
};
use crate::{
    app::SiteContext,
    repository::{
        RepoResponse, Repository, RepositoryAuthentication, RepositoryFactoryError,
        RepositoryRequest, utils::RepositoryExt,
    },
    utils::ResponseBuilder,
};

/// The default page size for `tags/list` and `_catalog`, and the cap on what `?n=` can ask for.
const DEFAULT_PAGE_SIZE: usize = 100;
const MAX_PAGE_SIZE: usize = 1000;

#[derive(derive_more::Debug)]
pub struct DockerRegistryInner {
    #[debug(skip)]
    pub site: SiteContext,
    /// Handed down from [`DockerRegistryType`](super::DockerRegistryType), which owns it. It used
    /// to be reached through the application state, which meant every repository type could see
    /// Docker's in-flight uploads.
    #[debug(skip)]
    pub uploads: BlobUploadManager,
    pub storage: DynStorage,
    pub id: Uuid,
    pub name: String,
    pub visibility: RwLock<Visibility>,
    pub active: AtomicBool,
}

/// The value `Repository::full_type()` reports, and what lands in the repository request
/// span and metric attributes. Pinned by `repository_type_ids_are_stable`.
pub static FULL_TYPE: &str = "docker/hosted";

#[derive(Debug, Clone, Deref)]
pub struct DockerHostedRegistry(Arc<DockerRegistryInner>);

impl DockerHostedRegistry {
    pub async fn load(
        site: SiteContext,
        storage: DynStorage,
        repository: DBRepository,
        uploads: BlobUploadManager,
    ) -> Result<Self, RepositoryFactoryError> {
        Ok(Self(Arc::new(DockerRegistryInner {
            site,
            uploads,
            storage,
            id: repository.id,
            name: repository.name.to_string(),
            visibility: RwLock::new(repository.visibility),
            active: AtomicBool::new(repository.active),
        })))
    }

    /// Where a blob's bytes live: `blobs/{algorithm}/{hex}`.
    ///
    /// Content-addressed, so two images sharing a base layer share the stored bytes. Both
    /// components come out of a parsed [`Digest`], so neither can escape the repository root.
    fn blob_path(digest: &Digest) -> Result<StoragePath, DockerError> {
        let (algorithm, hex) = digest.path_components();
        let mut path = StoragePath::parse("blobs")?;
        path.push_mut(algorithm);
        path.push_mut(hex);
        Ok(path)
    }

    /// Where a manifest's bytes live. Separate from blobs so a manifest is never mistaken for one.
    fn manifest_path(digest: &Digest) -> Result<StoragePath, DockerError> {
        let (algorithm, hex) = digest.path_components();
        let mut path = StoragePath::parse("manifests")?;
        path.push_mut(algorithm);
        path.push_mut(hex);
        Ok(path)
    }

    /// Refuses a read the caller may not make.
    ///
    /// A Docker client does not send credentials until it has been challenged, so an
    /// unauthenticated read of a private registry answers `401` with `WWW-Authenticate` rather than
    /// a flat `403`. That challenge is the whole handshake `docker login` performs.
    async fn check_read(
        &self,
        authentication: &RepositoryAuthentication,
        parts: &Parts,
        scope: Option<String>,
    ) -> Result<Option<DockerError>, DockerError> {
        let scope = scope.as_deref();
        if self.visibility().is_public() {
            return Ok(None);
        }
        if authentication.is_no_identification() {
            return Ok(Some(self.challenge(parts, scope, "pull")));
        }
        if !authentication
            .can_access_repository(RepositoryActions::Read, self.0.id, self.site.as_ref())
            .await?
        {
            return Ok(Some(DockerError::coded(
                ErrorCode::Denied,
                "you do not have read access to this registry",
            )));
        }
        Ok(None)
    }

    /// The `401` that tells a client where to get a token.
    ///
    /// The realm is built from the host the client is already talking to, not from `app_url`: a
    /// registry reached on a custom domain may be the only address that resolves from wherever
    /// Docker is running, and it is also what lets the token endpoint work out which repository the
    /// caller means when the image name carries no prefix.
    pub fn challenge(&self, parts: &Parts, image: Option<&str>, actions: &str) -> DockerError {
        let site = &self.0.site;
        let app_url = {
            let instance = site.instance.lock();
            instance.app_url.clone()
        };
        let base = crate::utils::host::request_origin(
            &parts.headers,
            &parts.uri,
            site.general_security_settings.trust_forwarded_host,
            &app_url,
        )
        .unwrap_or_default();
        DockerError::Challenge {
            // Under `/api`, not under `/v2`, so it can never collide with an image literally named
            // `token`. `/api` is nested before the fallback and served on every host, so it is
            // reachable on a custom Docker domain too.
            realm: format!("{base}/api/docker/token"),
            scope: image.map(|image| format!("repository:{image}:{actions}")),
        }
    }

    /// `image` must be the client's spelling — see [`Origin`]. A challenge naming an image the
    /// client did not ask for makes the token it fetches useless for the request it was retrying.
    async fn require_write(
        &self,
        authentication: &RepositoryAuthentication,
        parts: &Parts,
        image: &str,
    ) -> Result<i32, DockerError> {
        if authentication.is_no_identification() {
            return Err(self.challenge(parts, Some(image), "pull,push"));
        }
        let Some(user) = authentication
            .get_user_if_has_action(RepositoryActions::Write, self.0.id, self.site.as_ref())
            .await?
        else {
            return Err(DockerError::coded(
                ErrorCode::Denied,
                "you do not have push access to this registry",
            ));
        };
        Ok(user.id)
    }

    // ---- reads -------------------------------------------------------------------------------

    /// `GET`/`HEAD /v2/{name}/manifests/{reference}`.
    #[instrument(skip(self))]
    async fn read_manifest(
        &self,
        image: &str,
        reference: &Reference,
        head_only: bool,
    ) -> Result<RepoResponse, DockerError> {
        let digest = self.resolve_reference(image, reference).await?;
        let Some(record) =
            DBDockerManifest::get(self.0.id, image, &digest.to_string(), self.site.as_ref())
                .await?
        else {
            return Err(DockerError::coded(
                ErrorCode::ManifestUnknown,
                format!("`{image}:{reference}` is not in this registry"),
            ));
        };

        let path = Self::manifest_path(&digest)?;
        let Some(StorageFile::File { meta, content }) =
            self.storage.open_file(self.0.id, &path).await?
        else {
            // The row and the bytes disagree, which is a server-side inconsistency rather than a
            // missing image, so it is worth a log rather than a silent 404.
            warn!(%digest, image, "A recorded manifest has no stored bytes");
            return Err(DockerError::coded(
                ErrorCode::ManifestUnknown,
                format!("the manifest for `{image}:{reference}` is missing from storage"),
            ));
        };

        let mut response = ResponseBuilder::ok()
            .header(CONTENT_TYPE, record.media_type.clone())
            .header(CONTENT_LENGTH, record.size.to_string())
            .header(DOCKER_CONTENT_DIGEST_HEADER.clone(), digest.to_string())
            .header(
                DOCKER_API_VERSION_HEADER.clone(),
                DOCKER_API_VERSION_VALUE.clone(),
            );
        // A `HEAD` must carry exactly the headers the `GET` would, and no body. Clients use it to
        // resolve a tag to a digest without transferring the manifest.
        if head_only {
            let _ = meta;
            return Ok(RepoResponse::Other(response.empty()));
        }
        response = response.header("accept-ranges", "none");
        let bytes = read_all(content).await?;
        Ok(RepoResponse::Other(response.body(bytes)))
    }

    /// A tag resolves through `project_versions`; a digest is taken as given.
    async fn resolve_reference(
        &self,
        image: &str,
        reference: &Reference,
    ) -> Result<Digest, DockerError> {
        match reference {
            Reference::Digest(digest) => Ok(digest.clone()),
            Reference::Tag(tag) => {
                let project = self.project_or_unknown(image).await?;
                let Some(version) = self.get_project_version(project.id, tag).await? else {
                    return Err(DockerError::coded(
                        ErrorCode::ManifestUnknown,
                        format!("`{image}` has no tag `{tag}`"),
                    ));
                };
                let stored = self.stored_tag(&version)?;
                Digest::parse(&stored.digest)
                    .map_err(|error| DockerError::InvalidDigest(error.to_string()))
            }
        }
    }

    /// `GET`/`HEAD /v2/{name}/blobs/{digest}`.
    #[instrument(skip(self))]
    async fn read_blob(
        &self,
        image: &str,
        digest: &Digest,
        head_only: bool,
    ) -> Result<RepoResponse, DockerError> {
        let path = Self::blob_path(digest)?;
        let Some(StorageFile::File { meta, content }) =
            self.storage.open_file(self.0.id, &path).await?
        else {
            return Err(DockerError::coded(
                ErrorCode::BlobUnknown,
                format!("blob `{digest}` is not in this registry"),
            ));
        };
        let size = meta.file_type().file_size;

        let response = ResponseBuilder::ok()
            .header(CONTENT_TYPE, "application/octet-stream")
            .header(CONTENT_LENGTH, size.to_string())
            .header(DOCKER_CONTENT_DIGEST_HEADER.clone(), digest.to_string())
            .header(
                DOCKER_API_VERSION_HEADER.clone(),
                DOCKER_API_VERSION_VALUE.clone(),
            )
            // Range requests on blobs are not implemented; saying so keeps a client from asking and
            // then being handed the whole blob under a `206` it did not get.
            .header("accept-ranges", "none");

        if head_only {
            return Ok(RepoResponse::Other(response.empty()));
        }
        Ok(RepoResponse::Other(response.body(read_all(content).await?)))
    }

    /// `GET /v2/{name}/tags/list`.
    #[instrument(skip(self, parts))]
    async fn tags_list(
        &self,
        origin: &Origin,
        image: &str,
        parts: &Parts,
    ) -> Result<RepoResponse, DockerError> {
        let project = self.project_or_unknown(image).await?;
        let versions = DBProjectVersion::get_all_versions(project.id, self.site.as_ref()).await?;

        // The spec orders tags lexically, and pages with `?n=` and `?last=`.
        let mut tags: Vec<String> = versions
            .into_iter()
            .map(|version| version.version)
            .collect();
        tags.sort();
        let query: PageQuery = parts
            .uri
            .query()
            .and_then(|query| serde_urlencoded::from_str(query).ok())
            .unwrap_or_default();
        let tags = paginate(tags, &query);

        Ok(RepoResponse::Other(
            ResponseBuilder::ok()
                .header(
                    DOCKER_API_VERSION_HEADER.clone(),
                    DOCKER_API_VERSION_VALUE.clone(),
                )
                // `name` echoes the repository the client asked about, so it has to be spelled the
                // way the client spelled it — prefix included in prefix mode.
                .json(&serde_json::json!({ "name": origin.image_name(image), "tags": tags })),
        ))
    }

    /// `GET /v2/_catalog`.
    #[instrument(skip(self, parts))]
    async fn catalog(&self, origin: &Origin, parts: &Parts) -> Result<RepoResponse, DockerError> {
        let stored: Vec<String> =
            sqlx::query_scalar("SELECT key FROM projects WHERE repository_id = $1 ORDER BY key")
                .bind(self.0.id)
                .fetch_all(self.site.as_ref())
                .await?;
        // Listed the way they would be pulled, so a name copied out of the catalog works verbatim.
        let mut names: Vec<String> = stored
            .into_iter()
            .map(|name| origin.image_name(&name))
            .collect();
        names.sort();

        let query: PageQuery = parts
            .uri
            .query()
            .and_then(|query| serde_urlencoded::from_str(query).ok())
            .unwrap_or_default();
        let names = paginate(names, &query);

        Ok(RepoResponse::Other(
            ResponseBuilder::ok()
                .header(
                    DOCKER_API_VERSION_HEADER.clone(),
                    DOCKER_API_VERSION_VALUE.clone(),
                )
                .json(&serde_json::json!({ "repositories": names })),
        ))
    }

    /// `GET /v2/{name}/referrers/{digest}`.
    ///
    /// Answers `200` with an empty index when nothing refers to the subject — the spec is explicit
    /// that an unknown subject is not a 404 here, because a client uses this to *discover* whether
    /// anything exists.
    #[instrument(skip(self))]
    async fn referrers(&self, image: &str, subject: &Digest) -> Result<RepoResponse, DockerError> {
        let found =
            DBDockerManifest::referrers(self.0.id, image, &subject.to_string(), self.site.as_ref())
                .await?;

        let manifests: Vec<serde_json::Value> = found
            .into_iter()
            .map(|record| {
                serde_json::json!({
                    "mediaType": record.media_type,
                    "digest": record.digest,
                    "size": record.size,
                    "artifactType": record.artifact_type,
                })
            })
            .collect();

        Ok(RepoResponse::Other(
            ResponseBuilder::ok()
                .header(CONTENT_TYPE, super::types::manifest::MEDIA_TYPE_OCI_INDEX)
                .header(
                    DOCKER_API_VERSION_HEADER.clone(),
                    DOCKER_API_VERSION_VALUE.clone(),
                )
                .json(&serde_json::json!({
                    "schemaVersion": 2,
                    "mediaType": super::types::manifest::MEDIA_TYPE_OCI_INDEX,
                    "manifests": manifests,
                })),
        ))
    }

    // ---- uploads -----------------------------------------------------------------------------

    /// `POST /v2/{name}/blobs/uploads/`.
    #[instrument(skip(self, request))]
    async fn start_upload(
        &self,
        request: RepositoryRequest,
        image: &str,
    ) -> Result<RepoResponse, DockerError> {
        let origin = Origin::of(&request.parts, &request.path);
        self.require_write(
            &request.authentication,
            &request.parts,
            &origin.image_name(image),
        )
        .await?;

        let query: UploadQuery = request
            .parts
            .uri
            .query()
            .and_then(|query| serde_urlencoded::from_str(query).ok())
            .unwrap_or_default();

        // Cross-repository mount. Not supported, and the spec says to fall back to a normal upload
        // rather than to fail — the client then pushes the blob it wanted to borrow.
        if query.mount.is_some() {
            debug!("Ignoring a cross-repository mount and starting a normal upload");
        } else if let Some(raw) = &query.digest {
            // Monolithic upload: the whole blob came with the POST.
            let digest = Digest::parse(raw)
                .map_err(|error| DockerError::InvalidDigest(error.to_string()))?;
            let body = request.body.body_as_bytes().await?;
            return self.commit_blob(&origin, image, &digest, body).await;
        }

        let id = self
            .uploads
            .start(self.0.id, image)
            .await
            .map_err(DockerError::Upload)?;

        Ok(RepoResponse::Other(
            ResponseBuilder::default()
                .status(StatusCode::ACCEPTED)
                .header(LOCATION, origin.url(image, &format!("blobs/uploads/{id}")))
                .header(RANGE, "0-0")
                .header(CONTENT_LENGTH, "0")
                .header(DOCKER_UPLOAD_UUID_HEADER.clone(), id.to_string())
                .header(
                    DOCKER_API_VERSION_HEADER.clone(),
                    DOCKER_API_VERSION_VALUE.clone(),
                )
                .empty(),
        ))
    }

    /// `PATCH /v2/{name}/blobs/uploads/{uuid}`.
    #[instrument(skip(self, request))]
    async fn append_upload(
        &self,
        request: RepositoryRequest,
        image: &str,
        id: Uuid,
    ) -> Result<RepoResponse, DockerError> {
        let origin = Origin::of(&request.parts, &request.path);
        self.require_write(
            &request.authentication,
            &request.parts,
            &origin.image_name(image),
        )
        .await?;

        let start = content_range_start(&request.parts);
        let body = request.body.body_as_bytes().await?;
        let offset = self
            .uploads
            .append(id, self.0.id, start, body)
            .await
            .map_err(DockerError::Upload)?;

        Ok(RepoResponse::Other(
            ResponseBuilder::default()
                .status(StatusCode::ACCEPTED)
                .header(LOCATION, origin.url(image, &format!("blobs/uploads/{id}")))
                // Inclusive, so an upload of n bytes reports `0-(n-1)`. An empty one reports `0-0`,
                // which is what the spec's own example does.
                .header(RANGE, format!("0-{}", offset.saturating_sub(1)))
                .header(CONTENT_LENGTH, "0")
                .header(DOCKER_UPLOAD_UUID_HEADER.clone(), id.to_string())
                .header(
                    DOCKER_API_VERSION_HEADER.clone(),
                    DOCKER_API_VERSION_VALUE.clone(),
                )
                .empty(),
        ))
    }

    /// `PUT /v2/{name}/blobs/uploads/{uuid}?digest=`.
    #[instrument(skip(self, request))]
    async fn finish_upload(
        &self,
        request: RepositoryRequest,
        image: &str,
        id: Uuid,
    ) -> Result<RepoResponse, DockerError> {
        let origin = Origin::of(&request.parts, &request.path);
        self.require_write(
            &request.authentication,
            &request.parts,
            &origin.image_name(image),
        )
        .await?;

        let query: UploadQuery = request
            .parts
            .uri
            .query()
            .and_then(|query| serde_urlencoded::from_str(query).ok())
            .unwrap_or_default();
        let Some(raw) = query.digest else {
            return Err(DockerError::coded(
                ErrorCode::DigestInvalid,
                "committing an upload requires a `digest` query parameter",
            ));
        };
        let digest =
            Digest::parse(&raw).map_err(|error| DockerError::InvalidDigest(error.to_string()))?;

        // A commit may carry the last chunk in its body.
        let start = content_range_start(&request.parts);
        let body = request.body.body_as_bytes().await?;
        if !body.is_empty() {
            self.uploads
                .append(id, self.0.id, start, body)
                .await
                .map_err(DockerError::Upload)?;
        }

        // Verified against the digest the client committed under; a mismatch closes the session and
        // nothing reaches storage.
        let bytes = self
            .uploads
            .finish(id, self.0.id, &digest)
            .await
            .map_err(DockerError::Upload)?;

        self.commit_blob(&origin, image, &digest, bytes).await
    }

    /// Writes a verified blob and answers the `201` a client expects.
    async fn commit_blob(
        &self,
        origin: &Origin,
        image: &str,
        digest: &Digest,
        bytes: Bytes,
    ) -> Result<RepoResponse, DockerError> {
        if !digest.matches(&bytes) {
            return Err(DockerError::coded(
                ErrorCode::DigestInvalid,
                format!(
                    "the uploaded content is `{}`, but the request committed it as `{digest}`",
                    Digest::of(digest.algorithm(), &bytes)
                ),
            ));
        }
        let path = Self::blob_path(digest)?;
        self.storage
            .save_file(self.0.id, FileContent::Bytes(bytes), &path)
            .await?;

        Ok(RepoResponse::Other(
            ResponseBuilder::created()
                .header(LOCATION, origin.url(image, &format!("blobs/{digest}")))
                .header(CONTENT_LENGTH, "0")
                .header(DOCKER_CONTENT_DIGEST_HEADER.clone(), digest.to_string())
                .header(
                    DOCKER_API_VERSION_HEADER.clone(),
                    DOCKER_API_VERSION_VALUE.clone(),
                )
                .empty(),
        ))
    }

    // ---- manifest push -----------------------------------------------------------------------

    /// `PUT /v2/{name}/manifests/{reference}` — the commit that makes an image pullable.
    #[instrument(skip(self, request))]
    async fn put_manifest(
        &self,
        request: RepositoryRequest,
        image: &str,
        reference: Reference,
    ) -> Result<RepoResponse, DockerError> {
        let origin = Origin::of(&request.parts, &request.path);
        let publisher = self
            .require_write(
                &request.authentication,
                &request.parts,
                &origin.image_name(image),
            )
            .await?;

        let media_type = request
            .parts
            .headers
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.split(';').next().unwrap_or(value).trim().to_owned())
            .unwrap_or_else(|| DEFAULT_MANIFEST_MEDIA_TYPE.to_owned());

        let bytes = request.body.body_as_bytes().await?;
        if bytes.is_empty() {
            return Err(DockerError::coded(
                ErrorCode::ManifestInvalid,
                "the manifest body is empty",
            ));
        }
        let manifest: Manifest = serde_json::from_slice(&bytes).map_err(|error| {
            DockerError::coded(
                ErrorCode::ManifestInvalid,
                format!("the manifest will not parse: {error}"),
            )
        })?;

        // The digest is over the bytes exactly as they arrived, so they are stored and re-served
        // verbatim. Re-serialising would give the same manifest a different identity, and every
        // client that pinned the old digest would stop resolving.
        let digest = Digest::sha256_of(&bytes);
        if let Reference::Digest(claimed) = &reference
            && claimed != &digest
        {
            return Err(DockerError::coded(
                ErrorCode::DigestInvalid,
                format!("the manifest hashes to `{digest}`, but was pushed as `{claimed}`"),
            ));
        }

        // Everything a manifest points at must already be here. Without this a push can succeed and
        // every pull of it then fail on a missing layer, with nothing to say which push was at
        // fault.
        if is_index(&media_type) {
            for child in manifest.child_manifests() {
                if !DBDockerManifest::exists(
                    self.0.id,
                    image,
                    &child.to_string(),
                    self.site.as_ref(),
                )
                .await?
                {
                    return Err(DockerError::coded(
                        ErrorCode::ManifestBlobUnknown,
                        format!("the index refers to manifest `{child}`, which is not here yet"),
                    ));
                }
            }
        } else {
            for blob in manifest.required_blobs() {
                let path = Self::blob_path(blob)?;
                if !self.storage.file_exists(self.0.id, &path).await? {
                    return Err(DockerError::coded(
                        ErrorCode::ManifestBlobUnknown,
                        format!("the manifest refers to blob `{blob}`, which is not here yet"),
                    ));
                }
            }
        }

        let size = bytes.len() as i64;
        let path = Self::manifest_path(&digest)?;
        self.storage
            .save_file(self.0.id, FileContent::Bytes(bytes), &path)
            .await?;

        DBDockerManifest::upsert(
            self.0.id,
            image,
            &digest.to_string(),
            &media_type,
            size,
            manifest
                .subject
                .as_ref()
                .map(|subject| subject.digest.to_string())
                .as_deref(),
            manifest.artifact_type.as_deref(),
            self.site.as_ref(),
        )
        .await?;

        // Only a tag becomes a version. An untagged manifest — the per-platform child of an index,
        // or a referrers artifact — is reachable by digest and recorded above, but it is not
        // something to show on a project page.
        if let Reference::Tag(tag) = &reference {
            self.record_tag(image, tag, &digest, &media_type, size, &manifest, publisher)
                .await?;
        }

        debug!(image, %reference, %digest, "Accepted a manifest");
        Ok(RepoResponse::Other(
            ResponseBuilder::created()
                .header(
                    LOCATION,
                    origin.url(image, &format!("manifests/{reference}")),
                )
                .header(CONTENT_LENGTH, "0")
                .header(DOCKER_CONTENT_DIGEST_HEADER.clone(), digest.to_string())
                .header(
                    DOCKER_API_VERSION_HEADER.clone(),
                    DOCKER_API_VERSION_VALUE.clone(),
                )
                .empty(),
        ))
    }

    /// Points a tag at a manifest, as a project version.
    ///
    /// Re-tagging is normal — `:latest` moves with every push — so an existing tag is updated in
    /// place rather than refused, which is the opposite of what npm and Cargo do with a version.
    #[allow(clippy::too_many_arguments)]
    async fn record_tag(
        &self,
        image: &str,
        tag: &str,
        digest: &Digest,
        media_type: &str,
        size: i64,
        manifest: &Manifest,
        publisher: i32,
    ) -> Result<(), DockerError> {
        let project = self.get_or_create_project(image, publisher).await?;
        let stored = StoredTag {
            digest: digest.to_string(),
            media_type: media_type.to_owned(),
            size,
            platforms: manifest.platforms(),
        };
        let extra = VersionData {
            extra: Some(serde_json::to_value(&stored)?),
            ..Default::default()
        };

        match self.get_project_version(project.id, tag).await? {
            Some(existing) => {
                UpdateProjectVersion {
                    extra: Some(extra),
                    publisher: Some(Some(publisher)),
                    ..Default::default()
                }
                .update(existing.id, self.site.as_ref())
                .await?;
            }
            None => {
                let mut version_path = StoragePath::parse("manifests")?;
                let (algorithm, hex) = digest.path_components();
                version_path.push_mut(algorithm);
                version_path.push_mut(hex);
                NewVersion {
                    project_id: project.id,
                    version: tag.to_owned(),
                    release_type: ReleaseType::release_type_from_version(tag),
                    version_path: version_path.to_string(),
                    publisher: Some(publisher),
                    version_page: None,
                    extra,
                }
                .insert(self.site.as_ref())
                .await?;
            }
        }
        Ok(())
    }

    async fn get_or_create_project(
        &self,
        image: &str,
        publisher: i32,
    ) -> Result<DBProject, DockerError> {
        if let Some(project) =
            DBProject::find_by_project_key(image, self.0.id, self.site.as_ref()).await?
        {
            return Ok(project);
        }
        // The image name is the project key. `library/alpine` keeps its slash in the key and
        // becomes a nested storage path, which is what makes `_catalog` read back the name that was
        // pushed.
        let (scope, name) = match image.rsplit_once('/') {
            Some((scope, name)) => (Some(scope.to_owned()), name.to_owned()),
            None => (None, image.to_owned()),
        };
        let mut storage_path = StoragePath::parse("images")?;
        for component in image.split('/') {
            storage_path.push_mut(component);
        }
        let project = NewProject {
            scope,
            project_key: image.to_owned(),
            name,
            description: None,
            repository: self.0.id,
            storage_path: storage_path.to_string(),
        }
        .insert(self.site.as_ref())
        .await?;
        NewProjectMember::new_owner(publisher, project.id)
            .insert_no_return(self.site.as_ref())
            .await?;
        info!(image, "Created a new image");
        Ok(project)
    }

    // ---- deletes -----------------------------------------------------------------------------

    #[instrument(skip(self, authentication))]
    async fn delete_manifest(
        &self,
        authentication: &RepositoryAuthentication,
        parts: &Parts,
        origin: &Origin,
        image: &str,
        reference: &Reference,
    ) -> Result<RepoResponse, DockerError> {
        self.require_write(authentication, parts, &origin.image_name(image))
            .await?;

        match reference {
            // Deleting a tag removes the pointer, not the manifest — another tag or a digest pull
            // may still want it.
            Reference::Tag(tag) => {
                let project = self.project_or_unknown(image).await?;
                let Some(version) = self.get_project_version(project.id, tag).await? else {
                    return Err(DockerError::coded(
                        ErrorCode::ManifestUnknown,
                        format!("`{image}` has no tag `{tag}`"),
                    ));
                };
                DBProjectVersion::delete_by_id(version.id, self.site.as_ref()).await?;
            }
            Reference::Digest(digest) => {
                let removed = DBDockerManifest::delete(
                    self.0.id,
                    image,
                    &digest.to_string(),
                    self.site.as_ref(),
                )
                .await?;
                if !removed {
                    return Err(DockerError::coded(
                        ErrorCode::ManifestUnknown,
                        format!("manifest `{digest}` is not in this registry"),
                    ));
                }
                let path = Self::manifest_path(digest)?;
                self.storage.delete_file(self.0.id, &path).await?;

                // Any tag still aimed at it would resolve to bytes that are gone, and every pull
                // would 404 on a manifest the tag list still advertises.
                if let Some(project) =
                    DBProject::find_by_project_key(image, self.0.id, self.site.as_ref()).await?
                {
                    self.remove_tags_pointing_at(project.id, &digest.to_string())
                        .await?;
                }
            }
        }

        Ok(RepoResponse::Other(
            ResponseBuilder::default()
                .status(StatusCode::ACCEPTED)
                .header(CONTENT_LENGTH, "0")
                .header(
                    DOCKER_API_VERSION_HEADER.clone(),
                    DOCKER_API_VERSION_VALUE.clone(),
                )
                .empty(),
        ))
    }

    async fn remove_tags_pointing_at(
        &self,
        project: Uuid,
        digest: &str,
    ) -> Result<(), DockerError> {
        let versions = DBProjectVersion::get_all_versions(project, self.site.as_ref()).await?;
        for version in versions {
            let Ok(stored) = self.stored_tag(&version) else {
                continue;
            };
            if stored.digest == digest {
                DBProjectVersion::delete_by_id(version.id, self.site.as_ref()).await?;
            }
        }
        Ok(())
    }

    // ---- helpers -----------------------------------------------------------------------------

    async fn project_or_unknown(&self, image: &str) -> Result<DBProject, DockerError> {
        self.get_project_from_key(image).await?.ok_or_else(|| {
            DockerError::coded(
                ErrorCode::NameUnknown,
                format!("`{image}` is not in this registry"),
            )
        })
    }

    fn stored_tag(&self, version: &DBProjectVersion) -> Result<StoredTag, DockerError> {
        let Some(extra) = version.extra.0.extra.clone() else {
            return Err(DockerError::coded(
                ErrorCode::ManifestUnknown,
                format!("the tag `{}` has no recorded manifest", version.version),
            ));
        };
        serde_json::from_value(extra).map_err(|error| {
            DockerError::coded(
                ErrorCode::ManifestUnknown,
                format!(
                    "the recorded manifest for `{}` will not parse: {error}",
                    version.version
                ),
            )
        })
    }
}

/// Where the client thinks this registry lives.
///
/// A repository is reachable two ways, and the two disagree about what the image is called. On a
/// custom domain the client wrote `alpine`; in prefix mode it wrote `local/docker/alpine` and the
/// routing layer stripped the first two segments off before the handler ever saw them.
///
/// Every URL and every scope the registry hands back has to use the client's spelling. A `Location`
/// of `/v2/alpine/blobs/uploads/{id}` sent to a client that asked for `/v2/local/docker/alpine/...`
/// points at nothing, and the next `PATCH` 404s — which is exactly what happened before this
/// existed.
#[derive(Debug)]
struct Origin {
    /// The path prefix up to and including `/v2`, with no trailing slash.
    base: String,
}

impl Origin {
    /// Recovers the prefix by subtracting the path the handler was given from the path the client
    /// sent. Those differ by exactly the segments the routing layer consumed.
    fn of(parts: &Parts, path: &StoragePath) -> Self {
        // Both sides are trimmed of slashes before comparing: `StoragePath` only keeps a trailing
        // slash when the last component looks like a directory, so `/v2/x/y/blobs/uploads/` and
        // `x/y/blobs/uploads` describe the same request and must still line up.
        let full = parts.uri.path().trim_end_matches('/');
        let stripped = path.to_string();
        let stripped = stripped.trim_matches('/');

        let base = if stripped.is_empty() {
            full.to_owned()
        } else {
            match full.strip_suffix(stripped) {
                Some(prefix) => prefix.trim_end_matches('/').to_owned(),
                // Percent-encoding in the URI would make the two disagree. Falling back to the bare
                // mount point is right for a custom domain and wrong for prefix mode, but it beats
                // emitting a `Location` built from a prefix that does not match.
                None => "/v2".to_owned(),
            }
        };
        Self { base }
    }

    /// The image name as the client wrote it, prefix included.
    fn image_name(&self, image: &str) -> String {
        let prefix = self
            .base
            .strip_prefix("/v2")
            .unwrap_or_default()
            .trim_matches('/');
        if prefix.is_empty() {
            image.to_owned()
        } else {
            format!("{prefix}/{image}")
        }
    }

    fn url(&self, image: &str, suffix: &str) -> String {
        format!("{}/{image}/{suffix}", self.base)
    }
}

/// What a Docker tag carries in `project_versions.extra`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredTag {
    digest: String,
    media_type: String,
    size: i64,
    #[serde(default)]
    platforms: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct UploadQuery {
    digest: Option<String>,
    /// The digest a client wants to mount from another repository. Read only to notice the attempt
    /// and fall back to a normal upload; `from` (the source repository) is deliberately not
    /// captured, because nothing here acts on it.
    mount: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct PageQuery {
    n: Option<usize>,
    last: Option<String>,
}

/// Applies the spec's `?n=` / `?last=` paging to an already-sorted list.
fn paginate(items: Vec<String>, query: &PageQuery) -> Vec<String> {
    let size = query.n.unwrap_or(DEFAULT_PAGE_SIZE).clamp(1, MAX_PAGE_SIZE);
    let start = match &query.last {
        // `last` is exclusive: the page begins at the first entry *after* it.
        Some(last) => items.partition_point(|item| item <= last),
        None => 0,
    };
    items.into_iter().skip(start).take(size).collect()
}

/// The first byte offset a `Content-Range: {start}-{end}` header claims.
///
/// Absent on a monolithic `PATCH`, which is what Docker and BuildKit send in the common case, so a
/// missing header means "continue wherever the upload is" rather than an error.
fn content_range_start(parts: &Parts) -> Option<u64> {
    let value = parts.headers.get(CONTENT_RANGE)?.to_str().ok()?;
    let (start, _) = value.trim().split_once('-')?;
    start.trim().parse().ok()
}

async fn read_all(content: nr_storage::StorageFileReader) -> Result<Vec<u8>, DockerError> {
    use tokio::io::AsyncReadExt;
    let mut reader = content;
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).await?;
    Ok(bytes)
}

impl RepositoryExt for DockerHostedRegistry {}

impl Repository for DockerHostedRegistry {
    type Error = DockerError;

    fn get_storage(&self) -> DynStorage {
        self.0.storage.clone()
    }

    fn site(&self) -> SiteContext {
        self.0.site.clone()
    }

    fn get_type(&self) -> &'static str {
        "docker"
    }

    fn full_type(&self) -> &'static str {
        FULL_TYPE
    }

    fn config_types(&self) -> Vec<&str> {
        vec![
            DockerRegistryConfigType::get_type_static(),
            RepositoryPageType::get_type_static(),
            ProjectConfigType::get_type_static(),
        ]
    }

    fn name(&self) -> String {
        self.0.name.clone()
    }

    fn id(&self) -> Uuid {
        self.0.id
    }

    fn visibility(&self) -> Visibility {
        *self.0.visibility.read()
    }

    fn is_active(&self) -> bool {
        self.0.active.load(atomic::Ordering::Relaxed)
    }

    #[instrument(fields(repository_type = "docker/hosted"))]
    async fn reload(&self) -> Result<(), RepositoryFactoryError> {
        let Some(repository) = DBRepository::get_by_id(self.0.id, self.site.as_ref()).await? else {
            warn!("Repository no longer exists");
            self.0.active.store(false, atomic::Ordering::Relaxed);
            return Ok(());
        };
        self.0
            .active
            .store(repository.active, atomic::Ordering::Relaxed);
        *self.0.visibility.write() = repository.visibility;
        Ok(())
    }

    async fn handle_get(&self, request: RepositoryRequest) -> Result<RepoResponse, DockerError> {
        self.handle_read(request, false).await
    }

    /// `HEAD` is how a client checks whether a blob or manifest is already here before pushing it,
    /// so it is answered with the same headers as the `GET` and an empty body. The default trait
    /// implementation returns 405, which would make every push re-upload every layer.
    async fn handle_head(&self, request: RepositoryRequest) -> Result<RepoResponse, DockerError> {
        self.handle_read(request, true).await
    }

    async fn handle_post(&self, request: RepositoryRequest) -> Result<RepoResponse, DockerError> {
        match DockerPath::parse(&request.path)? {
            DockerPath::UploadStart { name } => self.start_upload(request, &name).await,
            other => {
                debug!(?other, "Route is not a POST");
                Err(DockerError::coded(
                    ErrorCode::Unsupported,
                    "this route does not accept POST",
                ))
            }
        }
    }

    async fn handle_patch(&self, request: RepositoryRequest) -> Result<RepoResponse, DockerError> {
        match DockerPath::parse(&request.path)? {
            DockerPath::UploadChunk { name, uuid } => {
                self.append_upload(request, &name, uuid).await
            }
            other => {
                debug!(?other, "Route is not a PATCH");
                Err(DockerError::coded(
                    ErrorCode::Unsupported,
                    "this route does not accept PATCH",
                ))
            }
        }
    }

    async fn handle_put(&self, request: RepositoryRequest) -> Result<RepoResponse, DockerError> {
        match DockerPath::parse(&request.path)? {
            DockerPath::UploadChunk { name, uuid } => {
                self.finish_upload(request, &name, uuid).await
            }
            DockerPath::Manifest { name, reference } => {
                self.put_manifest(request, &name, reference).await
            }
            other => {
                debug!(?other, "Route is not a PUT");
                Err(DockerError::coded(
                    ErrorCode::Unsupported,
                    "this route does not accept PUT",
                ))
            }
        }
    }

    async fn handle_delete(&self, request: RepositoryRequest) -> Result<RepoResponse, DockerError> {
        let origin = Origin::of(&request.parts, &request.path);
        match DockerPath::parse(&request.path)? {
            DockerPath::Manifest { name, reference } => {
                self.delete_manifest(
                    &request.authentication,
                    &request.parts,
                    &origin,
                    &name,
                    &reference,
                )
                .await
            }
            DockerPath::UploadChunk { name, uuid } => {
                self.require_write(
                    &request.authentication,
                    &request.parts,
                    &origin.image_name(&name),
                )
                .await?;
                self.uploads
                    .cancel(uuid, self.0.id)
                    .await
                    .map_err(DockerError::Upload)?;
                Ok(RepoResponse::Other(
                    ResponseBuilder::default()
                        .status(StatusCode::NO_CONTENT)
                        .empty(),
                ))
            }
            // Deleting a blob would corrupt every other manifest that shares it — layers are
            // content-addressed and nothing reference-counts them. Refused rather than silently
            // breaking images that were fine a moment ago.
            DockerPath::Blob { .. } => Err(DockerError::coded(
                ErrorCode::Unsupported,
                "this registry does not support deleting blobs; delete the manifest instead",
            )),
            other => {
                debug!(?other, "Route is not a DELETE");
                Err(DockerError::coded(
                    ErrorCode::Unsupported,
                    "this route does not accept DELETE",
                ))
            }
        }
    }
}

impl DockerHostedRegistry {
    /// `GET` and `HEAD` differ only in whether a body is written, so they share this.
    async fn handle_read(
        &self,
        request: RepositoryRequest,
        head_only: bool,
    ) -> Result<RepoResponse, DockerError> {
        let route = DockerPath::parse(&request.path)?;
        let origin = Origin::of(&request.parts, &request.path);

        // The image, for the `scope` on a challenge, spelled the way the client wrote it. `/v2/`
        // and `_catalog` name none.
        let scope = match &route {
            DockerPath::Manifest { name, .. }
            | DockerPath::Blob { name, .. }
            | DockerPath::TagsList { name }
            | DockerPath::Referrers { name, .. }
            | DockerPath::UploadStart { name }
            | DockerPath::UploadChunk { name, .. } => Some(origin.image_name(name)),
            DockerPath::Base | DockerPath::Catalog => None,
        };
        if let Some(denied) = self
            .check_read(&request.authentication, &request.parts, scope)
            .await?
        {
            return Err(denied);
        }

        match route {
            // The version check `docker login` and every client start with.
            DockerPath::Base => Ok(RepoResponse::Other(
                ResponseBuilder::ok()
                    .header(
                        DOCKER_API_VERSION_HEADER.clone(),
                        DOCKER_API_VERSION_VALUE.clone(),
                    )
                    .json(&serde_json::json!({})),
            )),
            DockerPath::Catalog => self.catalog(&origin, &request.parts).await,
            DockerPath::TagsList { name } => self.tags_list(&origin, &name, &request.parts).await,
            DockerPath::Manifest { name, reference } => {
                self.read_manifest(&name, &reference, head_only).await
            }
            DockerPath::Blob { name, digest } => self.read_blob(&name, &digest, head_only).await,
            DockerPath::Referrers { name, digest } => self.referrers(&name, &digest).await,
            // A status request on an in-progress upload.
            DockerPath::UploadChunk { name, uuid } => {
                self.require_write(
                    &request.authentication,
                    &request.parts,
                    &origin.image_name(&name),
                )
                .await?;
                let offset = self
                    .uploads
                    .offset(uuid, self.0.id)
                    .map_err(DockerError::Upload)?;
                Ok(RepoResponse::Other(
                    ResponseBuilder::default()
                        .status(StatusCode::NO_CONTENT)
                        .header(
                            LOCATION,
                            origin.url(&name, &format!("blobs/uploads/{uuid}")),
                        )
                        .header(RANGE, format!("0-{}", offset.saturating_sub(1)))
                        .header(DOCKER_UPLOAD_UUID_HEADER.clone(), uuid.to_string())
                        .empty(),
                ))
            }
            DockerPath::UploadStart { .. } => Err(DockerError::coded(
                ErrorCode::Unsupported,
                "an upload is opened with POST",
            )),
        }
    }
}

/// Present so the unused import of [`HeaderValue`] is not silently load-bearing elsewhere.
const _: Option<HeaderValue> = None;

#[cfg(test)]
mod tests {
    use http::{HeaderMap, HeaderValue, Request, header::CONTENT_RANGE};

    use super::{PageQuery, content_range_start, paginate};

    fn page(n: Option<usize>, last: Option<&str>) -> PageQuery {
        PageQuery {
            n,
            last: last.map(ToOwned::to_owned),
        }
    }

    fn items() -> Vec<String> {
        ["a", "b", "c", "d", "e"]
            .into_iter()
            .map(ToOwned::to_owned)
            .collect()
    }

    #[test]
    fn paging_defaults_to_everything_up_to_the_page_size() {
        assert_eq!(paginate(items(), &page(None, None)).len(), 5);
        assert_eq!(paginate(items(), &page(Some(2), None)), vec!["a", "b"]);
    }

    /// `last` is exclusive — a page starting *at* it would repeat the entry the client already has.
    #[test]
    fn last_starts_the_page_after_it() {
        assert_eq!(paginate(items(), &page(Some(2), Some("b"))), vec!["c", "d"]);
        assert_eq!(
            paginate(items(), &page(None, Some("e"))),
            Vec::<String>::new()
        );
        // A `last` that is not in the list still positions correctly.
        assert_eq!(paginate(items(), &page(Some(1), Some("bb"))), vec!["c"]);
    }

    #[test]
    fn a_page_size_is_clamped_rather_than_trusted() {
        assert_eq!(paginate(items(), &page(Some(0), None)).len(), 1);
        assert_eq!(paginate(items(), &page(Some(usize::MAX), None)).len(), 5);
    }

    #[test]
    fn a_content_range_start_is_read_when_present() {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_RANGE, HeaderValue::from_static("0-1023"));
        let parts = parts_with(headers);
        assert_eq!(content_range_start(&parts), Some(0));

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_RANGE, HeaderValue::from_static("1024-2047"));
        let parts = parts_with(headers);
        assert_eq!(content_range_start(&parts), Some(1024));
    }

    /// Docker and BuildKit send one unranged `PATCH` in the common case, so a missing header must
    /// mean "continue where the upload is" rather than an error.
    #[test]
    fn a_missing_or_unparseable_content_range_is_treated_as_absent() {
        assert_eq!(content_range_start(&parts_with(HeaderMap::new())), None);

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_RANGE, HeaderValue::from_static("nonsense"));
        assert_eq!(content_range_start(&parts_with(headers)), None);
    }

    fn parts_with(headers: HeaderMap) -> http::request::Parts {
        let mut request = Request::builder().uri("/").body(()).unwrap();
        *request.headers_mut() = headers;
        request.into_parts().0
    }
}
