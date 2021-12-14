use std::fs::{read_to_string, remove_dir_all};
use std::io::Cursor;
use std::path::Path;

use actix_files::Files;
use actix_web::{get, HttpRequest, HttpResponse, web};
use log::debug;
use zip::ZipArchive;

use crate::api_response::SiteResponse;

pub fn init(cfg: &mut web::ServiceConfig) {
    debug!("Loading Frontend!");
    web_data();
    cfg.service(index)
        .service(admin)
        .service(install)
        .service(browse)
        .service(browse_extend)
        .service(login)
        .service(Files::new("/", "frontend").show_files_listing());
}

fn web_data() {
    debug!("Loading Zip!");
    #[cfg(feature = "frontend")]
        {
            let data = include_bytes!(concat!(env!("OUT_DIR"), "/frontend.zip")).as_ref();
            let mut archive = ZipArchive::new(Cursor::new(data)).unwrap();
            let path = Path::new("frontend");
            if path.exists() {
                debug!("Deleting Old Frontend");
                remove_dir_all(&path).unwrap();
            }
            debug!("Extracting Zip!");
            archive.extract(&path).unwrap();
        }
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
    let content = read_to_string(Path::new("frontend").join("index.html"))?;
    return Ok(HttpResponse::Ok().content_type("text/html").body(content));
}
