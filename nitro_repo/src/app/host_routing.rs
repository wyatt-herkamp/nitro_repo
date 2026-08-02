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

use axum::{
    extract::{FromRequestParts, Request, State},
    response::{IntoResponse, Response},
};
use http::{HeaderMap, HeaderName, Uri};
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
    utils::{header::HeaderMapExt, request_logging::request_span::RequestSpan},
};

static X_FORWARDED_HOST: HeaderName = HeaderName::from_static("x-forwarded-host");

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
        &site,
        repository,
        path,
        authentication,
        &parent_span,
        request,
    )
    .await
}

/// The host this request is addressed to, normalised for lookup.
///
/// `Host` by default, falling back to the URI's authority — HTTP/2 has no `Host` header, and hyper
/// puts `:authority` in the URI instead.
///
/// `X-Forwarded-Host` is only read when the operator has said their proxy sets it, and then only
/// its first value: a proxy chain appends, so the left-most is the one nearest the client, which is
/// the one the operator's own proxy wrote.
pub fn request_host(headers: &HeaderMap, uri: &Uri, trust_forwarded: bool) -> Option<String> {
    if trust_forwarded
        && let Some(forwarded) = headers.get_str_ignore_empty(&X_FORWARDED_HOST)
        && let Some(host) = forwarded.split(',').next().and_then(normalize_host)
    {
        return Some(host);
    }
    headers
        .get_str_ignore_empty(&http::header::HOST)
        .and_then(normalize_host)
        .or_else(|| uri.host().and_then(normalize_host))
}

/// Lowercases a host and strips the port and any trailing root dot.
///
/// `None` means "this is not something to route on", which lands the request on the frontend, so
/// anything questionable is rejected rather than repaired: a value with a path or userinfo in it,
/// or a `:` suffix that is not a port, would otherwise be truncated into a host that could match a
/// registration it was never meant to.
pub fn normalize_host(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty()
        || raw
            .chars()
            .any(|c| matches!(c, '/' | '@' | '?' | '#') || c.is_whitespace() || c.is_control())
    {
        return None;
    }

    let host = if let Some(rest) = raw.strip_prefix('[') {
        // An IPv6 literal: `[::1]` or `[::1]:8443`. The only thing allowed after the bracket is a
        // port; anything else is malformed rather than something to trim off.
        let (address, after) = rest.split_once(']')?;
        match after.strip_prefix(':') {
            Some(port) if is_port(port) => {}
            _ if after.is_empty() => {}
            _ => return None,
        }
        format!("[{address}]")
    } else {
        match raw.rsplit_once(':') {
            Some((host, port)) if is_port(port) => host.to_owned(),
            Some(_) => return None,
            None => raw.to_owned(),
        }
    };

    let host = host.strip_suffix('.').unwrap_or(&host);
    if host.is_empty() || host == "[]" {
        return None;
    }
    Some(host.to_ascii_lowercase())
}

fn is_port(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use http::{HeaderMap, HeaderName, HeaderValue, Uri};

    use super::{X_FORWARDED_HOST, normalize_host, request_host};

    fn headers(pairs: &[(&HeaderName, &str)]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (name, value) in pairs {
            headers.insert((*name).clone(), HeaderValue::from_str(value).unwrap());
        }
        headers
    }

    #[test]
    fn normalizes_case_port_and_trailing_dot() {
        assert_eq!(
            normalize_host("MAVEN.Example.COM").as_deref(),
            Some("maven.example.com")
        );
        assert_eq!(
            normalize_host("maven.example.com:8443").as_deref(),
            Some("maven.example.com")
        );
        assert_eq!(
            normalize_host("maven.example.com.").as_deref(),
            Some("maven.example.com")
        );
        assert_eq!(
            normalize_host("  maven.example.com:443  ").as_deref(),
            Some("maven.example.com")
        );
    }

    #[test]
    fn keeps_ipv6_literals_intact() {
        assert_eq!(normalize_host("[::1]").as_deref(), Some("[::1]"));
        assert_eq!(normalize_host("[::1]:8443").as_deref(), Some("[::1]"));
        assert_eq!(normalize_host("[::1]junk"), None);
    }

    #[test]
    fn rejects_anything_that_is_not_a_bare_host() {
        for raw in [
            "",
            "   ",
            "host:notaport",
            "host:",
            "a/b",
            "user@example.com",
            "example.com/path",
            "exa mple.com",
            ".",
        ] {
            assert_eq!(normalize_host(raw), None, "raw: {raw:?}");
        }
    }

    #[test]
    fn the_host_header_wins_unless_forwarding_is_trusted() {
        let map = headers(&[
            (&http::header::HOST, "app.example.com"),
            (&X_FORWARDED_HOST, "maven.example.com"),
        ]);
        let uri = Uri::from_static("/some/path");

        assert_eq!(
            request_host(&map, &uri, false).as_deref(),
            Some("app.example.com")
        );
        assert_eq!(
            request_host(&map, &uri, true).as_deref(),
            Some("maven.example.com")
        );
    }

    #[test]
    fn a_forwarded_chain_uses_its_first_value() {
        let map = headers(&[(&X_FORWARDED_HOST, "maven.example.com, proxy.internal")]);
        let uri = Uri::from_static("/");
        assert_eq!(
            request_host(&map, &uri, true).as_deref(),
            Some("maven.example.com")
        );
    }

    #[test]
    fn falls_back_to_the_uri_authority_when_there_is_no_host_header() {
        // What HTTP/2 looks like: `:authority` lands in the URI and there is no `Host` header.
        let uri = Uri::from_static("https://maven.example.com/dev/kingtux");
        assert_eq!(
            request_host(&HeaderMap::new(), &uri, false).as_deref(),
            Some("maven.example.com")
        );
    }

    #[test]
    fn an_untrustworthy_forwarded_value_does_not_shadow_the_host_header() {
        let map = headers(&[
            (&http::header::HOST, "app.example.com"),
            (&X_FORWARDED_HOST, "not a host"),
        ]);
        let uri = Uri::from_static("/");
        assert_eq!(
            request_host(&map, &uri, true).as_deref(),
            Some("app.example.com")
        );
    }
}
