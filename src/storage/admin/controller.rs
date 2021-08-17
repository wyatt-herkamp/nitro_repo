use crate::repository::models::Repository;
use crate::DbPool;
use actix_web::{get, post, web, HttpRequest};
use crate::api_response::APIResponse;
use crate::apierror::{APIError, GenericError};
use crate::utils::{installed, get_current_time};
use crate::system::utils::get_user_by_header;
use crate::repository::action::{get_repositories, add_new_repository, get_repo_by_name_and_storage};
use serde::{Serialize, Deserialize};
use crate::apierror::APIError::NotFound;
use crate::storage::models::Storage;
use crate::storage::action::{get_storages, add_new_storage, get_storage_by_name};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListStorages {
    pub storages: Vec<Storage>,
}

#[get("/api/admin/storages/list")]
pub async fn list_storages(
    pool: web::Data<DbPool>,
    r: HttpRequest,
) -> Result<APIResponse<ListStorages>, APIError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let _user =
        get_user_by_header(r.headers(), &connection)?.ok_or_else(|| APIError::NotAuthorized)?;
    let vec = get_storages(&connection)?;

    let response = ListStorages { storages: vec };
    return Ok(APIResponse::new(true, Some(response)));
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewStorage {
    pub name: String,
}

#[post("/api/admin/storages/add")]
pub async fn add_server(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    nc: web::Json<NewStorage>,
) -> Result<APIResponse<Storage>, APIError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let _user =
        get_user_by_header(r.headers(), &connection)?.ok_or_else(|| APIError::NotAuthorized)?;

    let storage = Storage {
        id: 0,

        name: nc.name.clone(),
        created: get_current_time(),
    };
    add_new_storage(&storage, &connection)?;
    let option = get_storage_by_name(nc.name.clone(),  &connection)?.ok_or(NotFound)?;
    return Ok(APIResponse::new(true, Some(option)));
}
