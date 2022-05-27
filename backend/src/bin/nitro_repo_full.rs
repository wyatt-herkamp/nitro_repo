use std::env::current_dir;
use std::error::Error;
use std::io::ErrorKind;
use std::net::ToSocketAddrs;
use std::process::exit;
use std::rc::Rc;
use actix_web::{App, HttpServer, web};
use actix_web::guard::GuardContext;
use actix_web::web::Data;
use log::{info, trace};
use semver::Version;
use tokio::fs::read_to_string;
use tokio::sync::RwLock;
use api::cli::handle_cli;
use api::{NitroRepo, system};
use api::settings::load_configs;
use api::settings::models::GeneralSettings;
use api::storage::multi::MultiStorageController;
use api::utils::load_logger;
use actix_web::main;
use sea_orm::DatabaseConnection;
use api::authentication::middleware::HandleSession;
use api::authentication::session::SessionManager;

#[main]
async fn main() -> std::io::Result<()> {
    if handle_cli().await.map_err(convert_error)? {
        return Ok(());
    }
    let current_dir = current_dir()?;
    let configs = current_dir.join("cfg");
    let main_config = current_dir.join("nitro_repo.toml");
    if !main_config.exists() {
        eprintln!("Config not found. Should be at {:?}", main_config.as_os_str());
        exit(1)
    }

    let init_settings: GeneralSettings = toml::from_str(&read_to_string(&main_config).await?).map_err(convert_error)?;
    let version = Version::parse(&init_settings.internal.version).map_err(convert_error)?;
    // Sets the Log Location
    std::env::set_var("LOG_LOCATION", &init_settings.application.log);
    load_logger(&init_settings.application.mode);
    info!("Initializing Database Connection");
    let connection = sea_orm::Database::connect(init_settings.database.clone()).await.map_err(convert_error);
    info!("Initializing Session and Authorization");
    let session_manager = SessionManager::try_from(init_settings.session.clone()).unwrap();
    info!("Initializing State");
    let settings = load_configs(configs).await.map_err(convert_error)?;


    let storages = current_dir.join("storages.json");
    let storages = MultiStorageController::init(storages).await.map_err(convert_error)?;

    let nitro_repo = NitroRepo {
        settings: RwLock::new(settings),
        core: init_settings,
        current_version: version,
    };

    let application = nitro_repo.core.application.clone();

    let max_upload = application.max_upload;

    let address = application.address.clone();
    let storages_data = Data::new(storages);
    let site_state = Data::new(nitro_repo);
    let database_data = Data::new(connection);
    let session_data = Data::new(session_manager);

    let server = HttpServer::new(move || {
        App::new().
            app_data(storages_data.clone()).
            app_data(site_state.clone()).
            app_data(database_data.clone()).
            app_data(session_data.clone()).wrap(HandleSession {})
            .service(
                web::scope("/api")
                    .configure(system::web::init_public_routes)
                    .service(web::scope("/admin"))
            )
    });

    #[cfg(feature = "ssl")]
    {
        if let Some(private) = application.ssl_private_key {
            let cert = application
                .ssl_cert_key
                .expect("If Private Key is set. CERT Should be set");
            use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

            let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
            builder
                .set_private_key_file(private, SslFiletype::PEM)
                .unwrap();
            builder.set_certificate_chain_file(cert).unwrap();
            return server.bind_openssl(address, builder)?.run().await;
        }
    }

    return server.bind(address)?.run().await;
}

fn convert_error<E: Into<Box<dyn Error + Send + Sync>>>(e: E) -> std::io::Error {
    std::io::Error::new(ErrorKind::Other, e)
}