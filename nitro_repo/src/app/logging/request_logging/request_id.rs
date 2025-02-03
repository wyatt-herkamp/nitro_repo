use std::fmt::Display;

use axum::extract::{FromRef, FromRequestParts};
use derive_more::derive::{From, Into};
use http::{header::InvalidHeaderValue, request::Parts, HeaderValue};
use sqlx::types::Uuid;

use crate::{app::NitroRepo, error::MissingInternelExtension};

#[derive(Debug, Clone, Copy, From, Into)]
pub struct RequestId(pub Uuid);
impl Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}
impl RequestId {
    pub fn new_random() -> Self {
        Self(Uuid::new_v4())
    }
    pub fn extract_from_parts(parts: &Parts) -> Result<Self, MissingInternelExtension> {
        let extension = parts.extensions.get::<RequestId>();
        extension.copied()
            .ok_or(MissingInternelExtension("Request ID"))
    }
}
impl<S> FromRequestParts<S> for RequestId
where
    NitroRepo: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = MissingInternelExtension;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        RequestId::extract_from_parts(parts)
    }
}
impl TryFrom<RequestId> for HeaderValue {
    type Error = InvalidHeaderValue;

    fn try_from(value: RequestId) -> Result<Self, Self::Error> {
        HeaderValue::from_str(&value.0.to_string())
    }
}
