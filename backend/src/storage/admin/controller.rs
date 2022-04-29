use actix_web::{delete, get, post, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::api_response::{APIResponse, SiteResponse};

use crate::error::internal_error::InternalError;
use crate::error::response::{already_exists};
use crate::storage::models::{Storage, StorageConfig, StorageHandler};
use crate::system::permissions::options::CanIDo;
use crate::utils::get_current_time;
use log::warn;
use sea_orm::DatabaseConnection;
use std::fs::{canonicalize, create_dir_all};
use std::ops::Deref;
use std::path::Path;
use crate::authentication::Authentication;
use crate::settings::models::StringMap;
use crate::system::user::UserModel;
use crate::NitroRepoData;
use crate::storage::local_storage::LocalStorage;
use crate::storage::{StorageHandlerType, StorageManager};

#[get("/api/storages/list")]
pub async fn list_storages(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,storages: web::Data<StorageManager>,
    r: HttpRequest, auth: Authentication,
) -> Result<HttpResponse, InternalError> {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;

    APIResponse::new(true, Some(serde_json::to_value(&storages).unwrap())).respond(&r)
}

#[delete("/api/admin/storages/{id}")]
pub async fn delete_by_id(
    connection: web::Data<DatabaseConnection>,
    r: HttpRequest, auth: Authentication,
    site: NitroRepoData, storages: web::Data<StorageManager>,
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
    site: NitroRepoData, auth: Authentication,storages: web::Data<StorageManager>,
    id: web::Path<String>,
) -> SiteResponse {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;

    return APIResponse::new(true, storages.get_storage_by_name(id.as_ref()).await?).respond(&r);
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewStorage {
    pub name: String,
    pub public_name: String,
}

#[post("/api/admin/storages/add")]
pub async fn add_storage(
    connection: web::Data<DatabaseConnection>,
    r: HttpRequest, auth: Authentication,
    nc: web::Json<NewStorage>,
    storages: web::Data<StorageManager>,
    site: NitroRepoData,
) -> SiteResponse {
    let caller: UserModel = auth.get_user(&connection).await??;
    caller.can_i_edit_repos()?;
    let path = Path::new("storages").join(&nc.0.name);
    if !path.exists() {
        create_dir_all(&path)?;
    }
    let path = canonicalize(path)?;
    let string = nc.0.name;
    let storage = Storage {
        config: StorageConfig {
            public_name: nc.0.public_name,
            name: string.clone(),
            created: get_current_time(),
        },
        storage_handler: StorageHandler::LocalStorage(LocalStorage {
            location: path.to_str().unwrap().to_string()
        }),
    };

    storages.create_storage(storage.clone()).await?;
    APIResponse::new(true, Some(storage)).respond(&r)
}
