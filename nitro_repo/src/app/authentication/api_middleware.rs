use crate::app::authentication::AuthenticationRaw;
use crate::app::NitroRepo;
use crate::utils::headers::{AuthorizationHeader, HeaderValueExt};
use axum::body::Body;
use axum_extra::extract::CookieJar;
use derive_more::derive::From;
use http::header::AUTHORIZATION;
use http::{Request, Response};
use http_body_util::Either;
use pin_project::pin_project;
use std::task::ready;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::Layer;
use tower_service::Service;
use tracing::{debug, trace};
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
        if req.method() == http::Method::OPTIONS {
            trace!("Options Request");
            return ResponseFuture {
                inner: Kind::Ok {
                    future: self.inner.call(req),
                },
            };
        }
        let (mut parts, body) = req.into_parts();
        let cookie_jar = CookieJar::from_headers(&parts.headers);
        let authorization_header = parts
            .headers
            .get(AUTHORIZATION)
            .map(|header| header.parsed::<AuthorizationHeader>());

        let raw = if let Some(authorization_header) = authorization_header {
            match authorization_header {
                Ok(authorization) => AuthenticationRaw::new_from_header(authorization, &self.site),
                Err(error) => {
                    debug!("Invalid Header {}", error);
                    return ResponseFuture {
                        inner: Kind::InvalidAuthentication {
                            error: error.to_string(),
                        },
                    };
                }
            }
        } else {
            if let Some(cookie) = cookie_jar.get("session") {
                debug!("Session Cookie Found");
                AuthenticationRaw::new_from_cookie(cookie, &self.site)
            } else {
                debug!("No Authorization Header or Session Cookie Found");
                AuthenticationRaw::NoIdentification
            }
        };

        parts.extensions.insert(raw);
        ResponseFuture {
            inner: Kind::Ok {
                future: self.inner.call(Request::from_parts(parts, body)),
            },
        }
    }
}

#[pin_project]
pub struct ResponseFuture<F> {
    #[pin]
    inner: Kind<F>,
}

#[pin_project(project = KindProj)]
enum Kind<F> {
    Ok {
        #[pin]
        future: F,
    },
    InvalidAuthentication {
        error: String,
    },
}

impl<F, B, E> Future for ResponseFuture<F>
where
    F: Future<Output = Result<Response<B>, E>>,
{
    type Output = Result<ServiceResponse<B>, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project().inner.project() {
            KindProj::InvalidAuthentication { error } => {
                let body = Body::from(format!("Invalid Authentication Header: {}", error));
                let response = Response::new(Either::Right(body));

                Poll::Ready(Ok(response))
            }
            KindProj::Ok { future } => {
                let response: Response<B> = ready!(future.poll(cx))?;
                let response = response.map(Either::Left);
                Poll::Ready(Ok(response))
            }
        }
    }
}
