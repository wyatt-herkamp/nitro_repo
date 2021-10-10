#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate dotenv;
extern crate strum;
extern crate strum_macros;

use std::path::Path;

use actix_cors::Cors;
use actix_files::Files;
use actix_web::{get, middleware, web, App, HttpRequest, HttpResponse, HttpServer};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use log::error;
use log::info;
use log4rs::config::RawConfig;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

use crate::error::request_error::RequestError;

use crate::utils::{installed, Resources};
use actix_web::error::JsonPayloadError;
use actix_web::web::PayloadConfig;
use std::fs::read_to_string;

pub mod api_response;
pub mod error;
pub mod frontend;
pub mod install;
pub mod internal_error;
pub mod repository;
pub mod schema;
pub mod settings;
pub mod storage;
pub mod system;
pub mod utils;

type DbPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;
embed_migrations!();
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=trace");
    std::env::set_var("RUST_BACKTRACE", "1");
    dotenv::dotenv().ok();
    let config: RawConfig =
        serde_yaml::from_str(Resources::file_get_string("log.yml").as_str()).unwrap();
    log4rs::init_raw_config(config).unwrap();
    let connspec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<MysqlConnection>::new(connspec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    let connection = pool.get().unwrap();
    embedded_migrations::run_with_output(&connection, &mut std::io::stdout()).unwrap();
    info!("Test");

    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_header()
            .allow_any_method()
            .allow_any_origin();
        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .data(pool.clone())
            .data(PayloadConfig::new(1 * 1024 * 1024))
            .app_data(
                web::JsonConfig::default().limit(4096).error_handler(
                    |error, request| match error {
                        JsonPayloadError::Overflow => {
                            return actix_web::error::ErrorBadRequest(
                                RequestError::MissingArgument("Overflow".into())
                                    .to_json_response()
                                    .value,
                            );
                        }
                        JsonPayloadError::ContentType => {
                            return actix_web::error::ErrorBadRequest(
                                RequestError::MissingArgument("Invalid Content Type".into())
                                    .to_json_response()
                                    .value,
                            );
                        }
                        JsonPayloadError::Deserialize(serde) => {
                            return actix_web::error::ErrorBadRequest(
                                RequestError::MissingArgument(
                                    format!("Invalid Json {}", serde.to_string()).into(),
                                )
                                .to_json_response()
                                .value,
                            );
                        }
                        JsonPayloadError::Payload(payload) => {
                            return actix_web::error::ErrorBadRequest(
                                RequestError::MissingArgument(
                                    format!("Bad payload {}", payload.to_string()).into(),
                                )
                                .to_json_response()
                                .value,
                            );
                        }
                    },
                ),
            )
            .service(install::installed)
            .service(settings::controller::update_setting)
            .service(settings::controller::about_setting)
            .service(settings::controller::setting_report)
            .service(repository::admin::controller::add_repo)
            .service(repository::admin::controller::list_repos)
            .service(repository::admin::controller::modify_security)
            .service(repository::admin::controller::modify_settings)
            .service(storage::admin::controller::add_storage)
            .service(storage::admin::controller::list_storages)
            .service(storage::admin::controller::get_by_id)
            .configure(repository::init)
            .configure(frontend::install::init)
            .configure(system::controllers::init)
            .service(index)
            .service(admin)
            .service(browse)
            .service(browse_extend)
            .service(login)
        //.service(
        //    Files::new("/", format!("{}", std::env::var("SITE_DIR").unwrap()))
        //        .show_files_listing(),
        //)
    })
    .workers(2);
    if std::env::var("PRIVATE_KEY").is_ok() {
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        builder
            .set_private_key_file(std::env::var("PRIVATE_KEY").unwrap(), SslFiletype::PEM)
            .unwrap();
        builder
            .set_certificate_chain_file(std::env::var("CERT_KEY").unwrap())
            .unwrap();

        server
            .bind_openssl(std::env::var("ADDRESS").unwrap(), builder)?
            .run()
            .await
    } else {
        server.bind(std::env::var("ADDRESS").unwrap())?.run().await
    }
}

#[get("/")]
pub async fn index(pool: web::Data<DbPool>, _r: HttpRequest) -> Result<HttpResponse, RequestError> {
    let connection = pool.get()?;
    installed(&connection)?;
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
