use actix_cors::Cors;
use actix_files::Files;
use actix_web::{middleware, web, App, HttpServer};

use crate::api_response::{APIResponse, SiteResponse};

use crate::error::response::mismatching_passwords;
use crate::{frontend, installed, DbPool};
use actix_web::{post, HttpRequest};
use log::info;
use serde::{Deserialize, Serialize};

use crate::settings::utils::quick_add;
use crate::system::action::add_new_user;

use crate::system::models::{User, UserPermissions};
use crate::system::utils::hash;
use crate::utils::get_current_time;



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
    let user = User {
        id: 0,
        name: request.name,
        username: request.username,
        email: request.email,
        password: hash(request.password)?,
        permissions: UserPermissions::new_owner(),
        created: get_current_time(),
    };
    add_new_user(&user, &connection)?;

    quick_add("installed", "true".to_string(), &connection)?;
    quick_add(
        "version",
        env!("CARGO_PKG_VERSION").to_string(),
        &connection,
    )?;
    info!("Installation Complete");
    APIResponse::new(true, Some(true)).respond(&r)
}
