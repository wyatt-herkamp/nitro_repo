use std::sync::{
    Arc,
    atomic::{self, AtomicBool},
};

use derive_more::derive::Deref;
use http::{
    StatusCode,
    header::{CONTENT_TYPE, ETAG, IF_NONE_MATCH},
    request::Parts,
};
use nr_core::{
    database::entities::{
        project::{
            DBProject, NewProject, NewProjectMember, ProjectDBType,
            update::UpdateProject,
            versions::{DBProjectVersion, NewVersion, UpdateProjectVersion},
        },
        repository::DBRepository,
        user::UserSafeData,
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
use nr_storage::{DynStorage, FileContent, Storage};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{debug, info, instrument, warn};

use super::{
    CargoRegistryConfigType, CargoRegistryError,
    types::{
        CargoPath, IndexEntry, is_valid_crate_name, publish::PublishMetadata, split_publish_body,
    },
    utils::{crate_file_path, registry_base_url, to_hex},
};
use crate::{
    app::NitroRepo,
    repository::{
        RepoResponse, Repository, RepositoryAuthentication, RepositoryFactoryError,
        RepositoryRequest, utils::RepositoryExt,
    },
    utils::ResponseBuilder,
};

/// The default page size for `cargo search`, and the cap on what a caller can ask for. Without an
/// upper bound one request could ask the database for every crate in the registry.
const DEFAULT_SEARCH_SIZE: i64 = 10;
const MAX_SEARCH_SIZE: i64 = 100;

#[derive(derive_more::Debug)]
pub struct CargoRegistryInner {
    #[debug(skip)]
    pub site: NitroRepo,
    pub storage: DynStorage,
    pub id: uuid::Uuid,
    pub name: String,
    pub visibility: RwLock<Visibility>,
    pub active: AtomicBool,
}

#[derive(Debug, Clone, Deref)]
pub struct CargoHostedRegistry(Arc<CargoRegistryInner>);

impl CargoHostedRegistry {
    pub async fn load(
        site: NitroRepo,
        storage: DynStorage,
        repository: DBRepository,
    ) -> Result<Self, RepositoryFactoryError> {
        Ok(Self(Arc::new(CargoRegistryInner {
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
    /// Unlike npm, cargo is not prompted into sending credentials by a `WWW-Authenticate` challenge
    /// — it decides up front, from `auth-required` in `config.json`. A private registry therefore
    /// answers a credential-less request with a plain 401 and relies on `config.json` having told
    /// the client to send a token in the first place.
    async fn check_read(
        &self,
        authentication: &RepositoryAuthentication,
    ) -> Result<Option<RepoResponse>, CargoRegistryError> {
        if self.visibility().is_public() {
            return Ok(None);
        }
        if !authentication
            .can_access_repository(RepositoryActions::Read, self.id, self.site.as_ref())
            .await?
        {
            return Ok(Some(RepoResponse::unauthorized()));
        }
        Ok(None)
    }

    /// Resolves the caller, or `None` if they may not write here.
    async fn writer(
        &self,
        authentication: &RepositoryAuthentication,
    ) -> Result<Option<UserSafeData>, CargoRegistryError> {
        Ok(authentication
            .get_user_if_has_action(RepositoryActions::Write, self.id, self.site.as_ref())
            .await?
            .cloned())
    }

    async fn project_or_not_found(&self, name: &str) -> Result<DBProject, CargoRegistryError> {
        self.get_project_from_key(name)
            .await?
            .ok_or_else(|| CargoRegistryError::NotFound(format!("crate `{name}` not found")))
    }

    async fn version_or_not_found(
        &self,
        project: &DBProject,
        version: &str,
    ) -> Result<DBProjectVersion, CargoRegistryError> {
        self.get_project_version(project.id, version)
            .await?
            .ok_or_else(|| {
                CargoRegistryError::NotFound(format!(
                    "crate `{}` has no version `{version}`",
                    project.key
                ))
            })
    }

    /// `index/config.json`.
    ///
    /// Generated per request rather than stored, because the right answer depends on how the client
    /// reached the registry — a custom hostname and the `/repositories/...` path are both valid and
    /// must each describe themselves.
    fn config_json(&self, parts: &Parts) -> Result<RepoResponse, CargoRegistryError> {
        let base = registry_base_url(self, parts)?;
        // `dl` has no template markers, so cargo appends `/{crate}/{version}/download` itself.
        let config = serde_json::json!({
            "dl": format!("{base}/api/v1/crates"),
            "api": base,
            // Cargo only sends a token when it has been told the registry needs one. Without this
            // a private registry answers every request with a 401 that cargo reports as a hard
            // failure rather than retrying with credentials.
            "auth-required": !self.visibility().is_public(),
        });
        Ok(ResponseBuilder::ok().json(&config).into())
    }

    /// The stored index record for one version, with `yanked` taken from its current value.
    fn index_entry_for(
        &self,
        project: &DBProject,
        version: &DBProjectVersion,
    ) -> Result<IndexEntry, CargoRegistryError> {
        let Some(extra) = version.extra.0.extra.clone() else {
            return Err(CargoRegistryError::CorruptIndexRecord {
                name: project.key.clone(),
                version: version.version.clone(),
            });
        };
        let stored: StoredVersion = serde_json::from_value(extra).map_err(|error| {
            warn!(?error, version = %version.version, "Stored cargo index record will not parse");
            CargoRegistryError::CorruptIndexRecord {
                name: project.key.clone(),
                version: version.version.clone(),
            }
        })?;
        let mut entry = stored.index;
        entry.yanked = stored.yanked;
        Ok(entry)
    }

    /// `index/{prefix}/{name}` — the crate's versions, one JSON object per line.
    ///
    /// Cargo revalidates the index constantly, so this answers conditional requests. The body is
    /// generated from the database on every call, which means there is no stored `Last-Modified` to
    /// compare against; an `ETag` over the rendered bytes is the one validator that is always
    /// correct here.
    #[instrument(skip(self, parts))]
    async fn index_entry(
        &self,
        name: &str,
        parts: &Parts,
    ) -> Result<RepoResponse, CargoRegistryError> {
        let project = self.project_or_not_found(name).await?;
        let versions = DBProjectVersion::get_all_versions(project.id, self.site.as_ref()).await?;

        // Oldest first. Cargo does not require an order, but a stable one keeps the ETag from
        // changing when nothing has, and publication order is the only ordering this layer can
        // know is right (see `get_all_versions`).
        let mut body = String::new();
        for version in versions.iter().rev() {
            match self.index_entry_for(&project, version) {
                Ok(entry) => {
                    body.push_str(&serde_json::to_string(&entry)?);
                    body.push('\n');
                }
                Err(error) => {
                    // One unreadable row must not take the whole crate offline; every other version
                    // of it is still installable.
                    warn!(?error, "Skipping a version with an unreadable index record");
                }
            }
        }
        if body.is_empty() {
            return Err(CargoRegistryError::NotFound(format!(
                "crate `{name}` has no published versions"
            )));
        }

        let etag = format!("\"{}\"", to_hex(&Sha256::digest(body.as_bytes())));
        if parts
            .headers
            .get(IF_NONE_MATCH)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.split(',').any(|tag| tag.trim() == etag))
        {
            return Ok(RepoResponse::Other(
                ResponseBuilder::default()
                    .status(StatusCode::NOT_MODIFIED)
                    .header(ETAG, etag)
                    .empty(),
            ));
        }

        Ok(RepoResponse::Other(
            ResponseBuilder::ok()
                .header(ETAG, etag)
                .header(CONTENT_TYPE, "text/plain; charset=utf-8")
                .body(body),
        ))
    }

    /// `GET /api/v1/crates/{name}/{version}/download`.
    ///
    /// A yanked version is still served. Yanking removes a version from resolution, not from
    /// existence — a lockfile that already pins it must keep working, which is the whole point of
    /// yank being distinct from delete.
    async fn download(
        &self,
        name: &str,
        version: &str,
    ) -> Result<RepoResponse, CargoRegistryError> {
        let project = self.project_or_not_found(name).await?;
        let version_row = self.version_or_not_found(&project, version).await?;
        let path = crate_file_path(&project.key, &version_row.version)?;
        let Some(file) = self.storage.open_file(self.id, &path).await? else {
            return Err(CargoRegistryError::NotFound(format!(
                "the .crate file for `{name}@{version}` is missing from storage"
            )));
        };
        Ok(RepoResponse::FileResponse(Box::new(file)))
    }

    #[instrument(skip(self, request))]
    async fn publish(
        &self,
        request: RepositoryRequest,
    ) -> Result<RepoResponse, CargoRegistryError> {
        let Some(user) = self.writer(&request.authentication).await? else {
            info!("No acceptable user authentication provided");
            return Ok(RepoResponse::unauthorized());
        };

        let body = request.body.body_as_bytes().await?;
        let (metadata, crate_file) = split_publish_body(body)?;

        if !is_valid_crate_name(&metadata.name) {
            return Err(CargoRegistryError::InvalidCrateName(metadata.name));
        }
        if !is_plausible_semver(&metadata.vers) {
            return Err(CargoRegistryError::InvalidVersion(metadata.vers));
        }

        let project = self.get_or_create_project(&metadata, user.id).await?;

        // Refused before anything is written, so a rejected re-publish cannot have replaced the
        // stored .crate of the version it collided with.
        if self
            .get_project_version(project.id, &metadata.vers)
            .await?
            .is_some()
        {
            return Err(CargoRegistryError::VersionAlreadyExists {
                name: metadata.name,
                version: metadata.vers,
            });
        }

        // Cargo verifies every download against `cksum`, so it is computed from the bytes actually
        // stored rather than taken from anything the client said.
        let cksum = to_hex(&Sha256::digest(&crate_file));
        let path = crate_file_path(&project.key, &metadata.vers)?;

        let version = metadata.vers.clone();
        let stored = StoredVersion {
            index: IndexEntry::from_publish(metadata.clone(), cksum),
            yanked: false,
            publish: metadata.clone(),
        };

        NewVersion {
            project_id: project.id,
            version: version.clone(),
            release_type: ReleaseType::release_type_from_version(&version),
            version_path: path.to_string(),
            publisher: Some(user.id),
            version_page: metadata.readme.clone(),
            extra: VersionData {
                documentation_url: metadata.documentation.clone(),
                website: metadata.homepage.clone(),
                description: metadata.description.clone(),
                extra: Some(serde_json::to_value(&stored)?),
                ..Default::default()
            },
        }
        .insert(self.site.as_ref())
        .await?;

        self.storage
            .save_file(self.id, FileContent::Bytes(crate_file), &path)
            .await?;

        debug!(crate_name = %project.key, %version, "Published a crate");
        Ok(ResponseBuilder::ok()
            .json(&serde_json::json!({ "warnings": { "invalid_categories": [], "invalid_badges": [], "other": [] } }))
            .into())
    }

    async fn get_or_create_project(
        &self,
        metadata: &PublishMetadata,
        publisher: i32,
    ) -> Result<DBProject, CargoRegistryError> {
        if let Some(project) =
            DBProject::find_by_project_key(&metadata.name, self.id, self.site.as_ref()).await?
        {
            let description = metadata.description.as_deref();
            let update = UpdateProject {
                description: (description != project.description.as_deref())
                    .then(|| description.map(str::to_owned)),
                ..Default::default()
            };
            if !update.is_empty() {
                update.update(project.id, self.site.as_ref()).await?;
            }
            return Ok(project);
        }

        let mut storage_path = StoragePath::parse("crates")?;
        storage_path.push_mut(&metadata.name);
        let project = NewProject {
            scope: None,
            project_key: metadata.name.clone(),
            name: metadata.name.clone(),
            description: metadata.description.clone(),
            repository: self.id,
            storage_path: storage_path.to_string(),
        }
        .insert(self.site.as_ref())
        .await?;

        // Whoever publishes a crate first owns it, which is what `cargo owner` then manages.
        NewProjectMember::new_owner(publisher, project.id)
            .insert_no_return(self.site.as_ref())
            .await?;
        info!(?project, "Created a new crate");
        Ok(project)
    }

    /// `DELETE .../yank` and `PUT .../unyank`.
    #[instrument(skip(self, authentication))]
    async fn set_yanked(
        &self,
        authentication: &RepositoryAuthentication,
        name: &str,
        version: &str,
        yanked: bool,
    ) -> Result<RepoResponse, CargoRegistryError> {
        if self.writer(authentication).await?.is_none() {
            return Ok(RepoResponse::unauthorized());
        }
        let project = self.project_or_not_found(name).await?;
        let mut version_row = self.version_or_not_found(&project, version).await?;

        let Some(extra) = version_row.extra.0.extra.take() else {
            return Err(CargoRegistryError::CorruptIndexRecord {
                name: project.key.clone(),
                version: version.to_owned(),
            });
        };
        let mut stored: StoredVersion = serde_json::from_value(extra)?;
        stored.yanked = yanked;
        version_row.extra.0.extra = Some(serde_json::to_value(&stored)?);

        UpdateProjectVersion {
            extra: Some(version_row.extra.0.clone()),
            ..Default::default()
        }
        .update(version_row.id, self.site.as_ref())
        .await?;

        Ok(ResponseBuilder::ok()
            .json(&serde_json::json!({ "ok": true }))
            .into())
    }

    /// `GET /api/v1/crates/{name}/owners`.
    async fn list_owners(&self, name: &str) -> Result<RepoResponse, CargoRegistryError> {
        let project = self.project_or_not_found(name).await?;
        let owners: Vec<OwnerResponse> = sqlx::query_as::<_, (i32, String, Option<String>)>(
            r#"SELECT users.id, users.username, users.name
               FROM project_members
               JOIN users ON users.id = project_members.user_id
               WHERE project_members.project_id = $1
               ORDER BY users.username"#,
        )
        .bind(project.id)
        .fetch_all(self.site.as_ref())
        .await?
        .into_iter()
        .map(|(id, login, name)| OwnerResponse { id, login, name })
        .collect();

        Ok(ResponseBuilder::ok()
            .json(&serde_json::json!({ "users": owners }))
            .into())
    }

    /// `PUT` and `DELETE` on `/api/v1/crates/{name}/owners`.
    ///
    /// Managing owners is a stronger right than publishing: it is what lets someone hand the crate
    /// to a third party. Repository `Write` is not enough — the caller must already manage this
    /// crate, or be able to administer the repository.
    #[instrument(skip(self, request))]
    async fn modify_owners(
        &self,
        request: RepositoryRequest,
        name: &str,
        adding: bool,
    ) -> Result<RepoResponse, CargoRegistryError> {
        let Some(user) = self.writer(&request.authentication).await? else {
            return Ok(RepoResponse::unauthorized());
        };
        let project = self.project_or_not_found(name).await?;

        let manages_crate: Option<bool> = sqlx::query_scalar(
            "SELECT can_manage FROM project_members WHERE project_id = $1 AND user_id = $2",
        )
        .bind(project.id)
        .bind(user.id)
        .fetch_optional(self.site.as_ref())
        .await?;
        let can_manage = manages_crate.unwrap_or(false)
            || request
                .authentication
                .can_access_repository(RepositoryActions::Edit, self.id, self.site.as_ref())
                .await?;
        if !can_manage {
            return Ok(RepoResponse::forbidden());
        }

        #[derive(Debug, Deserialize)]
        struct OwnersRequest {
            #[serde(default)]
            users: Vec<String>,
        }
        let body = request.body.body_as_string().await?;
        let incoming: OwnersRequest = serde_json::from_str(&body)?;

        let mut changed = Vec::new();
        for login in incoming.users {
            let Some(target): Option<i32> =
                sqlx::query_scalar("SELECT id FROM users WHERE username = $1")
                    .bind(&login)
                    .fetch_optional(self.site.as_ref())
                    .await?
            else {
                return Err(CargoRegistryError::NotFound(format!(
                    "there is no user named `{login}`"
                )));
            };
            if adding {
                sqlx::query(
                    r#"INSERT INTO project_members (user_id, project_id, can_write, can_manage)
                       VALUES ($1, $2, TRUE, TRUE)
                       ON CONFLICT (project_id, user_id)
                       DO UPDATE SET can_write = TRUE, can_manage = TRUE"#,
                )
                .bind(target)
                .bind(project.id)
                .execute(self.site.as_ref())
                .await?;
            } else {
                sqlx::query("DELETE FROM project_members WHERE project_id = $1 AND user_id = $2")
                    .bind(project.id)
                    .bind(target)
                    .execute(self.site.as_ref())
                    .await?;
            }
            changed.push(login);
        }

        let verb = if adding { "added to" } else { "removed from" };
        Ok(ResponseBuilder::ok()
            .json(&serde_json::json!({
                "ok": true,
                "msg": format!("{} {verb} owners of {}", changed.join(", "), project.key),
            }))
            .into())
    }

    /// `GET /api/v1/crates?q=...` — what `cargo search` calls.
    #[instrument(skip(self))]
    async fn search(&self, query: Option<&str>) -> Result<RepoResponse, CargoRegistryError> {
        let query: SearchQuery = query
            .and_then(|query| serde_urlencoded::from_str(query).ok())
            .unwrap_or_default();
        let per_page = query
            .per_page
            .unwrap_or(DEFAULT_SEARCH_SIZE)
            .clamp(1, MAX_SEARCH_SIZE);

        // `projects.key` carries the `ignoreCase` collation, which Postgres refuses to run `ILIKE`
        // against ("nondeterministic collations are not supported for ILIKE") — every search
        // returned a 500. `COLLATE "C"` gives the comparison a deterministic collation to work in;
        // no case-sensitivity is lost, because `ILIKE` does that part itself.
        //
        // `%`, `_` and `\` in the search text would otherwise act as LIKE syntax.
        let escaped = query
            .q
            .replace('\\', r"\\")
            .replace('%', r"\%")
            .replace('_', r"\_");
        let pattern = format!("%{escaped}%");

        let total: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM projects
               WHERE repository_id = $1
                 AND ((key COLLATE "C") ILIKE $2 OR COALESCE(description, '') ILIKE $2)"#,
        )
        .bind(self.id)
        .bind(&pattern)
        .fetch_one(self.site.as_ref())
        .await?;

        let projects: Vec<DBProject> = sqlx::query_as(
            r#"SELECT * FROM projects
               WHERE repository_id = $1
                 AND ((key COLLATE "C") ILIKE $2 OR COALESCE(description, '') ILIKE $2)
               ORDER BY updated_at DESC
               LIMIT $3"#,
        )
        .bind(self.id)
        .bind(&pattern)
        .bind(per_page)
        .fetch_all(self.site.as_ref())
        .await?;

        let mut crates = Vec::with_capacity(projects.len());
        for project in projects {
            let versions =
                DBProjectVersion::get_all_versions(project.id, self.site.as_ref()).await?;
            let Some(newest) = versions.first() else {
                continue;
            };
            crates.push(serde_json::json!({
                "name": project.key,
                "max_version": newest.version,
                "description": project.description.clone().unwrap_or_default(),
            }));
        }

        Ok(ResponseBuilder::ok()
            .json(&serde_json::json!({
                "crates": crates,
                "meta": { "total": total },
            }))
            .into())
    }
}

/// What a `cargo` version carries in `project_versions.extra`.
///
/// The index record and the publish metadata are both kept: the index is what has to be served back
/// byte-for-byte on every resolve, and the publish metadata holds the fields the web UI wants
/// (categories, keywords, licence) that the index has no room for.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredVersion {
    index: IndexEntry,
    #[serde(default)]
    yanked: bool,
    publish: PublishMetadata,
}

#[derive(Debug, Default, Deserialize)]
struct SearchQuery {
    #[serde(default)]
    q: String,
    per_page: Option<i64>,
}

#[derive(Debug, Serialize)]
struct OwnerResponse {
    id: i32,
    login: String,
    name: Option<String>,
}

/// A cheap sanity check on the version string.
///
/// Not a full semver parse — cargo has already refused anything it cannot parse before it gets
/// here, and this registry has no reason to hold a second, subtly different opinion. What it does
/// catch is a version that cannot be used as a path component or would collide with the `.crate`
/// filename convention.
fn is_plausible_semver(version: &str) -> bool {
    !version.is_empty()
        && version.len() <= 64
        && version.starts_with(|c: char| c.is_ascii_digit())
        && version
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '+' | '_'))
}

impl RepositoryExt for CargoHostedRegistry {}

impl Repository for CargoHostedRegistry {
    type Error = CargoRegistryError;

    fn get_storage(&self) -> DynStorage {
        self.0.storage.clone()
    }

    fn site(&self) -> NitroRepo {
        self.0.site.clone()
    }

    fn get_type(&self) -> &'static str {
        "cargo"
    }

    fn full_type(&self) -> &'static str {
        "cargo/hosted"
    }

    fn config_types(&self) -> Vec<&str> {
        vec![
            CargoRegistryConfigType::get_type_static(),
            RepositoryPageType::get_type_static(),
            ProjectConfigType::get_type_static(),
        ]
    }

    fn name(&self) -> String {
        self.0.name.clone()
    }

    fn id(&self) -> uuid::Uuid {
        self.0.id
    }

    fn visibility(&self) -> Visibility {
        *self.0.visibility.read()
    }

    fn is_active(&self) -> bool {
        self.0.active.load(atomic::Ordering::Relaxed)
    }

    #[instrument(fields(repository_type = "cargo/hosted"))]
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

    async fn handle_get(
        &self,
        request: RepositoryRequest,
    ) -> Result<RepoResponse, CargoRegistryError> {
        // `config.json` is answered before the read check: cargo has to be able to read it to learn
        // that this registry wants a token at all.
        let route = CargoPath::try_from(request.path.clone())?;
        if matches!(route, CargoPath::Config) {
            return self.config_json(&request.parts);
        }
        if let Some(denied) = self.check_read(&request.authentication).await? {
            return Ok(denied);
        }

        match route {
            CargoPath::Config => unreachable!("handled above"),
            CargoPath::IndexEntry { name } => self.index_entry(&name, &request.parts).await,
            CargoPath::Download { name, version } => self.download(&name, &version).await,
            CargoPath::Owners { name } => self.list_owners(&name).await,
            CargoPath::Search => self.search(request.parts.uri.query()).await,
            other => {
                debug!(?other, "Route is not a GET");
                Ok(RepoResponse::unsupported_method_response(
                    request.parts.method,
                    "cargo",
                ))
            }
        }
    }

    async fn handle_put(
        &self,
        request: RepositoryRequest,
    ) -> Result<RepoResponse, CargoRegistryError> {
        match CargoPath::try_from(request.path.clone())? {
            CargoPath::Publish => self.publish(request).await,
            CargoPath::Unyank { name, version } => {
                self.set_yanked(&request.authentication, &name, &version, false)
                    .await
            }
            CargoPath::Owners { name } => self.modify_owners(request, &name, true).await,
            other => {
                debug!(?other, "Route is not a PUT");
                Ok(RepoResponse::unsupported_method_response(
                    request.parts.method,
                    "cargo",
                ))
            }
        }
    }

    async fn handle_delete(
        &self,
        request: RepositoryRequest,
    ) -> Result<RepoResponse, CargoRegistryError> {
        match CargoPath::try_from(request.path.clone())? {
            CargoPath::Yank { name, version } => {
                self.set_yanked(&request.authentication, &name, &version, true)
                    .await
            }
            CargoPath::Owners { name } => self.modify_owners(request, &name, false).await,
            other => {
                debug!(?other, "Route is not a DELETE");
                Ok(RepoResponse::unsupported_method_response(
                    request.parts.method,
                    "cargo",
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_plausible_semver;

    #[test]
    fn version_strings_are_sanity_checked() {
        for good in ["1.0.0", "0.1.0-alpha.1", "1.0.0+build.5", "2.0.0-rc1"] {
            assert!(is_plausible_semver(good), "{good}");
        }
        for bad in ["", "v1.0.0", "1.0.0/../etc", "1 0 0", "latest"] {
            assert!(!is_plausible_semver(bad), "{bad}");
        }
    }
}
