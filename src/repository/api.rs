

use crate::repository::action::get_repo_by_name_and_storage;
use crate::repository::maven::MavenHandler;
use crate::repository::models::Repository;
use crate::repository::repository::{RepositoryRequest, RepositoryType};

use crate::storage::action::get_storage_by_name;


use crate::DbPool;

use actix_web::{get, web, HttpRequest};

use serde::{Deserialize, Serialize};
use crate::api_response::SiteResponse;
use crate::error::response::not_found;

use crate::repository::controller::handle_result;

//

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositories {
    pub repositories: Vec<Repository>,
}

#[get("/api/version/{storage}/{repository}/{file:.*}")]
pub async fn get_versions(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
) -> SiteResponse {
    let connection = pool.get()?;

    let storage = get_storage_by_name(&path.0.0, &connection)?;
    if storage.is_none() {
        return not_found();
    }
    let storage = storage.unwrap();
    let repository = get_repo_by_name_and_storage(&path.0.1, &storage.id, &connection)?;
    if repository.is_none(){
        return not_found();
    }
    let repository = repository.unwrap();

    let t = repository.repo_type.clone();
    let string = path.0.2.clone();

    let request = RepositoryRequest {
        storage,
        repository,
        value: string,
    };
    let x = match t.as_str() {
        "maven" => MavenHandler::handle_versions(&request, &r,&connection),
        _ => {
            panic!("Unknown REPO")
        }
    }?;
    return handle_result(x, path.0.2.clone(), r);
}
