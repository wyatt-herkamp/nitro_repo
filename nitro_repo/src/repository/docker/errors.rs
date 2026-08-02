//! The OCI error envelope.
//!
//! Every non-2xx answer from a registry is expected to look like
//! `{"errors":[{"code":"...","message":"...","detail":...}]}`. Docker and containerd both surface
//! `message` to the user and branch on `code` — a plain-text body reaches the user as
//! "unexpected status" and nothing more.
//!
//! Codes are from
//! <https://github.com/opencontainers/distribution-spec/blob/main/spec.md#error-codes>.

use axum::response::Response;
use http::StatusCode;
use serde::Serialize;

/// The subset of the spec's codes this registry can actually produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    BlobUnknown,
    BlobUploadInvalid,
    BlobUploadUnknown,
    DigestInvalid,
    ManifestBlobUnknown,
    ManifestInvalid,
    ManifestUnknown,
    NameInvalid,
    NameUnknown,
    SizeInvalid,
    Unauthorized,
    Denied,
    Unsupported,
    TooManyRequests,
}

impl ErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BlobUnknown => "BLOB_UNKNOWN",
            Self::BlobUploadInvalid => "BLOB_UPLOAD_INVALID",
            Self::BlobUploadUnknown => "BLOB_UPLOAD_UNKNOWN",
            Self::DigestInvalid => "DIGEST_INVALID",
            Self::ManifestBlobUnknown => "MANIFEST_BLOB_UNKNOWN",
            Self::ManifestInvalid => "MANIFEST_INVALID",
            Self::ManifestUnknown => "MANIFEST_UNKNOWN",
            Self::NameInvalid => "NAME_INVALID",
            Self::NameUnknown => "NAME_UNKNOWN",
            Self::SizeInvalid => "SIZE_INVALID",
            Self::Unauthorized => "UNAUTHORIZED",
            Self::Denied => "DENIED",
            Self::Unsupported => "UNSUPPORTED",
            Self::TooManyRequests => "TOOMANYREQUESTS",
        }
    }

    /// The status the spec pairs with each code.
    pub fn status(self) -> StatusCode {
        match self {
            Self::BlobUnknown
            | Self::BlobUploadUnknown
            | Self::ManifestUnknown
            | Self::NameUnknown => StatusCode::NOT_FOUND,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Denied => StatusCode::FORBIDDEN,
            Self::Unsupported => StatusCode::METHOD_NOT_ALLOWED,
            Self::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::BAD_REQUEST,
        }
    }
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    errors: Vec<ErrorEntry>,
}

#[derive(Debug, Serialize)]
struct ErrorEntry {
    code: &'static str,
    message: String,
}

/// Builds the response body for one error code.
pub fn oci_error(code: ErrorCode, message: impl Into<String>) -> Response {
    oci_error_with_status(code.status(), code, message)
}

/// The same, with the status forced.
///
/// Needed for the challenge on `/v2/`: the code is `UNAUTHORIZED` but the response also has to
/// carry `WWW-Authenticate`, which is added by the caller.
pub fn oci_error_with_status(
    status: StatusCode,
    code: ErrorCode,
    message: impl Into<String>,
) -> Response {
    let body = ErrorBody {
        errors: vec![ErrorEntry {
            code: code.as_str(),
            message: message.into(),
        }],
    };
    Response::builder()
        .status(status)
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(
            super::DOCKER_API_VERSION_HEADER.clone(),
            super::DOCKER_API_VERSION_VALUE.clone(),
        )
        .body(serde_json::to_string(&body).unwrap_or_default().into())
        .unwrap()
}

#[cfg(test)]
mod tests {
    use http::StatusCode;

    use super::{ErrorCode, oci_error};

    #[test]
    fn codes_map_to_the_statuses_the_spec_pairs_them_with() {
        assert_eq!(ErrorCode::BlobUnknown.status(), StatusCode::NOT_FOUND);
        assert_eq!(ErrorCode::ManifestUnknown.status(), StatusCode::NOT_FOUND);
        assert_eq!(ErrorCode::Denied.status(), StatusCode::FORBIDDEN);
        assert_eq!(ErrorCode::Unauthorized.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(ErrorCode::DigestInvalid.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            ErrorCode::Unsupported.status(),
            StatusCode::METHOD_NOT_ALLOWED
        );
    }

    #[test]
    fn the_envelope_carries_the_api_version_header() {
        let response = oci_error(ErrorCode::NameUnknown, "nope");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            response
                .headers()
                .get("docker-distribution-api-version")
                .unwrap(),
            "registry/2.0"
        );
    }
}
