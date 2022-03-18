use actix_web::web::Data;
use actix_web::{get, HttpRequest};
use std::ops::Deref;

use crate::api_response::{APIResponse, SiteResponse};
use crate::error::response::unauthorized;
use crate::system::utils::get_user_by_header;
use crate::{Database, NitroRepo};

#[get("/api/settings/report")]
pub async fn setting_report(
    site: Data<NitroRepo>,
    database: Database,
    r: HttpRequest,
) -> SiteResponse {
    let connection = database.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() || !user.unwrap().permissions.admin {
        return unauthorized();
    }
    let settings = site.settings.lock().unwrap();
    APIResponse::from(Some(settings.deref())).respond(&r)
}
