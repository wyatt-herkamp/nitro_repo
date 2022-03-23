use std::fs::{read_to_string, remove_dir_all};
use std::io::Cursor;
use std::path::Path;

use actix_files::Files;
use actix_web::{web, HttpResponse};
use actix_web::web::Data;
use handlebars::Handlebars;
use log::debug;
use zip::ZipArchive;
use serde_json::json;

use crate::api_response::SiteResponse;
use crate::{NitroRepoData};

pub fn init(cfg: &mut web::ServiceConfig) {
    debug!("Loading Frontend!");
    web_data();
    let mut reg = Handlebars::new();
    let content = read_to_string(Path::new("frontend").join("index")).expect("Unable to read index.html");
    reg.register_template_string("index", content).expect("Unable to Parse Template");
    let reg = Data::new(reg);
    cfg
        .app_data(reg.clone())
        .route("/me", web::get().to(frontend_handler))
        .route("/browse/{file:.*}", web::get().to(frontend_handler))
        .route("/browse", web::get().to(frontend_handler))
        .route("/admin", web::get().to(frontend_handler))
        .route("/admin/{file:.*}", web::get().to(frontend_handler))
        .route("/upload/{file:.*}", web::get().to(frontend_handler))
        .route("/repository/{file:.*}", web::get().to(frontend_handler))
        .route("/project/{file:.*}", web::get().to(frontend_handler))
        .route("/", web::get().to(frontend_handler))
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


pub async fn frontend_handler(hb: web::Data<Handlebars<'_>>, site: NitroRepoData) -> SiteResponse {
    let value = json!({"base_url":     site.core.application.app_url});
    let content = hb.render("index", &value)?;
    return Ok(HttpResponse::Ok().content_type("text/html").body(content));
}
