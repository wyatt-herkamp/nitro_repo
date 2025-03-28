use crate::{error::InternalError, repository::Repository, utils::ResponseBuilder};
use axum::{
    body::Body,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use nr_core::{
    database::entities::{
        project::{DBProject, ProjectDBType, versions::DBProjectVersion},
        repository::DBRepositoryConfig,
    },
    repository::{
        config::{
            RepositoryConfigType,
            project::{BadgeSettings, ProjectConfig, ProjectConfigType},
        },
        project::ReleaseType,
    },
};
use serde::Deserialize;
use tracing::instrument;
use tracing::{Level, event};
use utoipa::{IntoParams, OpenApi};

use super::{NitroRepo, RepositoryStorageName, responses::RepositoryNotFound};
#[derive(OpenApi)]
#[openapi(
    paths(repository_badge, project_badge, supports_badges),
    components(schemas())
)]
pub struct BadgeRoutes;
pub fn badge_routes() -> axum::Router<NitroRepo> {
    axum::Router::new()
        .route(
            "/{storage}/{repository}",
            axum::routing::get(repository_badge),
        )
        .route(
            "/{storage}/{repository}/project/{project}",
            axum::routing::get(project_badge),
        )
        .route(
            "/{storage}/{repository}/supports",
            axum::routing::get(supports_badges),
        )
}
#[utoipa::path(
    get,
    path = "/{storage}/{repository}/supports",
    responses(
        (status = 204, description = "This Repository Supports Badges"),
        (status = 400, description = "This Repository does not support badges"),
        (status = 404, description = "Repository not found")
    )
)]
#[instrument]
async fn supports_badges(
    Path(RepositoryBadgeRequest {
        storage,
        repository,
    }): Path<RepositoryBadgeRequest>,
    State(site): State<NitroRepo>,
) -> Result<Response, InternalError> {
    let names = RepositoryStorageName::from((storage, repository));
    let Some(repo) = site.get_repository_from_names(&names).await? else {
        return Ok(RepositoryNotFound::RepositoryAndNameLookup(names).into_response());
    };
    if !repo
        .config_types()
        .contains(&ProjectConfigType::get_type_static())
    {
        return Ok(Response::builder()
            .status(http::StatusCode::BAD_REQUEST)
            .body("Repository does not have a project config".into())
            .unwrap());
    }
    Ok(Response::builder()
        .status(http::StatusCode::NO_CONTENT)
        .body(Body::empty())
        .unwrap())
}
#[derive(Deserialize)]
struct RepositoryBadgeRequest {
    pub storage: String,
    pub repository: String,
}
#[utoipa::path(
    get,
    path = "/{storage}/{repository}",
    responses(
        (status = 200, description = "Generates the Repository Badge", body = String)
    )
)]
#[instrument]
async fn repository_badge(
    Path(RepositoryBadgeRequest {
        storage,
        repository,
    }): Path<RepositoryBadgeRequest>,
    State(site): State<NitroRepo>,
) -> Result<Response, InternalError> {
    let names = RepositoryStorageName::from((storage, repository));
    let Some(repo) = site.get_repository_from_names(&names).await? else {
        return Ok(RepositoryNotFound::RepositoryAndNameLookup(names).into_response());
    };
    if !repo
        .config_types()
        .contains(&ProjectConfigType::get_type_static())
    {
        return Ok(ResponseBuilder::bad_request().body("This Repository does not support badges"));
    }

    let badge_settings = DBRepositoryConfig::<ProjectConfig>::get_config(
        repo.id(),
        ProjectConfigType::get_type_static(),
        site.as_ref(),
    )
    .await
    .map_err(InternalError::from)?
    .map(|c| c.value.0.badge_settings)
    .unwrap_or_default();

    let badge = match generate_badge(&badge_settings, "Repository", &repo.name()) {
        Ok(ok) => ok,
        Err(err) => {
            return Ok(ResponseBuilder::internal_server_error()
                .body(format!("Error generating badge: {}", err)));
        }
    };

    Ok(ResponseBuilder::ok()
        .content_type(mime::IMAGE_SVG)
        .body(badge.svg()))
}
#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
struct BadgeQuery {
    release_type: Option<Vec<ReleaseType>>,
}
#[derive(Deserialize)]
struct ProjectBadgeRequest {
    pub storage: String,
    pub repository: String,
    pub project: String,
}
#[utoipa::path(
    get,
    path = "/{storage}/{repository}/project/{project}",
    params(BadgeQuery),
    responses(
        (status = 200, description = "Generates the Repository Badge", body = String)
    )
)]
#[instrument]
async fn project_badge(
    Path(ProjectBadgeRequest {
        storage,
        repository,
        project,
    }): Path<ProjectBadgeRequest>,
    Query(query): Query<BadgeQuery>,
    State(site): State<NitroRepo>,
) -> Result<Response, InternalError> {
    let names = RepositoryStorageName::from((storage, repository));
    let Some(repo) = site.get_repository_from_names(&names).await? else {
        return Ok(RepositoryNotFound::RepositoryAndNameLookup(names).into_response());
    };
    if !repo
        .config_types()
        .contains(&ProjectConfigType::get_type_static())
    {
        return Ok(Response::builder()
            .status(http::StatusCode::BAD_REQUEST)
            .body("This Repository does not support badges".into())
            .unwrap());
    }

    let badge_settings = DBRepositoryConfig::<ProjectConfig>::get_config(
        repo.id(),
        ProjectConfigType::get_type_static(),
        site.as_ref(),
    )
    .await
    .map_err(InternalError::from)?
    .map(|c| c.value.0.badge_settings)
    .unwrap_or_default();
    let Some(project) = DBProject::find_by_project_key(&project, repo.id(), site.as_ref())
        .await
        .map_err(InternalError::from)?
    else {
        return Ok(ResponseBuilder::not_found().body("Project not found"));
    };
    let mut release_types = query
        .release_type
        .unwrap_or_else(|| vec![ReleaseType::Stable]);
    if release_types.is_empty() {
        release_types.push(ReleaseType::Stable);
    }
    let latest_releases: Vec<DBProjectVersion> = project
        .find_version_by_release_type(release_types, site.as_ref())
        .await?;

    let latest_release = if let Some(version) = latest_releases.get(0) {
        event!(Level::DEBUG, ?version, "Found latest version");
        version.version.to_owned()
    } else if let Some(version) = project.find_latest_version(site.as_ref()).await? {
        event!(
            Level::INFO,
            ?version,
            "Requested Version not found. Using latest version"
        );
        version.version.to_owned()
    } else {
        event!(Level::INFO, "No releases found");
        "No Release".to_owned()
    };
    let badge = match generate_badge(&badge_settings, &project.name, &latest_release) {
        Ok(ok) => ok,
        Err(err) => {
            return Ok(ResponseBuilder::internal_server_error()
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Error generating badge: {}", err)));
        }
    };

    Ok(ResponseBuilder::ok()
        .content_type(mime::IMAGE_SVG)
        .body(badge.svg()))
}
#[instrument]
fn generate_badge(
    settings: &BadgeSettings,
    label: &str,
    value: &str,
) -> Result<badge_maker::Badge, badge_maker::error::Error> {
    badge_maker::BadgeBuilder::new()
        .label_color_parse(&settings.label_color)
        .color_parse(&settings.color)
        .style(settings.style.0)
        .label(label)
        .message(value)
        .build()
}
