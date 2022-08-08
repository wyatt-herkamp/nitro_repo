use actix_web::{get, web, HttpResponse};

use log::warn;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

use crate::authentication::Authentication;
use crate::error::internal_error::InternalError;
use crate::generators::markdown::parse_to_html;
use crate::generators::GeneratorCache;

use crate::repository::handler::Repository;
use crate::repository::nitro::nitro_repository::NitroRepositoryHandler;

use crate::repository::settings::repository_page::{PageType, RepositoryPage};
use crate::repository::settings::{RepositoryConfig, RepositoryType, Visibility};
use crate::storage::models::Storage;
use crate::storage::multi::MultiStorageController;
use crate::storage::DynamicStorage;
use crate::system::permissions::options::CanIDo;
use crate::system::user::database::UserSafeData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicRepositoryResponse {
    pub name: String,
    pub repository_type: RepositoryType,
}

#[get("repositories/{storage_name}")]
pub async fn get_repositories(
    storage_handler: web::Data<MultiStorageController<DynamicStorage>>,
    database: web::Data<DatabaseConnection>,
    authentication: Authentication,
    path: web::Path<String>,
) -> actix_web::Result<HttpResponse> {
    let storage_name = path.into_inner();
    let value = crate::helpers::get_storage!(storage_handler, storage_name);

    let caller: Option<UserSafeData> = authentication.get_user(database.as_ref()).await?.ok();
    let vec: Vec<PublicRepositoryResponse> = value
        .get_repository_list()
        .map_err(actix_web::error::ErrorInternalServerError)?
        .into_iter()
        .filter(|repo| {
            if !repo.visibility.eq(&Visibility::Public) {
                match CanIDo::can_read_from(&caller, repo) {
                    Ok(ok) => ok.is_none(),
                    Err(error) => {
                        warn!("{}", error);
                        false
                    }
                }
            } else {
                true
            }
        })
        .map(|repo| PublicRepositoryResponse {
            name: repo.name.clone(),
            repository_type: repo.repository_type,
        })
        .collect();
    Ok(HttpResponse::Ok().json(vec))
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryPageResponse {
    pub name: String,
    pub repository_type: RepositoryType,
    pub page_content: String,
    pub last_updated: i64,
}
#[get("repositories/{storage_name}/{repository_name}")]
pub async fn get_repository(
    storage_handler: web::Data<MultiStorageController<DynamicStorage>>,
    database: web::Data<DatabaseConnection>,
    authentication: Authentication,
    path: web::Path<(String, String)>,
    generator: web::Data<GeneratorCache>,
) -> actix_web::Result<HttpResponse> {
    let (storage_name, repository_name) = path.into_inner();
    let storage = crate::helpers::get_storage!(storage_handler, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name);
    if !repository
        .get_repository()
        .visibility
        .eq(&Visibility::Public)
    {
        let caller = authentication.get_user(database.as_ref()).await??;
        if let Some(value) = caller.can_read_from(repository.get_repository())? {
            return Err(value.into());
        }
    }
    let page_content = get_readme::<DynamicStorage>(
        storage.as_ref(),
        repository.get_repository(),
        generator.into_inner(),
    )
    .await?;
    let v = if repository.supports_nitro() {
        if let Some(v) = repository.get_repository_listing().await? {
            v.last_updated
        } else {
            0
        }
    } else {
        0
    };
    let value = RepositoryPageResponse {
        name: repository.get_repository().name.clone(),
        repository_type: repository.get_repository().repository_type.clone(),
        page_content,
        last_updated: v,
    };

    Ok(HttpResponse::Ok().json(value))
}

pub async fn get_readme<StorageType: Storage>(
    storage: &StorageType,
    repo: &RepositoryConfig,
    generator: Arc<GeneratorCache>,
) -> Result<String, InternalError> {
    let data = repo
        .get_config::<RepositoryPage, StorageType>(storage)
        .await?;
    if let Some(data) = data {
        if PageType::None == data.page_type {
            Ok(String::new())
        } else {
            let cache_name = ".config.nitro_repo/README.html";
            if let Some(data) = generator.get_as_string(&cache_name).await? {
                Ok(data)
            } else {
                let option = storage
                    .get_file(repo, ".config.nitro_repo/README.md")
                    .await?;
                if let Some(data) = option {
                    let result = String::from_utf8(data.as_slice().to_vec())
                        .map_err(|e| InternalError::Error(e.to_string()))?;
                    parse_to_html(result, PathBuf::from(cache_name), generator)
                } else {
                    Ok(String::new())
                }
            }
        }
    } else {
        Ok(String::new())
    }
}
