use actix_web::{get, post, delete, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::api_response::{APIResponse, SiteResponse};

use crate::database::DbPool;
use crate::error::internal_error::InternalError;
use crate::error::response::{already_exists, not_found, unauthorized};
use crate::storage::action::{add_new_storage, delete_storage_by_id, get_storage_by_id, get_storage_by_name, get_storage_by_public_name, get_storages};
use crate::storage::models::Storage;
use crate::system::utils::get_user_by_header;
use crate::utils::get_current_time;
use std::fs::create_dir_all;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListStorages {
    pub storages: Vec<Storage>,
}

#[get("/api/storages/list")]
pub async fn list_storages(
    pool: web::Data<DbPool>,
    r: HttpRequest,
) -> Result<HttpResponse, InternalError> {
    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let vec = get_storages(&connection)?;

    let response = ListStorages { storages: vec };
    APIResponse::new(true, Some(response)).respond(&r)
}
#[delete("/api/admin/storages/{id}")]
pub async fn delete_by_id(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    id: web::Path<i64>,
) -> SiteResponse {
    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let result = delete_storage_by_id(&id.into_inner(), &connection)?;

    APIResponse::new(true, Some(result)).respond(&r)
}
#[get("/api/storages/id/{id}")]
pub async fn get_by_id(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    id: web::Path<i64>,
) -> SiteResponse {
    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let vec = get_storage_by_id(&id.into_inner(), &connection)?;
    if vec.is_none() {
        return not_found();
    }

    APIResponse::new(true, vec).respond(&r)
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
) -> SiteResponse {
    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    if get_storage_by_public_name(&nc.public_name, &connection)?.is_some() {
        return already_exists();
    }
    if get_storage_by_name(&nc.name, &connection)?.is_some() {
        return already_exists();
    }
    let storage = Storage {
        id: 0,

        public_name: nc.0.public_name,
        name: nc.0.name,
        created: get_current_time(),
    };
    add_new_storage(&storage, &connection)?;
    let buf = PathBuf::new().join("storages").join(&storage.name);
    if !buf.exists() {
        create_dir_all(buf)?;
    }
    let option = get_storage_by_name(&storage.name, &connection)?;
    if option.is_none() {
        return not_found();
    }
    APIResponse::new(true, option).respond(&r)
}
