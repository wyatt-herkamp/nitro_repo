use crate::app::NitroRepo;
use crate::app::authentication::AuthenticationRaw;
use crate::error::InternalError;
use crate::utils::header::HeaderValueExt;
use crate::utils::request_logging::request_span::RequestSpan;
use axum::body::Body;
use axum_extra::extract::CookieJar;
use derive_more::derive::From;
use future::ResponseFuture;
use http::header::AUTHORIZATION;
use http::request::Parts;
use http::{Request, Response};
use http_body_util::Either;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use std::borrow::Cow;
use std::task::{Context, Poll};
mod future;
use tower::Layer;
use tower_service::Service;
use tracing::field::Empty;
use tracing::{Span, debug, info_span, trace};

use super::header::AuthorizationHeader;
#[derive(Debug, Clone, From)]
pub struct AuthenticationLayer(pub NitroRepo);

impl<S> Layer<S> for AuthenticationLayer {
    type Service = AuthenticationMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthenticationMiddleware {
            inner,
            site: self.0.clone(),
        }
    }
}
type ServiceBody<T> = Either<T, Body>;
type ServiceResponse<T> = Response<ServiceBody<T>>;
#[derive(Debug, Clone)]
pub struct AuthenticationMiddleware<S> {
    inner: S,
    site: NitroRepo,
}
impl<S> AuthenticationMiddleware<S> {
    pub fn process_from_parts(&self, parts: &mut Parts, span: &Span) -> Result<(), InternalError> {
        let cookie_jar = CookieJar::from_headers(&parts.headers);

        let authorization_header = parts
            .headers
            .get(AUTHORIZATION)
            .map(|header| header.parsed::<AuthorizationHeader, _>())
            .transpose()?;
        let raw = if let Some(authorization_header) = authorization_header {
            AuthenticationRaw::new_from_header(authorization_header, &self.site)
        } else if let Some(cookie) = cookie_jar.get("session") {
            debug!("Session Cookie Found");
            AuthenticationRaw::new_from_cookie(cookie, &self.site)
        } else {
            debug!("No Authorization Header or Session Cookie Found");
            AuthenticationRaw::NoIdentification
        };
        span.record("auth.method", raw.method_name());

        parts.extensions.insert(raw);
        Ok(())
    }
}
impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for AuthenticationMiddleware<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>>,
    ReqBody: Default,
{
    type Response = ServiceResponse<ResBody>;
    type Error = S::Error;
    type Future = ResponseFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let parent_span = req
            .extensions()
            .get::<RequestSpan>()
            .map(|span| span.0.clone())
            .unwrap_or_else(Span::current);
        let (mut parts, body) = req.into_parts();

        {
            let span = info_span!(
                parent: &parent_span,
                "Authentication Middleware",
                auth.method = Empty,
            );
            let _guard = span.enter();
            if parts.method == http::Method::OPTIONS {
                trace!("Options Request");
                span.set_status(opentelemetry::trace::Status::Ok);
            } else if let Err(error) = self.process_from_parts(&mut parts, &span) {
                span.set_status(opentelemetry::trace::Status::Error {
                    description: Cow::Owned(error.to_string()),
                });
                return ResponseFuture::error(error.0);
            } else {
                span.set_status(opentelemetry::trace::Status::Ok);
            }
        }
        let request = Request::from_parts(parts, body);
        let inner = parent_span.in_scope(|| self.inner.call(request));

        ResponseFuture::from(inner)
    }
}
