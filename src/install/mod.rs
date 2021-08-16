use crate::api_response::{APIError, APIResponse};
use crate::settings::settings::DBSetting;
use crate::siteerror::SiteError;

use crate::settings::action::get_setting;
use crate::settings::utils::quick_add;
use crate::utils::{get_current_time, get_user_by_header, new_user, EmailChangeRequest, NewUser};
use crate::{action, settings, utils, DbPool};
use actix_web::{get, post, web, HttpRequest};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[get("/api/installed")]
pub async fn installed(pool: web::Data<DbPool>) -> Result<APIResponse<bool>, SiteError> {
    let connection = pool.get()?;
    utils::installed(&connection)?;
    Ok(APIResponse::new(true, Some(true)))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InstallRequest {
    pub new_user: NewUser,
    pub email: EmailChangeRequest,
}

#[post("/api/install")]
pub async fn install(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    request: web::Json<InstallRequest>,
) -> Result<APIResponse<Value>, SiteError> {
    let connection = pool.get()?;
    quick_add(
        "email.host",
        request
            .email
            .email_host
            .as_ref()
            .unwrap_or(&"".to_string())
            .clone(),
        &connection,
    )?;
    quick_add(
        "email.username",
        request
            .email
            .email_username
            .as_ref()
            .unwrap_or(&"".to_string())
            .clone(),
        &connection,
    )?;
    quick_add(
        "email.password",
        request
            .email
            .email_password
            .as_ref()
            .unwrap_or(&"".to_string())
            .clone(),
        &connection,
    )?;
    quick_add(
        "email.encryption",
        request
            .email
            .encryption
            .as_ref()
            .unwrap_or(&"NONE".to_string())
            .clone(),
        &connection,
    )?;
    quick_add(
        "email.from",
        request
            .email
            .from
            .as_ref()
            .unwrap_or(&"".to_string())
            .clone(),
        &connection,
    )?;
    quick_add(
        "email.port",
        request.email.port.as_ref().unwrap_or(&587).to_string(),
        &connection,
    )?;
    new_user(request.new_user.clone(), &connection)?;
    quick_add("installed", "true".to_string(), &connection)?;
    quick_add(
        "version",
        env!("CARGO_PKG_VERSION").to_string(),
        &connection,
    )?;
    for x in settings::settings::SettingManager::get_settings() {
        if let Some(default) = x.default {
            quick_add(x.key.as_str(), default, &connection)?;
        }
    }
    return Ok(APIResponse::<Value>::new(true, None));
}
