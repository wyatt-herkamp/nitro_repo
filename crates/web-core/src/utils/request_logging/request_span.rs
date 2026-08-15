use axum::extract::{FromRef, FromRequestParts, OptionalFromRequestParts};
use derive_more::From;
use http::request::Parts;

use crate::utils::extensions::MissingInternelExtension;

#[derive(Debug, Clone, From)]
pub struct RequestSpan(pub tracing::Span);
impl<S> FromRequestParts<S> for RequestSpan
where
    S: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = MissingInternelExtension;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let extension = parts.extensions.get::<RequestSpan>();
        extension
            .cloned()
            .ok_or(MissingInternelExtension("Request Span"))
    }
}
impl<S> OptionalFromRequestParts<S> for RequestSpan
where
    S: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = MissingInternelExtension;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Option<Self>, Self::Rejection> {
        let extension = parts.extensions.get::<RequestSpan>();
        Ok(extension.cloned())
    }
}
