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
    raw_host(headers, uri, trust_forwarded).and_then(|raw| normalize_host(&raw))
}

/// The host *with* its port, for building a URL to hand back to the client.
///
/// [`request_host`] deliberately strips the port, because a hostname is registered without one and
/// a lookup has to match whether or not the client wrote `:8443`. A URL is the opposite case: an
/// authentication realm of `http://localhost/...` when the instance is on `:6742` points at nothing,
/// and a Docker client that cannot reach the realm abandons the push without an error.
pub fn request_authority(headers: &HeaderMap, uri: &Uri, trust_forwarded: bool) -> Option<String> {
    let raw = raw_host(headers, uri, trust_forwarded)?;
    // Validated through `normalize_host` so anything that is not a bare host is rejected here too,
    // but the value returned keeps the port the client actually addressed.
    normalize_host(&raw)?;
    Some(raw.trim().to_ascii_lowercase())
}

/// Picks which of the three sources names the host, without normalising it.
fn raw_host(headers: &HeaderMap, uri: &Uri, trust_forwarded: bool) -> Option<String> {
    if trust_forwarded
        && let Some(forwarded) = headers.get_str_ignore_empty(&X_FORWARDED_HOST)
        && let Some(first) = forwarded.split(',').next()
        && normalize_host(first).is_some()
    {
        return Some(first.to_owned());
    }
    if let Some(host) = headers.get_str_ignore_empty(&http::header::HOST)
        && normalize_host(host).is_some()
    {
        return Some(host.to_owned());
    }
    // HTTP/2 has no `Host` header; hyper puts `:authority` in the URI instead. `authority()` rather
    // than `host()` so the port survives.
    uri.authority()
        .map(|authority| authority.as_str().to_owned())
        .filter(|authority| normalize_host(authority).is_some())
}

/// The scheme to build absolute URLs with.
///
/// The server may be terminating plain HTTP behind a TLS proxy, in which case the request itself
/// says `http` while the client is on `https`. `X-Forwarded-Proto` is only believed when the
/// operator has said their proxy sets it — the same switch that guards `X-Forwarded-Host` — and
/// otherwise the configured `app_url`'s scheme is the operator's own statement of how the instance
/// is reached.
pub fn request_scheme(site: &NitroRepo, headers: &HeaderMap, uri: &Uri) -> String {
    static X_FORWARDED_PROTO: HeaderName = HeaderName::from_static("x-forwarded-proto");

    if site.general_security_settings.trust_forwarded_host
        && let Some(forwarded) = headers.get_str_ignore_empty(&X_FORWARDED_PROTO)
        && let Some(first) = forwarded.split(',').next()
        && matches!(first.trim(), "http" | "https")
    {
        return first.trim().to_owned();
    }
    if let Some(scheme) = uri.scheme_str() {
        return scheme.to_owned();
    }
    let app_url = {
        let instance = site.instance.lock();
        instance.app_url.clone()
    };
    if app_url.starts_with("http://") {
        "http".to_owned()
    } else {
        "https".to_owned()
    }
}

/// `{scheme}://{host}` for the request as the client addressed it.
///
/// Built from the request rather than from `app_url` because a repository reachable on a custom
/// domain may not be reachable at `app_url` from wherever the client is running — a URL handed back
/// in a `Location` or an authentication realm has to point somewhere the caller can actually go.
/// Falls back to `app_url` when the request carries no usable host, and to `None` when there is no
/// `app_url` either.
pub fn request_origin(site: &NitroRepo, headers: &HeaderMap, uri: &Uri) -> Option<String> {
    match request_authority(
        headers,
        uri,
        site.general_security_settings.trust_forwarded_host,
    ) {
        Some(authority) => {
            let scheme = request_scheme(site, headers, uri);
            Some(format!("{scheme}://{authority}"))
        }
        None => {
            let app_url = {
                let instance = site.instance.lock();
                instance.app_url.trim_end_matches('/').to_owned()
            };
            (!app_url.is_empty()).then_some(app_url)
        }
    }
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

    use super::{X_FORWARDED_HOST, normalize_host, request_authority, request_host};

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

    /// A lookup ignores the port; a URL cannot. A realm of `http://localhost/...` for an instance
    /// on `:6742` is unreachable, and a Docker client that cannot reach the realm abandons the push
    /// with no error at all.
    #[test]
    fn the_authority_keeps_the_port_that_the_host_drops() {
        let map = headers(&[(&http::header::HOST, "localhost:6742")]);
        let uri = Uri::from_static("/v2/");

        assert_eq!(
            request_host(&map, &uri, false).as_deref(),
            Some("localhost")
        );
        assert_eq!(
            request_authority(&map, &uri, false).as_deref(),
            Some("localhost:6742")
        );
    }

    #[test]
    fn the_authority_is_normalised_and_validated_like_the_host() {
        let map = headers(&[(&http::header::HOST, "  Docker.Example.COM:8443 ")]);
        let uri = Uri::from_static("/v2/");
        assert_eq!(
            request_authority(&map, &uri, false).as_deref(),
            Some("docker.example.com:8443")
        );

        // Anything that is not a bare host is refused here too.
        for bad in ["host:notaport", "a/b", "user@example.com"] {
            let map = headers(&[(&http::header::HOST, bad)]);
            assert_eq!(request_authority(&map, &uri, false), None, "raw: {bad:?}");
        }
    }

    #[test]
    fn the_authority_falls_back_to_the_uri_for_http2() {
        let uri = Uri::from_static("https://docker.example.com:8443/v2/");
        assert_eq!(
            request_authority(&HeaderMap::new(), &uri, false).as_deref(),
            Some("docker.example.com:8443")
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
