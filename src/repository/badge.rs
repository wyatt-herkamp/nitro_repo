use std::fs::{create_dir_all, read_to_string, File};
use std::io::Write;
use std::path::PathBuf;

use actix_files::NamedFile;
use actix_web::{get, web, HttpRequest};
use badge_maker::{BadgeBuilder, Style};
use tiny_skia::Transform;
use usvg::Options;

use crate::api_response::SiteResponse;
use crate::DbPool;

use crate::repository::controller::to_request;
use crate::repository::maven::MavenHandler;
use crate::repository::models::BadgeSettings;
use crate::repository::npm::NPMHandler;
use crate::repository::types::RepositoryType;

fn file_name(b_s: &BadgeSettings, version: &str, t: &str) -> String {
    return format!(
        "badge-{}-{}-{}-{}.{}",
        b_s.style.to_badge_maker_style(),
        b_s.color,
        b_s.label_color,
        &version,
        t
    );
}

fn load_fonts() -> usvg::fontdb::Database {
    let mut fontdb = usvg::fontdb::Database::new();
    fontdb.load_system_fonts();

    fontdb.set_serif_family("Times New Roman");
    fontdb.set_sans_serif_family("Arial");
    fontdb.set_cursive_family("Comic Sans MS");
    fontdb.set_fantasy_family("Impact");
    fontdb.set_monospace_family("Courier New");

    fontdb
}

#[get("/badge/{storage}/{repository}/{file:.*}/badge.{typ}")]
pub async fn badge(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String, String)>,
) -> SiteResponse {
    let (storage, repository, file, typ) = path.into_inner();
    let connection = pool.get()?;

    let request = to_request(storage, repository, file, &connection)?;

    let x = if request.value.eq("nitro_repo_example") {
        "example".to_string()
    } else {
        match request.repository.repo_type.as_str() {
            "maven" => MavenHandler::latest_version(&request, &r, &connection),
            "npm" => NPMHandler::latest_version(&request, &r, &connection),
            _ => {
                panic!("Unknown REPO")
            }
        }?
    };
    let buf1 = PathBuf::new()
        .join("storages")
        .join(&request.storage.name)
        .join(&request.repository.name)
        .join(&request.value)
        .join(".nitro_repo");
    if !buf1.exists() {
        create_dir_all(&buf1)?;
    }
    let b_s = request.repository.settings.badge;
    let buf = buf1.join(file_name(&b_s, &x, typ.as_str()));
    if buf.exists() {
        let response = NamedFile::open(buf)?.into_response(&r);
        return SiteResponse::Ok(response);
    }
    let svg_file = buf1.join(file_name(&b_s, &x, typ.as_str()));

    if !svg_file.exists() {
        let svg: String = BadgeBuilder::new()
            .style(Style::Flat)
            .label(request.repository.name.as_str())
            .message(x.as_str())
            .style(b_s.style.to_badge_maker_style())
            .color_parse(b_s.color.as_str())
            .label_color_parse(b_s.label_color.as_str())
            .build()
            .unwrap()
            .svg();
        let mut file1 = File::create(&svg_file).unwrap();
        file1.write_all(svg.as_bytes())?;
    }
    if typ.eq("png") {
        //TODO Improve the SVG rendering process

        let string1 = read_to_string(svg_file)?;
        let options = Options {
            resources_dir: None,
            dpi: 0.0,
            font_family: "Times New Roman".to_string(),
            font_size: 12_f64,
            languages: Default::default(),
            shape_rendering: Default::default(),
            text_rendering: Default::default(),
            image_rendering: Default::default(),
            keep_named_groups: false,
            default_size: usvg::Size::new(100.0, 100.0).unwrap(),
            fontdb: load_fonts(),
            image_href_resolver: Default::default()
        };
        let result = usvg::Tree::from_str(string1.as_str(), &options.to_ref()).unwrap();

        let fit_to = usvg::FitTo::Original;
        let size = fit_to
            .fit_to(result.svg_node().size.to_screen_size())
            .unwrap();
        let mut pixmap1 = tiny_skia::Pixmap::new(size.width(), size.height()).unwrap();
        let pixmap = pixmap1.as_mut();
        resvg::render(&result, fit_to, Transform::default(), pixmap).unwrap();
        let svg_file = buf1.join(file_name(&b_s, &x, typ.as_str()));

        pixmap1.save_png(svg_file).unwrap();
    }

    let buf = buf1.join(format!("badge-{}.{}", &x, &typ));
    let response = NamedFile::open(buf)?.into_response(&r);
    SiteResponse::Ok(response)
}
