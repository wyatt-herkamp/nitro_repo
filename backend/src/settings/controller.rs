use actix_web::web::Data;
use actix_web::{get, HttpRequest, web};
use std::ops::Deref;
use sea_orm::DatabaseConnection;

use crate::api_response::{APIResponse, SiteResponse};
use crate::error::response::unauthorized;
use crate::system::utils::get_user_by_header;
use crate::{ NitroRepo};
use crate::system::permissions::options::CanIDo;

#[get("/api/settings/report")]
pub async fn setting_report(
    site: Data<NitroRepo>,
    database: web::Data<DatabaseConnection>,
    r: HttpRequest,
) -> SiteResponse {
    if get_user_by_header(r.headers(), &database).await?.can_i_admin().is_err() {
        return unauthorized();
    }
    let settings = site.settings.read().await;
    APIResponse::from(Some(settings.deref())).respond(&r)
}
