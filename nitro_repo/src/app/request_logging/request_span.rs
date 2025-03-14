use std::fmt::Display;

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
    utils::{ErrorReason, header::HeaderMapExt, ip_addr, request_logging::request_id::RequestId},
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

pub fn make_span<B>(request: &Request<B>, request_id: RequestId) -> tracing::Span {
    let user_agent = request
        .headers()
        .get_string_ignore_empty(&USER_AGENT)
        .unwrap_or_else(|| "<unknown>".to_string());

    let span = info_span!(target: "nitro_repo::requests","HTTP request",
        http.path = Empty,
        http.method = ?request.method(),
        http.version = ?request.version(),
        http.user_agent = user_agent,
        http.client_ip = Empty,
        otel.kind = ?opentelemetry::trace::SpanKind::Server,
        http.status_code = Empty,
        http.referer = Empty,
        http.raw_path = ?request.uri().path(),
        http.response_size = Empty,
        otel.status_code = Empty,
        otel.name = "HTTP request",
        trace_id = Empty,
        exception.message = Empty,
        request_id = display(request_id),
    );

    let context = global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderMapCarrier(request.headers()))
    });

    if context.has_active_span() {
        span.record(
            "trace_id",
            context.span().span_context().trace_id().to_string(),
        );
        span.set_parent(context);
    }

    span
}

pub fn on_request<B>(request: &Request<B>, span: &tracing::Span, state: &NitroRepo) {
    let path = request
        .extensions()
        .get::<MatchedPath>()
        .map_or(request.uri().path(), |p| p.as_str());
    let method = request.method().as_str();
    let client_ip = ip_addr::extract_ip_as_string(&request, state);

    span.record("http.path", path);
    span.record("otel.name", format!("{method} {path}"));
    span.record("http.client_ip", client_ip);

    let referer = request.headers().get_string_ignore_empty(&REFERER);
    if let Some(referer) = referer {
        span.record("http.referer", &referer);
    }
}
pub fn on_response<B>(
    response: &axum::http::Response<B>,
    _latency: std::time::Duration,
    span: &tracing::Span,
) {
    if response.status().is_client_error() || response.status().is_server_error() {
        let reason = response.extensions().get::<ErrorReason>();
        if let Some(reason) = reason {
            span.record("exception.message", reason.reason.as_ref());
        } else {
            span.record("exception.message", "Unknown error");
        }
    }

    span.record("http.status_code", response.status().as_u16());

    span.record("otel.status_code", "OK");
}
pub fn on_end_of_stream(body_size: u64, span: &tracing::Span) {
    span.record("http.response_size", body_size);
}
pub fn on_failure<E>(error: &E, _latency: std::time::Duration, span: &tracing::Span)
where
    E: Display,
{
    span.record("exception.message", error.to_string());
    span.record("otel.status_code", "ERROR");
}
