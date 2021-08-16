pub mod api_response;
pub mod install;
pub mod schema;
pub mod settings;
pub mod siteerror;
pub mod utils;
pub mod system;

#[macro_use]
extern crate lazy_static_include;
#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate dotenv;
extern crate strum;
extern crate strum_macros;

use std::path::Path;

use std::env;

use actix_web::{
    get, middleware, post, web, App, HttpRequest, HttpResponse, HttpServer, ResponseError,
};

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

use log::info;

use actix_cors::Cors;

use crate::settings::settings::get_file;
use crate::siteerror::SiteError;
use crate::utils::Resources;
use log4rs::config::RawConfig;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

type DbPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;
embed_migrations!();
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=trace");
    std::env::set_var("RUST_BACKTRACE", "1");
    let config: RawConfig =
        serde_yaml::from_str(Resources::file_get_string("log.yml").as_str()).unwrap();
    log4rs::init_raw_config(config).unwrap();
    dotenv::dotenv().ok();
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
            .service(install::install)
            .service(install::installed)
                 //.service(settings::controller::update_setting)
            .service(settings::controller::about_setting)
                    .default_service(web::route().to(|| SiteError::NotFound.error_response()))
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

        server.bind_openssl("0.0.0.0:6742", builder)?.run().await
    } else {
        server.bind("0.0.0.0:6742")?.run().await
    }
}
