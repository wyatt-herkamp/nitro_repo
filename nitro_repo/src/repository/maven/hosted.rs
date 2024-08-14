use std::sync::Arc;

use axum::response::{self, Response};
use derive_more::derive::Deref;
use http::{version, StatusCode};
use maven_rs::pom::Pom;
use nr_core::{
    database::{
        project::{DBProject, DBProjectVersion, NewProjectMember, ProjectDBType},
        repository::DBRepository,
    },
    repository::{
        config::{
            frontend::{BadgeSettings, BadgeSettingsType, Frontend, FrontendConfigType},
            get_repository_config_or_default, PushRulesConfig, PushRulesConfigType,
            RepositoryConfigType, SecurityConfig, SecurityConfigType,
        },
        Visibility,
    },
    storage::StoragePath,
    user::permissions::HasPermissions,
};
use nr_storage::{DynStorage, Storage};
use parking_lot::RwLock;
use tokio::io::AsyncReadExt;
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

use crate::{
    app::{authentication::RepositoryAuthentication, NitroRepo},
    repository::{
        maven::{self, MavenError, MavenRepositoryConfigType},
        Repository, RepositoryFactoryError, RepositoryHandlerError,
    },
};

use super::{RepoResponse, RepositoryRequest};
#[derive(derive_more::Debug)]
pub struct MavenHostedInner {
    pub id: Uuid,
    pub repository: RwLock<DBRepository>,
    pub push_rules: RwLock<PushRulesConfig>,
    pub security: RwLock<SecurityConfig>,
    pub frontend: RwLock<Frontend>,
    pub badge_settings: RwLock<BadgeSettings>,
    #[debug(skip)]
    pub storage: DynStorage,
    #[debug(skip)]
    pub site: NitroRepo,
}
impl MavenHostedInner {
    pub fn visibility(&self) -> Visibility {
        self.security.read().visibility
    }
}
#[derive(Debug, Clone, Deref)]
pub struct MavenHosted(Arc<MavenHostedInner>);
impl MavenHosted {
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
        let badge_settings_db =
            get_repository_config_or_default::<BadgeSettingsType, BadgeSettings>(
                repository.id,
                site.as_ref(),
            )
            .await?;
        debug!("Loaded Badge Settings Config: {:?}", badge_settings_db);
        let frontend_db = get_repository_config_or_default::<FrontendConfigType, Frontend>(
            repository.id,
            site.as_ref(),
        )
        .await?;
        debug!("Loaded Frontend Config: {:?}", frontend_db);
        let inner = MavenHostedInner {
            id: repository.id,
            repository: RwLock::new(repository),
            push_rules: RwLock::new(push_rules_db.value.0),
            security: RwLock::new(security_db.value.0),
            frontend: RwLock::new(frontend_db.value.0),
            badge_settings: RwLock::new(badge_settings_db.value.0),
            storage,
            site,
        };
        Ok(Self(Arc::new(inner)))
    }
    pub fn check_read(&self, authentication: &RepositoryAuthentication) -> Option<RepoResponse> {
        if self.visibility().is_private() {
            if authentication.is_no_identification() {
                return Some(RepoResponse::www_authenticate("Basic"));
            } else if !(authentication.can_read_repository(self.id)) {
                return Some(RepoResponse::forbidden());
            }
        }
        None
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

    fn get_type(&self) -> &'static str {
        "maven"
    }

    fn config_types(&self) -> Vec<&str> {
        vec![
            PushRulesConfigType::get_type_static(),
            SecurityConfigType::get_type_static(),
            BadgeSettingsType::get_type_static(),
            FrontendConfigType::get_type_static(),
            MavenRepositoryConfigType::get_type_static(),
        ]
    }

    async fn reload(&self) -> Result<(), RepositoryFactoryError> {
        let config = DBRepository::get_by_id(self.id, self.site.as_ref())
            .await?
            .ok_or(RepositoryFactoryError::LoadedRepositoryNotFound(self.id))?;
        {
            let mut repository = self.repository.write();
            *repository = config;
        }

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
        let badge_settings_db =
            get_repository_config_or_default::<BadgeSettingsType, BadgeSettings>(
                self.id,
                self.site.as_ref(),
            )
            .await?;
        let frontend_db = get_repository_config_or_default::<FrontendConfigType, Frontend>(
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
            let mut badge_settings = self.badge_settings.write();
            *badge_settings = badge_settings_db.value.0;
        }
        {
            let mut frontend = self.frontend.write();
            *frontend = frontend_db.value.0;
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
        if !self.repository.read().active {
            return Ok(RepoResponse::disabled_repository());
        }

        if let Some(err) = self.check_read(&authentication) {
            return Ok(err);
        }
        let visibility = self.visibility();
        let file = self.0.storage.open_file(self.id, &path).await?;
        if let Some(file) = &file {
            if file.is_directory()
                && visibility.is_hidden()
                && !authentication.can_read_repository(self.id)
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
        if !self.repository.read().active {
            return Ok(RepoResponse::disabled_repository());
        }
        let visibility = { self.security.read().visibility };

        if let Some(err) = self.check_read(&authentication) {
            return Ok(err);
        }
        let file = self.storage.get_file_information(self.id, &path).await?;
        if let Some(file) = &file {
            if file.is_directory()
                && visibility.is_hidden()
                && !authentication.can_read_repository(self.id)
            {
                return Ok(RepoResponse::indexing_not_allowed());
            }
        }
        Ok(RepoResponse::from(file))
    }
    #[instrument(name = "maven_hosted_put")]
    async fn handle_put(
        &self,
        RepositoryRequest {
            parts,
            body,
            path,
            authentication,
        }: RepositoryRequest,
    ) -> Result<RepoResponse, RepositoryHandlerError> {
        info!(
            "Handling PUT Request for Repository: {} Path: {}",
            self.id, path
        );
        let (save_path, user_id) = {
            let repository = self.repository.read();
            if !repository.active {
                return Ok(RepoResponse::disabled_repository());
            }
            let security = self.security.read();
            if security.must_use_auth_token_for_push && !authentication.has_auth_token() {
                info!("Repository requires an auth token for push");
                return Ok(RepoResponse::require_auth_token());
            }
            let Some(user) = authentication.get_user() else {
                info!("No acceptable user authentication provided");
                return Ok(RepoResponse::unauthorized());
            };
            if !user.can_write_to_repository(self.id) {
                info!(?repository, ?user, "User does not have write permissions");
                return Ok(RepoResponse::forbidden());
            }

            let save_path = format!(
                "/repositories/{}/{}/{}",
                self.storage.storage_config().storage_config.storage_name,
                repository.name,
                path
            );
            (save_path, user.id)
        };

        {
            let push_rules = self.push_rules.read();
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

        Ok(RepoResponse::put_response(created, save_path))
    }

    fn base_config(&self) -> DBRepository {
        self.0.repository.read().clone()
    }
}
