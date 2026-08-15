//! `/v2` at the host root.
//!
//! Every other repository type is reachable only under `/repositories/{storage}/{repository}`,
//! because every other client can be told a URL. A Docker client cannot: `docker pull host/x/y`
//! requests `https://host/v2/x/y/manifests/...` and there is no configuration that adds a prefix.
//!
//! So this router is mounted at the host root, *after* `/repositories`, `/api` and `/badge` and
//! *before* the host-routing fallback. It resolves the repository in two ways, in this order:
//!
//! 1. **By host.** The request arrived on a hostname registered to a Docker repository, so the
//!    whole path after `/v2` is the Docker sub-path and image names need no prefix:
//!    `docker pull docker.example.com/alpine`.
//! 2. **By name prefix.** The first two segments of the image name are the storage and the
//!    repository: `docker pull nitro.example.com/local/docker/alpine`.
//!
//! A host that resolves to a repository which is *not* a Docker one falls through to the normal
//! host dispatch, so an artifact whose path happens to begin with `v2/` is still served over a
//! custom domain — `/v2` does not become a fourth entry in the "cannot be served over a custom
//! domain" list in [`host_routing`](the application's host routing).

use axum::{
    extract::{FromRequestParts, Request},
    response::{IntoResponse, Response},
};
use http::{HeaderMap, Uri};
use nr_core::storage::StoragePath;
use nr_repository::{
    DynRepository, RepositoryAuthentication, RepositoryRouterState, RepositoryStorageName,
    dispatch_repository_request,
};
use nr_web_core::utils::{host::request_host, request_logging::request_span::RequestSpan};
use percent_encoding::percent_decode_str;
use tracing::{Span, debug};

use super::{DockerError, errors::ErrorCode};

/// The routes `/v2` needs. The application mounts these at the host root, after `/repositories`,
/// `/api` and `/badge` and before its own fallback.
///
/// Exported as patterns rather than as a `Router` so that this crate does not have to know what to
/// do when a request turns out not to be its own — see [`try_handle_v2`].
pub const V2_ROUTES: &[&str] = &["/v2", "/v2/", "/v2/{*path}"];

/// Serves a `/v2` request, or hands it back when it is not addressed to a Docker repository.
///
/// `Err(request)` is where this used to call straight into the application's frontend fallback.
/// The decision "this is not mine" belongs here; the decision "so serve the frontend instead"
/// belongs to the application. Nothing has been consumed from the request when it is handed back.
pub async fn try_handle_v2(
    state: &RepositoryRouterState,
    request: Request,
) -> Result<Response, Request> {
    // Headers and URI, not `&request`: a `&Request` is not `Send` (its `Body` is not `Sync`), so
    // holding one across the lookup's `.await` would make this future non-`Send` and axum would
    // refuse the handler — with an error that names neither the borrow nor the reason.
    let resolution = resolve(state, request.headers(), request.uri()).await;

    Ok(match resolution {
        Resolution::Docker { repository, path } => dispatch(state, repository, path, request).await,
        // A hostname pointing at something that is not a Docker repository. Handed back so the
        // request is served as that repository's own path, which is what it would have got
        // without this router in the way.
        Resolution::OtherRepository => return Err(request),
        Resolution::VersionCheck => version_check(state, request).await,
        Resolution::Unknown(message) => {
            DockerError::coded(ErrorCode::NameUnknown, message).into_response()
        }
        Resolution::InvalidPath(message) => {
            DockerError::coded(ErrorCode::NameInvalid, message).into_response()
        }
    })
}

/// `GET /v2/` with no repository behind it.
///
/// Credentials, if any, are checked so that `docker login` finds out about a typo now rather than
/// on the first push. No credentials is not an error: an anonymous pull of a public image starts
/// with this same probe, and answering `401` would make it give up before it ever named an image.
async fn version_check(state: &RepositoryRouterState, request: Request) -> Response {
    let (mut parts, _) = request.into_parts();
    let authenticated =
        <RepositoryAuthentication as FromRequestParts<RepositoryRouterState>>::from_request_parts(
            &mut parts, state,
        )
        .await;

    match authenticated {
        Ok(_) => Response::builder()
            .status(http::StatusCode::OK)
            .header(http::header::CONTENT_TYPE, "application/json")
            .header(
                super::DOCKER_API_VERSION_HEADER.clone(),
                super::DOCKER_API_VERSION_VALUE.clone(),
            )
            .body("{}".into())
            .unwrap(),
        Err(error) => {
            debug!(?error, "Refused a /v2/ probe carrying bad credentials");
            let base = state.context.app_url();
            DockerError::Challenge {
                realm: format!("{base}/api/docker/token"),
                scope: None,
            }
            .into_response()
        }
    }
}

enum Resolution {
    Docker {
        repository: DynRepository,
        path: StoragePath,
    },
    /// A bare `/v2/` addressed to the instance itself rather than to any one repository.
    VersionCheck,
    OtherRepository,
    Unknown(String),
    /// The path after `/v2` is not a path this server will accept.
    InvalidPath(String),
}

/// The Docker sub-path: everything after `/v2`, percent-decoded.
///
/// Read off the URI rather than from the `{*path}` capture so that `/v2` and `/v2/` reach the same
/// code as `/v2/{*path}`.
fn v2_path(uri: &Uri) -> Result<StoragePath, String> {
    let raw = uri.path().strip_prefix("/v2").unwrap_or_default();
    let decoded = percent_decode_str(raw).decode_utf8_lossy();
    // `parse`, not `From<&str>`: this is client-controlled, and `From` silently drops `..` rather
    // than rejecting it.
    StoragePath::parse(&decoded).map_err(|error| error.to_string())
}

/// Works out which repository a `/v2` request is for, and what the path means to it.
async fn resolve(state: &RepositoryRouterState, headers: &HeaderMap, uri: &Uri) -> Resolution {
    let host = request_host(headers, uri, state.context.trust_forwarded_host());

    // The host is checked before the path is parsed. The other way round, a malformed path on a
    // hostname belonging to a *Maven* repository answered with a Docker `NameInvalid` instead of
    // falling through to the repository that owns the host.
    if let Some(repository) = host
        .as_deref()
        .and_then(|host| state.resolver.repository_for_hostname(host))
    {
        if repository.get_type() != super::REPOSITORY_TYPE_ID {
            return Resolution::OtherRepository;
        }
        return match v2_path(uri) {
            Ok(path) => {
                debug!(?host, repository = %repository.name(), "Routing /v2 by host");
                Resolution::Docker { repository, path }
            }
            Err(message) => Resolution::InvalidPath(message),
        };
    }

    let path = match v2_path(uri) {
        Ok(path) => path,
        Err(message) => return Resolution::InvalidPath(message),
    };

    let components: Vec<String> = Vec::<nr_core::storage::StoragePathComponent>::from(path)
        .into_iter()
        .map(String::from)
        .collect();

    // A bare `/v2/` on the instance host names no repository, but it is the capability probe every
    // client — and every `docker login` — begins with. There is nothing to dispatch to, so it is
    // answered here rather than 404'd, which would make the whole registry look absent.
    if components.is_empty() {
        return Resolution::VersionCheck;
    }
    if components.len() < 2 {
        return Resolution::Unknown(
            "an image name must begin with `{storage}/{repository}`, or the registry must be \
             reached on a hostname registered to a Docker repository"
                .to_owned(),
        );
    }

    let names = RepositoryStorageName::from((components[0].clone(), components[1].clone()));
    let repository = match state.resolver.repository_from_names(&names).await {
        Ok(repository) => repository,
        Err(error) => {
            return Resolution::Unknown(format!("could not look up the repository: {error}"));
        }
    };
    let Some(repository) = repository else {
        return Resolution::Unknown(format!(
            "no repository named `{}/{}`",
            components[0], components[1]
        ));
    };
    if repository.get_type() != super::REPOSITORY_TYPE_ID {
        return Resolution::Unknown(format!(
            "`{}/{}` is a `{}` repository, not a Docker one",
            components[0],
            components[1],
            repository.get_type()
        ));
    }

    let remaining: StoragePath = components[2..]
        .iter()
        .fold(StoragePath::default(), |path, component| {
            path.push(component)
        });
    debug!(repository = %repository.name(), "Routing /v2 by name prefix");
    Resolution::Docker {
        repository,
        path: remaining,
    }
}

async fn dispatch(
    state: &RepositoryRouterState,
    repository: DynRepository,
    path: StoragePath,
    request: Request,
) -> Response {
    let (mut parts, body) = request.into_parts();

    let parent_span = parts
        .extensions
        .get::<RequestSpan>()
        .map(|span| span.0.clone())
        .unwrap_or_else(Span::current);

    // Extracted here rather than as a handler argument, for the same reason host routing does it:
    // `AuthenticationLayer` skips `OPTIONS`, and `RepositoryAuthentication` rejects a request it
    // did not annotate — which would turn every preflight into a 401.
    let authentication = match <RepositoryAuthentication as FromRequestParts<
        RepositoryRouterState,
    >>::from_request_parts(&mut parts, state)
    .await
    {
        Ok(authentication) => authentication,
        Err(error) => return error.into_response(),
    };

    let request = Request::from_parts(parts, body);
    dispatch_repository_request(
        &state.context,
        repository,
        path,
        authentication,
        &parent_span,
        request,
    )
    .await
}
