use actix_web::{get, web};

use crate::api_response::APIResponse;

use crate::error::request_error::RequestError;
use crate::{utils, DbPool};
use actix_web::{post, HttpRequest};
use serde::{Deserialize, Serialize};

use crate::settings::utils::quick_add;

use crate::system::models::UserPermissions;
use crate::system::utils::{new_user, NewPassword, NewUser};

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(install_post).service(installed);
}

#[get("/api/installed")]
pub async fn installed(pool: web::Data<DbPool>) -> Result<APIResponse<bool>, RequestError> {
    let connection = pool.get()?;
    let result = utils::installed(&connection);
    if result.is_err() {
        return Ok(APIResponse::new(true, Some(false)));
    }
    Ok(APIResponse::new(true, Some(true)))
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
    _r: HttpRequest,
    b: web::Bytes,
) -> Result<APIResponse<bool>, RequestError> {
    let string = String::from_utf8(b.to_vec()).unwrap();
    let request: InstallUser = serde_json::from_str(string.as_str()).unwrap();
    println!("HERe");
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
        }),
        permissions: UserPermissions::new_owner(),
    };
    let _result = new_user(user, &connection)?;

    quick_add("installed", "true".to_string(), &connection)?;
    quick_add(
        "version",
        env!("CARGO_PKG_VERSION").to_string(),
        &connection,
    )?;
    quick_add(
        "version",
        env!("CARGO_PKG_VERSION").to_string(),
        &connection,
    )?;
    return Ok(APIResponse::new(true, Some(true)));
}
