use actix_web::{get, HttpRequest, HttpResponse, Responder};

use crate::error::request_error::RequestError;

use std::fs::read_to_string;
use std::path::Path;

#[get("/")]
pub async fn index(_r: HttpRequest) -> Result< impl Responder, RequestError> {
    let result1 = read_to_string(Path::new(&std::env::var("SITE_DIR").unwrap()).join("index.html"));
    return Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(result1.unwrap()));
}

#[get("/browse/{file:.*}")]
pub async fn browse_extend(
    _r: HttpRequest,
) -> Result< impl Responder, RequestError> {
    let result1 = read_to_string(Path::new(&std::env::var("SITE_DIR").unwrap()).join("index.html"));
    return Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(result1.unwrap()));
}

#[get("/browse")]
pub async fn browse(
    _r: HttpRequest,
) -> Result< impl Responder, RequestError> {
    let result1 = read_to_string(Path::new(&std::env::var("SITE_DIR").unwrap()).join("index.html"));
    return Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(result1.unwrap()));
}

#[get("/admin")]
pub async fn admin(_r: HttpRequest) -> Result< impl Responder, RequestError> {
    let result1 = read_to_string(Path::new(&std::env::var("SITE_DIR").unwrap()).join("index.html"));
    return Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(result1.unwrap()));
}

#[get("/login")]
pub async fn login(_r: HttpRequest) -> Result< impl Responder, RequestError> {
    let result1 = read_to_string(Path::new(&std::env::var("SITE_DIR").unwrap()).join("index.html"));
    return Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(result1.unwrap()));
}
#[get("/install")]
pub async fn install(_r: HttpRequest) -> Result< impl Responder, RequestError> {
    let result1 = read_to_string(Path::new(&std::env::var("SITE_DIR").unwrap()).join("index.html"));
    return Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(result1.unwrap()));
}
