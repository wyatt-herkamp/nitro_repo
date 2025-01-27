use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use axum::{
    body::{Body, HttpBody},
    extract::MatchedPath,
};
use http::{header::InvalidHeaderValue, HeaderValue, Request, Response};
use opentelemetry::KeyValue;
use pin_project::pin_project;

use tower_service::Service;
use tracing::{debug, error, info};

use crate::app::NitroRepo;

use super::{request_id::RequestId, response_body::TraceResponseBody, RequestSpan, X_REQUEST_ID};

/// Middleware that handles the authentication of the user
#[derive(Debug, Clone)]
pub struct AppTraceMiddleware<S> {
    pub(super) inner: S,
    pub(super) site: NitroRepo,
}

impl<S> Service<Request<Body>> for AppTraceMiddleware<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Send + Sync + Clone + 'static,
    S::Future: Send + 'static,
    S::Error: std::fmt::Display + 'static,
{
    type Response = Response<TraceResponseBody>;
    type Error = S::Error;
    //type Future = BoxFuture<'static, Result<Self::Response, S::Error>>;
    type Future = TraceResponseFuture<S::Future>;
    // Async Stuff we can ignore
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        let path = req
            .extensions()
            .get::<MatchedPath>()
            .map_or(req.uri().path(), |p| p.as_str());
        let request_id = RequestId::new_random();
        let attributes = vec![
            KeyValue::new("http.route", path.to_owned()),
            KeyValue::new("http.request.method", req.method().as_str().to_string()),
            KeyValue::new("request_id", request_id.to_string()),
        ];
        let site: NitroRepo = self.site.clone();
        let body_size = req.body().size_hint().lower();

        // Continue the request
        let mut inner = self.inner.clone();
        let start = std::time::Instant::now();

        let request_span = super::make_span(&req, request_id);
        req.extensions_mut()
            .insert(RequestSpan(request_span.clone()));
        req.extensions_mut().insert(request_id);

        super::on_request(&req, &request_span);

        let result = request_span.in_scope(|| inner.call(req));
        TraceResponseFuture {
            inner: result,
            instant: start,
            state: site,
            span: Some(request_span),
            request_body_size: body_size,
            attributes: attributes,
            request_id,
        }
    }
}

#[pin_project]
pub struct TraceResponseFuture<F> {
    #[pin]
    inner: F,
    instant: std::time::Instant,
    state: NitroRepo,
    attributes: Vec<KeyValue>,
    span: Option<tracing::Span>,
    request_body_size: u64,

    request_id: RequestId,
}

impl<F, E> Future for TraceResponseFuture<F>
where
    E: std::fmt::Display + 'static,
    F: Future<Output = Result<Response<Body>, E>>,
{
    type Output = Result<Response<TraceResponseBody>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if this.span.is_none() {
            panic!("ResponseFuture polled after completion");
        }
        // Attempt to poll the inner future
        let result = {
            let span_ref = this.span.as_ref().unwrap();
            match span_ref.in_scope(|| this.inner.poll(cx)) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(result) => result,
            }
        };
        // One it has completed we can take the span and the classifier
        let span = this.span.take().unwrap();
        let _guard = span.enter();

        let duration = this.instant.elapsed();
        let state = this.state.clone();
        let request_body_size = *this.request_body_size;
        match result {
            Ok(mut response) => {
                let request_id_header: Result<HeaderValue, InvalidHeaderValue> =
                    this.request_id.clone().try_into();
                match request_id_header {
                    Ok(header) => {
                        response.headers_mut().insert(X_REQUEST_ID, header);
                    }
                    Err(e) => {
                        error!("Failed to set request id header: {}", e);
                    }
                }
                this.attributes.push(KeyValue::new(
                    "http.response.status_code",
                    response.status().as_u16().to_string(),
                ));

                super::on_response(&response, duration, &span);
                if response.status().is_server_error() {
                    super::on_failure(&response.status(), duration, &span);
                }

                final_metrics(&state, duration, request_body_size, &this.attributes);

                let span = span.clone();

                let res: Response<TraceResponseBody> = response.map(|body| TraceResponseBody {
                    inner: body,
                    start: *this.instant,
                    span,
                    state: state.clone(),
                    attributes: this.attributes.clone(),
                    total_bytes: request_body_size,
                });

                Poll::Ready(Ok(res))
            }
            Err(err) => {
                super::on_failure(&err, duration, &span);

                final_metrics(&state, duration, request_body_size, &this.attributes);

                Poll::Ready(Err(err))
            }
        }
    }
}

fn final_metrics(
    state: &NitroRepo,
    duration: std::time::Duration,
    body_size: u64,
    attrs: &[KeyValue],
) {
    state.metrics.request_size_bytes.record(body_size, attrs);
    let duration = duration.as_millis();
    state
        .metrics
        .request_duration
        .record(duration as f64 / 1000f64, attrs);
}
