use axum::response::Response;
use http::header::CONTENT_TYPE;
use thiserror::Error;

#[cfg(feature = "frontend")]
mod hosted;
#[cfg(feature = "frontend")]
pub use hosted::*;
#[cfg(not(feature = "frontend"))]
pub use no_frontend::*;

use crate::utils::{
    IntoErrorResponse, ResponseBuilder, api_error_response::APIErrorResponse,
    other::PLAIN_TEXT_MEDIA_TYPE,
};

#[derive(Debug, Error)]
pub enum FrontendError {
    #[error("Index Page Missing")]
    IndexPageMissing,
    #[error("Failed to read frontend data")]
    IOError(#[from] std::io::Error),
    #[error("File not found")]
    FileNotFound,
    #[error(transparent)]
    HandlebarsError(#[from] handlebars::RenderError),
    #[error(transparent)]
    ZipError(#[from] zip::result::ZipError),
    #[error(transparent)]
    JSONError(#[from] serde_json::Error),
    #[error("Invalid route path {error} in route {path}")]
    InvalidRoutePath { error: &'static str, path: String },
}
impl IntoErrorResponse for FrontendError {
    fn into_response_boxed(self: Box<Self>) -> Response {
        let response = APIErrorResponse::<(), Box<Self>> {
            message: "Frontend Error".into(),
            error: Some(self),
            details: None,
        };

        let response_text = response.to_string();

        ResponseBuilder::internal_server_error()
            .header(CONTENT_TYPE, PLAIN_TEXT_MEDIA_TYPE)
            .body(response_text)
    }
}
#[cfg(not(feature = "frontend"))]
mod no_frontend {
    use axum::extract::{Request, State};

    use crate::{app::NitroRepo, utils::response::ResponseBuilder};

    pub async fn frontend_request(
        State(_): State<NitroRepo>,
        _request: Request,
    ) -> Result<axum::response::Response, crate::error::InternalError> {
        Ok(ResponseBuilder::not_found().empty())
    }
}
