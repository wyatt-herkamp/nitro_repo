use axum::{
    extract::{Path, State},
    response::Response,
    routing::{get, post},
    Json,
};
use axum_extra::{headers::UserAgent, TypedHeader};
use http::StatusCode;
use nr_core::{
    database::user::{auth_token::NewAuthToken, UserType},
    user::{permissions::RepositoryActions, scopes::NRScope, token::AuthTokenFullResponse},
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    app::{
        authentication::OnlySessionAllowedAuthentication, responses::ResponseBuilderExt, NitroRepo,
    },
    error::InternalError,
};

pub fn token_routes() -> axum::Router<NitroRepo> {
    axum::Router::new()
        .route("/create", post(create))
        .route("/list", get(list))
        .route("/get/:id", get(get_token))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NewAuthTokenRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub scopes: Vec<NRScope>,
    #[serde(default)]
    pub repository_scopes: Vec<NewRepositoryScope>,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NewRepositoryScope {
    pub repository_id: Uuid,
    pub scopes: Vec<RepositoryActions>,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NewAuthTokenResponse {
    pub id: i32,
    pub token: String,
}
#[utoipa::path(
    post,
    path = "/token/create",
    request_body = NewAuthToken,
    responses(
        (status = 200, description = "A New Auth Token was created"),
    ),
)]
async fn create(
    auth: OnlySessionAllowedAuthentication,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    State(site): State<NitroRepo>,
    Json(new_token): Json<NewAuthTokenRequest>,
) -> Result<Response, InternalError> {
    let source = format!("API Request ({})", user_agent);
    if new_token.repository_scopes.len() == 0 && new_token.scopes.len() == 0 {
        return Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("No Scopes Provided".into())
            .unwrap());
    }
    let repositories: Vec<(Uuid, Vec<RepositoryActions>)> = new_token
        .repository_scopes
        .into_iter()
        .map(|scope| (scope.repository_id, scope.scopes))
        .collect();
    let new_token = NewAuthToken {
        user_id: auth.get_id(),
        name: new_token.name,
        description: new_token.description,
        source: source,
        scopes: new_token.scopes,
        repositories: repositories,
    };
    let (id, token) = new_token.insert(site.as_ref()).await?;
    let response = NewAuthTokenResponse { id, token };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .json_body(&response)
        .unwrap())
}
#[utoipa::path(
    get,
    path = "/token/list",
    responses(
        (status = 200, description = "A New Auth Token was created", body=[AuthTokenFullResponse]),
    ),
)]
#[instrument]
async fn list(
    auth: OnlySessionAllowedAuthentication,
    State(site): State<NitroRepo>,
) -> Result<Response, InternalError> {
    let tokens = AuthTokenFullResponse::get_all_for_user(auth.get_id(), site.as_ref()).await?;

    Response::builder()
        .status(StatusCode::OK)
        .json_body(&tokens)
}
#[utoipa::path(
    get,
    path = "/token/get/{id}",
    responses(
        (status = 200, description = "A New Auth Token was created", body=AuthTokenFullResponse),
    ),
)]
#[instrument]

async fn get_token(
    auth: OnlySessionAllowedAuthentication,
    Path(id): Path<i32>,
    State(site): State<NitroRepo>,
) -> Result<Response, InternalError> {
    let tokens =
        AuthTokenFullResponse::find_by_id_and_user_id(id, auth.get_id(), site.as_ref()).await?;

    Response::builder()
        .status(StatusCode::OK)
        .json_body(&tokens)
}
