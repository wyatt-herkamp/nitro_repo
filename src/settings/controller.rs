use crate::api_response::APIResponse;
use crate::settings::settings::DBSetting;
use crate::apierror::APIError;

use crate::settings::action::get_setting;
use crate::utils::{installed};
use crate::{settings, DbPool};
use actix_web::{get, post, web, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::system::utils::get_user_by_header;
use crate::apierror::APIError::NotAuthorized;

#[get("/api/setting/{setting}")]
pub async fn about_setting(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    web::Path(setting): web::Path<String>,
) -> Result<APIResponse<DBSetting>, APIError> {
    let connection = pool.get()?;
    installed(&connection)?;

    let option = get_setting(setting.as_str(), &connection)?.ok_or_else(|| APIError::NotFound)?;
    if !option.setting.public.unwrap_or(false) {
        return Err(APIError::NotAuthorized);
    }
    return Ok(APIResponse::new(true, Some(option)));
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateSettingRequest {
    pub value: String,
}

#[post("/api/admin/setting/{setting}/update")]
pub async fn update_setting(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    request: web::Json<UpdateSettingRequest>,
    web::Path(setting): web::Path<String>,
) -> Result<APIResponse<DBSetting>, APIError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let user =
        get_user_by_header(r.headers(), &connection)?.ok_or_else(|| APIError::NotAuthorized)?;
    if !user.permissions.permissions.contains(&"ADMIN".to_string()) {
        return Err(NotAuthorized);
    }
    let mut option =
        get_setting(setting.as_str(), &connection)?.ok_or_else(|| APIError::NotFound)?;
    option.set_value(request.value.clone());
    settings::action::update_setting(&option, &connection)?;
    let option = get_setting(setting.as_str(), &connection)?.ok_or_else(|| APIError::NotFound)?;
    return Ok(APIResponse::new(true, Some(option)));
}
