use std::{
    ops::Deref,
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

use ahash::HashMap;
use axum::response::Response;
use bytes::Bytes;
use http::StatusCode;
use maven_rs::pom::Pom;
use nr_core::{
    database::entities::{
        project::{DBProject, ProjectDBType, versions::DBProjectVersion},
        repository::{DBRepository, DBRepositoryConfig},
    },
    repository::{
        Visibility,
        config::{
            RepositoryConfigType as _, get_repository_config_or_default,
            project::{ProjectConfig, ProjectConfigType},
            repository_page::RepositoryPageType,
        },
        project::ProjectResolution,
        proxy_url::ProxyURL,
    },
    storage::StoragePath,
};
use nr_storage::{DynStorage, FileContent, Storage, StorageFile};
use parking_lot::RwLock;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, instrument, warn};
use uuid::Uuid;

use super::{
    MavenError, MavenRepositoryConfig, MavenRepositoryConfigType, REPOSITORY_TYPE_ID, RepoResponse,
    RepositoryRequest, metadata, repo_type::RepositoryFactoryError, utils::MavenRepositoryExt,
};
use crate::{app::NitroRepo, repository::Repository};
/// How long a cached artifact is served before the upstream is asked again.
///
/// Zero means forever, which is right for a released artifact — those are immutable by
/// convention. It is wrong for `maven-metadata.xml` and for snapshots, which is why they get their
/// own, much shorter default below.
const fn default_cache_ttl_seconds() -> u64 {
    0
}
/// Metadata and snapshots change upstream, so they expire quickly regardless of the artifact TTL.
const fn default_mutable_ttl_seconds() -> u64 {
    15 * 60
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct MavenProxyConfig {
    pub routes: Vec<MavenProxyRepositoryRoute>,
    /// Seconds a cached artifact stays valid. `0` keeps it forever.
    ///
    /// Nothing expired before this existed: once a file had been fetched it was served from
    /// storage for good, so a corrected upstream artifact or a moved snapshot never arrived.
    #[schemars(title = "Cache TTL (seconds)")]
    pub cache_ttl_seconds: u64,
    /// Seconds a cached `maven-metadata.xml` or snapshot build stays valid.
    #[schemars(title = "Metadata and snapshot TTL (seconds)")]
    pub mutable_ttl_seconds: u64,
}
impl Default for MavenProxyConfig {
    fn default() -> Self {
        Self {
            routes: Vec::new(),
            cache_ttl_seconds: default_cache_ttl_seconds(),
            mutable_ttl_seconds: default_mutable_ttl_seconds(),
        }
    }
}
impl MavenProxyConfig {
    /// Orders routes by priority, lowest number first.
    ///
    /// This existed and was never called, so `priority` had no effect and routes were tried in
    /// whatever order they happened to be stored in.
    pub fn sort(&mut self) {
        self.routes.sort_by(|a, b| match (a.priority, b.priority) {
            (Some(a), Some(b)) => a.cmp(&b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
    }

    /// How long a given path may be served from cache.
    pub fn ttl_for(&self, path: &StoragePath) -> Option<Duration> {
        let as_string = path.to_string();
        let file_name = as_string.rsplit('/').next().unwrap_or_default();
        let is_mutable = file_name.starts_with(metadata::METADATA_FILE_NAME)
            || as_string.to_uppercase().contains("-SNAPSHOT");
        let seconds = if is_mutable {
            self.mutable_ttl_seconds
        } else {
            self.cache_ttl_seconds
        };
        (seconds > 0).then(|| Duration::from_secs(seconds))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MavenProxyRepositoryRoute {
    pub url: ProxyURL,
    pub name: Option<String>,
    /// If Null then it will be the lowest priority
    pub priority: Option<i32>,
    /// Username for an upstream that requires authentication.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(title = "Username")]
    pub username: Option<String>,
    /// Password or token for an upstream that requires authentication.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(title = "Password")]
    pub password: Option<String>,
}
/// Everything worth pulling when one file of a version is asked for.
///
/// The issue's ask is to "start a task to download all files in the project version when one is
/// requested". A Maven repository offers no listing to enumerate from, so this is the conventional
/// set — the classified jars plus the checksums Maven verifies against, which are what a build
/// asks for next and what used to cost a second round trip upstream each.
fn project_download_files(pom: &Pom) -> Result<Vec<String>, MavenError> {
    let version = pom
        .get_version()
        .ok_or(MavenError::MissingFromPom("version"))?;
    let artifact = &pom.artifact_id;
    let mut files = Vec::new();
    for base in [
        format!("{artifact}-{version}.jar"),
        format!("{artifact}-{version}-sources.jar"),
        format!("{artifact}-{version}-javadoc.jar"),
        format!("{artifact}-{version}.pom"),
    ] {
        files.push(format!("{base}.sha1"));
        files.push(format!("{base}.md5"));
        files.push(base);
    }
    Ok(files)
}
/// How many directory-to-upstream mappings to remember. See [`MavenProxy::remember_route`].
const ROUTE_MEMO_LIMIT: usize = 4096;

/// The value `Repository::full_type()` reports, and what lands in the repository request
/// span and metric attributes. Pinned by `repository_type_ids_are_stable`.
pub static FULL_TYPE: &str = "maven/proxy";

#[derive(Debug)]
pub struct MavenProxyInner {
    pub storage: DynStorage,
    pub site: NitroRepo,
    pub id: Uuid,
    pub name: String,
    pub visibility: RwLock<Visibility>,
    pub active: AtomicBool,
    pub project: RwLock<ProjectConfig>,
    pub config: RwLock<MavenProxyConfig>,
    /// One client for the repository's lifetime.
    ///
    /// A fresh `reqwest::Client` was built for every proxied request and every project download,
    /// so nothing was pooled — each miss paid a new TCP and TLS handshake against the upstream.
    pub http_client: reqwest::Client,
    /// Which upstream last served a given directory.
    pub route_memo: RwLock<HashMap<String, MavenProxyRepositoryRoute>>,
}
#[derive(Debug, Clone)]
pub struct MavenProxy(Arc<MavenProxyInner>);
impl Deref for MavenProxy {
    type Target = MavenProxyInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl MavenProxy {
    pub async fn load(
        repository: DBRepository,
        storage: DynStorage,
        site: NitroRepo,
        mut proxy_config: MavenProxyConfig,
    ) -> Result<Self, RepositoryFactoryError> {
        let project_config_db =
            get_repository_config_or_default::<ProjectConfigType, ProjectConfig>(
                repository.id,
                site.as_ref(),
            )
            .await?;
        // `sort` existed and was never called, so `priority` did nothing.
        proxy_config.sort();
        let inner = MavenProxyInner {
            id: repository.id,
            name: repository.name.into(),
            active: AtomicBool::new(repository.active),
            visibility: RwLock::new(repository.visibility),
            config: RwLock::new(proxy_config),
            project: RwLock::new(project_config_db.value.0),
            http_client: build_http_client(),
            route_memo: RwLock::new(HashMap::default()),
            storage,
            site,
        };
        Ok(Self(Arc::new(inner)))
    }
    async fn save_bytes(
        &self,
        bytes: Bytes,
        to: &StoragePath,
    ) -> Result<(), nr_storage::StorageError> {
        self.storage
            .save_file(self.id, FileContent::Bytes(bytes), to)
            .await?;
        Ok(())
    }

    /// Builds a request against an upstream, applying its credentials if it has any.
    fn upstream_request(
        &self,
        method: reqwest::Method,
        url: reqwest::Url,
        route: &MavenProxyRepositoryRoute,
    ) -> reqwest::RequestBuilder {
        let request = self.http_client.request(method, url);
        match (&route.username, &route.password) {
            (Some(username), password) => request.basic_auth(username, password.as_ref()),
            (None, _) => request,
        }
    }

    /// Remembers which upstream served a directory, so the next file from the same version does
    /// not walk every configured route again.
    ///
    /// A miss used to try each route in turn on every single request, and a build resolving one
    /// version asks for the POM, the jar and two checksums — four full walks where one suffices.
    fn remembered_route(&self, path: &StoragePath) -> Option<MavenProxyRepositoryRoute> {
        let directory = path.clone().parent().to_string();
        let routes = self.route_memo.read();
        routes.get(&directory).cloned()
    }
    fn remember_route(&self, path: &StoragePath, route: &MavenProxyRepositoryRoute) {
        let directory = path.clone().parent().to_string();
        let mut routes = self.route_memo.write();
        // A cap, so a proxy that is asked for thousands of distinct paths does not grow without
        // bound. Losing an entry costs one extra walk, which is what happened every time before.
        if routes.len() >= ROUTE_MEMO_LIMIT {
            routes.clear();
        }
        routes.insert(directory, route.clone());
    }

    /// The routes to try for a path, best guess first.
    fn routes_for(&self, path: &StoragePath) -> Vec<MavenProxyRepositoryRoute> {
        let configured = self.config.read().routes.clone();
        let Some(remembered) = self.remembered_route(path) else {
            return configured;
        };
        let mut routes = vec![remembered.clone()];
        routes.extend(
            configured
                .into_iter()
                .filter(|route| route.url != remembered.url),
        );
        routes
    }

    #[instrument(skip(self, pom), fields(nr.repository.id = %self.id, nr.repository.name = %self.name))]
    pub async fn proxy_project_download(
        &self,
        path: StoragePath,
        proxy_config: MavenProxyRepositoryRoute,
        pom: Bytes,
    ) -> Result<(), MavenError> {
        let pom = self.parse_pom(pom.to_vec())?;
        let version_dir = path.clone().parent();

        for file in project_download_files(&pom)? {
            let mut file_path = version_dir.clone();
            file_path.push_mut(&file);
            if self.storage.file_exists(self.id, &file_path).await? {
                continue;
            }
            debug!(?file, "Downloading file");
            let url_string = format!("{}/{}", proxy_config.url, file_path);
            let Ok(url) = reqwest::Url::parse(&url_string) else {
                warn!(?url_string, "Failed to parse URL");
                continue;
            };
            match self
                .upstream_request(reqwest::Method::GET, url, &proxy_config)
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    let bytes = response.bytes().await?;
                    self.save_bytes(bytes, &file_path).await?;
                }
                Ok(response) => {
                    // A version having no sources or javadoc jar is normal, not a problem.
                    debug!(?url_string, status = ?response.status(), "Upstream does not have this file");
                }
                Err(err) => {
                    warn!(?url_string, ?err, "Failed to download file");
                }
            }
        }

        // Proxied artifacts were never registered, so a proxy repository had no projects at all —
        // invisible to browse, badges and search, and `resolve_project_and_version_for_path`
        // always came back empty.
        if let Err(error) = self.post_pom_upload_inner(path, None, pom).await {
            error!(?error, "Failed to index a proxied project");
        }
        Ok(())
    }

    #[instrument(skip(self), fields(nr.repository.id = %self.id, nr.repository.name = %self.name))]
    pub async fn get_from_proxy(
        &self,
        path: StoragePath,
    ) -> Result<Option<StorageFile>, MavenError> {
        for route in self.routes_for(&path) {
            let mut path_as_string = path.to_string();
            if path_as_string.starts_with("/") {
                path_as_string = path_as_string[1..].into();
            }
            let url_string = format!("{}/{}", route.url, path_as_string);
            debug!(?url_string, "Proxying request");
            let url = match url::Url::parse(&url_string) {
                Ok(ok) => ok,
                Err(err) => {
                    error!(?err, ?url_string, "Failed to parse URL");
                    continue;
                }
            };
            let response = match self
                .upstream_request(reqwest::Method::GET, url, &route)
                .send()
                .await
            {
                Ok(ok) => ok,
                Err(err) => {
                    error!(?err, ?url_string, "Failed to send request");
                    continue;
                }
            };
            if response.status().is_success() {
                let response_bytes = response.bytes().await?;
                self.remember_route(&path, &route);
                if path_as_string.ends_with(".pom") {
                    let self_clone = self.clone();
                    let path = path.clone();
                    let pom = response_bytes.clone();
                    tokio::spawn(async move {
                        if let Err(error) =
                            self_clone.proxy_project_download(path, route, pom).await
                        {
                            error!(?error, "Failed to download project files");
                        };
                    });
                }
                self.storage
                    .save_file(self.id, FileContent::Bytes(response_bytes), &path)
                    .await?;
                return Ok(self.storage.open_file(self.id, &path).await?);
            } else {
                debug!(?url_string, status = ?response.status(), "Upstream does not have this path");
            }
        }
        Ok(None)
    }

    /// Asks the upstreams whether a path exists, without fetching it.
    ///
    /// `handle_head` only looked in local storage, so a `HEAD` 404d for anything not yet cached
    /// while a `GET` for the same path succeeded — and Maven uses `HEAD` to decide whether to
    /// bother with the `GET`.
    #[instrument(skip(self), fields(nr.repository.id = %self.id, nr.repository.name = %self.name))]
    pub async fn head_from_proxy(&self, path: &StoragePath) -> bool {
        for route in self.routes_for(path) {
            let url_string = format!("{}/{}", route.url, path.to_string().trim_start_matches('/'));
            let Ok(url) = reqwest::Url::parse(&url_string) else {
                continue;
            };
            match self
                .upstream_request(reqwest::Method::HEAD, url, &route)
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    self.remember_route(path, &route);
                    return true;
                }
                Ok(_) => {}
                Err(err) => {
                    debug!(?url_string, ?err, "HEAD against upstream failed");
                }
            }
        }
        false
    }

    /// Whether a cached file has outlived its TTL and should be fetched again.
    async fn is_stale(&self, path: &StoragePath) -> Result<bool, MavenError> {
        let Some(ttl) = self.config.read().ttl_for(path) else {
            return Ok(false);
        };
        let Some(meta) = self.storage.get_file_information(self.id, path).await? else {
            return Ok(false);
        };
        let age = chrono::Local::now().fixed_offset() - *meta.modified();
        let Ok(age) = age.to_std() else {
            // A modification time in the future is not something to act on.
            return Ok(false);
        };
        Ok(age > ttl)
    }
}

fn build_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(concat!("Nitro Repo/", env!("CARGO_PKG_VERSION")))
        .build()
        .expect("Failed to build HTTP Client")
}

impl Repository for MavenProxy {
    type Error = MavenError;
    fn get_storage(&self) -> nr_storage::DynStorage {
        self.0.storage.clone()
    }
    /// Read from the database like every other repository type.
    ///
    /// This returned `Public` unconditionally, so `check_read` never blocked anything and a
    /// private proxy repository was readable by anyone — including whatever it had cached from an
    /// authenticated upstream.
    fn visibility(&self) -> Visibility {
        *self.0.visibility.read()
    }

    fn get_type(&self) -> &'static str {
        REPOSITORY_TYPE_ID
    }
    fn full_type(&self) -> &'static str {
        FULL_TYPE
    }
    fn config_types(&self) -> Vec<&str> {
        vec![
            RepositoryPageType::get_type_static(),
            ProjectConfigType::get_type_static(),
            MavenRepositoryConfigType::get_type_static(),
        ]
    }

    fn name(&self) -> String {
        self.0.name.clone()
    }

    fn id(&self) -> Uuid {
        self.0.id
    }

    fn is_active(&self) -> bool {
        self.0.active.load(std::sync::atomic::Ordering::Relaxed)
    }
    #[instrument(fields(repository_type = "maven/proxy"))]
    async fn reload(&self) -> Result<(), RepositoryFactoryError> {
        // Same as the hosted repository: without this the cached visibility never changes, and a
        // proxy made private keeps serving whatever it has cached.
        if let Some(repository) = DBRepository::get_by_id(self.id, self.site.as_ref()).await? {
            self.0
                .active
                .store(repository.active, std::sync::atomic::Ordering::Relaxed);
            *self.0.visibility.write() = repository.visibility;
        }

        let project_config_db =
            get_repository_config_or_default::<ProjectConfigType, ProjectConfig>(
                self.id,
                self.site.as_ref(),
            )
            .await?;
        let Some(maven_config_db) = DBRepositoryConfig::<MavenRepositoryConfig>::get_config(
            self.id,
            MavenRepositoryConfigType::get_type_static(),
            self.site.as_ref(),
        )
        .await?
        else {
            return Err(RepositoryFactoryError::MissingConfig(
                MavenRepositoryConfigType::get_type_static(),
            ));
        };
        {
            let mut project_config = self.project.write();
            *project_config = project_config_db.value.0;
        }
        {
            match maven_config_db.value.0 {
                MavenRepositoryConfig::Proxy(mut proxy_config) => {
                    proxy_config.sort();
                    let mut maven_config = self.config.write();
                    *maven_config = proxy_config;
                }
                _ => {
                    return Err(RepositoryFactoryError::InvalidConfig(
                        MavenRepositoryConfigType::get_type_static(),
                        "Expected Proxy Config".into(),
                    ));
                }
            }
        }
        Ok(())
    }
    async fn handle_get(
        &self,
        RepositoryRequest {
            parts,
            path,
            authentication,
            ..
        }: RepositoryRequest,
    ) -> Result<RepoResponse, MavenError> {
        if let Some(err) = self.check_read(&authentication).await? {
            return Ok(err);
        }
        let cached = self.0.storage.open_file(self.id, &path).await?;
        // Once a file had been fetched it was served from storage forever, so a corrected upstream
        // artifact — or, much more often, a moved snapshot or an updated `maven-metadata.xml` —
        // never arrived. A stale hit falls through to the upstream, and keeps what it has if the
        // upstream cannot be reached.
        let refetch = match &cached {
            Some(_) => self.is_stale(&path).await.unwrap_or(false),
            None => true,
        };
        if refetch {
            debug!(?path, cached = cached.is_some(), "Fetching from upstream");
            match self.get_from_proxy(path.clone()).await {
                Ok(Some(file)) => return self.indexing_check(file, &authentication).await,
                Ok(None) if cached.is_none() => {
                    return Ok(RepoResponse::from(Option::<StorageFile>::None));
                }
                Ok(None) => {}
                Err(err) if cached.is_none() => {
                    warn!(?err, "Failed to proxy request");
                    return Ok(Response::builder()
                        .status(StatusCode::SERVICE_UNAVAILABLE)
                        .body(format!("Failed to proxy request: {}", err).into())
                        .into());
                }
                Err(err) => {
                    warn!(?err, "Upstream is unreachable; serving the cached copy");
                }
            }
        }
        let Some(file) = cached else {
            return Ok(RepoResponse::from(Option::<StorageFile>::None));
        };
        self.indexing_check(file, &authentication).await
    }
    async fn handle_head(
        &self,
        RepositoryRequest {
            path,
            authentication,
            ..
        }: RepositoryRequest,
    ) -> Result<RepoResponse, MavenError> {
        if let Some(err) = self.check_read(&authentication).await? {
            return Ok(err);
        }
        if let Some(file) = self.storage.get_file_information(self.id, &path).await? {
            return self.indexing_check(file, &authentication).await;
        }
        // Only local storage was consulted, so `HEAD` 404d for anything not yet cached while a
        // `GET` for the same path succeeded. Maven uses `HEAD` to decide whether to issue the
        // `GET` at all.
        if self.head_from_proxy(&path).await {
            return Ok(RepoResponse::Other(
                Response::builder()
                    .status(StatusCode::OK)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            ));
        }
        Ok(RepoResponse::from(
            Option::<nr_storage::StorageFileMeta<nr_storage::FileType>>::None,
        ))
    }
    /// Resolves a browse path to a project.
    ///
    /// The default implementation returns nothing, so browse could never resolve a project in a
    /// proxy repository even once one had been indexed.
    #[instrument(fields(repository_type = "maven/proxy"))]
    async fn resolve_project_and_version_for_path(
        &self,
        path: &StoragePath,
    ) -> Result<ProjectResolution, MavenError> {
        let path_as_string = path.to_string();
        if let Some(meta) = self.storage.get_repository_meta(self.id, path).await?
            && let Some(project_id) = meta.project_id
        {
            return Ok(ProjectResolution {
                project_id: Some(project_id),
                version_id: meta.project_version_id,
            });
        }
        if let Some(version) =
            DBProjectVersion::find_ids_by_version_dir(&path_as_string, self.id, self.site.as_ref())
                .await?
        {
            return Ok(version.into());
        }
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
    fn site(&self) -> NitroRepo {
        self.0.site.clone()
    }
}
impl MavenRepositoryExt for MavenProxy {}
