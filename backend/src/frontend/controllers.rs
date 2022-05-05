use std::fs::read_to_string;
use std::path::Path;

use actix_files::Files;
use actix_web::web::Data;
use actix_web::{web, HttpResponse, Responder};
use handlebars::Handlebars;
use log::{debug, trace, warn};
use serde_json::json;

use crate::NitroRepoData;

pub fn init(cfg: &mut web::ServiceConfig) {
    debug!("Loading Frontend!");
    let frontend_string = std::env::var("FRONTEND").unwrap_or_else(|_| "frontend".to_string());
    let frontend_path = Path::new(&frontend_string);
    let index = frontend_path.join("index.html");
    trace!("Frontend Path {}", frontend_path.display());
    if !frontend_path.exists() {
        warn!("Frontend Not Found");
        return;
    }
    let mut reg = Handlebars::new();
    let content = read_to_string(index).expect("Unable to read index.html");
    reg.register_template_string("index", content)
        .expect("Unable to Parse Template");
    let reg = Data::new(reg);
    cfg.app_data(reg.clone())
        .route("/me", web::get().to(frontend_handler))
        .route("/browse/{file:.*}", web::get().to(frontend_handler))
        .route("/browse", web::get().to(frontend_handler))
        .route("/admin", web::get().to(frontend_handler))
        .route("/admin/{file:.*}", web::get().to(frontend_handler))
        .route("/upload/{file:.*}", web::get().to(frontend_handler))
        .route("/repository/{file:.*}", web::get().to(frontend_handler))
        .route("/project/{file:.*}", web::get().to(frontend_handler))
        .route("/", web::get().to(frontend_handler))
        .service(Files::new("/", frontend_path).show_files_listing());
}

pub async fn frontend_handler(
    hb: web::Data<Handlebars<'_>>,
    site: NitroRepoData,
) -> impl Responder {
    let guard = site.settings.read().await;

    let value = json!({"base_url":     site.core.application.app_url, "title": guard.site.name,"description": guard.site.description});
    let content = hb.render("index", &value)?;
    return Ok(HttpResponse::Ok().content_type("text/html").body(content));
}
