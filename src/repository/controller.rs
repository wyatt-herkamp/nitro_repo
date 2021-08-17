use crate::api_response::APIResponse;
use crate::settings::settings::DBSetting;
use crate::siteerror::SiteError;

use crate::settings::action::get_setting;
use crate::utils::{installed};
use crate::{settings, DbPool};
use actix_web::{get, post, patch, put, delete, head, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::system::utils::get_user_by_header;
use crate::siteerror::SiteError::NotAuthorized;
use crate::system::models::User;
use futures::StreamExt;
use actix_web::web::Bytes;

//

#[get("/storages/{storage}/{repository}/{file:.*}")]
pub async fn get_repository(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>, ) -> Result<APIResponse<User>, SiteError> {
    let connection = pool.get()?;
     installed(&connection)?;
    println!("GET {}", path.0.0);
    println!("{}", path.0.1);
    println!("{}", path.0.2);
    println!("{}", r.headers().get("Authorization").unwrap().to_str().unwrap());
    return Ok(APIResponse::new(true, None));
}

#[post("/storages/{storage}/{repository}/{file:.*}")]
pub async fn post_repository(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>, bytes: Bytes) -> Result<APIResponse<User>, SiteError> {
    let connection = pool.get()?;
    installed(&connection)?;
    println!("POST {}", path.0.0);
    println!("{}", path.0.1);
    println!("{}", path.0.2);
    println!("{}", r.headers().get("Authorization").unwrap().to_str().unwrap());

    return Ok(APIResponse::new(true, None));
}

#[patch("/storages/{storage}/{repository}/{file:.*}")]
pub async fn patch_repository(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,bytes: Bytes ) -> Result<APIResponse<User>, SiteError> {
    let connection = pool.get()?;
    installed(&connection)?;
    println!("PATCH {}", path.0.0);
    println!("{}", path.0.1);
    println!("{}", path.0.2);
    println!("{}", r.headers().get("Authorization").unwrap().to_str().unwrap());

    return Ok(APIResponse::new(true, None));
}

#[put("/storages/{storage}/{repository}/{file:.*}")]
pub async fn put_repository(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>, bytes: Bytes) -> Result<APIResponse<User>, SiteError> {
    let connection = pool.get()?;
     installed(&connection)?;
    println!("HEAD {}", path.0.0);
    println!("{}", path.0.1);

    println!("{}", path.0.2);
    for x in r.headers().keys() {
        println!("{}: {}", &x, r.headers().get(x).unwrap().to_str().unwrap());
    }
    return Ok(APIResponse::new(true, None));
}

#[head("/storages/{storage}/{repository}/{file:.*}")]
pub async fn head_repository(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String)>, ) -> Result<APIResponse<User>, SiteError> {
    let connection = pool.get()?;
      installed(&connection)?;
    println!("PUT {}", path.0.0);
    println!("{}", path.0.1);

    println!("{}", path.0.2);
    for x in r.headers().keys() {
        println!("{}: {}", &x, r.headers().get(x).unwrap().to_str().unwrap());
    }
    return Ok(APIResponse::new(true, None));
}