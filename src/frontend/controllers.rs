use actix_web::{get, web, HttpRequest, HttpResponse};

use crate::error::request_error::RequestError;

use crate::utils::installed;
use crate::DbPool;
use std::fs::read_to_string;
use std::path::Path;

#[get("/")]
pub async fn index(pool: web::Data<DbPool>, _r: HttpRequest) -> Result<HttpResponse, RequestError> {
    let result1 = read_to_string(Path::new(&std::env::var("SITE_DIR").unwrap()).join("index.html"));
    return Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(result1.unwrap()));
}

#[get("/browse/{file:.*}")]
pub async fn browse_extend(
    pool: web::Data<DbPool>,
    _r: HttpRequest,
) -> Result<HttpResponse, RequestError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let result1 = read_to_string(
        Path::new(&std::env::var("SITE_DIR").unwrap()).join("browse/[...browse].html"),
    );
    return Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(result1.unwrap()));
}

#[get("/browse")]
pub async fn browse(
    pool: web::Data<DbPool>,
    _r: HttpRequest,
) -> Result<HttpResponse, RequestError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let result1 =
        read_to_string(Path::new(&std::env::var("SITE_DIR").unwrap()).join("browse/browse.html"));
    return Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(result1.unwrap()));
}

#[get("/admin")]
pub async fn admin(pool: web::Data<DbPool>, _r: HttpRequest) -> Result<HttpResponse, RequestError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let result1 = read_to_string(Path::new(&std::env::var("SITE_DIR").unwrap()).join("admin.html"));
    return Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(result1.unwrap()));
}

#[get("/login")]
pub async fn login(pool: web::Data<DbPool>, _r: HttpRequest) -> Result<HttpResponse, RequestError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let result1 = read_to_string(Path::new(&std::env::var("SITE_DIR").unwrap()).join("login.html"));
    return Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(result1.unwrap()));
}
