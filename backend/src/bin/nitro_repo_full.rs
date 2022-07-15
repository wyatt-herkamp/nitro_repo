use std::env::{current_dir, set_var};
use std::error::Error;
use std::io::ErrorKind;
use std::process::exit;

use actix_cors::Cors;
use actix_web::main;
use actix_web::middleware::DefaultHeaders;
use actix_web::web::{Data, PayloadConfig};
use actix_web::{web, App, HttpServer};
use log::info;
use semver::Version;
use tokio::fs::read_to_string;
use tokio::sync::RwLock;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use api::api::ApiDoc;
use api::authentication::middleware::HandleSession;
use api::authentication::session::SessionManager;
use api::cli::handle_cli;
use api::settings::load_configs;
use api::settings::models::GeneralSettings;
use api::storage::multi::MultiStorageController;
use api::utils::load_logger;
use api::{frontend, repository, storage, system, NitroRepo};

#[main]
async fn main() -> std::io::Result<()> {
    if handle_cli().await.map_err(convert_error)? {
        return Ok(());
    }
    let current_dir = current_dir()?;
    let configs = current_dir.join("cfg");
    let main_config = current_dir.join("nitro_repo.toml");
    if !main_config.exists() {
        eprintln!(
            "Config not found. Should be at {:?}",
            main_config.as_os_str()
        );
        exit(1)
    }

    let init_settings: GeneralSettings =
        toml::from_str(&read_to_string(&main_config).await?).map_err(convert_error)?;
    let version = Version::parse(&init_settings.internal.version).map_err(convert_error)?;
    // Sets the Log Location
    set_var("LOG_LOCATION", &init_settings.application.log);
    load_logger(&init_settings.application.mode);
    info!("Initializing Database Connection");
    let connection = sea_orm::Database::connect(init_settings.database.clone())
        .await
        .map_err(convert_error)?;
    info!("Initializing Session and Authorization");
    let session_manager = SessionManager::try_from(init_settings.session.clone()).unwrap();
    info!("Initializing State");
    let settings = load_configs(configs).await.map_err(convert_error)?;

    let storages = current_dir.join("storages.json");
    let storages = MultiStorageController::init(storages)
        .await
        .map_err(convert_error)?;

    let nitro_repo = NitroRepo {
        settings: RwLock::new(settings),
        core: init_settings,
        current_version: version,
    };

    let application = nitro_repo.core.application.clone();
    set_var("STORAGE_LOCATION", &application.storage_location);
    let max_upload = Data::new(PayloadConfig::default().limit(application.max_upload));

    let address = application.address.clone();
    let storages_data = Data::new(storages);
    let site_state = Data::new(nitro_repo);
    let database_data = Data::new(connection);
    let session_data = Data::new(session_manager);
    let openapi = ApiDoc::openapi();

    let server = HttpServer::new(move || {
        App::new()
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-doc/openapi.json", openapi.clone()),
            )
            .app_data(storages_data.clone())
            .app_data(site_state.clone())
            .app_data(database_data.clone())
            .app_data(session_data.clone())
            .app_data(max_upload.clone())
            .wrap(DefaultHeaders::new().add(("X-Powered-By", "Nitro Repo powered by Actix.rs")))
            .wrap(
                Cors::default()
                    .allow_any_header()
                    .allow_any_method()
                    .allow_any_origin()
                    .supports_credentials(),
            )
            .service(
                web::scope("/api")
                    .wrap(DefaultHeaders::new().add(("Content-Type", "application/json")))
                    .wrap(HandleSession {})
                    .configure(system::web::init_public_routes)
                    .configure(system::web::user_routes)
                    .service(
                        web::scope("/admin")
                            .configure(system::web::init_user_manager_routes)
                            .configure(storage::multi::web::init_admin_routes)
                            .configure(repository::web::multi::init_admin),
                    )
                    .configure(storage::multi::web::init_public_routes)
                    .configure(repository::web::multi::public::init_public),
            )
            .service(
                web::scope("/nitro_repo/help")
                    .service(repository::web::multi::helpers::help_update_type),
            )
            .configure(repository::web::multi::init_repository_handlers)
            .configure(frontend::init)
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
