use std::{
    ops::Deref,
    sync::{atomic::AtomicBool, Arc},
};

use axum::response::Response;
use bytes::Bytes;
use http::StatusCode;
use maven_rs::pom::Pom;
use nr_core::{
    database::repository::{DBRepository, DBRepositoryConfig},
    repository::{
        config::{
            get_repository_config_or_default,
            project::{ProjectConfig, ProjectConfigType},
            repository_page::RepositoryPageType,
            RepositoryConfigType as _,
        },
        proxy_url::ProxyURL,
        Visibility,
    },
    storage::StoragePath,
};
use nr_storage::{DynStorage, FileContent, Storage, StorageFile};
use parking_lot::RwLock;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::{app::NitroRepo, repository::Repository};

use super::{
    repo_type::RepositoryFactoryError, utils::MavenRepositoryExt, MavenError,
    MavenRepositoryConfig, MavenRepositoryConfigType, RepoResponse, RepositoryRequest,
    REPOSITORY_TYPE_ID,
};
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MavenProxyConfig {
    pub routes: Vec<MavenProxyRepositoryRoute>,
}
impl MavenProxyConfig {
    pub fn sort(&mut self) {
        self.routes.sort_by(|a, b| match (a.priority, b.priority) {
            (Some(a), Some(b)) => a.cmp(&b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        });
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MavenProxyRepositoryRoute {
    pub url: ProxyURL,
    pub name: Option<String>,
    /// If Null then it will be the lowest priority
    pub priority: Option<i32>,
    // TODO: Credentials
}
fn project_download_files(pom: &Pom) -> Result<Vec<String>, MavenError> {
    let version = pom
        .get_version()
        .ok_or(MavenError::MissingFromPom("version"))?;
    Ok(vec![
        format!("{}-{}.jar", pom.artifact_id, version),
        format!("{}-{}-sources.jar", pom.artifact_id, version),
        format!("{}-{}-javadoc.jar", pom.artifact_id, version),
    ])
}
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
        proxy_config: MavenProxyConfig,
    ) -> Result<Self, RepositoryFactoryError> {
        let project_config_db =
            get_repository_config_or_default::<ProjectConfigType, ProjectConfig>(
                repository.id,
                site.as_ref(),
            )
            .await?;
        let inner = MavenProxyInner {
            id: repository.id,
            name: repository.name.into(),
            active: AtomicBool::new(repository.active),
            visibility: RwLock::new(repository.visibility),
            config: RwLock::new(proxy_config),
            project: RwLock::new(project_config_db.value.0),
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
    #[instrument(name = "maven_proxy_project_download")]
    pub async fn proxy_project_download(
        &self,
        path: StoragePath,
        proxy_config: MavenProxyRepositoryRoute,
        pom: Bytes,
    ) -> Result<(), MavenError> {
        let pom = self.parse_pom(pom.to_vec())?;
        let version_dir = path.parent();
        let http_client = reqwest::Client::builder()
            .user_agent("Nitro Repo")
            .build()
            .expect("Failed to build HTTP Client");

        for file in project_download_files(&pom)? {
            debug!(?file, "Downloading file");
            let mut path = version_dir.clone();
            path.push_mut(&file);
            let url = format!("{}/{}", proxy_config.url, path);
            match http_client.get(&url).send().await {
                Ok(ok) => {
                    if ok.status().is_success() {
                        let bytes = ok.bytes().await?;
                        self.save_bytes(bytes, &path).await?;
                    } else {
                        warn!(?url, ?file, ?ok, "Failed to download file");
                    }
                }
                Err(err) => {
                    warn!(?url, ?file, ?err, "Failed to download file");
                }
            }
        }
        // TODO: Trigger project indexing
        Ok(())
    }
    #[instrument(name = "maven_proxy_get_from_proxy")]
    pub async fn get_from_proxy(
        &self,
        path: StoragePath,
    ) -> Result<Option<StorageFile>, MavenError> {
        // TODO: Setup internal cache to check the following
        //  If a recent previous request was made with a similar path use that proxy config.
        //  Similar path being both starting with /dev/kingtux/tms/... They should be in the same proxy
        // TODO: Handle projects. When requesting a path such as /dev/kingtux/tms/1.0.0/tms-1.0.0.pom. Go ahead and download all files in that directory.
        let proxy_config = self.config.read().clone();
        let http_client = reqwest::Client::builder()
            .user_agent("Nitro Repo")
            .build()
            .expect("Failed to build HTTP Client");
        for route in proxy_config.routes {
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
            let response = match http_client.get(url).send().await {
                Ok(ok) => ok,
                Err(err) => {
                    error!(?err, ?url_string, "Failed to send request");
                    continue;
                }
            };
            if response.status().is_success() {
                let response_bytes = response.bytes().await?;
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
                info!(?response, ?url_string, "Failed to proxy request");
            }
        }
        Ok(None)
    }
}

impl Repository for MavenProxy {
    type Error = MavenError;
    fn get_storage(&self) -> nr_storage::DynStorage {
        self.0.storage.clone()
    }
    fn visibility(&self) -> Visibility {
        Visibility::Public
    }

    fn get_type(&self) -> &'static str {
        &REPOSITORY_TYPE_ID
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
    async fn reload(&self) -> Result<(), RepositoryFactoryError> {
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
                MavenRepositoryConfig::Proxy(proxy_config) => {
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
    #[instrument(name = "maven_proxy_get")]
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
        let visibility = self.visibility();
        let Some(file) = self.0.storage.open_file(self.id, &path).await? else {
            debug!(?path, "File not found in storage. Proxying request");
            return match self.get_from_proxy(path).await {
                Ok(ok) => Ok(RepoResponse::from(ok)),
                Err(err) => {
                    warn!(?err, "Failed to proxy request");
                    Ok(Response::builder()
                        .status(StatusCode::SERVICE_UNAVAILABLE)
                        .body(format!("Failed to proxy request: {}", err).into())
                        .into())
                }
            };
        };
        // TODO: Check file age. If it is older than the configured time then re-download the file.
        return self.indexing_check(file, &authentication).await;
    }
    #[instrument(name = "maven_proxy_head")]
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
        // TODO: Proxy HEAD request
        if let Some(err) = self.check_read(&authentication).await? {
            return Ok(err);
        }
        let file = self.storage.get_file_information(self.id, &path).await?;
        return self.indexing_check_option(file, &authentication).await;
    }
    fn site(&self) -> NitroRepo {
        self.0.site.clone()
    }
}
impl MavenRepositoryExt for MavenProxy {}
