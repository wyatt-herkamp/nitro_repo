use std::{path::PathBuf, sync::Arc};

use actix_web::{get, web, HttpResponse};
use chrono::{DateTime, FixedOffset, Local};
use log::warn;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::{
    authentication::{Authentication, TrulyAuthenticated},
    error::internal_error::InternalError,
    generators::{markdown::parse_to_html, GeneratorCache},
    repository::{
        handler::Repository,
        nitro::nitro_repository::NitroRepositoryHandler,
        settings::{
            repository_page::{PageType, RepositoryPage},
            RepositoryConfig, RepositoryType, Visibility,
        },
    },
    storage::{models::Storage, multi::MultiStorageController, DynamicStorage},
    system::{permissions::permissions_checker::CanIDo, user::database::UserSafeData},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicRepositoryResponse {
    pub name: String,
    pub repository_type: RepositoryType,
}

#[get("repositories/{storage_name}")]
pub async fn get_repositories(
    storage_handler: web::Data<MultiStorageController<DynamicStorage>>,
    database: web::Data<DatabaseConnection>,
    authentication: TrulyAuthenticated,
    path: web::Path<String>,
) -> actix_web::Result<HttpResponse> {
    let storage_name = path.into_inner();
    let value = crate::helpers::get_storage!(storage_handler, storage_name);

    let caller: UserSafeData = authentication.into_user();
    let vec: Vec<PublicRepositoryResponse> = value
        .get_repository_list()?
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

#[derive(Debug, Clone, Serialize)]
pub struct RepositoryPageResponse<'v> {
    pub name: &'v str,
    pub repository_type: &'v RepositoryType,
    pub page_content: Option<String>,
    pub last_updated: DateTime<FixedOffset>,
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
    crate::helpers::read_check_web!(
        authentication,
        database.as_ref(),
        repository.get_repository()
    );

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
            Local::now().into()
        }
    } else {
        Local::now().into()
    };
    let value = RepositoryPageResponse {
        name: repository.get_repository().name.as_str(),
        repository_type: &repository.get_repository().repository_type,
        page_content: page_content,
        last_updated: v,
    };

    Ok(HttpResponse::Ok().json(value))
}

pub async fn get_readme<StorageType: Storage>(
    storage: &StorageType,
    repo: &RepositoryConfig,
    generator: Arc<GeneratorCache>,
) -> Result<Option<String>, InternalError> {
    let data = repo
        .get_config::<RepositoryPage, StorageType>(storage)
        .await?;
    if let Some(data) = data {
        if PageType::None == data.page_type {
            Ok(None)
        } else {
            let cache_name = format!(
                "{}/{}/.config.nitro_repo/README.html",
                repo.storage, repo.name
            );
            if let Some(data) = generator.get_as_string(&cache_name).await? {
                Ok(Some(data))
            } else {
                let option = storage
                    .get_file(repo, ".config.nitro_repo/README.md")
                    .await?;
                if let Some(data) = option {
                    let result = String::from_utf8(data.as_slice().to_vec())
                        .map_err(|e| InternalError::Error(e.to_string()))?;
                    parse_to_html(result, PathBuf::from(cache_name), generator)
                        .map(|v| String::from_utf8(v).ok())
                } else {
                    Ok(None)
                }
            }
        }
    } else {
        Ok(None)
    }
}
