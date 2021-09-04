use actix_web::{get, HttpRequest, post, web};
use serde::{Deserialize, Serialize};

use crate::api_response::APIResponse;
use crate::apierror::{APIError, GenericError};
use crate::apierror::APIError::NotFound;
use crate::DbPool;
use crate::repository::action::{add_new_repository, get_repo_by_name_and_storage, get_repositories};
use crate::repository::models::{Repository, RepositorySettings};
use crate::system::utils::get_user_by_header;
use crate::utils::{get_current_time, installed};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositories {
    pub repositories: Vec<Repository>,
}

#[get("/api/repositories/list")]
pub async fn list_repos(
    pool: web::Data<DbPool>,
    r: HttpRequest,
) -> Result<APIResponse<ListRepositories>, APIError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let user =
        get_user_by_header(r.headers(), &connection)?.ok_or_else(|| APIError::NotAuthorized)?;
    if !user.permissions.admin{
        return Err(APIError::NotAuthorized);
    }
    let vec = get_repositories(&connection)?;

    let response = ListRepositories { repositories: vec };
    return Ok(APIResponse::new(true, Some(response)));
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewRepo {
    pub name: String,
    pub storage: String,
    pub repo: String,
    pub settings: RepositorySettings,
}

#[post("/api/admin/repository/add")]
pub async fn add_repo(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    nc: web::Json<NewRepo>,
) -> Result<APIResponse<Repository>, APIError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let user =
        get_user_by_header(r.headers(), &connection)?.ok_or_else(|| APIError::NotAuthorized)?;
    if !user.permissions.admin{
        return Err(APIError::NotAuthorized);
    }
    let option1 = crate::storage::action::get_storage_by_name(nc.storage.clone(), &connection)?.
        ok_or_else(|| APIError::from("Unable to find storage"))?;
    let repository = Repository {
        id: 0,

        name: nc.name.clone(),
        repo_type: nc.repo.clone(),
        storage: option1.id.clone(),
        settings: nc.settings.clone(),
        created: get_current_time(),
    };
    add_new_repository(&repository, &connection)?;
    let option = get_repo_by_name_and_storage(nc.name.clone(), option1.id, &connection)?.ok_or(NotFound)?;
    return Ok(APIResponse::new(true, Some(option)));
}
