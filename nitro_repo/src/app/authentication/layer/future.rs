use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use axum::{body::Body, response::Response};
use futures::ready;
use http_body_util::Either;
use pin_project::pin_project;

use crate::error::IntoErrorResponse;

use super::ServiceResponse;

#[pin_project]
pub struct ResponseFuture<F> {
    #[pin]
    pub(super) inner: Kind<F>,
}
impl<F> ResponseFuture<F> {
    pub fn error(error: Box<dyn IntoErrorResponse>) -> Self {
        Self {
            inner: Kind::InvalidAuthentication { error },
        }
    }
}
impl<F> From<F> for ResponseFuture<F> {
    fn from(inner: F) -> Self {
        Self {
            inner: Kind::Ok { future: inner },
        }
    }
}

#[pin_project(project = KindProj)]
pub(super) enum Kind<F> {
    Ok {
        #[pin]
        future: F,
    },
    InvalidAuthentication {
        error: Box<dyn IntoErrorResponse>,
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
