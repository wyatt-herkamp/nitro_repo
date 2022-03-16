use std::collections::HashMap;
use actix_web::{get, post, delete, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::api_response::{APIResponse, SiteResponse};

use crate::database::DbPool;
use crate::error::internal_error::InternalError;
use crate::error::response::{already_exists, unauthorized};
use crate::storage::models::{LocationType, save_storages, Storage};
use crate::system::utils::get_user_by_header;
use crate::utils::get_current_time;
use std::fs::{canonicalize, create_dir_all};
use std::ops::Deref;
use std::path::{Path};
use log::warn;
use uuid::Uuid;
use crate::{NitroRepoData};


#[get("/api/storages/list")]
pub async fn list_storages(
    pool: web::Data<DbPool>,
    site: NitroRepoData,
    r: HttpRequest,
) -> Result<HttpResponse, InternalError> {
    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let guard = site.storages.lock().unwrap();

    APIResponse::new(true, Some(guard.deref())).respond(&r)
}

#[delete("/api/admin/storages/{id}")]
pub async fn delete_by_id(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    site: NitroRepoData,
    id: web::Path<Uuid>,
) -> SiteResponse {
    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let mut guard = site.storages.lock().unwrap();
    if let Some(storage) = guard.remove(&id.into_inner()) {
        //Yes I am exporting everything being deleted
        warn!(" Deleted Storage {}",serde_json::to_string(&storage).unwrap());
        save_storages(guard.deref());
        APIResponse::new(true, Some(true)).respond(&r)
    } else {
        APIResponse::new(true, Some(false)).respond(&r)
    }
}

#[get("/api/storages/id/{id}")]
pub async fn get_by_id(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    site: NitroRepoData,
    id: web::Path<Uuid>,
) -> SiteResponse {
    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let guard = site.storages.lock().unwrap();
    return APIResponse::new(true, guard.get(&id.into_inner())).respond(&r);
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewStorage {
    pub name: String,
    pub public_name: String,
}

#[post("/api/admin/storages/add")]
pub async fn add_storage(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    nc: web::Json<NewStorage>,
    site: NitroRepoData,
) -> SiteResponse {
    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let mut guard = site.storages.lock().unwrap();
    for (id, storage) in guard.iter() {
        if storage.name.eq(&nc.name) || storage.public_name.eq(&nc.public_name) {
            return already_exists();
        }
    }
    let path = Path::new("storages").join(&nc.0.name);
    if !path.exists() {
        create_dir_all(&path);
    }
    let path = canonicalize(path)?;
    let uuid = Uuid::new_v4();
    let storage = Storage {
        public_name: nc.0.public_name,
        name: nc.0.name,
        created: get_current_time(),
        location_type: LocationType::LocalStorage,
        location: HashMap::from([("location".to_string(), path.to_str().unwrap().to_string())]),
    };
    guard.insert(uuid.clone(), storage);
    save_storages(guard.deref());

    APIResponse::new(true, guard.get(&uuid)).respond(&r)
}
