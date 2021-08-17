use crate::api_response::{APIErrorResponse, APIResponse};
use crate::settings::settings::DBSetting;
use crate::apierror::APIError;

use crate::settings::action::get_setting;
use crate::settings::utils::quick_add;
use crate::utils::{get_current_time, NewUser};
use crate::{ settings, utils, DbPool};
use actix_web::{get, post, web, HttpRequest};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[get("/api/installed")]
pub async fn installed(pool: web::Data<DbPool>) -> Result<APIResponse<bool>, APIError> {
    let connection = pool.get()?;
    utils::installed(&connection)?;
    Ok(APIResponse::new(true, Some(true)))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstallRequest {
    pub new_user: NewUser,
}

#[post("/api/install")]
pub async fn install(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    request: web::Json<InstallRequest>,
) -> Result<APIResponse<Value>, APIError> {
    let connection = pool.get()?;

    quick_add("installed", "true".to_string(), &connection)?;
    quick_add(
        "version",
        env!("CARGO_PKG_VERSION").to_string(),
        &connection,
    )?;

    return Ok(APIResponse::<Value>::new(true, None));
}
