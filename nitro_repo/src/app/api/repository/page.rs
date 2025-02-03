use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing::get,
};
use http::StatusCode;
use nr_core::{
    database::entities::repository::DBRepositoryConfig,
    repository::config::{
        repository_page::{RepositoryPage, RepositoryPageType},
        RepositoryConfigType,
    },
    user::permissions::{HasPermissions, RepositoryActions},
};
use tracing::instrument;
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{
    app::{
        authentication::Authentication,
        responses::{
            InvalidRepositoryConfig, MissingPermission, RepositoryNotFound, ResponseBuilderExt,
        },
        NitroRepo, RepositoryStorageName,
    },
    error::InternalError,
    repository::Repository,
};
#[derive(OpenApi)]
#[openapi(
    paths(get_repository_page_by_id, get_repository_page_by_names),
    components(schemas(RepositoryPage,))
)]
pub struct RepositoryPageRoutes;
pub fn page_apis() -> axum::Router<NitroRepo> {
    axum::Router::new()
        .route("/{repository_id}", get(get_repository_page_by_id))
        .route(
            "/{storage-name}/{repository-name}",
            get(get_repository_page_by_names),
        )
}
#[utoipa::path(
    get,
    path = "/{repository_id}",
    params(
        ("repository_id" = Uuid, Path, description = "The Repository Id"),
    ),
    responses(
        (status = 200, description = "Found Repository by Ids", body = RepositoryPage),
        (status = 404, description = "Repository not found"),
    )
)]
#[instrument]
pub async fn get_repository_page_by_id(
    State(site): State<NitroRepo>,
    auth: Option<Authentication>,
    Path(repository): Path<Uuid>,
) -> Result<Response, InternalError> {
    let Some(repository) = site.get_repository(repository) else {
        return Ok(RepositoryNotFound::Uuid(repository).into_response());
    };
    if repository.visibility().is_private()
        && !auth
            .has_action(RepositoryActions::Read, repository.id(), &site.database)
            .await?
    {
        return Ok(MissingPermission::EditRepository(repository.id()).into_response());
    }
    if !repository
        .config_types()
        .contains(&RepositoryPageType::get_type_static())
    {
        return Ok(InvalidRepositoryConfig::RepositoryTypeDoesntSupportConfig {
            repository_type: repository.get_type().to_owned(),
            config_key: RepositoryPageType::get_type_static().to_owned(),
        }
        .into_response());
    }
    let page = DBRepositoryConfig::<RepositoryPage>::get_config(
        repository.id(),
        RepositoryPageType::get_type_static(),
        site.as_ref(),
    )
    .await?
    .map(|x| x.value.0)
    .unwrap_or_default();
    Response::builder().status(StatusCode::OK).json_body(&page)
}
#[utoipa::path(
    get,
    path = "/{storage_name}/{repository_name}",
    params(
        ("storage-name" = String, Path, description = "The Storage Name"),
        ("repository-name" = String, Path, description = "The Repository Name"),
    ),
    responses(
        (status = 200, description = "Found Repository by Names", body = RepositoryPage),
        (status = 404, description = "Repository not found"),
    )
)]
#[instrument]
pub async fn get_repository_page_by_names(
    State(site): State<NitroRepo>,
    auth: Option<Authentication>,
    Path(names): Path<(String, String)>,
) -> Result<Response, InternalError> {
    let names: RepositoryStorageName = names.into();
    let Some(repository) = site.get_repository_from_names(&names).await? else {
        return Ok(RepositoryNotFound::RepositoryAndNameLookup(names).into_response());
    };
    if repository.visibility().is_private()
        && !auth
            .has_action(RepositoryActions::Read, repository.id(), &site.database)
            .await?
    {
        return Ok(MissingPermission::EditRepository(repository.id()).into_response());
    }
    if !repository
        .config_types()
        .contains(&RepositoryPageType::get_type_static())
    {
        return Ok(InvalidRepositoryConfig::RepositoryTypeDoesntSupportConfig {
            repository_type: repository.get_type().to_owned(),
            config_key: RepositoryPageType::get_type_static().to_owned(),
        }
        .into_response());
    }
    let page = DBRepositoryConfig::<RepositoryPage>::get_config(
        repository.id(),
        RepositoryPageType::get_type_static(),
        site.as_ref(),
    )
    .await?
    .map(|x| x.value.0)
    .unwrap_or_default();
    Response::builder().status(StatusCode::OK).json_body(&page)
}
