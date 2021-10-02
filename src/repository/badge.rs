use crate::error::request_error::RequestError;

use crate::repository::action::get_repo_by_name_and_storage;
use crate::repository::maven::MavenHandler;
use crate::repository::models::Repository;
use crate::repository::repository::{RepositoryRequest, RepositoryType};

use crate::storage::action::get_storage_by_name;

use crate::utils::installed;
use crate::DbPool;
use actix_files::NamedFile;

use actix_web::{get, HttpRequest, HttpResponse, web};

use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, read_to_string, File};
use std::path::PathBuf;

use badge_maker::{BadgeBuilder, Style};

use std::io::Write;
use usvg::Options;

//

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRepositories {
    pub repositories: Vec<Repository>,
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

#[get("/badge/{storage}/{repository}/{file:.*}/badge.{type}")]
pub async fn badge(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    path: web::Path<(String, String, String, String)>,
) -> Result<HttpResponse, RequestError> {
    println!("HELLO");
    let connection = pool.get()?;
    installed(&connection)?;
    let storage = get_storage_by_name(path.0 .0, &connection)?.ok_or(RequestError::NotFound)?;
    let repository =
        get_repo_by_name_and_storage(path.0 .1.clone(), storage.id.clone(), &connection)?
            .ok_or(RequestError::NotFound)?;

    let t = repository.repo_type.clone();
    let string = path.0 .2.clone();

    let request = RepositoryRequest {
        //TODO DONT DO THIS
        request: r.clone(),
        storage: storage.clone(),
        repository: repository.clone(),
        value: string.clone(),
    };
    let x = match t.as_str() {
        "maven" => MavenHandler::latest_version(request, &connection),
        _ => {
            panic!("Unknown REPO")
        }
    }?;
    let buf1 = PathBuf::new()
        .join("storages")
        .join(storage.name.clone())
        .join(repository.name.clone())
        .join(string.clone())
        .join(".nitro_repo");
    if !buf1.exists() {
        create_dir_all(&buf1)?;
    }
    let typ = path.0 .3.clone();
    let buf = buf1
        .clone()
        .join(format!("badge-{}.{}", x.clone(), typ.clone()));
    if buf.exists() {
        return Ok(NamedFile::open(buf)?.into_response(&r)?);
    }
    let svg_file = buf1.clone().join(format!("badge-{}.svg", x.clone()));

    if !svg_file.exists() {
        let svg: String = BadgeBuilder::new()
            .style(Style::Flat)
            .label(repository.name.as_str())
            .message(x.as_str())
            .style(repository.settings.badge.style.to_badge_maker_style())
            .color_parse(repository.settings.badge.color.as_str())
            .label_color_parse(repository.settings.badge.label_color.as_str())
            .build()
            .unwrap()
            .svg();
        let mut file1 = File::create(&svg_file).unwrap();
        file1.write_all(svg.as_bytes())?;
    }
    if typ.eq("png") {
        let string1 = read_to_string(svg_file)?;
        let options = Options {
            resources_dir: None,
            dpi: 0.0,
            font_family: "Times New Roman".to_string(),
            font_size: 12 as f64,
            languages: Default::default(),
            shape_rendering: Default::default(),
            text_rendering: Default::default(),
            image_rendering: Default::default(),
            keep_named_groups: false,
            default_size: usvg::Size::new(100.0, 100.0).unwrap(),
            fontdb: load_fonts(),
        };
        let result = usvg::Tree::from_str(string1.as_str(), &options.to_ref()).unwrap();

        let fit_to = usvg::FitTo::Original;
        let size = fit_to
            .fit_to(result.svg_node().size.to_screen_size())
            .unwrap();
        let mut pixmap1 = tiny_skia::Pixmap::new(size.width(), size.height()).unwrap();
        let pixmap = pixmap1.as_mut();
        resvg::render(&result, fit_to, pixmap).unwrap();
        let svg_file = buf1.clone().join(format!("badge-{}.png", x.clone()));

        pixmap1.save_png(svg_file).unwrap();
    }

    let buf = buf1
        .clone()
        .join(format!("badge-{}.{}", x.clone(), typ.clone()));
    return Ok(NamedFile::open(buf)?.into_response(&r)?);
}
