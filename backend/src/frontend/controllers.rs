use std::borrow::Cow;
use std::fs::{create_dir_all, read_to_string, File};
use std::path::{Path, PathBuf};

use actix_files::Files;
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use actix_web::web::Data;
use actix_web::{web, HttpRequest, HttpResponse};
use handlebars::Handlebars;
use log::{debug, error, info, trace, warn};


use crate::{NitroRepoData, Serialize};

pub fn init(cfg: &mut web::ServiceConfig) {
    debug!("Loading Frontend!");
    let frontend_string = std::env::var("FRONTEND").unwrap_or_else(|_| "frontend".to_string());
    let frontend_path = if frontend_string.ends_with(".zip") {
        let zip_to = Path::new(frontend_string.trim_end_matches(".zip"));
        info!("Unzipping frontend to {}", zip_to.display());
        if !zip_to.exists() {
            create_dir_all(zip_to).unwrap();
            let zip_path = Path::new(&frontend_string);
            if let Err(error) = extract(zip_to, zip_path) {
                error!("Error extracting frontend: {}", error);
            }
            zip_to.to_path_buf()
        } else {
            info!("Frontend already unzipped");
            zip_to.to_path_buf()
        }
    } else {
        Path::new(&frontend_string).to_path_buf()
    };
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

#[derive(Serialize, Debug)]
struct FrontendData<'a> {
    base_url: Cow<'a, str>,
    title: &'a str,
    description: &'a str,
}

pub async fn frontend_handler(
    req: HttpRequest,
    hb: Data<Handlebars<'_>>,
    site: NitroRepoData,
) -> Result<HttpResponse, actix_web::Error> {
    let guard = site.settings.read().await;
    let mut data = FrontendData {
        base_url: Default::default(),
        title: guard.site.name.as_str(),
        description: guard.site.description.as_str(),
    };
    if let Some(value) = site.core.application.app_url.as_ref() {
        data.base_url = value.as_str().into();
    } else {
        let host = if let Some(v) = req.headers().get("host") {
            v.to_str()
                .map_err(|_| ErrorBadRequest("Invalid Host Header"))?
        } else {
            return Err(ErrorBadRequest("No Host Header Found"));
        };
        let schema = req.uri().scheme_str().unwrap_or("http");
        data.base_url = format!("{schema}://{host}").into();
    }
    trace!("Frontend Data: {data:?}");
    let content = hb
        .render("index", &data)
        .map_err(ErrorInternalServerError)?;
    return Ok(HttpResponse::Ok().content_type("text/html").body(content));
}

fn extract(extract_to: impl AsRef<Path>, archive: impl AsRef<Path>) -> Result<(), std::io::Error> {
    let file = File::open(&archive)?;

    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => {
                let components = path.components();
                let buf = components
                    .skip(1)
                    .fold(PathBuf::default(), |buf, component| buf.join(component));
                extract_to.as_ref().join(buf)
            }
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            debug!("File {} extracted to \"{}\"", i, outpath.display());
            create_dir_all(&outpath)?;
        } else {
            debug!(
                "File {} extracted to \"{}\" ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    create_dir_all(&p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}
