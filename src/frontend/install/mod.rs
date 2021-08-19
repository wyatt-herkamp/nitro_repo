use actix_web::{get, HttpRequest, post, web};
use serde::{Deserialize, Serialize};
use tera::Context;

use crate::{DbPool, url_raw};
use crate::internal_error::InternalError;
use crate::settings::utils::quick_add;
use crate::site_response::SiteResponse;
use crate::system::models::UserPermissions;
use crate::system::utils::{new_user, NewPassword, NewUser};
use crate::utils::installed;
use crate::error::request_error::RequestError;
use crate::api_response::APIResponse;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(install_post);
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstallUser {
    pub name: String,
    pub username: String,
    pub email: String,
    pub password: String,
    pub password_two: String,
}

#[post("/install")]
pub async fn install_post(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    request: web::Json<InstallUser>,
) -> Result<APIResponse<bool>, RequestError> {
    let connection = pool.get()?;
    if request.password != request.password_two {
        return Err(RequestError::MismatchingPasswords);
    }
    let user = NewUser {
        name: request.name.clone(),
        username: Some(request.username.clone()),
        email: Some(request.email.clone()),
        password: Some(NewPassword {
            password: request.password.clone(),
            password_two: request.password_two.clone(),
        })
        ,
        permissions: UserPermissions::new_owner(),
    };
    let result = new_user(user, &connection)?;

    quick_add("installed", "true".to_string(), &connection)?;
    quick_add(
        "version",
        env!("CARGO_PKG_VERSION").to_string(),
        &connection,
    )?;
    let mut context = Context::new();
    context.insert("url", &url_raw(""));
    return Ok(APIResponse::new(true, Some(true)));
}
