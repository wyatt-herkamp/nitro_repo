use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

use crate::apierror::APIError;
use crate::internal_error::InternalError;

#[derive(Debug)]
pub struct SiteResponse {
    pub template: String,
    pub context: Context,
    pub status_code: Option<StatusCode>,
}

impl SiteResponse {
    pub fn new(
        template: &str,
        context: Context,
        status_code: Option<StatusCode>,
    ) -> Result<SiteResponse, InternalError> {
        return Ok(SiteResponse {
            template: template.to_string(),
            context,
            status_code,
        });
    }
}

impl Responder for SiteResponse {
    type Error = InternalError;
    type Future = futures_util::future::Ready<Result<HttpResponse, Self::Error>>;

    fn respond_to(self, req: &HttpRequest) -> Self::Future {
        let tera: &actix_web::web::Data<Tera> = req.app_data().unwrap();
        let result1 = tera.render(self.template.as_str(), &self.context);
        if let Err(e) = result1 {
            return futures_util::future::err(InternalError::TeraError(e));
        }
        let result = HttpResponse::Ok()
            .status(self.status_code.unwrap_or(StatusCode::OK))
            .content_type("text/html")
            .body(result1.unwrap());
        return futures_util::future::ok(result);
    }
}
