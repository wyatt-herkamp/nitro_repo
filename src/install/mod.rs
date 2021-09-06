use actix_web::{get, post, web, HttpRequest};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::api_response::{APIErrorResponse, APIResponse};
use crate::apierror::APIError;
use crate::settings::action::get_setting;
use crate::settings::settings::DBSetting;
use crate::settings::utils::quick_add;
use crate::system::action::add_new_user;
use crate::system::utils::{new_user, NewUser};
use crate::utils::get_current_time;
use crate::{settings, utils, DbPool};

#[get("/api/installed")]
pub async fn installed(pool: web::Data<DbPool>) -> Result<APIResponse<bool>, APIError> {
    let connection = pool.get()?;
    let result = utils::installed(&connection);
    if result.is_err() {
        return Ok(APIResponse::new(true, Some(false)));
    }
    Ok(APIResponse::new(true, Some(true)))
}
