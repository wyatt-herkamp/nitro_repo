use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
};
use http::StatusCode;
use nr_core::{
    database::{
        project::{DBProject, ProjectDBType},
        repository::DBRepositoryConfig,
    },
    repository::config::{
        project::{BadgeSettings, ProjectConfig, ProjectConfigType},
        RepositoryConfigType,
    },
};
use serde::Deserialize;
use utoipa::OpenApi;

use crate::{error::InternalError, repository::Repository};

use super::NitroRepo;
#[derive(OpenApi)]
#[openapi(
    paths(repository_badge, project_badge, supports_badges),
    components(schemas())
)]
pub struct BadgeRoutes;
pub fn badge_routes() -> axum::Router<NitroRepo> {
    axum::Router::new()
        .route(
            "/:storage/:repository",
            axum::routing::get(repository_badge),
        )
        .route(
            "/:storage/:repository/project/:project",
            axum::routing::get(project_badge),
        )
        .route(
            "/:storage/:repository/supports",
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
async fn supports_badges(
    Path(RepositoryBadgeRequest {
        storage,
        repository,
    }): Path<RepositoryBadgeRequest>,
    State(site): State<NitroRepo>,
) -> Result<Response, InternalError> {
    let Some(repo) = site
        .get_repository_from_names((storage, repository))
        .await
        .map_err(InternalError::from)?
    else {
        return Ok(Response::builder()
            .status(http::StatusCode::NOT_FOUND)
            .body("Repository not found".into())
            .unwrap());
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
    return Ok(Response::builder()
        .status(http::StatusCode::NO_CONTENT)
        .body(Body::empty())
        .unwrap());
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
async fn repository_badge(
    Path(RepositoryBadgeRequest {
        storage,
        repository,
    }): Path<RepositoryBadgeRequest>,
    State(site): State<NitroRepo>,
) -> Result<Response, InternalError> {
    let Some(repo) = site
        .get_repository_from_names((storage, repository))
        .await
        .map_err(InternalError::from)?
    else {
        return Ok(Response::builder()
            .status(http::StatusCode::NOT_FOUND)
            .body("Repository not found".into())
            .unwrap());
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

    let badge = match generate_badge(&badge_settings, "Repository", &repo.name()) {
        Ok(ok) => ok,
        Err(err) => {
            return Ok(Response::builder()
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Error generating badge: {}", err).into())
                .unwrap());
        }
    };

    Ok(Response::builder()
        .header(http::header::CONTENT_TYPE, "image/svg+xml")
        .status(StatusCode::OK)
        .body(badge.svg().into())
        .unwrap())
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
    responses(
        (status = 200, description = "Generates the Repository Badge", body = String)
    )
)]
async fn project_badge(
    Path(ProjectBadgeRequest {
        storage,
        repository,
        project,
    }): Path<ProjectBadgeRequest>,
    State(site): State<NitroRepo>,
) -> Result<Response, InternalError> {
    let Some(repo) = site
        .get_repository_from_names((storage, repository))
        .await
        .map_err(InternalError::from)?
    else {
        return Ok(Response::builder()
            .status(http::StatusCode::NOT_FOUND)
            .body("Repository not found".into())
            .unwrap());
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
        return Ok(Response::builder()
            .status(http::StatusCode::NOT_FOUND)
            .body("Project not found".into())
            .unwrap());
    };
    let latest_release = project
        .latest_release
        .as_deref()
        .or(project.latest_pre_release.as_deref())
        .unwrap_or("No Release");

    let badge = match generate_badge(&badge_settings, &project.name, &latest_release) {
        Ok(ok) => ok,
        Err(err) => {
            return Ok(Response::builder()
                .status(http::StatusCode::INTERNAL_SERVER_ERROR)
                .body(format!("Error generating badge: {}", err).into())
                .unwrap());
        }
    };

    Ok(Response::builder()
        .header(http::header::CONTENT_TYPE, "image/svg+xml")
        .status(StatusCode::OK)
        .body(badge.svg().into())
        .unwrap())
}

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
