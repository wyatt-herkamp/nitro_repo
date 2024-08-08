use std::sync::Arc;

use derive_more::derive::Deref;
use nr_core::{
    database::repository::DBRepository,
    repository::{
        config::{
            frontend::{BadgeSettings, BadgeSettingsType, Frontend, FrontendConfigType},
            get_repository_config_or_default, PushRulesConfig, PushRulesConfigType,
            RepositoryConfigType, SecurityConfig, SecurityConfigType,
        },
        Visibility,
    },
    user::permissions::HasPermissions,
};
use nr_storage::{DynStorage, Storage};
use parking_lot::RwLock;
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::{
    app::NitroRepo,
    repository::{Repository, RepositoryFactoryError, RepositoryHandlerError},
};

use super::{RepoResponse, RepositoryRequest};
#[derive(Debug)]
pub struct MavenHostedInner {
    pub id: Uuid,
    pub repository: RwLock<DBRepository>,
    pub push_rules: RwLock<PushRulesConfig>,
    pub security: RwLock<SecurityConfig>,
    pub frontend: RwLock<Frontend>,
    pub badge_settings: RwLock<BadgeSettings>,
    pub storage: DynStorage,
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

        let visibility = { self.security.read().visibility };

        if visibility.is_private() && !authentication.can_read_repository(self.id) {
            return Ok(RepoResponse::Unauthorized);
        }

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

        if visibility.is_private() && !authentication.can_read_repository(self.id) {
            return Ok(RepoResponse::Unauthorized);
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
    #[instrument]
    async fn handle_put(
        &self,
        RepositoryRequest {
            parts,
            body,
            path,
            authentication,
        }: RepositoryRequest,
    ) -> Result<RepoResponse, RepositoryHandlerError> {
        let repository = self.repository.read();
        if !repository.active {
            return Ok(RepoResponse::disabled_repository());
        }
        let Some(user) = authentication.get_user() else {
            return Ok(RepoResponse::Unauthorized);
        };
        let security_config = self.security.read();
        let push_rules = self.push_rules.read();
        if security_config.must_use_auth_token_for_push && !authentication.has_auth_token() {
            return Ok(RepoResponse::Unauthorized);
        }
        if !user.can_write_to_repository(self.id) {
            return Ok(RepoResponse::Unauthorized);
        }
        let body = body.body_as_bytes().await?;
        // TODO: Validate Against Push Rules

        let (size, created) = self.storage.save_file(self.id, body.into(), &path).await?;
        // Trigger Push Event if it is the .pom file
        if path.has_extension("pom") {
            // TODO: Trigger Push Event
        }
        let save_path = format!(
            "/repositories/{}/{}/{}",
            self.storage.storage_config().storage_config.storage_name,
            repository.name,
            path
        );
        Ok(RepoResponse::put_response(created, save_path))
    }

    fn base_config(&self) -> DBRepository {
        self.0.repository.read().clone()
    }
}
