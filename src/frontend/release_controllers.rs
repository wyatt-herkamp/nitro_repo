use std::fs::{read_to_string, remove_dir_all};
use std::io::Cursor;
use std::path::Path;

use actix_files::Files;
use actix_web::{get, web, HttpRequest, HttpResponse};
use log::debug;
use zip::ZipArchive;

use crate::api_response::SiteResponse;

pub fn init(cfg: &mut web::ServiceConfig) {
    debug!("Loading Frontend!");
    web_data();
    cfg.service(index)
        .service(me)
        .service(admin)
        .service(admin_extra)
        .service(browse)
        .service(browse_extend)
        .service(upload)
        .service(repository)
        .service(project)

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
}#[get("/me")]
pub async fn me(_r: HttpRequest) -> SiteResponse {
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

#[get("/admin/{file:.*}")]
pub async fn admin_extra(_r: HttpRequest) -> SiteResponse {
    get_file()
}

#[get("/upload/{file:.*}")]
pub async fn upload(_r: HttpRequest) -> SiteResponse {
    get_file()
}

#[get("/repository/{file:.*}")]
pub async fn repository(_r: HttpRequest) -> SiteResponse {
    get_file()
}
#[get("/project/{file:.*}")]
pub async fn project(_r: HttpRequest) -> SiteResponse {
    get_file()
}


fn get_file() -> SiteResponse {
    //TODO cache this value at runtime
    let content = read_to_string(Path::new("frontend").join("index.html"))?;
    return Ok(HttpResponse::Ok().content_type("text/html").body(content));
}
