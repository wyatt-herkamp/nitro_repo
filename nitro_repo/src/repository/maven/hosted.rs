use std::sync::Arc;

use derive_more::derive::Deref;
use nr_core::{
    database::repository::DBRepository,
    repository::{
        config::{PushRulesConfig, SecurityConfig},
        Visibility,
    },
    user::permissions::HasPermissions,
};
use nr_storage::{DynStorage, Storage};
use tracing::instrument;
use uuid::Uuid;

use crate::repository::{
    http::{RepoResponse, RepositoryRequest},
    Repository, RepositoryHandlerError,
};
#[derive(Debug)]
pub struct MavenHostedInner {
    pub id: Uuid,
    pub repository: DBRepository,
    pub push_rules: PushRulesConfig,
    pub security: SecurityConfig,
    pub storage: DynStorage,
}
impl MavenHostedInner {
    pub fn visibility(&self) -> Visibility {
        self.security.visibility
    }
}
#[derive(Debug, Clone, Deref)]
pub struct MavenHosted(Arc<MavenHostedInner>);

impl Repository for MavenHosted {
    fn get_storage(&self) -> nr_storage::DynStorage {
        self.0.storage.clone()
    }

    fn get_type(&self) -> &'static str {
        "maven"
    }

    fn config_types(&self) -> Vec<String> {
        vec!["push_rules".to_string(), "security".to_string()]
    }

    fn reload(&self) {}

    async fn handle_get(
        &self,
        RepositoryRequest {
            parts,
            path,
            authentication,
            ..
        }: RepositoryRequest,
    ) -> Result<RepoResponse, RepositoryHandlerError> {
        if self.visibility().is_private() && !authentication.can_read_repository(self.repository.id)
        {
            return Ok(RepoResponse::Unauthorized);
        }

        let file = self.0.storage.open_file(self.id, &path).await?;
        if let Some(file) = &file {
            if file.is_directory()
                && self.visibility().is_hidden()
                && !authentication.can_read_repository(self.repository.id)
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
        if self.visibility().is_private() && !authentication.can_read_repository(self.repository.id)
        {
            return Ok(RepoResponse::Unauthorized);
        }
        let file = self.storage.get_file_information(self.id, &path).await?;
        if let Some(file) = &file {
            if file.is_directory()
                && self.visibility().is_hidden()
                && !authentication.can_read_repository(self.repository.id)
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
        let Some(user) = authentication.get_user() else {
            return Ok(RepoResponse::Unauthorized);
        };
        if self.security.must_use_auth_token_for_push && !authentication.has_auth_token() {
            return Ok(RepoResponse::Unauthorized);
        }
        if !user.can_write_to_repository(self.repository.id) {
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
            self.repository.name,
            path
        );
        Ok(RepoResponse::put_response(created, save_path))
    }
}
