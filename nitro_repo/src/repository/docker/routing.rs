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
//! domain" list in [`host_routing`](crate::app::host_routing).

use axum::{
    Router,
    extract::{FromRequestParts, Request, State},
    response::{IntoResponse, Response},
    routing::any,
};
use http::{HeaderMap, Uri};
use nr_core::storage::StoragePath;
use percent_encoding::percent_decode_str;
use tracing::{Span, debug};

use super::{DockerError, errors::ErrorCode};
use crate::{
    app::{NitroRepo, RepositoryStorageName},
    repository::{
        DynRepository, Repository, RepositoryAuthentication, dispatch_repository_request,
    },
    utils::{host::request_host, request_logging::request_span::RequestSpan},
};

pub fn v2_router() -> Router<NitroRepo> {
    Router::new()
        .route("/v2", any(handle_v2))
        .route("/v2/", any(handle_v2))
        .route("/v2/{*path}", any(handle_v2))
}

async fn handle_v2(State(site): State<NitroRepo>, request: Request) -> Response {
    // The URI path is still percent-encoded here: `{*path}` is captured but this handler reads the
    // URI directly so that `/v2` and `/v2/` reach the same code as `/v2/{*path}`.
    let raw = request
        .uri()
        .path()
        .strip_prefix("/v2")
        .unwrap_or_default()
        .to_owned();
    let decoded = percent_decode_str(&raw).decode_utf8_lossy();
    // `parse`, not `From<&str>`: this is client-controlled, and `From` silently drops `..` rather
    // than rejecting it.
    let path = match StoragePath::parse(&decoded) {
        Ok(path) => path,
        Err(error) => {
            return DockerError::coded(ErrorCode::NameInvalid, error.to_string()).into_response();
        }
    };

    // Headers and URI, not `&request`: a `&Request` is not `Send` (its `Body` is not `Sync`), so
    // holding one across the lookup's `.await` would make this handler's future non-`Send` and
    // axum would refuse it — with an error that names neither the borrow nor the reason.
    match resolve(&site, request.headers(), request.uri(), path).await {
        Resolution::Docker { repository, path } => dispatch(site, repository, path, request).await,
        // A hostname pointing at something that is not a Docker repository: serve the request as
        // that repository's own path, which is what it would have got without this router.
        Resolution::OtherRepository => {
            crate::app::host_routing::host_or_frontend(State(site), request)
                .await
                .unwrap_or_else(|error| error.into_response())
        }
        Resolution::VersionCheck => version_check(&site, request).await,
        Resolution::Unknown(message) => {
            DockerError::coded(ErrorCode::NameUnknown, message).into_response()
        }
    }
}

/// `GET /v2/` with no repository behind it.
///
/// Credentials, if any, are checked so that `docker login` finds out about a typo now rather than
/// on the first push. No credentials is not an error: an anonymous pull of a public image starts
/// with this same probe, and answering `401` would make it give up before it ever named an image.
async fn version_check(site: &NitroRepo, request: Request) -> Response {
    let (mut parts, _) = request.into_parts();
    let authenticated =
        <RepositoryAuthentication as FromRequestParts<NitroRepo>>::from_request_parts(
            &mut parts, site,
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
            let base = {
                let instance = site.instance.lock();
                instance.app_url.trim_end_matches('/').to_owned()
            };
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
}

/// Works out which repository a `/v2` request is for, and what the path means to it.
async fn resolve(
    site: &NitroRepo,
    headers: &HeaderMap,
    uri: &Uri,
    path: StoragePath,
) -> Resolution {
    let host = request_host(
        headers,
        uri,
        site.general_security_settings.trust_forwarded_host,
    );

    if let Some(repository) = host
        .as_deref()
        .and_then(|host| site.repository_for_hostname(host))
    {
        if repository.get_type() == super::REPOSITORY_TYPE_ID {
            debug!(?host, repository = %repository.name(), "Routing /v2 by host");
            return Resolution::Docker { repository, path };
        }
        return Resolution::OtherRepository;
    }

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
    let repository = match site.get_repository_from_names(&names).await {
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
    site: NitroRepo,
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
    let authentication =
        match <RepositoryAuthentication as FromRequestParts<NitroRepo>>::from_request_parts(
            &mut parts, &site,
        )
        .await
        {
            Ok(authentication) => authentication,
            Err(error) => return error.into_response(),
        };

    let request = Request::from_parts(parts, body);
    dispatch_repository_request(
        &site,
        repository,
        path,
        authentication,
        &parent_span,
        request,
    )
    .await
}
