//! The token endpoint a Docker client is sent to.
//!
//! A registry that wants credentials answers `GET /v2/...` with `401` and
//! `WWW-Authenticate: Bearer realm="...",scope="repository:{image}:pull,push"`. The client then
//! calls the realm with its Basic credentials, gets a bearer token back, and retries. That is the
//! whole of `docker login`.
//!
//! The realm lives here, under `/api`, rather than under `/v2`: an image can be named anything,
//! including `token`, and `/api` is nested before the router fallback so it is reachable on a
//! custom Docker domain as well as on the instance host.
//!
//! The token minted is an ordinary Nitro auth token — the same rows `/api/user/token/create`
//! writes — so there is no second credential store to keep in step, and revoking a token from the
//! profile UI logs the Docker client out too.

use axum::{
    extract::{Query, State},
    response::Response,
};
use chrono::{Duration, Local};
use nr_core::{
    database::entities::user::auth_token::NewRepositoryToken, user::permissions::RepositoryActions,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};
use utoipa::{OpenApi, ToSchema};

// Still on `NitroRepo` rather than `SiteContext`: `resolve_repository` below has to turn a name or
// a hostname into a loaded repository, and resolving repositories is deliberately not something
// `SiteContext` can do (see its module docs on the reference cycle). This moves to the repository
// resolver along with `routing.rs`, which is blocked on the same thing.
use crate::{
    app::{NitroRepo, RepositoryStorageName},
    error::InternalError,
    repository::RepositoryAuthentication,
    utils::ResponseBuilder,
};

/// How long a minted token lives.
///
/// Long enough for a large push to finish, short enough that a token scraped out of a CI log is not
/// a standing key. Docker re-requests one whenever it gets a 401, so a short life is invisible.
const TOKEN_LIFETIME: Duration = Duration::minutes(30);

#[derive(OpenApi)]
#[openapi(paths(token), components(schemas(TokenResponse)))]
pub struct DockerAPI;

pub fn docker_routes() -> axum::Router<NitroRepo> {
    axum::Router::new().route("/token", axum::routing::get(token))
}

#[derive(Debug, Default, Deserialize)]
pub struct TokenQuery {
    /// `repository:{image}:{actions}` — what the client is about to do.
    pub scope: Option<String>,
    pub service: Option<String>,
    pub account: Option<String>,
    /// Set when the client only wants an anonymous token.
    #[serde(default)]
    pub offline_token: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TokenResponse {
    /// The spec names this `token`; the OAuth2 flow names the same value `access_token`. Both are
    /// sent because different clients read different ones.
    pub token: String,
    pub access_token: String,
    pub expires_in: i64,
    pub issued_at: String,
}

/// Exchanges Basic credentials for a bearer token.
#[utoipa::path(
    get,
    path = "/token",
    responses(
        (status = 200, description = "A bearer token for the requested scope", body = TokenResponse),
        (status = 401, description = "The credentials were not accepted"),
    )
)]
#[instrument(skip(site, query, authentication, parts))]
pub async fn token(
    State(site): State<NitroRepo>,
    Query(query): Query<TokenQuery>,
    authentication: RepositoryAuthentication,
    parts: http::request::Parts,
) -> Result<Response, InternalError> {
    // `RepositoryAuthentication` is the extractor the artifact routes use: it resolves Basic
    // credentials against the user table and, failing that, treats the password as an auth token.
    // That second attempt is what makes `docker login -u me -p <token>` work, which is the form
    // every CI pipeline uses.
    let Some(user) = authentication.get_user() else {
        debug!("Refused a Docker token request with no usable credentials");
        return Ok(unauthorized());
    };

    let (image, actions) = match query.scope.as_deref().and_then(parse_scope) {
        Some((image, actions)) => (Some(image), actions),
        // `docker login` with no scope: the client is only checking the credentials work. A token
        // scoped to nothing is still useful — it authenticates the next `/v2/` probe, which is what
        // makes `docker login` report success.
        None => (None, vec![RepositoryActions::Read]),
    };
    let repository = resolve_repository(&site, image.as_deref(), &parts).await;

    let expires_at = Local::now().fixed_offset() + TOKEN_LIFETIME;
    let source = format!(
        "Docker client ({})",
        query.service.as_deref().unwrap_or("-")
    );

    let new_token = match repository {
        Some(repository_id) => NewRepositoryToken {
            user_id: user.id,
            source,
            repositories: vec![(repository_id, actions)],
            expires_at: Some(expires_at),
        },
        None => NewRepositoryToken {
            user_id: user.id,
            source,
            repositories: Vec::new(),
            expires_at: Some(expires_at),
        },
    };
    let (_, secret) = new_token.insert(site.as_ref()).await?;

    Ok(ResponseBuilder::ok().json(&TokenResponse {
        access_token: secret.clone(),
        token: secret,
        expires_in: TOKEN_LIFETIME.num_seconds(),
        issued_at: Local::now().fixed_offset().to_rfc3339(),
    }))
}

fn unauthorized() -> Response {
    ResponseBuilder::unauthorized()
        .header(http::header::WWW_AUTHENTICATE, "Basic realm=\"nitro_repo\"")
        .json(&serde_json::json!({
            "errors": [{
                "code": "UNAUTHORIZED",
                "message": "the credentials were not accepted",
            }]
        }))
}

/// Parses `repository:{image}:{action},{action}`.
///
/// The image itself contains `/` but never `:`, so splitting on the first and last colon is
/// unambiguous.
fn parse_scope(scope: &str) -> Option<(String, Vec<RepositoryActions>)> {
    let rest = scope.strip_prefix("repository:")?;
    let (image, actions) = rest.rsplit_once(':')?;
    if image.is_empty() {
        return None;
    }
    let actions: Vec<RepositoryActions> = actions
        .split(',')
        .filter_map(|action| match action.trim() {
            "pull" => Some(RepositoryActions::Read),
            "push" => Some(RepositoryActions::Write),
            "delete" => Some(RepositoryActions::Write),
            _ => None,
        })
        .collect();
    if actions.is_empty() {
        return None;
    }
    Some((image.to_owned(), actions))
}

/// Finds the repository the caller is asking for a token against.
///
/// Two ways, matching the two ways a registry is addressed:
///
/// * **prefix mode** — the scope's image name begins with `{storage}/{repository}`;
/// * **hostname mode** — the image name carries no prefix, but the challenge sent the client to a
///   realm on the registry's own hostname, so the `Host` on this request identifies it.
///
/// `None` means the token is minted unscoped. That is not a way to widen access: every request is
/// still checked against the repository it names, and a token with no repository scope passes no
/// such check. It only keeps `docker login` working when there is nothing to scope to.
async fn resolve_repository(
    site: &NitroRepo,
    image: Option<&str>,
    parts: &http::request::Parts,
) -> Option<uuid::Uuid> {
    if let Some(image) = image {
        let mut components = image.split('/');
        if let (Some(storage), Some(repository)) = (components.next(), components.next()) {
            let names = RepositoryStorageName::from((storage.to_owned(), repository.to_owned()));
            // Through the cache-then-database lookup rather than the in-memory table alone: that
            // table is filled lazily, so a miss on a registry nobody has pulled from yet would mint
            // a token that grants nothing and make the first push of the day fail.
            if let Ok(Some(found)) = site.get_repository_from_names(&names).await
                && found.get_type() == super::REPOSITORY_TYPE_ID
            {
                return Some(found.id());
            }
        }
    }

    let host = crate::utils::host::request_host(
        &parts.headers,
        &parts.uri,
        site.general_security_settings.trust_forwarded_host,
    )?;
    site.repository_for_hostname(&host)
        .filter(|repository| repository.get_type() == super::REPOSITORY_TYPE_ID)
        .map(|repository| repository.id())
}

#[cfg(test)]
mod tests {
    use nr_core::user::permissions::RepositoryActions;

    use super::parse_scope;

    #[test]
    fn a_pull_push_scope_maps_onto_repository_actions() {
        let (image, actions) = parse_scope("repository:local/docker/alpine:pull,push").unwrap();
        assert_eq!(image, "local/docker/alpine");
        assert_eq!(
            actions,
            vec![RepositoryActions::Read, RepositoryActions::Write]
        );
    }

    #[test]
    fn a_pull_only_scope_grants_only_read() {
        let (image, actions) = parse_scope("repository:alpine:pull").unwrap();
        assert_eq!(image, "alpine");
        assert_eq!(actions, vec![RepositoryActions::Read]);
    }

    /// The image name contains slashes but never a colon, so the last colon separates the actions.
    #[test]
    fn a_multi_segment_image_name_survives_parsing() {
        let (image, _) = parse_scope("repository:a/b/c/d:pull").unwrap();
        assert_eq!(image, "a/b/c/d");
    }

    #[test]
    fn a_scope_that_names_nothing_useful_is_ignored() {
        for scope in [
            "",
            "registry:catalog:*",
            "repository:alpine",
            "repository::pull",
            "repository:alpine:nonsense",
        ] {
            assert!(parse_scope(scope).is_none(), "{scope:?}");
        }
    }
}
