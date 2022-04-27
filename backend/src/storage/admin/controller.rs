use actix_web::{delete, get, post, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::api_response::{APIResponse, SiteResponse};

use crate::error::internal_error::InternalError;
use crate::error::response::{already_exists, unauthorized};
use crate::storage::models::{save_storages, LocationType, Storage};
use crate::system::utils::get_user_by_header;
use crate::utils::get_current_time;
use crate::{NitroRepoData, StringMap};
use log::warn;
use std::fs::{canonicalize, create_dir_all};
use std::ops::Deref;
use std::path::Path;
use sea_orm::DatabaseConnection;
use crate::system::permissions::options::CanIDo;

#[get("/api/storages/list")]
pub async fn list_storages(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
) -> Result<HttpResponse, InternalError> {
    if get_user_by_header(r.headers(), &connection).await?.can_i_edit_repos().is_err() {
        return unauthorized();
    }
    let guard = site.storages.read().await;
    let values: Vec<Storage<StringMap>> = guard.values().cloned().collect();
    APIResponse::new(true, Some(values)).respond(&r)
}

#[delete("/api/admin/storages/{id}")]
pub async fn delete_by_id(
    connection: web::Data<DatabaseConnection>,
    r: HttpRequest,
    site: NitroRepoData,
    id: web::Path<String>,
) -> SiteResponse {
    if get_user_by_header(r.headers(), &connection).await?.can_i_edit_repos().is_err() {
        return unauthorized();
    }
    let mut guard = site.storages.write().await;
    if let Some(storage) = guard.remove(&id.into_inner()) {
        //Yes I am exporting everything being deleted
        warn!(
            " Deleted Storage {}",
            serde_json::to_string(&storage).unwrap()
        );
        save_storages(guard.deref())?;
        APIResponse::from(true).respond(&r)
    } else {
        APIResponse::from(false).respond(&r)
    }
}

#[get("/api/storages/id/{id}")]
pub async fn get_by_id(
    connection: web::Data<DatabaseConnection>,
    r: HttpRequest,
    site: NitroRepoData,
    id: web::Path<String>,
) -> SiteResponse {
    if get_user_by_header(r.headers(), &connection).await?.can_i_edit_repos().is_err() {
        return unauthorized();
    }
    let guard = site.storages.read().await;

    return APIResponse::new(true, guard.get(&id.into_inner())).respond(&r);
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
    nc: web::Json<NewStorage>,
    site: NitroRepoData,
) -> SiteResponse {
    if get_user_by_header(r.headers(), &connection).await?.can_i_edit_repos().is_err() {
        return unauthorized();
    }
    let mut guard = site.storages.write().await;
    for (_, storage) in guard.iter() {
        if storage.name.eq(&nc.name) || storage.public_name.eq(&nc.public_name) {
            return already_exists();
        }
    }
    let path = Path::new("storages").join(&nc.0.name);
    if !path.exists() {
        create_dir_all(&path)?;
    }
    let path = canonicalize(path)?;
    let string = nc.0.name;
    let storage = Storage {
        public_name: nc.0.public_name,
        name: string.clone(),
        created: get_current_time(),
        location_type: LocationType::LocalStorage,
        location: HashMap::from([("location".to_string(), path.to_str().unwrap().to_string())]),
    };
    guard.insert(string.clone(), storage);
    save_storages(guard.deref())?;

    APIResponse::new(true, guard.get(&string)).respond(&r)
}
