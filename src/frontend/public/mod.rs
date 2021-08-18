use actix_web::{get, HttpRequest, HttpResponse, web};
use tera::Context;

use crate::DbPool;
use crate::internal_error::InternalError;
use crate::site_response::SiteResponse;
use crate::utils::installed;

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
    return SiteResponse::new("index.html",Context::new(), None);
}
