use actix_web::{get, web, HttpRequest, HttpResponse};
use crate::DbPool;
use crate::site_response::SiteResponse;
use crate::internal_error::InternalError;
use crate::utils::installed;
use tera::Context;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
}

#[get("/")]
pub async fn index(
    pool: web::Data<DbPool>,
    r: HttpRequest,
) -> Result<SiteResponse, InternalError> {
    let connection = pool.get()?;
    //installed(&connection)?;
    return Ok(SiteResponse::new("index.html",Context::new(), None));
}
