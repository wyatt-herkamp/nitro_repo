
use std::sync::Arc;


use actix_web::{delete, get, post, web, HttpResponse};
use lockfree::map::Removed;

use sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::authentication::Authentication;

use crate::error::internal_error::InternalError;

use crate::repository::handler::{Repository};

use crate::repository::settings::{Policy, Visibility};

use crate::repository::RepositoryType;

use crate::storage::models::Storage;
use crate::storage::multi::MultiStorageController;
use crate::storage::DynamicStorage;
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;
use paste::paste;
use serde_json::Value;


/// Get all repositories from the storage
#[get("/repositories/{storage_name}")]
pub async fn get_repositories(
    storage_handler: web::Data<MultiStorageController<DynamicStorage>>,
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
    storage_name: web::Path<String>,
) -> actix_web::Result<HttpResponse> {
    let user: UserModel = auth.get_user(&database).await??;
    user.can_i_edit_repos()?;

    let storage = crate::helpers::get_storage!(storage_handler, storage_name);

    Ok(HttpResponse::Ok().json(storage.get_repository_list().map_err(InternalError::from)?))
}

/// Create a new repository
#[post("/repositories/{storage_name}/new/{repository_name}/{repository_type}")]
pub async fn create_repository(
    storage_handler: web::Data<MultiStorageController<DynamicStorage>>,
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
    query_params: web::Path<(String, String, RepositoryType)>,
    _inner_config: web::Json<Option<Value>>,
) -> actix_web::Result<HttpResponse> {
    let user: UserModel = auth.get_user(&database).await??;
    user.can_i_edit_repos()?;
    let (storage_name, _repository_name, _repository_type) = query_params.into_inner();

    let _storage = crate::helpers::get_storage!(storage_handler, storage_name);
    todo!("Create Repository");
}

#[derive(Deserialize)]
pub struct GetRepositoryQuery {
    #[serde(default)]
    pub all_info: bool,
}

/// Get a repository by the name and storage name
/// If the query param all_info is present. It will include other repository configs such as Frontend and Badge
#[get("/repositories/{storage_name}/{repository_name}")]
pub async fn get_repository(
    storage_handler: web::Data<MultiStorageController<DynamicStorage>>,
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
    path_params: web::Path<(String, String)>,
    _query_params: web::Query<GetRepositoryQuery>,
) -> actix_web::Result<HttpResponse> {
    let user: UserModel = auth.get_user(&database).await??;
    user.can_i_edit_repos()?;
    let (storage_name, repository_name) = path_params.into_inner();
    let storage = crate::helpers::get_storage!(storage_handler, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name);

    Ok(HttpResponse::Ok().json(repository.get_repository()))
}

#[derive(Deserialize)]
pub struct DeleteRepositoryQuery {
    #[serde(default)]
    pub purge_repository: bool,
}

#[delete("/repositories/{storage_name}/{repository_name}")]
pub async fn delete_repository(
    storage_handler: web::Data<MultiStorageController<DynamicStorage>>,
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
    path_params: web::Path<(String, String)>,
    query_params: web::Query<DeleteRepositoryQuery>,
) -> actix_web::Result<HttpResponse> {
    let user: UserModel = auth.get_user(&database).await??;
    user.can_i_edit_repos()?;
    let (storage_name, repository_name) = path_params.into_inner();
    let storage = crate::helpers::get_storage!(storage_handler, storage_name);
    let _repository = crate::helpers::get_repository!(storage, repository_name);
    storage
        .delete_repository(repository_name, query_params.purge_repository)
        .await
        .map_err(InternalError::from)?;
    Ok(HttpResponse::NoContent().finish())
}

macro_rules! update_repository_core_prop {
    ($($name:ident,$value_type:tt),*) => {
        $(
        paste! {
        pub async fn [<update_repository_ $name>](
            storage_handler: web::Data<MultiStorageController<DynamicStorage>>,
            database: web::Data<DatabaseConnection>,
            auth: Authentication,
            path_params: web::Path<(String, String, $value_type)>,
        ) -> actix_web::Result<HttpResponse> {
            let user: UserModel = auth.get_user(&database).await??;
            user.can_i_edit_repos()?;
            let (storage_name, repository_name, value) = path_params.into_inner();
            let storage = crate::helpers::get_storage!(storage_handler, storage_name);
            let (name, mut repository) = crate::helpers::take_repository!(storage, repository_name);
            repository.get_mut_config().$name = value;
            storage.add_repository_for_updating(name, repository).expect("Failed to add repository for updating");

            Ok(HttpResponse::NoContent().finish())
        }
        }
        )*
        pub fn register_core_updates(cfg: &mut actix_web::web::ServiceConfig){
            $(
            paste! {
            cfg.service(actix_web::web::resource([concat!("/repositories/{storage}/{repository}/config/", stringify!($name), "{value}")])
                .route(actix_web::web::put().to([<update_repository_ $name>])));
            }
            )*
        }
    };
}
update_repository_core_prop!(visibility, Visibility, active, bool, policy, Policy);
