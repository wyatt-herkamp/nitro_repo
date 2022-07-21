use actix_web::{get, web, HttpResponse};
use comrak::Arena;
use log::warn;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::authentication::Authentication;

use crate::repository::handler::Repository;

use crate::repository::settings::repository_page::{PageType, RepositoryPage};
use crate::repository::settings::{RepositoryType, Visibility};
use crate::storage::models::Storage;
use crate::storage::multi::MultiStorageController;
use crate::storage::DynamicStorage;
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;
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

    let caller: Option<UserModel> = authentication.get_user(database.as_ref()).await?.ok();
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
}
#[get("repositories/{storage_name}/{repository_name}")]
pub async fn get_repository(
    storage_handler: web::Data<MultiStorageController<DynamicStorage>>,
    database: web::Data<DatabaseConnection>,
    authentication: Authentication,
    path: web::Path<(String, String)>,
) -> actix_web::Result<HttpResponse> {
    let (storage_name, repository_name) = path.into_inner();
    let storage = crate::helpers::get_storage!(storage_handler, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name);
    if !repository
        .get_repository()
        .visibility
        .eq(&Visibility::Public)
    {
        let caller: UserModel = authentication.get_user(database.as_ref()).await??;
        if let Some(value) = caller.can_read_from(repository.get_repository())? {
            return Err(value.into());
        }
    }
    let repository_page: Option<RepositoryPage> = repository
        .get_repository()
        .get_config::<RepositoryPage, DynamicStorage>(&storage)
        .await?;
    let page_content = if let Some(value) = repository_page {
        match value.page_type {
            PageType::None => String::new(),
            PageType::Markdown(markdown) => {
                let arena = Arena::new();
                let html =
                    comrak::parse_document(&arena, &markdown, &comrak::ComrakOptions::default());
                let mut content = vec![];
                comrak::format_html(html, &comrak::ComrakOptions::default(), &mut content);
                String::from_utf8(content).unwrap()
            }
        }
    } else {
        String::new()
    };
    let value = RepositoryPageResponse {
        name: repository.get_repository().name.clone(),
        repository_type: repository.get_repository().repository_type.clone(),
        page_content,
    };

    Ok(HttpResponse::Ok().json(value))
}
