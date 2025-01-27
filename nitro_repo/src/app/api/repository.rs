use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    routing::get,
};
use management::NewRepositoryRequest;
use nr_core::{
    database::entities::repository::{
        DBRepository, DBRepositoryNames, DBRepositoryNamesWithVisibility,
        DBRepositoryWithStorageName,
    },
    repository::{
        browse::{BrowseFile, BrowseResponse},
        config::repository_page::{PageType, RepositoryPage},
        project::ProjectResolution,
        Visibility,
    },
    user::permissions::{HasPermissions, RepositoryActions},
};

use serde::Deserialize;
use tracing::instrument;
use utoipa::{IntoParams, OpenApi};
use uuid::Uuid;

use crate::{
    app::{
        authentication::Authentication,
        responses::{MissingPermission, RepositoryNotFound},
        NitroRepo,
    },
    error::InternalError,
    repository::RepositoryTypeDescription,
    utils::response_builder::ResponseBuilder,
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
        get_repository_names,
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
        BrowseResponse,
        ProjectResolution,
        DBRepositoryNames,
        DBRepositoryNamesWithVisibility
    ))
)]
pub struct RepositoryAPI;
pub fn repository_routes() -> axum::Router<NitroRepo> {
    axum::Router::new()
        .route("/list", get(list_repositories))
        .route("/{repository_id}", get(get_repository))
        .route("/{repository_id}/names", get(get_repository_names))
        .route("/page/{repository_id}", get(page::get_repository_page))
        .route("/types", get(types::repository_types))
        .merge(browse::browse_routes())
        .merge(management::management_routes())
        .merge(config::config_routes())
}

#[utoipa::path(
    get,
    path = "/{repository_id}",
    params(
        ("repository_id" = Uuid,Path, description = "The Repository ID"),
    ),
    responses(
        (status = 200, description = "Repository Types", body = DBRepositoryWithStorageName),
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

    Ok(ResponseBuilder::ok().json(&config))
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
    Ok(ResponseBuilder::ok().json(&repositories))
}
#[derive(Debug, Clone, Copy, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct QueryRepositoryNames {
    /// Rather or not to include the visibility of the repository
    #[serde(default)]
    #[param(default = false)]
    pub include_visibility: bool,
}
#[utoipa::path(
    get,
    path = "/{repository_id}/names",
    params(
        QueryRepositoryNames,
        ("repository_id" = Uuid, Path, description = "The Repository ID"),
    ),
    responses(
        (status = 200, description = "The Storage Name/ID and the Repository Name/ID for the given Repository ID", body = DBRepositoryNames),
        (status = 200, description = "The Storage Name/ID and the Repository Name/ID for the given Repository ID", body = DBRepositoryNamesWithVisibility),
        (status = 404, description = "Repository not found"),
        (status = 403, description = "Missing permission"),
    )
)]
#[instrument]
pub async fn get_repository_names(
    State(site): State<NitroRepo>,
    auth: Option<Authentication>,
    Query(query): Query<QueryRepositoryNames>,
    Path(repository_id): Path<Uuid>,
) -> Result<Response, InternalError> {
    let Some(repository) =
        DBRepositoryNamesWithVisibility::get_by_id(repository_id, site.as_ref()).await?
    else {
        return Ok(RepositoryNotFound::Uuid(repository_id).into_response());
    };
    if repository.visibility.is_private()
        && !auth
            .has_action(RepositoryActions::Read, repository_id, site.as_ref())
            .await?
    {
        return Ok(MissingPermission::ReadRepository(repository_id).into_response());
    }
    if query.include_visibility {
        Ok(ResponseBuilder::ok().json(&repository))
    } else {
        Ok(ResponseBuilder::ok().json(&DBRepositoryNames::from(repository)))
    }
}
