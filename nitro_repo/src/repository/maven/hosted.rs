use std::sync::{
    atomic::{self, AtomicBool},
    Arc,
};

use axum::response::Response;
use derive_more::derive::Deref;
use http::StatusCode;
use maven_rs::pom::Pom;
use nr_core::{
    database::{
        project::{DBProject, DBProjectVersion, NewProjectMember, ProjectDBType},
        repository::DBRepository,
    },
    repository::{
        config::{
            get_repository_config_or_default,
            project::{ProjectConfig, ProjectConfigType},
            repository_page::RepositoryPageType,
            PushRulesConfig, PushRulesConfigType, RepositoryConfigType, SecurityConfig,
            SecurityConfigType,
        },
        project::ProjectResolution,
        Visibility,
    },
    storage::StoragePath,
    user::permissions::{HasPermissions, RepositoryActions},
};
use nr_storage::{DynStorage, Storage};
use parking_lot::RwLock;
use tokio::io::AsyncReadExt;
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

use crate::{
    app::NitroRepo,
    repository::{
        maven::{MavenError, MavenRepositoryConfigType},
        Repository, RepositoryFactoryError, RepositoryHandlerError,
    },
};

use super::{RepoResponse, RepositoryAuthentication, RepositoryRequest};
#[derive(derive_more::Debug)]
pub struct MavenHostedInner {
    pub id: Uuid,
    pub name: String,
    pub active: AtomicBool,
    pub visibility: RwLock<Visibility>,
    pub push_rules: RwLock<PushRulesConfig>,
    pub security: RwLock<SecurityConfig>,
    pub project: RwLock<ProjectConfig>,
    #[debug(skip)]
    pub storage: DynStorage,
    #[debug(skip)]
    pub site: NitroRepo,
}
impl MavenHostedInner {}
#[derive(Debug, Clone, Deref)]
pub struct MavenHosted(Arc<MavenHostedInner>);
impl MavenHosted {
    #[instrument]
    pub async fn standard_maven_deploy(
        &self,
        RepositoryRequest {
            parts,
            body,
            path,
            authentication,
        }: RepositoryRequest,
    ) -> Result<RepoResponse, RepositoryHandlerError> {
        let user_id = if let Some(user) = authentication.get_user() {
            user.id
        } else {
            return Ok(RepoResponse::unauthorized());
        };

        {
            let push_rules = self.push_rules.read();
            if push_rules.require_nitro_deploy {
                return Ok(RepoResponse::require_nitro_deploy());
            }
        }
        info!("Saving File: {}", path);
        let body = body.body_as_bytes().await?;
        // TODO: Validate Against Push Rules

        let (size, created) = self.storage.save_file(self.id, body.into(), &path).await?;
        // Trigger Push Event if it is the .pom file
        if path.has_extension("pom") {
            let file = self
                .storage
                .open_file(self.id, &path)
                .await?
                .and_then(|x| x.file());
            let Some((mut file, meta)) = file else {
                return Ok(Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body("Failed to open file".into())
                    .unwrap()
                    .into());
            };
            let mut pom_file = String::with_capacity(size);
            file.read_to_string(&mut pom_file).await?;
            let pom: maven_rs::pom::Pom =
                maven_rs::quick_xml::de::from_str(&pom_file).map_err(MavenError::from)?;
            debug!(?pom, "Parsed POM File");
            self.post_pom_upload(path.clone(), user_id, pom).await;
        }
        let save_path = format!(
            "/repositories/{}/{}/{}",
            self.storage.storage_config().storage_config.storage_name,
            self.name,
            path
        );

        Ok(RepoResponse::put_response(created, save_path))
    }
    pub async fn load(
        repository: DBRepository,
        storage: DynStorage,
        site: NitroRepo,
    ) -> Result<Self, RepositoryFactoryError> {
        let security_db = get_repository_config_or_default::<SecurityConfigType, SecurityConfig>(
            repository.id,
            site.as_ref(),
        )
        .await?;
        debug!("Loaded Security Config: {:?}", security_db);
        let push_rules_db =
            get_repository_config_or_default::<PushRulesConfigType, PushRulesConfig>(
                repository.id,
                site.as_ref(),
            )
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
            active: active,
            visibility: RwLock::new(repository.visibility),
            push_rules: RwLock::new(push_rules_db.value.0),
            security: RwLock::new(security_db.value.0),
            project: RwLock::new(project_db.value.0),
            storage,
            site,
        };
        Ok(Self(Arc::new(inner)))
    }
    pub async fn check_read(
        &self,
        authentication: &RepositoryAuthentication,
    ) -> Result<Option<RepoResponse>, RepositoryHandlerError> {
        if self.visibility().is_private() {
            if authentication.is_no_identification() {
                return Ok(Some(RepoResponse::www_authenticate("Basic")));
            } else if !(authentication
                .has_action(RepositoryActions::Read, self.id, self.site.as_ref())
                .await?)
            {
                return Ok(Some(RepoResponse::forbidden()));
            }
        }
        Ok(None)
    }
    async fn add_or_update_version(
        &self,
        version_directory: StoragePath,
        project_id: Uuid,
        publisher: i32,
        pom: Pom,
    ) -> Result<(), MavenError> {
        let db_version = DBProjectVersion::find_by_version_and_project(
            &pom.version,
            project_id,
            &self.site.database,
        )
        .await?;
        if let Some(version) = db_version {
            info!(?version, "Version already exists");
            // TODO: Update Version
        } else {
            let version = super::pom_to_db_project_version(
                project_id,
                version_directory,
                publisher,
                pom.clone(),
            )?;
            version.insert_no_return(&self.site.database).await?;
            info!("Created Version");
        };
        Ok(())
    }
    #[instrument]
    async fn post_pom_upload_inner(
        &self,
        pom_directory: StoragePath,
        publisher: i32,
        pom: Pom,
    ) -> Result<(), MavenError> {
        let project_key = format!("{}:{}", pom.group_id, pom.artifact_id);
        let version_directory = pom_directory.clone().parent();
        let db_project =
            DBProject::find_by_project_key(&project_key, self.id, &self.site.database).await?;
        let project_id = if let Some(project) = db_project {
            project.id
        } else {
            let project_directory = version_directory.clone().parent();
            let project = super::pom_to_db_project(project_directory, self.id, pom.clone())?;
            let project = project.insert(&self.site.database).await?;

            let new_member = NewProjectMember::new_owner(publisher, project.id);
            new_member.insert_no_return(&self.site.database).await?;
            info!(?project, "Created Project");
            project.id
        };

        self.add_or_update_version(version_directory, project_id, publisher, pom)
            .await?;
        Ok(())
    }

    pub async fn post_pom_upload(&self, pom_directory: StoragePath, publisher: i32, pom: Pom) {
        match self
            .post_pom_upload_inner(pom_directory, publisher, pom)
            .await
        {
            Ok(()) => {}
            Err(e) => {
                error!(?e, "Failed to handle POM Upload");
            }
        }
    }
}
impl Repository for MavenHosted {
    fn get_storage(&self) -> nr_storage::DynStorage {
        self.0.storage.clone()
    }
    fn visibility(&self) -> Visibility {
        self.visibility.read().clone()
    }
    fn get_type(&self) -> &'static str {
        "maven"
    }

    fn config_types(&self) -> Vec<&str> {
        vec![
            RepositoryPageType::get_type_static(),
            PushRulesConfigType::get_type_static(),
            SecurityConfigType::get_type_static(),
            ProjectConfigType::get_type_static(),
            MavenRepositoryConfigType::get_type_static(),
        ]
    }

    async fn reload(&self) -> Result<(), RepositoryFactoryError> {
        let Some(is_active) = DBRepository::get_active_by_id(self.id, self.site.as_ref()).await?
        else {
            error!("Failed to get repository");
            self.0.active.store(false, atomic::Ordering::Relaxed);
            return Ok(());
        };
        self.0.active.store(is_active, atomic::Ordering::Relaxed);

        let push_rules_db =
            get_repository_config_or_default::<PushRulesConfigType, PushRulesConfig>(
                self.id,
                self.site.as_ref(),
            )
            .await?;
        let security_db = get_repository_config_or_default::<SecurityConfigType, SecurityConfig>(
            self.id,
            self.site.as_ref(),
        )
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
            let mut security = self.security.write();
            *security = security_db.value.0;
        }
        {
            let mut project_config = self.project.write();
            *project_config = project_config_db.value.0;
        }

        Ok(())
    }
    #[instrument(name = "maven_hosted_get")]
    async fn handle_get(
        &self,
        RepositoryRequest {
            parts,
            path,
            authentication,
            ..
        }: RepositoryRequest,
    ) -> Result<RepoResponse, RepositoryHandlerError> {
        if let Some(err) = self.check_read(&authentication).await? {
            return Ok(err);
        }
        let visibility = self.visibility();
        let file = self.0.storage.open_file(self.id, &path).await?;
        if let Some(file) = &file {
            if file.is_directory()
                && visibility.is_hidden()
                && !authentication
                    .has_action(RepositoryActions::Read, self.id, self.site.as_ref())
                    .await?
            {
                return Ok(RepoResponse::indexing_not_allowed());
            }
        }
        Ok(RepoResponse::from(file))
    }
    #[instrument(name = "maven_hosted_head")]
    async fn handle_head(
        &self,
        RepositoryRequest {
            parts,
            path,
            authentication,
            ..
        }: RepositoryRequest,
    ) -> Result<RepoResponse, RepositoryHandlerError> {
        let visibility = self.visibility();

        if let Some(err) = self.check_read(&authentication).await? {
            return Ok(err);
        }
        let file = self.storage.get_file_information(self.id, &path).await?;
        if let Some(file) = &file {
            if file.is_directory()
                && visibility.is_hidden()
                && !authentication
                    .has_action(RepositoryActions::Read, self.id, self.site.as_ref())
                    .await?
            {
                return Ok(RepoResponse::indexing_not_allowed());
            }
        }
        Ok(RepoResponse::from(file))
    }
    #[instrument(name = "maven_hosted_put")]
    async fn handle_put(
        &self,
        request: RepositoryRequest,
    ) -> Result<RepoResponse, RepositoryHandlerError> {
        info!("Handling PUT Request for Repository: {}", self.id);
        {
            let security = self.security.read();
            if security.must_use_auth_token_for_push && !request.authentication.has_auth_token() {
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

        let Some(nitro_deploy_version) = request.get_nitro_repo_deploy_header()? else {
            return self.standard_maven_deploy(request).await;
        };
        info!(?nitro_deploy_version, "Handling Nitro Deploy Version");

        Ok(RepoResponse::unsupported_method_response(
            request.parts.method,
            self.get_type(),
        ))
    }
    async fn handle_post(
        &self,
        request: RepositoryRequest,
    ) -> Result<RepoResponse, RepositoryHandlerError> {
        let Some(nitro_deploy_version) = request.get_nitro_repo_deploy_header()? else {
            return Ok(RepoResponse::unsupported_method_response(
                request.parts.method,
                self.get_type(),
            ));
        };
        info!(?nitro_deploy_version, "Handling Nitro Deploy Version");
        todo!()
    }
    fn name(&self) -> String {
        self.0.name.clone()
    }

    fn id(&self) -> Uuid {
        self.0.id
    }

    fn is_active(&self) -> bool {
        self.active.load(atomic::Ordering::Relaxed)
    }
    #[instrument]
    async fn resolve_project_and_version_for_path(
        &self,
        path: StoragePath,
    ) -> Result<ProjectResolution, RepositoryHandlerError> {
        let path_as_string = path.to_string();
        let version = DBProjectVersion::find_by_version_directory(
            &path_as_string,
            self.id,
            self.site.as_ref(),
        )
        .await?;

        let project = if let Some(version) = version.as_ref() {
            DBProject::find_by_id(version.project_id, self.site.as_ref()).await?
        } else {
            DBProject::find_by_project_directory(&path_as_string, self.id, self.site.as_ref())
                .await?
        };

        Ok(ProjectResolution { project, version })
    }
}
