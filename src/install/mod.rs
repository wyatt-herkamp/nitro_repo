use actix_cors::Cors;
use actix_files::Files;
use actix_web::{App, HttpServer, middleware, web};

use crate::api_response::{APIResponse, SiteResponse};

use crate::error::response::mismatching_passwords;
use crate::{DbPool, frontend, installed};
use actix_web::{post, HttpRequest};
use log::info;
use serde::{Deserialize, Serialize};

use crate::settings::utils::quick_add;
use crate::system::action::add_new_user;

use crate::system::models::{User, UserPermissions};
use crate::system::utils::{hash, NewPassword, NewUser};
use crate::utils::get_current_time;

pub async fn load_installer(pool: DbPool) -> std::io::Result<()> {
    let result = HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_header()
                    .allow_any_method()
                    .allow_any_origin(),
            )
            .wrap(middleware::Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .configure(init)
            .configure(frontend::init)
            .service(installed)
            // TODO Make sure this is the correct way of handling vue and actix together. Also learn about packaging the website.
            .service(Files::new("/", std::env::var("SITE_DIR").unwrap()).show_files_listing())
    })
        .workers(1).bind(std::env::var("ADDRESS").unwrap())?.run().await;
    info!("Installer Loaded. Only 1 web worker. Please Setup your Environment ");
    return result;
}

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
