use super::types::{
    request::{GetPath, InvalidNPMCommand, NPMCommand, PublishVersion},
    NpmRegistryPackageResponse, NPM_COMMAND_HEADER,
};
use super::utils::{npm_time, NpmRegistryExt};
use crate::{
    app::{responses::no_content_response, NitroRepo},
    repository::{
        npm::{types::PublishRequest, NPMRegistryConfigType, NPMRegistryError},
        utils::RepositoryExt,
        RepoResponse, Repository, RepositoryFactoryError, RepositoryRequest,
    },
};
use ahash::{HashMap, HashMapExt};
use axum::response::{IntoResponse, Response};
use derive_more::derive::Deref;
use http::{header::CONTENT_TYPE, StatusCode};
use nr_core::{
    database::entities::{project::versions::DBProjectVersion, repository::DBRepository},
    repository::config::RepositoryConfigType,
    storage::StoragePath,
    user::permissions::RepositoryActions,
};
use nr_storage::{DynStorage, FileContent, Storage};
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

#[derive(derive_more::Debug)]
pub struct NpmRegistryInner {
    #[debug(skip)]
    pub site: NitroRepo,
    pub storage: DynStorage,
    pub id: uuid::Uuid,
    pub repository: DBRepository,
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
            repository,
        })))
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
        debug!(?body, "Handling publish request");
        let PublishRequest {
            name,
            attachments,
            versions,
            other,
        }: PublishRequest = serde_json::from_str(&body)?;
        if versions.len() != 1 {
            return Err(NPMRegistryError::OnlyOneReleaseOrAttachmentAtATime);
        }
        let (version, data) = versions.into_iter().next().unwrap();
        {
            let storage_config: nr_storage::BorrowedStorageConfig = self.storage.storage_config();
            data.dist.validate_tarball(
                &storage_config.storage_config.storage_name,
                &self.repository.name,
            )?;
        }
        let project_path = StoragePath::from(name.clone());
        let project = self.get_or_create_project(&project_path, &data).await?;
        let mut version_path = project_path.clone();
        version_path.push_mut(&version);

        self.create_or_update_version(user.id, &version_path, &project, &data)
            .await?;

        for (file, attachment) in attachments.into_iter() {
            info!(?file, ?attachment, "Saving Attachment");
            let mut path = version_path.clone();
            if file.starts_with("@") && file.contains("/") {
                let split = file.split("/").collect::<Vec<&str>>();
                path.push_mut(split.last().unwrap());
            } else {
                path.push_mut(&file);
            }
            let attachment_data = attachment.read_data()?;
            let storage = self.get_storage();
            storage
                .save_file(self.id, FileContent::Content(attachment_data), &path)
                .await?;
        }

        Ok(no_content_response().into())
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
        "npm/hosted"
    }

    fn config_types(&self) -> Vec<&str> {
        vec![NPMRegistryConfigType::get_type_static()]
    }

    fn name(&self) -> String {
        self.0.repository.name.to_string()
    }

    fn id(&self) -> uuid::Uuid {
        self.id
    }

    fn visibility(&self) -> nr_core::repository::Visibility {
        nr_core::repository::Visibility::Public
    }

    fn is_active(&self) -> bool {
        true
    }
    async fn handle_get(
        &self,
        request: RepositoryRequest,
    ) -> Result<RepoResponse, NPMRegistryError> {
        let headers = request.headers();
        let path_as_string = request.path.to_string();
        debug!(?headers, ?path_as_string, "Handling NPM GET request");
        let get_path = match GetPath::try_from(request.path.clone()) {
            Ok(ok) => ok,
            Err(err) => return Ok(err.into_response().into()),
        };
        match get_path {
            GetPath::GetPackageInfo { name } => {
                let Some(project) = self.get_project_from_key(&name).await? else {
                    return Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(format!("Project {} not found in repository", name).into())
                        .into());
                };
                debug!(?project, "Got project");
                let versions =
                    DBProjectVersion::get_all_versions(project.id, self.site.as_ref()).await?;
                let mut dist_tags = HashMap::new();
                let mut times = HashMap::new();
                times.insert(
                    "created".to_owned(),
                    npm_time::format_date_time(&project.created_at),
                );
                times.insert(
                    "modified".to_owned(),
                    npm_time::format_date_time(&project.updated_at),
                );
                if let Some(latest) = project.latest_release {
                    dist_tags.insert("latest".to_string(), latest);
                }
                let mut versions_map = HashMap::new();
                for version in versions {
                    times.insert(
                        version.version.clone(),
                        npm_time::format_date_time(&version.created_at),
                    );
                    debug!(?version, "Got Version");
                    if let Some(extra) = version.extra.0.extra {
                        let extra: PublishVersion = match serde_json::from_value(extra) {
                            Ok(ok) => ok,
                            Err(err) => {
                                warn!("Invalid NPM Project");
                                continue;
                            }
                        };
                        versions_map.insert(version.version.clone(), extra);
                    } else {
                        warn!(?version, "Invalid NPM Project");
                    }
                }
                let project_response = NpmRegistryPackageResponse {
                    id: project.project_key.clone(),
                    name: project.name.clone(),
                    description: project.description.clone(),
                    dist_tags,
                    versions: versions_map,
                    time: times,
                };
                debug!(?project_response, "Returning Project");
                let as_string = serde_json::to_string(&project_response).unwrap();
                Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header(CONTENT_TYPE, "application/json")
                    .body(as_string.into())
                    .into())
            }
            GetPath::VersionInfo { name, version } => {
                let Some(project) = self.get_project_from_key(&name).await? else {
                    return Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(format!("Project {} not found in repository", name).into())
                        .into());
                };
                debug!(?project, ?version, "Getting version");
                let Some(version) = self.get_project_version(project.id, &version).await? else {
                    return Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(format!("Version {} not found in project {}", version, name).into())
                        .into());
                };
                debug!(?version, "Got Version");
                if let Some(extra) = version.extra.0.extra {
                    let as_string = serde_json::to_string(&extra).unwrap();
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header(CONTENT_TYPE, "application/json")
                        .body(as_string.into())
                        .into())
                } else {
                    Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body("Invalid NPM Project".into())
                        .into())
                }
            }
            GetPath::GetTar {
                name,
                version,
                file,
            } => {
                let Some(project) = self.get_project_from_key(&name).await? else {
                    return Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(format!("Project {} not found in repository", name).into())
                        .into());
                };
                debug!(?project, ?version, "Getting version");
                let Some(version) = self.get_project_version(project.id, &version).await? else {
                    return Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(format!("Version {} not found in project {}", version, name).into())
                        .into());
                };
                debug!(?version, "Got Version");
                let mut storage_path = StoragePath::from(version.version_path.as_str());
                storage_path.push_mut(&file);
                debug!(?storage_path, "Getting file");
                let storage = self.get_storage();
                let file = storage.open_file(self.id, &storage_path).await?;
                Ok(RepoResponse::from(file))
            }
            _ => {
                Ok(Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body("Not Found".into())
                    .into())
            }
        }
    }

    async fn handle_put(
        &self,
        request: RepositoryRequest,
    ) -> Result<RepoResponse, NPMRegistryError> {
        let path_as_string = request.path.to_string();
        debug!(
            ?path_as_string,
            headers = ?request.headers(),
        );
        if path_as_string.starts_with(r#"-/user/org.couchdb.user:"#) {
            return super::login::couch_db::perform_login(self, request).await;
        } else if path_as_string.eq("-/v1/login") {
            return super::login::web_login::perform_login(self, request).await;
        }
        let Some(user) = request
            .authentication
            .get_user_if_has_action(RepositoryActions::Write, self.id, self.site.as_ref())
            .await?
        else {
            info!("No acceptable user authentication provided");
            return Ok(RepoResponse::unauthorized());
        };
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
        }
    }
}
