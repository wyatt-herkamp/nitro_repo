mod layer;
mod request_span;
use derive_more::derive::From;
use http::HeaderName;
use layer::AppTraceMiddleware;
pub use request_span::*;
pub mod response_body;

use tower::Layer;

use crate::app::NitroRepo;

#[allow(clippy::declare_interior_mutable_const)]
const X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

#[derive(Debug, Clone, From)]
pub struct AppTracingLayer(pub NitroRepo);

impl<S> Layer<S> for AppTracingLayer {
    type Service = AppTraceMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AppTraceMiddleware {
            inner,
            site: self.0.clone(),
        }
    }
}
