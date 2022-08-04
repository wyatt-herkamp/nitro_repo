use crate::authentication::Authentication;
use crate::error::internal_error::InternalError;
use crate::repository::handler::Repository;

use crate::repository::response::RepoResponse;
use crate::repository::settings::{Policy, RepositoryConfig, RepositoryConfigType};
use crate::storage::file::StorageFileResponse;
use crate::storage::models::Storage;
use crate::system::permissions::options::CanIDo;

use actix_web::http::header::HeaderMap;
use actix_web::http::StatusCode;
use actix_web::web::Bytes;
use async_trait::async_trait;

use crate::repository::nitro::nitro_repository::NitroRepositoryHandler;
use crate::repository::settings::badge::BadgeSettings;
use crate::repository::settings::frontend::Frontend;

use crate::repository::maven::error::MavenError;
use crate::repository::maven::settings::MavenSettings;
use crate::repository::maven::validate_policy;
use log::error;
use maven_rs::pom::Pom;
use schemars::JsonSchema;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug)]
pub struct HostedMavenRepository<S: Storage> {
    pub config: RepositoryConfig,
    pub storage: Arc<S>,
    pub badge: BadgeSettings,
    pub frontend: Frontend,
    pub hosted: MavenHosted,
}

crate::repository::settings::define_configs_on_handler!(
    HostedMavenRepository<StorageType>,
    badge,
    BadgeSettings,
    frontend,
    Frontend,
    hosted,
    MavenHosted
);

impl<S: Storage> Clone for HostedMavenRepository<S> {
    fn clone(&self) -> Self {
        HostedMavenRepository {
            config: self.config.clone(),
            storage: self.storage.clone(),
            badge: self.badge.clone(),
            frontend: self.frontend.clone(),
            hosted: self.hosted.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct MavenHosted {
    pub allow_pushing: bool,
    #[serde(default)]
    policy: Policy,
}
impl RepositoryConfigType for MavenHosted {
    fn config_name() -> &'static str {
        "maven_hosted.json"
    }
}

impl Default for MavenHosted {
    fn default() -> Self {
        MavenHosted {
            allow_pushing: true,
            policy: Default::default(),
        }
    }
}

#[async_trait]
impl<S: Storage> Repository<S> for HostedMavenRepository<S> {
    fn get_repository(&self) -> &RepositoryConfig {
        &self.config
    }

    fn get_mut_config(&mut self) -> &mut RepositoryConfig {
        &mut self.config
    }

    fn get_storage(&self) -> &S {
        self.storage.as_ref()
    }
    #[inline(always)]
    fn features(&self) -> Vec<&'static str> {
        vec!["badge", "frontend", "hosted"]
    }
    async fn handle_get(
        &self,
        path: &str,
        _: &HeaderMap,
        conn: &DatabaseConnection,
        authentication: Authentication,
    ) -> Result<RepoResponse, actix_web::Error> {
        crate::helpers::read_check!(authentication, conn, self.config);

        match self
            .storage
            .get_file_as_response(&self.config, path)
            .await
            .map_err(InternalError::from)?
        {
            StorageFileResponse::List(list) => {
                let files = self.process_storage_files(list, path).await?;
                Ok(RepoResponse::try_from((files, StatusCode::OK))?)
            }
            value => Ok(RepoResponse::FileResponse(value)),
        }
    }

    async fn handle_put(
        &self,
        path: &str,
        _: &HeaderMap,
        conn: &DatabaseConnection,
        authentication: Authentication,
        bytes: Bytes,
    ) -> Result<RepoResponse, actix_web::Error> {
        let caller = crate::helpers::write_check!(authentication, conn, self.config);

        validate_policy(&self.hosted.policy, path)?;

        let exists = self
            .storage
            .save_file(&self.config, bytes.as_ref(), path)
            .await
            .map_err(InternalError::from)?;

        //  Post Deploy Handler
        if path.ends_with(".pom") {
            let vec = bytes.as_ref().to_vec();
            let result = String::from_utf8(vec).map_err(|_| MavenError::PomError)?;
            let pom: Pom =
                maven_rs::serde_xml_rs::from_str(&result).map_err(|_| MavenError::PomError)?;

            let project_folder = format!("{}/{}", pom.group_id.replace('.', "/"), pom.artifact_id);
            let version_folder = format!("{}/{}", &project_folder, &pom.version);
            if let Err(error) = self
                .post_deploy(project_folder, version_folder, caller, pom.into())
                .await
            {
                error!("Unable to complete post processing Tasks {}", error);
            }
        }
        // Everything was ok
        Ok(RepoResponse::PUTResponse(
            exists,
            format!(
                "/storages/{}/{}/{}",
                &self.storage.storage_config().generic_config.id,
                &self.config.name,
                path
            ),
        ))
    }
}
impl<S: Storage> NitroRepositoryHandler<S> for HostedMavenRepository<S> {
    #[inline(always)]
    fn parse_project_to_directory<V: Into<String>>(value: V) -> String {
        value.into().replace('.', "/").replace(':', "/")
    }
}
pub mod multi_web {
    crate::repository::maven::settings::macros::define_repository_config_handlers_group!(
        super::MavenHosted,
        maven_hosted,
        Hosted
    );
}
