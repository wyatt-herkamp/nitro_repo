pub mod install;

use actix_web::{get, web};

use crate::api_response::{APIResponse, SiteResponse};

use crate::error::response::mismatching_passwords;
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
pub async fn installed(pool: web::Data<DbPool>, r: HttpRequest) -> SiteResponse {
    let connection = pool.get()?;
    let result = utils::installed(&connection)?;
    APIResponse::new(true, Some(result)).respond(&r)
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
pub async fn install_post(pool: web::Data<DbPool>, r: HttpRequest, b: web::Bytes) -> SiteResponse {
    let connection = pool.get()?;
    let x = crate::utils::installed(&connection)?;
    if x {
        return APIResponse::new(true, Some(true)).respond(&r);
    }
    let string = String::from_utf8(b.to_vec()).unwrap();
    let request: InstallUser = serde_json::from_str(string.as_str()).unwrap();
    if request.password != request.password_two {
        return mismatching_passwords();
    }
    let user = NewUser {
        name: request.name,
        username: Some(request.username),
        email: Some(request.email),
        password: Some(NewPassword {
            password: request.password,
            password_two: request.password_two,
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
    APIResponse::new(true, Some(true)).respond(&r)
}
