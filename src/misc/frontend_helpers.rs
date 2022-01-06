use actix_web::{get, HttpRequest, web};
use diesel::MysqlConnection;
use log::debug;
use serde::{Deserialize, Serialize};

use crate::{APIResponse, Database, DbPool, SiteResponse};
use crate::error::internal_error::InternalError;
use crate::settings::controller::get_setting_or_empty;

#[derive(Serialize, Deserialize)]
pub struct SiteInfo {
    pub name: String,
    pub description: String,
}

pub fn get_site_info(connection: &MysqlConnection) -> Result<SiteInfo, InternalError> {
    let name = get_setting_or_empty("name.public", &connection)?.value;
    let description = get_setting_or_empty("description", &connection)?.value;
    return Ok(SiteInfo {
        name,
        description,
    });
}

#[get("/api/info/site")]
pub async fn site_info(
    pool: Database,
    request: HttpRequest,
) -> SiteResponse {
    let connection = pool.get()?;

    let info = get_site_info(&connection)?;
    return APIResponse::respond_new(Some(info), &request);
}