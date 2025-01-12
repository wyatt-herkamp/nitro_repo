use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::get,
};
use browse::{BrowseFile, BrowseResponse};
use http::{header::CONTENT_TYPE, StatusCode};
use management::NewRepositoryRequest;
use nr_core::{
    database::repository::{DBRepository, DBRepositoryWithStorageName},
    repository::{
        config::repository_page::{PageType, RepositoryPage},
        Visibility,
    },
    user::permissions::{HasPermissions, RepositoryActions},
};

use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{
    app::{
        authentication::Authentication,
        responses::{MissingPermission, RepositoryNotFound, ResponseBuilderExt},
        NitroRepo,
    },
    error::InternalError,
    repository::RepositoryTypeDescription,
};
mod browse;
mod config;
mod management;
mod page;
mod types;
#[derive(OpenApi)]
#[openapi(
    paths(
        list_repositories,
        get_repository,
        types::repository_types,
        config::config_schema,
        config::config_validate,
        config::config_default,
        config::config_description,
        management::new_repository,
        management::get_config,
        management::update_config,
        management::get_configs_for_repository,
        management::delete_repository,
        page::get_repository_page,
        browse::browse,
    ),
    components(schemas(
        DBRepository,
        DBRepositoryWithStorageName,
        RepositoryTypeDescription,
        RepositoryPage,
        NewRepositoryRequest,
        PageType,
        BrowseFile,
        BrowseResponse
    ))
)]
pub struct RepositoryAPI;
pub fn repository_routes() -> axum::Router<NitroRepo> {
    axum::Router::new()
        .route("/list", get(list_repositories))
        .route("/{id}", get(get_repository))
        .route("/page/{id}", get(page::get_repository_page))
        .route("/types", get(types::repository_types))
        .merge(browse::browse_routes())
        .merge(management::management_routes())
        .merge(config::config_routes())
}

#[utoipa::path(
    get,
    path = "/{repository_id}",
    responses(
        (status = 200, description = "Repository Types", body = DBRepository),
    )
)]
#[instrument]
pub async fn get_repository(
    State(site): State<NitroRepo>,
    auth: Option<Authentication>,
    Path(repository): Path<Uuid>,
) -> Result<Response, InternalError> {
    let Some(config) = DBRepositoryWithStorageName::get_by_id(repository, site.as_ref()).await?
    else {
        return Ok(RepositoryNotFound::Uuid(repository).into_response());
    };
    if config.visibility.is_private()
        && !auth
            .has_action(RepositoryActions::Read, repository, site.as_ref())
            .await?
    {
        return Ok(MissingPermission::ReadRepository(repository).into_response());
    }
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&config).unwrap().into())
        .unwrap();
    Ok(response)
}

#[utoipa::path(
    get,
    path = "/list",
    responses(
        (status = 200, description = "List Repositories", body = [DBRepositoryWithStorageName]),
    )
)]
#[instrument]
pub async fn list_repositories(
    auth: Option<Authentication>,
    State(site): State<NitroRepo>,
) -> Result<Response, InternalError> {
    let repositories: Vec<_> = DBRepositoryWithStorageName::get_all(site.as_ref())
        .await?
        .into_iter()
        .filter(|repo| match repo.visibility {
            Visibility::Private | Visibility::Public => true, // TODO FIX
            _ => true,
        })
        .collect();
    Response::builder()
        .status(StatusCode::OK)
        .json_body(&repositories)
}
