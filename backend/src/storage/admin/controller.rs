use actix_web::{delete, get, post, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::api_response::{APIResponse, SiteResponse};

use crate::error::internal_error::InternalError;

use crate::storage::models::{Storage, StorageConfig, UnloadedStorage};
use crate::system::permissions::options::CanIDo;
use crate::utils::get_current_time;

use sea_orm::DatabaseConnection;
use std::fs::{canonicalize, create_dir_all};

use crate::authentication::Authentication;
use std::path::Path;

use crate::storage::local_storage::{LocalConfig, LocalStorage};
use crate::system::user::UserModel;
use crate::NitroRepoData;
use crate::storage::handler::StorageHandlerFactory;
use crate::storage::multi::MultiStorageController;

#[get("/api/storages/list")]
pub async fn list_storages(
    connection: web::Data<DatabaseConnection>,
    _site: NitroRepoData,
    storages: web::Data<MultiStorageController>,
    r: HttpRequest,
    auth: Authentication,
) -> Result<HttpResponse, InternalError> {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;

    APIResponse::new(true, Some(serde_json::to_value(&storages).unwrap())).respond(&r)
}

#[delete("/api/admin/storages/{id}")]
pub async fn delete_by_id(
    connection: web::Data<DatabaseConnection>,
    r: HttpRequest,
    auth: Authentication,
    _site: NitroRepoData,
    storages: web::Data<MultiStorageController>,
    id: web::Path<String>,
) -> SiteResponse {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;
    if storages.delete_storage(id.as_str()).await? {
        APIResponse::from(true).respond(&r)
    } else {
        APIResponse::from(false).respond(&r)
    }
}

#[get("/api/storages/id/{id}")]
pub async fn get_by_id(
    connection: web::Data<DatabaseConnection>,
    r: HttpRequest,
    _site: NitroRepoData,
    auth: Authentication,
    storages: web::Data<MultiStorageController>,
    id: web::Path<String>,
) -> SiteResponse {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;

    let option = storages.get_storage_by_name(id.as_ref()).await?;
    if option.is_none() {
        return APIResponse::new(true, Some(false)).respond(&r);
    }
    //TODO serialize
    return APIResponse::new(true, Some(false)).respond(&r);
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewStorage {
    pub name: String,
    pub public_name: String,
}

#[post("/api/admin/storages/add")]
pub async fn add_storage(
    connection: web::Data<DatabaseConnection>,
    r: HttpRequest,
    auth: Authentication,
    nc: web::Json<NewStorage>,
    storages: web::Data<MultiStorageController>,
    _site: NitroRepoData,
) -> SiteResponse {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;
    let path = Path::new("storages").join(&nc.0.name);
    if !path.exists() {
        create_dir_all(&path)?;
    }
    let path = canonicalize(path)?;
    let string = nc.0.name;
    let config = LocalConfig {
        location: "".to_string()
    };
    let storage = UnloadedStorage {
        config: StorageConfig {
            public_name: nc.0.public_name,
            name: string.clone(),
            created: get_current_time(),
        },
        storage_handler: StorageHandlerFactory {
            storage_type: "local".to_string(),
            config: serde_json::to_value(config)?,
        },
    };

    storages.create_storage(storage.clone()).await?;
    APIResponse::new(true, Some(storage)).respond(&r)
}
