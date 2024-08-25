use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use http::StatusCode;
use nr_core::{
    database::repository::DBRepositoryConfig,
    repository::config::{
        repository_page::{RepositoryPage, RepositoryPageType},
        RepositoryConfigType,
    },
    user::permissions::{HasPermissions, RepositoryActions},
};
use tracing::instrument;
use uuid::Uuid;

use crate::{
    app::{
        authentication::Authentication,
        responses::{
            InvalidRepositoryConfig, MissingPermission, RepositoryNotFound, ResponseBuilderExt,
        },
        NitroRepo,
    },
    error::InternalError,
    repository::Repository,
};

#[utoipa::path(
    get,
    path = "/page/{repository_id}",
    responses(
        (status = 200, description = "Create new Repository", body = RepositoryPage),
    )
)]
#[instrument]
pub async fn get_repository_page(
    State(site): State<NitroRepo>,
    auth: Option<Authentication>,
    Path(repository): Path<Uuid>,
) -> Result<Response, InternalError> {
    let Some(repository) = site.get_repository(repository) else {
        return Ok(RepositoryNotFound::Uuid(repository).into_response());
    };
    if repository.visibility().is_private() {
        if !auth
            .has_action(RepositoryActions::Read, repository.id(), &site.database)
            .await?
        {
            return Ok(MissingPermission::EditRepository(repository.id()).into_response());
        }
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
