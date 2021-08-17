use crate::repository::models::Repository;
use crate::DbPool;
use actix_web::{get, post, web, HttpRequest};
use crate::api_response::APIResponse;
use crate::siteerror::{SiteError, GenericError};
use crate::utils::{installed, get_current_time};
use crate::system::utils::get_user_by_header;
use crate::repository::action::{get_repositories, add_new_repository, get_repo_by_name_and_storage};
use serde::{Serialize, Deserialize};
use crate::siteerror::SiteError::NotFound;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositories {
    pub repositories: Vec<Repository>,
}

#[get("/api/admin/repositories/list")]
pub async fn list_servers(
    pool: web::Data<DbPool>,
    r: HttpRequest,
) -> Result<APIResponse<ListRepositories>, SiteError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let _user =
        get_user_by_header(r.headers(), &connection)?.ok_or_else(|| SiteError::NotAuthorized)?;
    let vec = get_repositories(&connection)?;

    let response = ListRepositories { repositories: vec };
    return Ok(APIResponse::new(true, Some(response)));
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NewRepo {
    pub name: String,
    pub storage: String,
    pub repo: String,
}

#[post("/api/admin/repository/add")]
pub async fn add_server(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    nc: web::Json<NewRepo>,
) -> Result<APIResponse<Repository>, SiteError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let _user =
        get_user_by_header(r.headers(), &connection)?.ok_or_else(|| SiteError::NotAuthorized)?;
    let option1 = crate::storage::action::get_storage_by_name(nc.storage.clone(), &connection)?.
        ok_or_else(|| SiteError::from("Unable to find storage"))?;
    let repository = Repository {
        id: 0,

        name: nc.name.clone(),
        repo_type: nc.repo.clone(),
        storage: option1.id.clone(),
        created: get_current_time(),
    };
    add_new_repository(&repository, &connection)?;
    let option = get_repo_by_name_and_storage(nc.name.clone(), option1.id, &connection)?.ok_or(NotFound)?;
    return Ok(APIResponse::new(true, Some(option)));
}
