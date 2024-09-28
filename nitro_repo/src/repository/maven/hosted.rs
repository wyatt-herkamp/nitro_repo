use std::sync::{
    atomic::{self, AtomicBool},
    Arc,
};

use derive_more::derive::Deref;
use maven_rs::pom::Pom;
use nr_core::{
    database::{
        project::{DBProject, DBProjectVersion, ProjectDBType},
        repository::DBRepository,
    },
    repository::{
        config::{
            get_repository_config_or_default,
            project::{ProjectConfig, ProjectConfigType},
            repository_page::RepositoryPageType,
            RepositoryConfigType,
        },
        project::ProjectResolution,
        Visibility,
    },
    storage::StoragePath,
    user::permissions::{HasPermissions, RepositoryActions},
};
use nr_storage::{DynStorage, Storage};
use parking_lot::RwLock;
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

use crate::{
    app::NitroRepo,
    repository::{
        maven::{configs::MavenPushRulesConfigType, MavenRepositoryConfigType},
        utils::RepositoryExt,
        Repository, RepositoryFactoryError,
    },
};

use super::{
    configs::MavenPushRules, utils::MavenRepositoryExt, MavenError, RepoResponse,
    RepositoryRequest, REPOSITORY_TYPE_ID,
};
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
    pub site: NitroRepo,
}
impl MavenHostedInner {}
#[derive(Debug, Clone, Deref)]
pub struct MavenHosted(Arc<MavenHostedInner>);
impl MavenRepositoryExt for MavenHosted {}
impl RepositoryExt for MavenHosted {}
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
    ) -> Result<RepoResponse, MavenError> {
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
        if path.has_extension("pom") {
            let pom: Pom = self.parse_pom(body.to_vec())?;
            debug!(?pom, "Parsed POM File");
            self.post_pom_upload(path.clone(), Some(user_id), pom).await;
        }
        let (size, created) = self.storage.save_file(self.id, body.into(), &path).await?;
        // Trigger Push Event if it is the .pom file
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
    fn site(&self) -> NitroRepo {
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

    async fn reload(&self) -> Result<(), RepositoryFactoryError> {
        let Some(is_active) = DBRepository::get_active_by_id(self.id, self.site.as_ref()).await?
        else {
            error!("Failed to get repository");
            self.0.active.store(false, atomic::Ordering::Relaxed);
            return Ok(());
        };
        self.0.active.store(is_active, atomic::Ordering::Relaxed);

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
    #[instrument(name = "maven_hosted_get")]
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
        let file = self.0.storage.open_file(self.id, &path).await?;
        return self.indexing_check_option(file, &authentication).await;
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
    ) -> Result<RepoResponse, MavenError> {
        let visibility = self.visibility();
        if let Some(err) = self.check_read(&authentication).await? {
            return Ok(err);
        }
        let file = self.storage.get_file_information(self.id, &path).await?;
        return self.indexing_check_option(file, &authentication).await;
    }
    #[instrument(name = "maven_hosted_put")]
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

        let Some(nitro_deploy_version) = request.get_nitro_repo_deploy_header()? else {
            return self.standard_maven_deploy(request).await;
        };
        info!(?nitro_deploy_version, "Handling Nitro Deploy Version");

        Ok(RepoResponse::unsupported_method_response(
            request.parts.method,
            self.get_type(),
        ))
    }
    async fn handle_post(&self, request: RepositoryRequest) -> Result<RepoResponse, MavenError> {
        let Some(nitro_deploy_version) = request.get_nitro_repo_deploy_header()? else {
            return Ok(RepoResponse::unsupported_method_response(
                request.parts.method,
                self.get_type(),
            ));
        };
        info!(?nitro_deploy_version, "Handling Nitro Deploy Version");
        todo!()
    }

    #[instrument]
    async fn resolve_project_and_version_for_path(
        &self,
        path: StoragePath,
    ) -> Result<ProjectResolution, MavenError> {
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
