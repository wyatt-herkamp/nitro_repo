use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use axum::{
    body::{Body, HttpBody},
    extract::MatchedPath,
};
use derive_more::derive::From;
use futures::ready;
use http::{Request, Response};
use opentelemetry::KeyValue;
use pin_project::pin_project;
use tower::Layer;
use tower_http::{
    classify::{ClassifyResponse, MakeClassifier, ServerErrorsAsFailures, SharedClassifier},
    trace::HttpMakeClassifier,
};
use tower_service::Service;

use crate::app::NitroRepo;

#[derive(Debug, Clone, From)]
pub struct AppTracingLayer(pub NitroRepo);

impl<S> Layer<S> for AppTracingLayer {
    type Service = AppTraceMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AppTraceMiddleware {
            inner,
            site: self.0.clone(),
            classifier: SharedClassifier::new(ServerErrorsAsFailures::new()),
        }
    }
}

/// Middleware that handles the authentication of the user
#[derive(Debug, Clone)]
pub struct AppTraceMiddleware<S> {
    inner: S,
    site: NitroRepo,
    classifier: HttpMakeClassifier,
}

impl<S> Service<Request<Body>> for AppTraceMiddleware<S>
where
    S: Service<Request<Body>, Response = Response<Body>> + Send + Sync + Clone + 'static,
    S::Future: Send + 'static,
    S::Error: std::fmt::Display + 'static,
{
    type Response = axum::response::Response<Body>;
    type Error = S::Error;
    //type Future = BoxFuture<'static, Result<Self::Response, S::Error>>;
    type Future = ResponseFuture<S::Future>;
    // Async Stuff we can ignore
    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let path = req
            .extensions()
            .get::<MatchedPath>()
            .map_or(req.uri().path(), |p| p.as_str());

        let attributes = vec![
            KeyValue::new("http.route", path.to_owned()),
            KeyValue::new("http.request.method", req.method().as_str().to_string()),
        ];
        let site: NitroRepo = self.site.clone();
        let body_size = req.body().size_hint().lower();

        // Continue the request
        let mut inner = self.inner.clone();
        let start = std::time::Instant::now();

        let request_span = super::make_span(&req);
        let classifier = self.classifier.make_classifier(&req);
        let result = {
            super::on_request(&req, &request_span);
            let _enter = request_span.enter();
            inner.call(req)
        };
        ResponseFuture {
            inner: result,
            instant: start,
            state: site,
            classifier: Some(classifier),
            span: request_span,
            request_body_size: body_size,
            attributes: attributes,
        }
    }
}

#[pin_project]
pub struct ResponseFuture<F> {
    #[pin]
    inner: F,

    instant: std::time::Instant,

    state: NitroRepo,

    classifier: Option<ServerErrorsAsFailures>,
    span: tracing::Span,
    request_body_size: u64,
    attributes: Vec<KeyValue>,
}

impl<F, E> Future for ResponseFuture<F>
where
    E: std::fmt::Display + 'static,
    F: Future<Output = Result<Response<Body>, E>>,
{
    type Output = Result<Response<Body>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _guard = this.span.enter();
        let result = ready!(this.inner.poll(cx));
        let duration = this.instant.elapsed();

        let classifier = this.classifier.take().unwrap();
        let state = this.state.clone();
        let request_body_size = *this.request_body_size;
        match result {
            Ok(response) => {
                this.attributes.push(KeyValue::new(
                    "http.response.status_code",
                    response.status().as_u16().to_string(),
                ));
                //let classification = classifier.classify_response(&response);
                super::on_response(&response, duration, &this.span);

                state
                    .metrics
                    .response_size_bytes
                    .record(response.body().size_hint().lower(), &this.attributes);

                final_metrics(&state, duration, request_body_size, &this.attributes);

                Poll::Ready(Ok(response))
            }
            Err(err) => {
                let failure_class = classifier.classify_error(&err);

                super::on_failure(failure_class, duration, &this.span);

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
