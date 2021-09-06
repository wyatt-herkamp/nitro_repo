use actix_web::{get, post, web, HttpRequest};
use serde::{Deserialize, Serialize};

use crate::api_response::APIResponse;
use crate::apierror::APIError::NotFound;
use crate::apierror::{APIError};


use crate::storage::action::{add_new_storage, get_storage_by_name, get_storages};
use crate::storage::models::Storage;
use crate::system::utils::get_user_by_header;
use crate::utils::{get_current_time, installed};
use crate::DbPool;
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
) -> Result<APIResponse<ListStorages>, APIError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let user =
        get_user_by_header(r.headers(), &connection)?.ok_or_else(|| APIError::NotAuthorized)?;
    if !user.permissions.admin {
        return Err(APIError::NotAuthorized);
    }
    let vec = get_storages(&connection)?;

    let response = ListStorages { storages: vec };
    return Ok(APIResponse::new(true, Some(response)));
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
) -> Result<APIResponse<Storage>, APIError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let user =
        get_user_by_header(r.headers(), &connection)?.ok_or_else(|| APIError::NotAuthorized)?;
    if !user.permissions.admin {
        return Err(APIError::NotAuthorized);
    }
    let storage = Storage {
        id: 0,

        public_name: nc.public_name.clone(),
        name: nc.name.clone(),
        created: get_current_time(),
    };
    add_new_storage(&storage, &connection)?;
    let buf = PathBuf::new().join("storages").join(nc.name.clone());
    if !buf.exists() {
        create_dir_all(buf)?;
    }
    let option = get_storage_by_name(nc.name.clone(), &connection)?.ok_or(NotFound)?;
    return Ok(APIResponse::new(true, Some(option)));
}
