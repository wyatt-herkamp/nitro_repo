use actix_web::{post, web, HttpRequest};
use serde::{Deserialize, Serialize};

use crate::api_response::APIResponse;
use crate::error::request_error::RequestError;

use crate::settings::utils::quick_add;

use crate::system::models::UserPermissions;
use crate::system::utils::{new_user, NewPassword, NewUser};

use crate::DbPool;

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
    _r: HttpRequest,
    request: web::Json<InstallUser>,
) -> Result<APIResponse<bool>, RequestError> {
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
