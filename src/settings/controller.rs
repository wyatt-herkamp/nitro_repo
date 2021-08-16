use crate::api_response::APIResponse;
use crate::settings::settings::DBSetting;
use crate::siteerror::SiteError;

use crate::settings::action::get_setting;
use crate::utils::{get_user_by_header, installed};
use crate::{action, settings, DbPool};
use actix_web::{get, post, web, HttpRequest};
use serde::{Deserialize, Serialize};

#[get("/api/setting/{setting}")]
pub async fn about_setting(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    web::Path(setting): web::Path<String>,
) -> Result<APIResponse<DBSetting>, SiteError> {
    let connection = pool.get()?;
    installed(&connection)?;

    let option = get_setting(setting.as_str(), &connection)?.ok_or_else(|| SiteError::NotFound)?;
    if !option.setting.public.unwrap_or(false) {
        get_user_by_header(r.headers(), &connection)?.ok_or_else(|| SiteError::NotAuthorized)?;
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
) -> Result<APIResponse<DBSetting>, SiteError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let _user =
        get_user_by_header(r.headers(), &connection)?.ok_or_else(|| SiteError::NotAuthorized)?;
    let mut option =
        get_setting(setting.as_str(), &connection)?.ok_or_else(|| SiteError::NotFound)?;
    option.set_value(request.value.clone());
    settings::action::update_setting(&option, &connection)?;
    let option = get_setting(setting.as_str(), &connection)?.ok_or_else(|| SiteError::NotFound)?;
    return Ok(APIResponse::new(true, Some(option)));
}
