use actix_web::http::header::HeaderMap;
use actix_web::http::StatusCode;
use actix_web::web::Bytes;

use log::error;
use sea_orm::DatabaseConnection;

use crate::repository::maven::models::Pom;
use crate::repository::settings::security::Visibility;
use crate::repository::settings::Policy;

use crate::authentication::Authentication;
use crate::repository::data::RepositoryConfig;
use crate::repository::handler::RepositoryHandler;
use crate::repository::nitro::nitro_repository::NitroRepositoryHandler;

use crate::error::api_error::APIError;
use crate::error::internal_error::InternalError;
use crate::repository::response::RepoResponse;
use crate::storage::file::StorageFileResponse;
use crate::storage::models::Storage;
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;
use async_trait::async_trait;

use crate::storage::DynamicStorage;
use tokio::sync::RwLockReadGuard;

pub mod error;
pub mod models;
mod utils;

pub struct MavenHandler<'a> {
    config: RepositoryConfig,
    storage: RwLockReadGuard<'a, DynamicStorage>,
}

impl<'a> MavenHandler<'a> {
    pub fn create(
        repository: RepositoryConfig,
        storage: RwLockReadGuard<'a, DynamicStorage>,
    ) -> MavenHandler<'a> {
        MavenHandler::<'a> {
            config: repository,
            storage,
        }
    }
}

#[async_trait]
impl<'a> RepositoryHandler<'a> for MavenHandler<'a> {
    async fn handle_get(
        &self,
        path: &str,
        _: &HeaderMap,
        conn: &DatabaseConnection,
        authentication: Authentication,
    ) -> Result<RepoResponse, actix_web::Error> {
        if self.config.visibility == Visibility::Private {
            let caller: UserModel = authentication.get_user(conn).await??;
            if let Some(value) = caller.can_read_from(&self.config)? {
                return Err(value.into());
            }
        }

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
        let caller: UserModel = authentication.get_user(conn).await??;
        if let Some(_value) = caller.can_deploy_to(&self.config)? {}
        match self.config.policy {
            Policy::Release => {
                if path.contains("-SNAPSHOT") {
                    return Err(APIError::from((
                        "SNAPSHOT in release only",
                        StatusCode::BAD_REQUEST,
                    ))
                    .into());
                }
            }
            Policy::Snapshot => {
                if !path.contains("-SNAPSHOT") {
                    return Err(APIError::from((
                        "Release in a snapshot only",
                        StatusCode::BAD_REQUEST,
                    ))
                    .into());
                }
            }
            Policy::Mixed => {}
        }
        let exists = self
            .storage
            .save_file(&self.config, bytes.as_ref(), path)
            .await
            .map_err(InternalError::from)?;

        //  Post Deploy Handler
        if path.ends_with(".pom") {
            let vec = bytes.as_ref().to_vec();
            let result = String::from_utf8(vec).map_err(APIError::bad_request)?;
            let pom: Pom = serde_xml_rs::from_str(&result).map_err(APIError::bad_request)?;

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
                &self.storage.storage_config().name,
                &self.config.name,
                path
            ),
        ))
    }
}

impl NitroRepositoryHandler for MavenHandler<'_> {
    fn parse_project_to_directory<S: Into<String>>(value: S) -> String {
        value.into().replace('.', "/").replace(':', "/")
    }

    fn storage(&self) -> &DynamicStorage {
        &self.storage
    }

    fn repository(&self) -> &RepositoryConfig {
        &self.config
    }
}
