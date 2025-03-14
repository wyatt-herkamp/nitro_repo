use std::borrow::Cow;

use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::Value;
use utoipa::ToSchema;

use super::{ErrorReason, ResponseBuilder, api_error_response::APIErrorResponse};

#[derive(Serialize, ToSchema)]
pub struct ConflictResponse {
    /// Field that caused the conflict
    pub field: Cow<'static, str>,
}

impl utoipa::IntoResponses for ConflictResponse {
    fn responses() -> std::collections::BTreeMap<
        String,
        utoipa::openapi::RefOr<utoipa::openapi::response::Response>,
    > {
        let missing_permission_response = APIErrorResponse::<&str, ()>::name();
        utoipa::openapi::response::ResponsesBuilder::new()
            .responses_from_iter([(
                "409",
                utoipa::openapi::ResponseBuilder::new()
                    .description("A conflict occurred")
                    .content(
                        "application/json",
                        utoipa::openapi::content::ContentBuilder::new()
                            .schema(Some(
                                utoipa::openapi::schema::RefBuilder::new()
                                    .ref_location_from_schema_name(missing_permission_response),
                            ))
                            .example(Some(example()))
                            .into(),
                    )
                    .build(),
            )])
            .build()
            .into()
    }
}

fn example() -> Value {
    let response: APIErrorResponse<&str, ()> = APIErrorResponse {
        message: "Conflict".into(),
        details: Some("Some_Field"),
        error: None,
    };
    serde_json::to_value(response).unwrap()
}

impl From<&'static str> for ConflictResponse {
    fn from(field: &'static str) -> Self {
        ConflictResponse {
            field: Cow::Borrowed(field),
        }
    }
}
impl From<String> for ConflictResponse {
    fn from(field: String) -> Self {
        ConflictResponse {
            field: Cow::Owned(field),
        }
    }
}

impl IntoResponse for ConflictResponse {
    fn into_response(self) -> Response {
        let response: APIErrorResponse<&str, ()> = APIErrorResponse {
            message: "Conflict".into(),
            details: Some(self.field.as_ref()),
            error: None,
        };

        ResponseBuilder::conflict()
            .extension(ErrorReason::from(format!(
                "Conflict on field: {}",
                self.field
            )))
            .json(&response)
    }
}
