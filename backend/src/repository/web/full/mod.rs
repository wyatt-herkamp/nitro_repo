use actix_web::{HttpRequest, HttpResponse, web as ActixWeb, web};
use actix_web::http::StatusCode;
use log::trace;
use crate::api_response::{APIResponse, SiteResponse};
use crate::error::internal_error::InternalError;
use crate::error::response::{bad_request, i_am_a_teapot, not_found};
use crate::NitroRepoData;
use crate::repository::types::{RepoResponse, RepositoryRequest};
use crate::storage::multi::MultiStorageController;
use crate::utils::get_accept;

pub mod api;
pub mod badge;
pub mod controller;
pub mod admin;

pub fn init(cfg: &mut ActixWeb::ServiceConfig) {
    cfg.service(controller::browse_storage)
        .service(ActixWeb::resource(["/storages/", "/storages"]).to(controller::browse))
        .service(
            ActixWeb::resource([
                "/storages/{storage}/{repository}",
                "/storages/{storage}/{repository}/{file:.*}",
                "/storages/{storage}/{repository}/",
            ])
                .route(ActixWeb::get().to(controller::get_repository))
                .route(ActixWeb::put().to(controller::put_repository))
                .route(ActixWeb::head().to(controller::head_repository))
                .route(ActixWeb::patch().to(controller::patch_repository))
                .route(ActixWeb::post().to(controller::post_repository)),
        )
        .service(api::get_versions)
        .service(api::get_version)
        .service(api::get_project)
        .service(badge::badge)
        .service(api::get_repo);
}


pub async fn to_request(
    storage_name: String,
    repo_name: String,
    file: String,
    storages: web::Data<MultiStorageController>,
) -> Result<RepositoryRequest, InternalError> {
    let storage = storages.get_storage_by_name(&storage_name).await?;
    if storage.is_none() {
        trace!("Storage {} not found", &storage_name);
        return Err(InternalError::NotFound);
    }
    let storage = storage.unwrap().clone();
    let repository = storage.get_repository(&repo_name).await?;
    if repository.is_none() {
        trace!("Repository {} not found", repo_name);
        return Err(InternalError::NotFound);
    }
    let repository = repository.unwrap();

    let request = RepositoryRequest {
        storage,
        repository,
        value: file,
    };
    Ok(request)
}

pub fn handle_result(response: RepoResponse, r: HttpRequest) -> SiteResponse {
    let x = get_accept(r.headers())?.unwrap_or_else(|| "text/html".to_string());
    return match response {
        RepoResponse::FileList(files) => {
            if x.contains(&"application/json".to_string()) {
                APIResponse::new(true, Some(files)).respond(&r)
            } else {
                trace!("Access to Simple Dir Listing {}", r.uri());
                Ok(HttpResponse::Ok()
                    .content_type("text/html")
                    .body("Coming Soon(Simple DIR Listing)"))
            }
        }
        RepoResponse::FileResponse(file) => file,
        RepoResponse::Ok => APIResponse::new(true, Some(false)).respond(&r),
        RepoResponse::NotFound => {
            if x.contains(&"application/json".to_string()) {
                return not_found();
            } else {
                Ok(HttpResponse::NotFound()
                    .content_type("text/html")
                    .body("NOT FOUND"))
            }
        }

        RepoResponse::NotAuthorized => {
            let r = APIResponse::<bool> {
                success: false,
                data: None,
                status_code: Some(401),
            };
            let result = HttpResponse::Ok()
                .status(StatusCode::UNAUTHORIZED)
                .content_type("application/json")
                .append_header(("WWW-Authenticate", "Basic realm=nitro_repo"))
                .body(serde_json::to_string(&r).unwrap());
            return Ok(result);
        }
        RepoResponse::BadRequest(e) => {
            return bad_request(e);
        }
        RepoResponse::IAmATeapot(e) => {
            return i_am_a_teapot(e);
        }
        RepoResponse::ProjectResponse(project) => APIResponse::new(true, Some(project)).respond(&r),
        RepoResponse::VersionListingResponse(versions) => {
            APIResponse::new(true, Some(versions)).respond(&r)
        }
        RepoResponse::CreatedWithJSON(json) => {
            let result = HttpResponse::Ok()
                .status(StatusCode::CREATED)
                .content_type("application/json")
                .body(json);
            return Ok(result);
        }
        RepoResponse::OkWithJSON(json) => {
            let result = HttpResponse::Ok()
                .status(StatusCode::OK)
                .content_type("application/json")
                .body(json);
            return Ok(result);
        }
        RepoResponse::NitroVersionListingResponse(value) => {
            APIResponse::new(true, Some(value)).respond(&r)
        }
        RepoResponse::NitroVersionResponse(value) => {
            APIResponse::new(true, Some(value)).respond(&r)
        }
        RepoResponse::NitroProjectResponse(value) => {
            APIResponse::new(true, Some(value)).respond(&r)
        }

        RepoResponse::NitroFileList(value) => APIResponse::new(true, Some(value)).respond(&r),
    };
}
