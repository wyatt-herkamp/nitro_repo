use actix_web::{delete, get, web, HttpResponse};

use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::authentication::Authentication;

use crate::error::internal_error::InternalError;

use crate::repository::handler::Repository;

use crate::repository::settings::{RepositoryConfigLayout, Visibility};

use crate::storage::models::Storage;
use crate::storage::multi::MultiStorageController;
use crate::storage::DynamicStorage;
use crate::system::permissions::permissions_checker::CanIDo;

use paste::paste;
use schemars::schema::RootSchema;

/// Get all repositories from the storage
#[get("/repositories/{storage_name}")]
pub async fn get_repositories(
    storage_handler: web::Data<MultiStorageController<DynamicStorage>>,
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
    storage_name: web::Path<String>,
) -> actix_web::Result<HttpResponse> {
    let user = auth.get_user(&database).await??;
    user.can_i_edit_repos()?;
    let storage = storage_name.into_inner();
    let storage = crate::helpers::get_storage!(storage_handler,storage);

    Ok(HttpResponse::Ok().json(storage.get_repository_list().map_err(InternalError::from)?))
}

#[derive(Serialize, Debug)]
pub struct CreateRepositoryLayout {
    pub name: &'static str,
    pub layout: RootSchema,
}
macro_rules! create_repository {
    ($($repository:ident,$repository_type:path, $repository_config:path),*) => {
        pub async fn repository_layout(_storage_handler: web::Data<MultiStorageController<DynamicStorage>>,
            database: web::Data<DatabaseConnection>,
            auth: Authentication) -> actix_web::Result<HttpResponse> {
                let user = auth.get_user(&database).await??;
                user.can_i_edit_repos()?;

                let layouts = vec![$(CreateRepositoryLayout{
                    name: stringify!($repository),
                    layout: schemars::schema_for!($repository_config),
                }),*];
                Ok(HttpResponse::Ok().json(layouts))
        }
            $(
        paste! {
        pub async fn [<create_repository_ $repository>](
            storage_handler: web::Data<MultiStorageController<DynamicStorage>>,
            database: web::Data<DatabaseConnection>,
            auth: Authentication,
            path_params: web::Path<(String, String)>,
            inner_config: web::Json<$repository_config>,
        ) -> actix_web::Result<HttpResponse> {
            let user = auth.get_user(&database).await??;
            user.can_i_edit_repos()?;
            let (storage_name, repository_name) = path_params.into_inner();
            let storage = crate::helpers::get_storage!(storage_handler, storage_name);
            use crate::repository::handler::CreateRepository;
            let (repository,config) = $repository_type::create_repository(
                inner_config.into_inner(),
                repository_name,
                storage.clone()
            )?;
           let repository = storage.create_repository(repository).await.map_err(InternalError::from)?;
            storage.save_repository_config(repository.get_repository(), &config).await.map_err(InternalError::from)?;
            Ok(HttpResponse::NoContent().finish())
        }
        }
            )*


        pub fn register_new_repos(cfg: &mut actix_web::web::ServiceConfig){
            $(
            paste! {
            cfg.service(actix_web::web::resource([concat!("/repositories/new/", stringify!($repository),"/{storage_name}/{repository_name}")])
                .route(actix_web::web::post().to([<create_repository_ $repository>])));
            }
            )*
            cfg.service(actix_web::web::resource("/tools/repositories/new/layout")
                .route(actix_web::web::get().to(repository_layout)));
        }
    };
}
create_repository!(
    maven,
    crate::repository::maven::MavenHandler,
    crate::repository::maven::settings::MavenSettings,
    npm,
    crate::repository::npm::NPMHandler,
    crate::repository::npm::NPMSettings,
    raw,
    crate::repository::raw::RawHandler,
    crate::repository::raw::RawSettings
);

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
    let user = auth.get_user(&database).await??;
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
    let user = auth.get_user(&database).await??;
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

#[get("/repositories/{storage_name}/{repository_name}/layout")]
pub async fn get_config_layout(
    storage_handler: web::Data<MultiStorageController<DynamicStorage>>,
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
    path_params: web::Path<(String, String)>,
) -> actix_web::Result<HttpResponse> {
    let user = auth.get_user(&database).await??;
    user.can_i_edit_repos()?;
    let (storage_name, repository_name) = path_params.into_inner();
    let storage = crate::helpers::get_storage!(storage_handler, storage_name);
    let repository = crate::helpers::get_repository!(storage, repository_name);
    let vec = repository.get_config_layout();
    Ok(HttpResponse::Ok().json(vec))
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
            let user = auth.get_user(&database).await??;
            user.can_i_edit_repos()?;
            let (storage_name, repository_name, value) = path_params.into_inner();
            let storage = crate::helpers::get_storage!(storage_handler, storage_name);
            let (name, mut repository) = crate::helpers::take_repository!(storage, repository_name);
            repository.get_mut_config().$name = value;
            storage.add_repository_for_updating(name, repository, true).await.expect("Failed to add repository for updating");

            Ok(HttpResponse::NoContent().finish())
        }
        }
        )*
        pub fn register_core_updates(cfg: &mut actix_web::web::ServiceConfig){
            $(
            paste! {
            cfg.service(actix_web::web::resource([concat!("/repositories/{storage}/{repository}/config/", stringify!($name), "/{value}")])
                .route(actix_web::web::put().to([<update_repository_ $name>])));
            }
            )*
        }
    };
}
update_repository_core_prop!(visibility, Visibility, active, bool, require_token_over_basic,bool);
