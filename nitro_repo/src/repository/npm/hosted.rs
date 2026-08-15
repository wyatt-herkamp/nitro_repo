use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{self, AtomicBool},
    },
};

use ahash::{HashMap, HashMapExt};
use axum::response::{IntoResponse, Response};
use derive_more::derive::Deref;
use http::{StatusCode, header::CONTENT_TYPE};
use nr_core::{
    database::entities::{
        project::{
            DBProject, ProjectDBType,
            dist_tags::{DBNpmDistTag, LATEST_TAG},
            versions::{DBProjectVersion, UpdateProjectVersion},
        },
        repository::DBRepository,
    },
    repository::{
        Visibility,
        config::{
            RepositoryConfigType, project::ProjectConfigType, repository_page::RepositoryPageType,
        },
        project::ProjectResolution,
    },
    storage::StoragePath,
    user::permissions::{HasPermissions, RepositoryActions},
};
use nr_storage::{DynStorage, FileContent, Storage};
use parking_lot::RwLock;
use serde::Deserialize;
use serde_json::Value;
use tracing::{debug, info, instrument, warn};

use super::{
    types::{
        NPM_COMMAND_HEADER, NpmRegistryPackageResponse,
        request::{GetPath, InvalidNPMCommand, NPMCommand, PublishVersion},
    },
    utils::{NpmRegistryExt, npm_time},
};
use crate::{
    app::NitroRepo,
    repository::{
        RepoResponse, Repository, RepositoryAuthentication, RepositoryFactoryError,
        RepositoryRequest,
        npm::{
            NPMRegistryConfigType, NPMRegistryError,
            integrity::{verify_integrity, verify_shasum},
            login::web_login,
            search,
            types::PublishRequest,
        },
        utils::RepositoryExt,
    },
    utils::ResponseBuilder,
};

/// The value `Repository::full_type()` reports, and what lands in the repository request
/// span and metric attributes. Pinned by `repository_type_ids_are_stable`.
pub static FULL_TYPE: &str = "npm/hosted";

#[derive(derive_more::Debug)]
pub struct NpmRegistryInner {
    #[debug(skip)]
    pub site: NitroRepo,
    pub storage: DynStorage,
    pub id: uuid::Uuid,
    pub name: String,
    /// Read from the database rather than assumed. `visibility()` returned `Public` and
    /// `is_active()` returned `true` unconditionally, so a private or disabled npm registry served
    /// every packument and tarball in it to anyone who asked.
    pub visibility: RwLock<Visibility>,
    pub active: AtomicBool,
}
#[derive(Debug, Clone, Deref)]
pub struct NPMHostedRegistry(Arc<NpmRegistryInner>);
impl NPMHostedRegistry {
    pub async fn load(
        site: NitroRepo,
        storage: DynStorage,
        repository: DBRepository,
    ) -> Result<Self, RepositoryFactoryError> {
        Ok(Self(Arc::new(NpmRegistryInner {
            site,
            storage,
            id: repository.id,
            name: repository.name.to_string(),
            visibility: RwLock::new(repository.visibility),
            active: AtomicBool::new(repository.active),
        })))
    }

    /// Refuses a read the caller is not allowed to make.
    ///
    /// Unauthenticated callers get a `WWW-Authenticate` challenge rather than a flat denial, so
    /// npm knows to send credentials instead of giving up.
    async fn check_read(
        &self,
        authentication: &RepositoryAuthentication,
    ) -> Result<Option<RepoResponse>, NPMRegistryError> {
        if self.visibility().is_public() {
            return Ok(None);
        }
        if authentication.is_no_identification() {
            return Ok(Some(RepoResponse::www_authenticate("Basic")));
        }
        if !authentication
            .has_action(RepositoryActions::Read, self.id, self.site.as_ref())
            .await?
        {
            return Ok(Some(RepoResponse::forbidden()));
        }
        Ok(None)
    }

    async fn project_or_not_found(&self, name: &str) -> Result<DBProject, NPMRegistryError> {
        self.get_project_from_key(name)
            .await?
            .ok_or_else(|| NPMRegistryError::NotFound {
                message: format!("Project {name} not found in repository"),
            })
    }

    async fn version_or_not_found(
        &self,
        project: &DBProject,
        version: &str,
    ) -> Result<DBProjectVersion, NPMRegistryError> {
        self.get_project_version(project.id, version)
            .await?
            .ok_or_else(|| NPMRegistryError::NotFound {
                message: format!("Version {version} not found in project {}", project.key),
            })
    }

    /// Every dist-tag for a project.
    ///
    /// A project published before dist-tags were storable has no rows, and npm cannot install a
    /// package whose `latest` is missing — so the newest version stands in.
    async fn dist_tags_for(
        &self,
        project: &DBProject,
        versions: &[DBProjectVersion],
    ) -> Result<BTreeMap<String, String>, NPMRegistryError> {
        let mut map: BTreeMap<String, String> =
            DBNpmDistTag::get_all_for_project(project.id, self.site.as_ref())
                .await?
                .into_iter()
                .map(|tag| (tag.tag, tag.version))
                .collect();
        if !map.contains_key(LATEST_TAG)
            && let Some(newest) = versions.first()
        {
            map.insert(LATEST_TAG.to_owned(), newest.version.clone());
        }
        Ok(map)
    }

    /// Builds the packument — what `GET /{package}` returns.
    async fn packument(
        &self,
        project: &DBProject,
    ) -> Result<NpmRegistryPackageResponse, NPMRegistryError> {
        let versions = DBProjectVersion::get_all_versions(project.id, self.site.as_ref()).await?;
        let dist_tags = self.dist_tags_for(project, &versions).await?;

        let mut times = HashMap::new();
        times.insert(
            "created".to_owned(),
            npm_time::format_date_time(&project.created_at),
        );
        times.insert(
            "modified".to_owned(),
            npm_time::format_date_time(&project.updated_at),
        );

        let mut versions_map = HashMap::new();
        for version in &versions {
            times.insert(
                version.version.clone(),
                npm_time::format_date_time(&version.created_at),
            );
            let Some(extra) = version.extra.0.extra.clone() else {
                warn!(version = %version.version, "Version has no stored npm metadata");
                continue;
            };
            match serde_json::from_value::<PublishVersion>(extra) {
                Ok(published) => {
                    versions_map.insert(version.version.clone(), published);
                }
                Err(error) => {
                    // The error used to be discarded, so a packument could quietly lose versions
                    // with nothing in the log explaining why.
                    warn!(?error, version = %version.version, "Stored npm metadata will not parse");
                }
            }
        }

        // npm takes the fields describing the package as a whole from its newest version.
        let newest = dist_tags
            .get(LATEST_TAG)
            .and_then(|version| versions_map.get(version));
        let field = |key: &str| newest.and_then(|version| version.extra_field(key)).cloned();

        Ok(NpmRegistryPackageResponse {
            id: project.key.clone(),
            // npm only ever echoes `_rev` back to us, so anything that changes with the document
            // serves. It is required for `npm unpublish` and `npm dist-tag` to proceed.
            rev: format!("{}-{}", versions.len(), project.updated_at.timestamp()),
            name: project.key.clone(),
            description: project.description.clone(),
            dist_tags: dist_tags.into_iter().collect(),
            time: times,
            maintainers: field("maintainers")
                .and_then(|value| serde_json::from_value(value).ok())
                .unwrap_or_default(),
            readme: newest
                .map(|version| version.readme.clone())
                .filter(|readme| !readme.is_empty()),
            readme_filename: newest
                .map(|version| version.readme_file_name.clone())
                .filter(|name| !name.is_empty()),
            license: field("license"),
            keywords: field("keywords")
                .and_then(|value| serde_json::from_value(value).ok())
                .unwrap_or_default(),
            repository: field("repository"),
            bugs: field("bugs"),
            homepage: field("homepage"),
            versions: versions_map,
        })
    }

    #[instrument]
    async fn handle_publish(
        &self,
        request: RepositoryRequest,
    ) -> Result<RepoResponse, NPMRegistryError> {
        let Some(user) = request
            .authentication
            .get_user_if_has_action(RepositoryActions::Write, self.id, self.site.as_ref())
            .await?
        else {
            info!("No acceptable user authentication provided");
            return Ok(RepoResponse::unauthorized());
        };
        let body = request.body.body_as_string().await?;
        let PublishRequest {
            attachments,
            versions,
            other,
            ..
        }: PublishRequest = serde_json::from_str(&body)?;
        if versions.len() != 1 {
            return Err(NPMRegistryError::OnlyOneReleaseOrAttachmentAtATime);
        }
        let (version, data) = versions.into_iter().next().unwrap();
        {
            let storage_config: nr_storage::BorrowedStorageConfig = self.storage.storage_config();
            data.dist
                .validate_tarball(&storage_config.storage_config.storage_name, &self.name)?;
        }

        // Decode and check every attachment before anything is written. `dist.integrity` and
        // `dist.shasum` were parsed, stored and never verified, so a tarball that did not match
        // its own checksums was accepted and then failed every later `npm install`.
        let mut decoded = Vec::with_capacity(attachments.len());
        for (file, attachment) in attachments.into_iter() {
            let bytes = attachment.read_data()?;
            verify_integrity(&data.dist.integrity, &bytes)?;
            if !data.dist.shasum.is_empty() {
                verify_shasum(&data.dist.shasum, &bytes)?;
            }
            decoded.push((file, bytes));
        }

        // The name comes from the version document rather than the envelope's `name` field, so it
        // has been through `NPMPackageName` parsing and carries the scope in canonical form.
        let project_path = StoragePath::from(data.name.to_string());
        let project = self.get_or_create_project(&project_path, &data).await?;
        let mut version_path = project_path.clone();
        version_path.push_mut(&version);

        // Refuses a re-publish rather than overwriting, so this happens before any file is
        // written.
        self.create_version(user.id, &version_path, &project, &data)
            .await?;

        for (file, bytes) in decoded {
            info!(?file, "Saving Attachment");
            let mut path = version_path.clone();
            // A scoped package names its attachment `@scope/pkg-1.0.0.tgz`; only the last segment
            // is a filename.
            let file_name = file.rsplit('/').next().unwrap_or(&file);
            path.push_mut(file_name);
            self.storage
                .save_file(self.id, FileContent::Content(bytes), &path)
                .await?;
        }

        // `npm publish --tag next` names the tag in the request. Without reading it, everything
        // published landed on `latest` no matter what was asked for.
        let tag = other
            .get("dist-tags")
            .and_then(Value::as_object)
            .and_then(|tags| tags.keys().next().cloned())
            .unwrap_or_else(|| LATEST_TAG.to_owned());
        DBNpmDistTag::set(project.id, &tag, &version, self.site.as_ref()).await?;

        Ok(ResponseBuilder::ok()
            .json(&serde_json::json!({ "success": true }))
            .into())
    }

    /// `npm deprecate`. npm sends the packument back with `deprecated` set on the versions it wants
    /// marked, so the change is whichever versions differ from what is stored.
    #[instrument(skip(self, request))]
    async fn handle_deprecate(
        &self,
        request: RepositoryRequest,
    ) -> Result<RepoResponse, NPMRegistryError> {
        if request
            .authentication
            .get_user_if_has_action(RepositoryActions::Write, self.id, self.site.as_ref())
            .await?
            .is_none()
        {
            return Ok(RepoResponse::unauthorized());
        }
        let name = request.path.to_string();
        let project = self.project_or_not_found(&name).await?;

        #[derive(Deserialize)]
        struct DeprecateRequest {
            #[serde(default)]
            versions: HashMap<String, Value>,
        }
        let body = request.body.body_as_string().await?;
        let incoming: DeprecateRequest = serde_json::from_str(&body)?;

        for (version_string, incoming_version) in incoming.versions {
            let message = incoming_version
                .get("deprecated")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let Some(mut version) = self
                .get_project_version(project.id, &version_string)
                .await?
            else {
                continue;
            };
            let Some(object) = version
                .extra
                .0
                .extra
                .as_mut()
                .and_then(Value::as_object_mut)
            else {
                continue;
            };
            if message.is_empty() {
                // `npm deprecate pkg@1.0.0 ""` clears the notice.
                object.remove("deprecated");
            } else {
                object.insert("deprecated".to_owned(), Value::String(message.to_owned()));
            }
            UpdateProjectVersion {
                extra: Some(version.extra.0.clone()),
                ..Default::default()
            }
            .update(version.id, self.site.as_ref())
            .await?;
        }
        Ok(ResponseBuilder::ok()
            .json(&serde_json::json!({ "success": true }))
            .into())
    }

    /// `npm dist-tag add`. The body is the version, as a bare JSON string.
    #[instrument(skip(self, request))]
    async fn handle_dist_tag_write(
        &self,
        request: RepositoryRequest,
        name: String,
        tag: String,
    ) -> Result<RepoResponse, NPMRegistryError> {
        if request
            .authentication
            .get_user_if_has_action(RepositoryActions::Write, self.id, self.site.as_ref())
            .await?
            .is_none()
        {
            return Ok(RepoResponse::unauthorized());
        }
        let project = self.project_or_not_found(&name).await?;
        let body = request.body.body_as_string().await?;
        let version: String =
            serde_json::from_str(&body).unwrap_or_else(|_| body.trim().to_owned());

        // A tag aimed at a version that does not exist surfaces as a 404 at install time, which is
        // far harder to trace back to here than refusing it now.
        self.version_or_not_found(&project, &version).await?;
        DBNpmDistTag::set(project.id, &tag, &version, self.site.as_ref()).await?;
        Ok(ResponseBuilder::ok()
            .json(&serde_json::json!({ tag: version }))
            .into())
    }

    #[instrument(skip(self, authentication))]
    async fn handle_dist_tag_delete(
        &self,
        authentication: &RepositoryAuthentication,
        name: String,
        tag: String,
    ) -> Result<RepoResponse, NPMRegistryError> {
        if !authentication
            .has_action(RepositoryActions::Write, self.id, self.site.as_ref())
            .await?
        {
            return Ok(RepoResponse::unauthorized());
        }
        if tag == LATEST_TAG {
            // Removing it leaves `npm install pkg` with nothing to resolve.
            return Ok(RepoResponse::basic_text_response(
                StatusCode::BAD_REQUEST,
                "The `latest` tag cannot be removed",
            ));
        }
        let project = self.project_or_not_found(&name).await?;
        DBNpmDistTag::delete(project.id, &tag, self.site.as_ref()).await?;
        Ok(ResponseBuilder::ok()
            .json(&serde_json::json!({ "success": true }))
            .into())
    }

    /// `npm unpublish`. There was no `handle_delete` at all, so every DELETE answered 405.
    #[instrument(skip(self, request))]
    async fn handle_unpublish(
        &self,
        request: RepositoryRequest,
    ) -> Result<RepoResponse, NPMRegistryError> {
        if request
            .authentication
            .get_user_if_has_action(RepositoryActions::Write, self.id, self.site.as_ref())
            .await?
            .is_none()
        {
            return Ok(RepoResponse::unauthorized());
        }
        // npm appends `/-rev/{rev}` to an unpublish. The revision is advisory here — there is no
        // optimistic-concurrency story to enforce it against — so it is dropped before parsing.
        let path = strip_revision(&request.path);
        let get_path = GetPath::try_from(path)?;

        match get_path {
            // `DELETE /{package}/-/{file}` removes one version.
            GetPath::GetTar { name, version, .. } => {
                let project = self.project_or_not_found(&name).await?;
                let version_row = self.version_or_not_found(&project, &version).await?;
                let version_path = StoragePath::from(version_row.path.as_str());
                self.storage.delete_file(self.id, &version_path).await?;
                DBProjectVersion::delete_by_id(version_row.id, self.site.as_ref()).await?;
                // A tag left aimed at a version that no longer exists turns every install into a
                // 404 on a tarball that was deleted on purpose.
                let orphaned =
                    DBNpmDistTag::delete_pointing_at(project.id, &version, self.site.as_ref())
                        .await?;
                if !orphaned.is_empty() {
                    debug!(
                        ?orphaned,
                        "Dropped dist-tags pointing at the removed version"
                    );
                }
                // npm removes a package outright once its last version goes.
                if DBProjectVersion::count_for_project(project.id, self.site.as_ref()).await? == 0 {
                    self.remove_project(&project).await?;
                }
                Ok(ResponseBuilder::ok()
                    .json(&serde_json::json!({ "success": true }))
                    .into())
            }
            // `DELETE /{package}` removes the whole package.
            GetPath::GetPackageInfo { name } => {
                let project = self.project_or_not_found(&name).await?;
                self.remove_project(&project).await?;
                Ok(ResponseBuilder::ok()
                    .json(&serde_json::json!({ "success": true }))
                    .into())
            }
            other => {
                info!(?other, "Unsupported unpublish path");
                Err(NPMRegistryError::InvalidGetRequest)
            }
        }
    }

    async fn remove_project(&self, project: &DBProject) -> Result<(), NPMRegistryError> {
        let project_path = StoragePath::from(project.path.as_str());
        self.storage.delete_file(self.id, &project_path).await?;
        // Versions, members and dist-tags go with it by cascade.
        DBProject::delete_by_id(project.id, self.site.as_ref()).await?;
        Ok(())
    }
}

/// Drops the trailing `/-rev/{revision}` npm appends to a mutation.
fn strip_revision(path: &StoragePath) -> StoragePath {
    let as_string = path.to_string();
    match as_string.split_once("/-rev/") {
        Some((before, _)) => StoragePath::from(before),
        None => path.clone(),
    }
}

impl NpmRegistryExt for NPMHostedRegistry {}
impl RepositoryExt for NPMHostedRegistry {}
impl Repository for NPMHostedRegistry {
    type Error = NPMRegistryError;
    fn get_storage(&self) -> DynStorage {
        self.0.storage.clone()
    }
    fn site(&self) -> NitroRepo {
        self.0.site.clone()
    }

    fn get_type(&self) -> &'static str {
        "npm"
    }
    fn full_type(&self) -> &'static str {
        FULL_TYPE
    }

    fn config_types(&self) -> Vec<&str> {
        vec![
            NPMRegistryConfigType::get_type_static(),
            RepositoryPageType::get_type_static(),
            ProjectConfigType::get_type_static(),
        ]
    }

    fn name(&self) -> String {
        self.0.name.clone()
    }

    fn id(&self) -> uuid::Uuid {
        self.id
    }

    fn visibility(&self) -> Visibility {
        *self.0.visibility.read()
    }

    fn is_active(&self) -> bool {
        self.0.active.load(atomic::Ordering::Relaxed)
    }

    #[instrument(fields(repository_type = "npm/hosted"))]
    async fn reload(&self) -> Result<(), RepositoryFactoryError> {
        let Some(repository) = DBRepository::get_by_id(self.id, self.site.as_ref()).await? else {
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

    /// Tells the file browser which project — and which version — a path belongs to.
    ///
    /// npm stored the right paths from the start but never implemented this, so the default
    /// resolved nothing and browsing a package showed a bare directory listing with no link to its
    /// project page. Both lookups match the `path` columns written at publish time:
    /// `{package}` is the project's and `{package}/{version}` is the version's.
    #[instrument(fields(repository_type = "npm/hosted"))]
    async fn resolve_project_and_version_for_path(
        &self,
        path: &StoragePath,
    ) -> Result<ProjectResolution, NPMRegistryError> {
        let path = path.to_string();

        if let Some(version) =
            DBProjectVersion::find_ids_by_version_dir(&path, self.id, self.site.as_ref()).await?
        {
            debug!(?path, ?version, "Path is a package version");
            return Ok(version.into());
        }
        if let Some(project) =
            DBProject::find_by_project_directory(&path, self.id, self.site.as_ref()).await?
        {
            debug!(?path, "Path is a package");
            return Ok(ProjectResolution {
                project_id: Some(project.id),
                version_id: None,
            });
        }
        Ok(ProjectResolution::default())
    }

    async fn handle_get(
        &self,
        request: RepositoryRequest,
    ) -> Result<RepoResponse, NPMRegistryError> {
        let path_as_string = request.path.to_string();
        debug!(?path_as_string, "Handling NPM GET request");
        let get_path = match GetPath::try_from(request.path.clone()) {
            Ok(ok) => ok,
            Err(err) => return Ok(err.into_response().into()),
        };

        // The login poll stays reachable without credentials — it is how a client obtains them.
        if let GetPath::LoginDone { session } = &get_path {
            return web_login::poll_login(self, session).await;
        }
        if let Some(denied) = self.check_read(&request.authentication).await? {
            return Ok(denied);
        }

        match get_path {
            GetPath::RegistryBase | GetPath::Ping => Ok(ResponseBuilder::ok()
                .json(&serde_json::json!({
                    "db_name": self.name,
                    "engine": "nitro_repo",
                }))
                .into()),
            GetPath::Whoami => match request.authentication.get_user() {
                Some(user) => Ok(ResponseBuilder::ok()
                    .json(&serde_json::json!({ "username": user.username }))
                    .into()),
                None => Ok(RepoResponse::unauthorized()),
            },
            GetPath::Search => {
                let query = request.parts.uri.query().unwrap_or_default();
                search::handle_search(self, query).await
            }
            GetPath::LoginDone { .. } => unreachable!("answered above"),
            GetPath::DistTags { name } => {
                let project = self.project_or_not_found(&name).await?;
                let versions =
                    DBProjectVersion::get_all_versions(project.id, self.site.as_ref()).await?;
                let tags = self.dist_tags_for(&project, &versions).await?;
                Ok(ResponseBuilder::ok().json(&tags).into())
            }
            GetPath::GetPackageInfo { name } => {
                let project = self.project_or_not_found(&name).await?;
                let packument = self.packument(&project).await?;
                Ok(ResponseBuilder::ok().json(&packument).into())
            }
            GetPath::VersionInfo { name, version } => {
                let project = self.project_or_not_found(&name).await?;
                // `npm install pkg@latest` asks for a tag here, not a version number.
                let version =
                    match DBNpmDistTag::get(project.id, &version, self.site.as_ref()).await? {
                        Some(tag) => tag.version,
                        None => version,
                    };
                let version = self.version_or_not_found(&project, &version).await?;
                let Some(extra) = version.extra.0.extra else {
                    return Ok(RepoResponse::basic_text_response(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "This version has no stored npm metadata",
                    ));
                };
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header(CONTENT_TYPE, "application/json")
                    .body(serde_json::to_string(&extra).unwrap().into())
                    .into())
            }
            GetPath::GetTar {
                name,
                version,
                file,
            } => {
                let project = self.project_or_not_found(&name).await?;
                let version = self.version_or_not_found(&project, &version).await?;
                let mut storage_path = StoragePath::from(version.path.as_str());
                storage_path.push_mut(&file);
                debug!(?storage_path, "Getting file");
                let file = self.storage.open_file(self.id, &storage_path).await?;
                Ok(RepoResponse::from(file))
            }
        }
    }

    /// npm opens a browser login with `POST /-/v1/login`.
    async fn handle_post(
        &self,
        request: RepositoryRequest,
    ) -> Result<RepoResponse, NPMRegistryError> {
        if request.path.to_string() == "-/v1/login" {
            return web_login::perform_login(self, request).await;
        }
        Ok(RepoResponse::unsupported_method_response(
            request.parts.method.clone(),
            self.get_type(),
        ))
    }

    async fn handle_delete(
        &self,
        request: RepositoryRequest,
    ) -> Result<RepoResponse, NPMRegistryError> {
        let path_as_string = request.path.to_string();
        // `npm dist-tag rm` goes through the registry route rather than the package one.
        if let Some((name, tag)) = dist_tag_route(&path_as_string) {
            return self
                .handle_dist_tag_delete(&request.authentication, name, tag)
                .await;
        }
        self.handle_unpublish(request).await
    }

    async fn handle_put(
        &self,
        request: RepositoryRequest,
    ) -> Result<RepoResponse, NPMRegistryError> {
        let path_as_string = request.path.to_string();
        debug!(?path_as_string, "Handling NPM PUT request");
        if path_as_string.starts_with(r#"-/user/org.couchdb.user:"#) {
            return super::login::couch_db::perform_login(self, request).await;
        } else if path_as_string == "-/v1/login" {
            // npm uses POST for this, but older clients and some proxies send PUT.
            return web_login::perform_login(self, request).await;
        }
        // `npm dist-tag add` writes here.
        if let Some((name, tag)) = dist_tag_route(&path_as_string) {
            return self.handle_dist_tag_write(request, name, tag).await;
        }

        let command_header = match request
            .headers()
            .get(NPM_COMMAND_HEADER)
            .ok_or(InvalidNPMCommand::NoHeaderFound)
            .and_then(NPMCommand::try_from)
        {
            Ok(ok) => ok,
            Err(err) => return Ok(err.into_response().into()),
        };

        match command_header {
            NPMCommand::Publish => self.handle_publish(request).await,
            NPMCommand::Deprecate => self.handle_deprecate(request).await,
            NPMCommand::Unpublish => self.handle_unpublish(request).await,
            // These have no registry-side model here yet. They are refused with a message npm
            // shows the user, rather than the "Invalid command" 400 every one of them used to get.
            other @ (NPMCommand::DistTag
            | NPMCommand::Access
            | NPMCommand::Owner
            | NPMCommand::Star
            | NPMCommand::AddUser) => Err(NPMRegistryError::UnsupportedCommand(other.to_string())),
        }
    }
}

/// Matches `-/package/{name}/dist-tags/{tag}`, where the name may itself contain a `/` when the
/// package is scoped.
fn dist_tag_route(path: &str) -> Option<(String, String)> {
    let rest = path.strip_prefix("-/package/")?;
    let (name, tag) = rest.rsplit_once("/dist-tags/")?;
    if name.is_empty() || tag.is_empty() {
        return None;
    }
    Some((name.to_owned(), tag.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dist_tag_routes_are_recognised() {
        assert_eq!(
            dist_tag_route("-/package/mylib/dist-tags/next"),
            Some(("mylib".to_owned(), "next".to_owned()))
        );
        assert_eq!(
            dist_tag_route("-/package/@nr/mylib/dist-tags/beta"),
            Some(("@nr/mylib".to_owned(), "beta".to_owned()))
        );
        assert_eq!(dist_tag_route("-/package/mylib/dist-tags"), None);
        assert_eq!(dist_tag_route("mylib"), None);
    }

    #[test]
    fn revisions_are_stripped_from_a_mutation_path() {
        assert_eq!(
            strip_revision(&StoragePath::from("mylib/-rev/3-abc")).to_string(),
            "mylib"
        );
        assert_eq!(
            strip_revision(&StoragePath::from("@nr/mylib/-/mylib-1.0.0.tgz/-rev/3-abc"))
                .to_string(),
            "@nr/mylib/-/mylib-1.0.0.tgz"
        );
        assert_eq!(
            strip_revision(&StoragePath::from("mylib")).to_string(),
            "mylib"
        );
    }
}
