use std::fs::read_to_string;
use std::path::Path;
use actix_files::Files;

use actix_web::{get, HttpRequest, HttpResponse, web};
use log::debug;

use crate::api_response::SiteResponse;

pub fn init(cfg: &mut web::ServiceConfig) {
    debug!("Loading Development Frontend!");
    cfg.service(index)
        .service(admin)
        .service(install)
        .service(browse)
        .service(browse_extend)
        .service(login)
        .service(Files::new("/", std::env::var("SITE_DIR").unwrap()).show_files_listing())
    ;
}

#[get("/")]
pub async fn index(_r: HttpRequest) -> SiteResponse {
    get_file()
}

#[get("/browse/{file:.*}")]
pub async fn browse_extend(_r: HttpRequest) -> SiteResponse {
    get_file()
}

#[get("/browse")]
pub async fn browse(_r: HttpRequest) -> SiteResponse {
    get_file()
}

#[get("/admin")]
pub async fn admin(_r: HttpRequest) -> SiteResponse {
    get_file()
}

#[get("/login")]
pub async fn login(_r: HttpRequest) -> SiteResponse {
    get_file()
}

#[get("/install")]
pub async fn install(_r: HttpRequest) -> SiteResponse {
    get_file()
}

fn get_file() -> SiteResponse {
    //TODO cache this value at runtime
    let content =
        read_to_string(Path::new(&std::env::var("SITE_DIR").unwrap()).join("index.html"))?;
    return Ok(HttpResponse::Ok().content_type("text/html").body(content));
}
