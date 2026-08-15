use std::sync::{
    Arc,
    atomic::{self, AtomicBool},
};

use derive_more::derive::Deref;
use http::StatusCode;
use maven_rs::pom::Pom;
use nr_core::{
    database::entities::{
        project::{
            DBProject, ProjectDBType, info::ProjectInfo, members::DBProjectMember,
            versions::DBProjectVersion,
        },
        repository::DBRepository,
    },
    repository::{
        Visibility,
        config::{
            RepositoryConfigType, get_repository_config_or_default,
            project::{ProjectConfig, ProjectConfigType},
            repository_page::RepositoryPageType,
        },
        project::ProjectResolution,
    },
    storage::StoragePath,
    user::permissions::{HasPermissions, RepositoryActions},
};
use nr_storage::{DynStorage, FileType, Storage, StorageFile};
use parking_lot::RwLock;
use tracing::{debug, error, event, info, instrument};
use uuid::Uuid;

use super::{
    MavenError, REPOSITORY_TYPE_ID, RepoResponse, RepositoryRequest, checksum,
    configs::MavenPushRules, metadata, push_rules, utils::MavenRepositoryExt,
};
use crate::{
    app::SiteContext,
    repository::{
        Repository, RepositoryFactoryError,
        maven::{MavenRepositoryConfigType, configs::MavenPushRulesConfigType},
        utils::RepositoryExt,
    },
    utils::ResponseBuilder,
};

/// The value `Repository::full_type()` reports, and what lands in the repository request
/// span and metric attributes. Pinned by `repository_type_ids_are_stable`.
pub static FULL_TYPE: &str = "maven/hosted";

#[derive(derive_more::Debug)]
pub struct MavenHostedInner {
    pub id: Uuid,
    pub name: String,
    pub active: AtomicBool,
    pub visibility: RwLock<Visibility>,
    pub push_rules: RwLock<MavenPushRules>,
    pub project: RwLock<ProjectConfig>,
    #[debug(skip)]
    pub storage: DynStorage,
    #[debug(skip)]
    pub site: SiteContext,
}
impl MavenHostedInner {}
#[derive(Debug, Clone, Deref)]
pub struct MavenHosted(Arc<MavenHostedInner>);
impl MavenRepositoryExt for MavenHosted {}
impl RepositoryExt for MavenHosted {}
impl MavenHosted {
    #[instrument(skip(self))]
    pub async fn standard_maven_deploy(
        &self,
        RepositoryRequest {
            parts,
            body,
            path,
            authentication,
            trace,
            ..
        }: RepositoryRequest,
    ) -> Result<RepoResponse, MavenError> {
        let user_id = if let Some(user) = authentication.get_user() {
            user.id
        } else {
            return Ok(RepoResponse::unauthorized());
        };

        if let Some(rejection) = self.check_push_rules(&path, user_id).await? {
            info!(?rejection, %path, "Refused a deploy");
            return Ok(RepoResponse::basic_text_response(
                StatusCode::FORBIDDEN,
                rejection.to_string(),
            ));
        }

        let parent_path = path.clone().parent();
        if let Some(meta) = self
            .storage
            .get_repository_meta(self.id, &parent_path)
            .await?
        {
            let project_info = if let Some(version_id) = meta.project_version_id {
                ProjectInfo::query_from_version_id(version_id, self.site.as_ref()).await?
            } else if let Some(project_id) = meta.project_id {
                ProjectInfo::query_from_project_id(project_id, self.site.as_ref()).await?
            } else {
                None
            };
            if let Some(project) = project_info {
                trace.set_project(
                    project.project_scope,
                    project.project_name,
                    project.project_key,
                    project.project_version,
                );
            }
        };
        info!("Saving File: {}", path);

        let body = body.body_as_bytes().await?;
        trace.metrics.project_write_bytes(body.len() as u64);

        // A checksum upload is checked against the artifact it describes rather than stored as an
        // opaque blob. Maven uploads the artifact first, so it is already here to compare against.
        if let Some((artifact_path, kind)) = checksum::split_checksum_path(&path)
            && let Some(rejection) = self
                .verify_uploaded_checksum(&artifact_path, kind, &body)
                .await?
        {
            return Ok(rejection);
        }

        let pom = if path.has_extension("pom") {
            let pom: Pom = self.parse_pom(body.to_vec())?;
            // Checked once the POM is parsed, because the coordinates it declares are what the
            // path has to agree with.
            if let Some(rejection) = push_rules::check_pom_matches_path(&pom, &path) {
                info!(?rejection, %path, "Refused a deploy");
                return Ok(RepoResponse::basic_text_response(
                    StatusCode::BAD_REQUEST,
                    rejection.to_string(),
                ));
            }
            Some(pom)
        } else {
            None
        };
        let (_size, created) = self.storage.save_file(self.id, body.into(), &path).await?;
        // Trigger Push Event if it is the .pom file
        let save_path = format!(
            "/repositories/{}/{}/{}",
            self.storage.storage_config().storage_config.storage_name,
            self.name,
            path
        );
        if let Some(pom) = pom {
            debug!(?pom, "Parsed POM File");
            // A failure here used to be logged and swallowed, so a deploy whose project or version
            // row could not be written still answered `201` and the artifact became invisible to
            // browse, badges and search with nothing to say why.
            if let Err(error) = self
                .post_pom_upload_inner(path.clone(), Some(user_id), pom)
                .await
            {
                error!(?error, %path, "Failed to register the deployed POM");
                return Err(error);
            }
        };
        Ok(RepoResponse::put_response(created, save_path))
    }

    /// Applies the repository's push rules to a deploy.
    async fn check_push_rules(
        &self,
        path: &StoragePath,
        user_id: i32,
    ) -> Result<Option<push_rules::PushRejection>, MavenError> {
        let (push_policy, allow_overwrite, must_be_project_member) = {
            let rules = self.push_rules.read();
            (
                rules.push_policy.clone(),
                rules.allow_overwrite,
                rules.must_be_project_member,
            )
        };
        let require_semver = self.project.read().require_semver;
        let version = push_rules::version_directory_of(path);

        if let Some(rejection) = push_rules::check_policy(push_policy, version.as_deref()) {
            return Ok(Some(rejection));
        }
        if let Some(version) = version.as_deref()
            && let Some(rejection) = push_rules::check_semver(require_semver, version)
        {
            return Ok(Some(rejection));
        }
        if !allow_overwrite
            && !push_rules::is_rewritable(path)
            && self.storage.file_exists(self.id, path).await?
        {
            return Ok(Some(push_rules::PushRejection::OverwriteNotAllowed(
                path.to_string(),
            )));
        }
        if must_be_project_member {
            // Only meaningful once the project exists — the first push is what creates it, and
            // that pusher is made an owner.
            let project_directory = path.clone().parent().parent();
            if let Some(project) = DBProject::find_by_project_directory(
                &project_directory.to_string(),
                self.id,
                self.site.as_ref(),
            )
            .await?
                && !DBProjectMember::can_write(project.id, user_id, self.site.as_ref()).await?
            {
                return Ok(Some(push_rules::PushRejection::NotAProjectMember));
            }
        }
        Ok(None)
    }

    /// Compares an uploaded checksum against the artifact it names.
    async fn verify_uploaded_checksum(
        &self,
        artifact_path: &StoragePath,
        kind: checksum::ChecksumKind,
        body: &[u8],
    ) -> Result<Option<RepoResponse>, MavenError> {
        let Ok(text) = std::str::from_utf8(body) else {
            return Ok(Some(RepoResponse::basic_text_response(
                StatusCode::BAD_REQUEST,
                "Checksum file is not valid UTF-8",
            )));
        };
        let Some(expected) = checksum::parse_checksum_body(text) else {
            return Ok(Some(RepoResponse::basic_text_response(
                StatusCode::BAD_REQUEST,
                "Checksum file does not contain a hex digest",
            )));
        };
        let Some(actual) = self.checksum_for(artifact_path, kind).await? else {
            // The artifact is not here yet. Maven uploads it first, so this normally means the
            // client is uploading a checksum for something it never sent — but refusing would
            // break any client that reorders them, so it is stored and left alone.
            debug!(%artifact_path, "Checksum uploaded before its artifact");
            return Ok(None);
        };
        if actual != expected {
            info!(
                %artifact_path,
                ?kind,
                "Uploaded checksum does not match the stored artifact"
            );
            return Ok(Some(RepoResponse::basic_text_response(
                StatusCode::BAD_REQUEST,
                format!(
                    "The uploaded {} does not match the artifact: expected {actual}, got {expected}",
                    kind.extension()
                ),
            )));
        }
        Ok(None)
    }

    /// Answers a request for `maven-metadata.xml`, or one of its checksums.
    ///
    /// `None` means the path is not a metadata request, or there is nothing to describe — the
    /// caller falls through to storage, which is what serves a metadata file for an artifact this
    /// repository does not know about.
    #[instrument(skip(self))]
    async fn serve_metadata(&self, path: &StoragePath) -> Result<Option<RepoResponse>, MavenError> {
        let Some((request, suffix)) = metadata::MetadataRequest::parse(path) else {
            return Ok(None);
        };
        let document = match request {
            metadata::MetadataRequest::Artifact(directory) => {
                let Some(project) = DBProject::find_by_project_directory(
                    &directory.to_string(),
                    self.id,
                    self.site.as_ref(),
                )
                .await?
                else {
                    return Ok(None);
                };
                let versions =
                    DBProjectVersion::get_all_versions(project.id, self.site.as_ref()).await?;
                if versions.is_empty() {
                    return Ok(None);
                }
                metadata::to_xml(&metadata::artifact_metadata(&project, &versions)?)?
            }
            metadata::MetadataRequest::Snapshot(directory) => {
                let Some(document) = self.snapshot_metadata_for(&directory).await? else {
                    return Ok(None);
                };
                document
            }
        };

        // The checksums are derived from the same bytes that are about to be served, so they
        // cannot disagree with the document the way a stored pair can.
        let body = match suffix {
            "" => document,
            other => {
                let Some(kind) =
                    checksum::ChecksumKind::from_extension(other.trim_start_matches('.'))
                else {
                    return Ok(None);
                };
                kind.compute(document.as_bytes())
            }
        };
        let content_type = if suffix.is_empty() {
            "application/xml"
        } else {
            "text/plain"
        };
        Ok(Some(RepoResponse::Other(
            ResponseBuilder::ok()
                .header(http::header::CONTENT_TYPE, content_type)
                .body(body),
        )))
    }

    /// Builds the snapshot document for a version directory by reading what is in it.
    ///
    /// The individual timestamped builds are files, not rows — nothing in the database knows about
    /// build number 3 of `1.0.0-SNAPSHOT` — so the directory listing is the only source.
    async fn snapshot_metadata_for(
        &self,
        directory: &StoragePath,
    ) -> Result<Option<String>, MavenError> {
        let Some(project) = DBProject::find_by_project_directory(
            &directory.clone().parent().to_string(),
            self.id,
            self.site.as_ref(),
        )
        .await?
        else {
            return Ok(None);
        };
        let Some((group_id, artifact_id)) = project.key.split_once(':') else {
            return Ok(None);
        };
        let Some(version) = directory.clone().into_iter().next_back() else {
            return Ok(None);
        };
        let version = version.as_ref().to_owned();
        let base_version = version
            .strip_suffix("-SNAPSHOT")
            .or_else(|| version.strip_suffix("-snapshot"))
            .unwrap_or(&version)
            .to_owned();

        let Some(StorageFile::Directory { files, .. }) =
            self.storage.open_file(self.id, directory).await?
        else {
            return Ok(None);
        };
        let builds: Vec<_> = files
            .iter()
            .filter_map(|file| {
                metadata::parse_snapshot_file(&file.name, artifact_id, &base_version)
            })
            .collect();

        let Some(document) = metadata::snapshot_metadata(
            group_id.to_owned(),
            artifact_id.to_owned(),
            version,
            &builds,
        ) else {
            return Ok(None);
        };
        Ok(Some(metadata::to_xml(&document)?))
    }

    /// Generates a checksum file for an artifact that has one computed but never stored.
    ///
    /// A `GET` for `foo.jar.sha1` was a 404 unless a client had uploaded one, even though the
    /// storage layer had been computing the digest all along.
    async fn serve_generated_checksum(
        &self,
        path: &StoragePath,
    ) -> Result<Option<RepoResponse>, MavenError> {
        let Some((artifact_path, kind)) = checksum::split_checksum_path(path) else {
            return Ok(None);
        };
        let Some(value) = self.checksum_for(&artifact_path, kind).await? else {
            return Ok(None);
        };
        Ok(Some(RepoResponse::Other(
            ResponseBuilder::ok()
                .header(http::header::CONTENT_TYPE, "text/plain")
                .body(value),
        )))
    }

    /// The checksum of a stored artifact, preferring what the storage layer already recorded.
    async fn checksum_for(
        &self,
        artifact_path: &StoragePath,
        kind: checksum::ChecksumKind,
    ) -> Result<Option<String>, MavenError> {
        let Some(meta) = self
            .storage
            .get_file_information(self.id, artifact_path)
            .await?
        else {
            return Ok(None);
        };
        let FileType::File(file) = &meta.file_type else {
            return Ok(None);
        };
        if let Some(stored) = kind.from_stored(&file.file_hash) {
            return Ok(Some(stored));
        }
        // sha512 is not among the hashes the storage layer keeps, so it costs a read.
        let Some(StorageFile::File { content, .. }) =
            self.storage.open_file(self.id, artifact_path).await?
        else {
            return Ok(None);
        };
        let bytes = content.read_to_vec(file.file_size as usize).await?;
        Ok(Some(kind.compute(&bytes)))
    }
    pub async fn load(
        repository: DBRepository,
        storage: DynStorage,
        site: SiteContext,
    ) -> Result<Self, RepositoryFactoryError> {
        let push_rules_db = get_repository_config_or_default::<
            MavenPushRulesConfigType,
            MavenPushRules,
        >(repository.id, site.as_ref())
        .await?;
        debug!("Loaded Push Rules Config: {:?}", push_rules_db);

        let project_db = get_repository_config_or_default::<ProjectConfigType, ProjectConfig>(
            repository.id,
            site.as_ref(),
        )
        .await?;
        let active = AtomicBool::new(repository.active);
        debug!("Loaded Frontend Config: {:?}", project_db);
        let inner = MavenHostedInner {
            id: repository.id,
            name: repository.name.into(),
            active,
            visibility: RwLock::new(repository.visibility),
            push_rules: RwLock::new(push_rules_db.value.0),
            project: RwLock::new(project_db.value.0),
            storage,
            site,
        };
        Ok(Self(Arc::new(inner)))
    }
}
impl Repository for MavenHosted {
    type Error = MavenError;
    #[inline(always)]
    fn site(&self) -> SiteContext {
        self.0.site.clone()
    }
    #[inline(always)]
    fn get_storage(&self) -> nr_storage::DynStorage {
        self.0.storage.clone()
    }
    #[inline(always)]
    fn visibility(&self) -> Visibility {
        *self.visibility.read()
    }
    #[inline(always)]
    fn get_type(&self) -> &'static str {
        REPOSITORY_TYPE_ID
    }
    fn full_type(&self) -> &'static str {
        FULL_TYPE
    }
    #[inline(always)]
    fn name(&self) -> String {
        self.0.name.clone()
    }
    #[inline(always)]
    fn id(&self) -> Uuid {
        self.0.id
    }
    #[inline(always)]
    fn is_active(&self) -> bool {
        self.active.load(atomic::Ordering::Relaxed)
    }

    fn config_types(&self) -> Vec<&str> {
        vec![
            RepositoryPageType::get_type_static(),
            MavenPushRulesConfigType::get_type_static(),
            ProjectConfigType::get_type_static(),
            MavenRepositoryConfigType::get_type_static(),
        ]
    }
    #[instrument(fields(repository_type = "maven/hosted"))]
    async fn reload(&self) -> Result<(), RepositoryFactoryError> {
        // The whole row, not just `active`: this read `get_active_by_id` and never refreshed the
        // cached `visibility`, so making a repository private took effect in the database and in
        // `/api/repository/list` while the running repository carried on serving its files to
        // anyone — until a restart. The artifact bytes are the most sensitive thing here, so this
        // was the worst of the visibility leaks.
        let Some(repository) = DBRepository::get_by_id(self.id, self.site.as_ref()).await? else {
            error!("Failed to get repository");
            self.0.active.store(false, atomic::Ordering::Relaxed);
            return Ok(());
        };
        self.0
            .active
            .store(repository.active, atomic::Ordering::Relaxed);
        *self.0.visibility.write() = repository.visibility;

        let push_rules_db = get_repository_config_or_default::<
            MavenPushRulesConfigType,
            MavenPushRules,
        >(self.id, self.site.as_ref())
        .await?;

        let project_config_db =
            get_repository_config_or_default::<ProjectConfigType, ProjectConfig>(
                self.id,
                self.site.as_ref(),
            )
            .await?;

        {
            let mut push_rules = self.push_rules.write();
            *push_rules = push_rules_db.value.0;
        }

        {
            let mut project_config = self.project.write();
            *project_config = project_config_db.value.0;
        }

        Ok(())
    }
    async fn handle_get(
        &self,
        RepositoryRequest {
            parts,
            path,
            authentication,
            trace,
            ..
        }: RepositoryRequest,
    ) -> Result<RepoResponse, MavenError> {
        if let Some(err) = self.check_read(&authentication).await? {
            return Ok(err);
        }
        // Generated ahead of storage: a `maven-metadata.xml` uploaded by a client is a snapshot of
        // what that one client knew, and serving it back would hide versions deployed from
        // anywhere else.
        if let Some(response) = self.serve_metadata(&path).await? {
            return Ok(response);
        }
        let file = self.0.storage.open_file(self.id, &path).await?;
        // A checksum that was never uploaded is generated from the artifact rather than 404d.
        if file.is_none()
            && let Some(response) = self.serve_generated_checksum(&path).await?
        {
            return Ok(response);
        }
        if let Some(StorageFile::File { meta, .. }) = &file {
            trace.metrics.project_access_bytes(meta.file_type.file_size);
            let parent = path.parent();
            let meta = self
                .0
                .storage
                .get_repository_meta(self.id, &parent)
                .await?
                .unwrap_or_default();
            let project_info = if let Some(version_id) = meta.project_version_id {
                ProjectInfo::query_from_version_id(version_id, self.site.as_ref()).await?
            } else if let Some(project_id) = meta.project_id {
                ProjectInfo::query_from_project_id(project_id, self.site.as_ref()).await?
            } else {
                None
            };
            if let Some(project) = project_info {
                trace.set_project(
                    project.project_scope,
                    project.project_name,
                    project.project_key,
                    project.project_version,
                );
            }
        }
        return self.indexing_check_option(file, &authentication).await;
    }
    async fn handle_head(
        &self,
        RepositoryRequest {
            parts,
            path,
            authentication,
            ..
        }: RepositoryRequest,
    ) -> Result<RepoResponse, MavenError> {
        let visibility = self.visibility();
        if let Some(err) = self.check_read(&authentication).await? {
            return Ok(err);
        }
        let file = self.storage.get_file_information(self.id, &path).await?;
        return self.indexing_check_option(file, &authentication).await;
    }
    async fn handle_put(&self, request: RepositoryRequest) -> Result<RepoResponse, MavenError> {
        info!("Handling PUT Request for Repository: {}", self.id);
        {
            let push_rules = self.push_rules.read();
            if push_rules.must_use_auth_token_for_push && !request.authentication.has_auth_token() {
                info!("Repository requires an auth token for push");
                return Ok(RepoResponse::require_auth_token());
            }
        }

        let Some(user) = request
            .authentication
            .get_user_if_has_action(RepositoryActions::Write, self.id, self.site.as_ref())
            .await?
        else {
            info!("No acceptable user authentication provided");
            return Ok(RepoResponse::unauthorized());
        };
        if !user
            .has_action(RepositoryActions::Write, self.id, self.site.as_ref())
            .await?
        {
            info!(?self.id, ?user, "User does not have write permissions");
            return Ok(RepoResponse::forbidden());
        }

        self.standard_maven_deploy(request).await
    }
    #[instrument(fields(repository_type = "maven/hosted"))]
    async fn resolve_project_and_version_for_path(
        &self,
        path: &StoragePath,
    ) -> Result<ProjectResolution, MavenError> {
        let path_as_string = path.to_string();
        event!(
            tracing::Level::DEBUG,
            "Resolving Project and Version for Path: {}",
            path_as_string
        );
        let Some(meta) = self.storage.get_repository_meta(self.id, path).await? else {
            return Ok(ProjectResolution::default());
        };
        if let Some(project_id) = meta.project_id {
            let version_id = meta.project_version_id;
            event!(
                tracing::Level::DEBUG,
                ?project_id,
                ?version_id,
                "Found Project ID in Meta"
            );

            return Ok(ProjectResolution {
                project_id: Some(project_id),
                version_id,
            });
        }
        event!(
            tracing::Level::DEBUG,
            "No Project ID in Meta looking project dirs in DB"
        );
        let version =
            DBProjectVersion::find_ids_by_version_dir(&path_as_string, self.id, self.site.as_ref())
                .await?;
        if let Some(version) = version {
            event!(
                tracing::Level::DEBUG,
                "Found Project Version in DB Versions: {:?}",
                version
            );
            return Ok(version.into());
        }
        event!(
            tracing::Level::DEBUG,
            "No Project Version in DB looking for Project dir"
        );
        if let Some(project) =
            DBProject::find_by_project_directory(&path_as_string, self.id, self.site.as_ref())
                .await?
        {
            return Ok(ProjectResolution {
                project_id: Some(project.id),
                version_id: None,
            });
        }

        Ok(ProjectResolution::default())
    }
}
