//! Routing a request by the host it was addressed to.
//!
//! A repository can have custom hostnames attached to it. A request that arrives on one of them is
//! served by that repository, with the whole request path taken as the artifact path relative to
//! the repository root — so `https://maven.example.com/dev/kingtux/tms/1.0.0/tms-1.0.0.jar` serves
//! what `/repositories/local/releases/dev/kingtux/tms/1.0.0/tms-1.0.0.jar` serves.
//!
//! This is wired in as the router's *fallback*, not as a layer above it. `/api`, `/badge` and
//! `/repositories` are matched first and on every host, so a custom domain never shadows the API or
//! makes the web UI unreachable. The cost is that an artifact whose path starts with one of those
//! segments cannot be fetched over a custom domain.
//!
//! The header parsing this is built on — `request_host`, `request_origin`, `normalize_host` and
//! friends — lives in [`crate::utils::host`], which depends on nothing but the request itself so
//! that repository code can reach it without reaching the app state.

use axum::{
    extract::{FromRequestParts, Request, State},
    response::{IntoResponse, Response},
};
use nr_core::storage::StoragePath;
use percent_encoding::percent_decode_str;
use tracing::{Span, debug};

use crate::{
    app::NitroRepo,
    error::InternalError,
    repository::{
        DynRepository, Repository, RepositoryAuthentication, RepositoryRequestError,
        dispatch_repository_request,
    },
    utils::{host::request_host, request_logging::request_span::RequestSpan},
};

/// The router's fallback: a repository when the request's host is registered to one, the frontend
/// otherwise.
pub async fn host_or_frontend(
    State(site): State<NitroRepo>,
    request: Request,
) -> Result<Response, InternalError> {
    let host = request_host(
        request.headers(),
        request.uri(),
        site.general_security_settings.trust_forwarded_host,
    );
    let repository = host
        .as_deref()
        .and_then(|host| site.repository_for_hostname(host));

    let Some(repository) = repository else {
        return super::frontend::frontend_request(State(site), request).await;
    };
    debug!(?host, repository = %repository.name(), "Routing a request by its host");
    Ok(dispatch_by_host(site, repository, request).await)
}

async fn dispatch_by_host(
    site: NitroRepo,
    repository: DynRepository,
    request: Request,
) -> Response {
    let (mut parts, body) = request.into_parts();

    let parent_span = parts
        .extensions
        .get::<RequestSpan>()
        .map(|span| span.0.clone())
        .unwrap_or_else(Span::current);

    // Extracted here, after the host has matched, rather than as an argument of the handler.
    // `AuthenticationLayer` skips `OPTIONS` entirely, so `RepositoryAuthentication` rejects one —
    // as a handler argument that would turn every preflight to `/` into a 401 on hosts that are not
    // repositories at all, where the frontend should have answered.
    let authentication =
        match <RepositoryAuthentication as FromRequestParts<NitroRepo>>::from_request_parts(
            &mut parts, &site,
        )
        .await
        {
            Ok(authentication) => authentication,
            Err(error) => return error.into_response(),
        };

    // The URI path is still percent-encoded. Axum decodes the `{*path}` capture before it reaches
    // `StoragePath` on the `/repositories` route, and npm asks for scoped packages as
    // `/@scope%2fpkg`, so decoding here is what makes the two routes agree.
    let decoded = percent_decode_str(parts.uri.path()).decode_utf8_lossy();
    // `parse`, not `From<&str>`: this is client-controlled, and `From` drops `..` components
    // silently instead of rejecting them.
    let path = match StoragePath::parse(&decoded) {
        Ok(path) => path,
        Err(error) => return RepositoryRequestError::InvalidPath(error).into_response(),
    };

    let request = Request::from_parts(parts, body);
    dispatch_repository_request(
        &site.context(),
        repository,
        path,
        authentication,
        &parent_span,
        request,
    )
    .await
}
