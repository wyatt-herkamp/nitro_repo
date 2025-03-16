//! Attempts to follow the http server semantic conventions of opentelemetry.
//! https://opentelemetry.io/docs/specs/semconv/http/http-spans/#http-server-semantic-conventions
use std::{borrow::Cow, fmt::Display};

use axum::extract::MatchedPath;
use http::{
    HeaderMap, HeaderName, Request,
    header::{REFERER, USER_AGENT},
};
use opentelemetry::{global, propagation::Extractor, trace::TraceContextExt};
use tracing::{field::Empty, info_span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::{
    app::NitroRepo,
    utils::{
        ErrorReason,
        header::HeaderMapExt,
        ip_addr,
        request_logging::{HttpTraceValue, request_id::RequestId},
    },
};

pub struct HeaderMapCarrier<'a>(pub &'a HeaderMap);

impl Extractor for HeaderMapCarrier<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(HeaderName::as_str).collect()
    }
}

pub fn make_span<B>(
    request: &Request<B>,
    request_id: RequestId,
    state: &NitroRepo,
) -> tracing::Span {
    let user_agent = request.headers().get_string_ignore_empty(&USER_AGENT);
    let client_ip = ip_addr::extract_ip_as_string(request, state);

    let span: tracing::Span = info_span!(target: "nitro_repo::requests","HTTP request",
        http.route = Empty,
        url.path = request.uri().path(),
        url.query = request.uri().query(),
        url.schema = request.uri().scheme_str().unwrap_or("http"),
        http.request.method = %request.method(),
        http.request.body.size = Empty,

        http.response.status_code = Empty,
        http.response.body.size = Empty,

        user_agent.original = user_agent,

        network.protocol.version = request.version().value(),

        client.address = client_ip,
        // The port is not included because I run this behind a reverse proxy

        request_id = display(request_id),
        otel.name = "HTTP request",
        otel.kind = ?opentelemetry::trace::SpanKind::Server,
        error.type = Empty,
    );

    let context = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderMapCarrier(request.headers()))
    });

    if context.has_active_span() {
        span.set_parent(context);
    }

    span
}

pub fn on_request<B>(request: &Request<B>, span: &tracing::Span, request_body_size: Option<u64>) {
    let path = request
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str());
    let method = request.method().as_str();

    let otel_name = if let Some(path) = &path {
        format!("{method} {path}")
    } else {
        format!(
            "{method} {path}",
            method = request.method(),
            path = request.uri().path()
        )
    };

    span.record("http.route", path);
    span.record("otel.name", otel_name);
    if let Some(size) = request_body_size {
        span.record("http.request.body.size", size);
    }

    let referer = request.headers().get_string_ignore_empty(&REFERER);
    if let Some(referer) = referer {
        span.set_attribute("http.request.header.referer", referer);
    }
}
pub fn on_response<B>(
    response: &axum::http::Response<B>,
    _latency: std::time::Duration,
    span: &tracing::Span,
    body_size: Option<u64>,
) {
    if response.status().is_client_error() || response.status().is_server_error() {
        let reason = response.extensions().get::<ErrorReason>();
        let reason_value = reason
            .cloned()
            .map_or(Cow::Borrowed("Unknown error"), |r| r.reason);
        span.record("error.type", reason_value.as_ref());
        span.set_status(opentelemetry::trace::Status::Error {
            description: reason_value,
        });
    } else {
        span.set_status(opentelemetry::trace::Status::Ok);
    }

    span.record("http.response.status_code", response.status().value());
    if let Some(size) = body_size {
        span.record("http.response.body.size", size);
    }
}
pub fn on_end_of_stream(body_size: u64, span: &tracing::Span) {
    span.record("http.response.body.size", body_size);
}
pub fn on_failure(err: &impl Display, _latency: std::time::Duration, span: &tracing::Span) {
    span.record("error.type", err.to_string());
    span.set_status(opentelemetry::trace::Status::Error {
        description: Cow::Owned(err.to_string()),
    });
}
