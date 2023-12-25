use std::{
    env::{current_dir, set_var},
    error::Error,
    fs::File,
    io::{BufReader, ErrorKind},
    path::PathBuf,
    process::exit,
};

use actix_cors::Cors;
use actix_web::{
    middleware::{DefaultHeaders, Logger},
    web,
    web::{Data, PayloadConfig},
    App, HttpServer,
};
use api::{
    authentication,
    authentication::{middleware::HandleSession, session::SessionManager},
    frontend,
    generators::GeneratorCache,
    repository,
    settings::{load_configs, models::GeneralSettings},
    storage,
    storage::multi::MultiStorageController,
    system, tracing_setup, NitroRepo, Version,
};
use log::{info, trace};
use rustls::{Certificate, PrivateKey, ServerConfig as RustlsServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use tempfile::tempdir;
use tokio::{fs::read_to_string, sync::RwLock};
use tracing_actix_web::TracingLogger;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config_dir = std::env::var("NITRO_CONFIG_DIR")
        .map(|x| PathBuf::from(x))
        .unwrap_or_else(|_| current_dir().unwrap());
    let configs = config_dir.join("cfg");
    let main_config = config_dir.join("nitro_repo.toml");
    if !main_config.exists() {
        eprintln!(
            "Config not found. Should be at {:?}",
            main_config.as_os_str()
        );
        exit(1)
    }

    let init_settings: GeneralSettings =
        toml::from_str(&read_to_string(&main_config).await?).map_err(convert_error)?;
    let version = Version::new(init_settings.internal.version.clone());
    if init_settings.internal.version.major != version.cargo_version.major
        && init_settings.internal.version.minor != version.cargo_version.minor
    {
        eprintln!(
            "Version mismatch. Expected {:?} but found {:?}",
            version.cargo_version, init_settings.internal.version
        );
        exit(1)
    }

    // Sets the Log Location
    tracing_setup::setup(init_settings.application.logging.clone())
        .expect("Failed to setup tracing");

    set_var("FRONTEND", &init_settings.application.frontend);
    trace!("Frontend {:?}", init_settings.application.frontend);
    set_var(
        "STORAGE_LOCATION",
        &init_settings.application.storage_location,
    );
    trace!(
        "Storage Location {:?}",
        init_settings.application.storage_location
    );

    info!("Initializing Database Connection");
    let connection = sea_orm::Database::connect(init_settings.database.clone())
        .await
        .map_err(convert_error)?;
    info!("Initializing Session and Authorization");
    let session_manager = SessionManager::new(init_settings.session.clone()).unwrap();
    info!("Initializing State");
    let settings = load_configs(configs).await.map_err(convert_error)?;

    let storages = init_settings
        .application
        .storage_location
        .join("storages.json");
    trace!("Loading Storages from {storages:?}");
    let storages = MultiStorageController::init(storages)
        .await
        .map_err(convert_error)?;

    let nitro_repo = NitroRepo {
        settings: RwLock::new(settings),
        core: init_settings,
        current_version: version,
    };
    info!("Version: {:?}", nitro_repo.current_version);
    let application = nitro_repo.core.application.clone();
    let max_upload =
        Data::new(PayloadConfig::default().limit(application.max_upload * 1024 * 1024));
    let dir = tempdir()?;
    let cache = GeneratorCache {
        local_path: dir.into_path(),
    };

    let address = application.address.clone();
    let storages_data = Data::new(storages);
    let site_state = Data::new(nitro_repo);
    let database_data = Data::new(connection);
    let session_data = Data::new(session_manager);

    
    let cache = Data::new(cache);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(storages_data.clone())
            .app_data(site_state.clone())
            .app_data(database_data.clone())
            .app_data(session_data.clone())
            .app_data(max_upload.clone())
            .app_data(cache.clone())
            .wrap(TracingLogger::default())
            .wrap(DefaultHeaders::new().add(("X-Powered-By", "Nitro Repo powered by Actix.rs")))
            .wrap(
                Cors::default()
                    .allow_any_header()
                    .allow_any_method()
                    .allow_any_origin()
                    .supports_credentials(),
            )
            .wrap(Logger::default())
            .service(
                web::scope("/api")
                    .wrap(HandleSession(session_data.clone().into_inner()))
                    .wrap(DefaultHeaders::new().add(("Content-Type", "application/json")))
                    .configure(system::web::init_public_routes)
                    .configure(system::web::user_routes)
                    .configure(authentication::auth_token::web::authentication_router)
                    .configure(storage::multi::web::init_public_routes)
                    .configure(repository::web::multi::public::init_public)
                    .service(
                        web::scope("/admin")
                            .configure(system::web::init_user_manager_routes)
                            .configure(storage::multi::web::init_admin_routes)
                            .configure(repository::web::multi::init_admin),
                    ),
            )
            .service(
                web::scope("")
                    .wrap(HandleSession(session_data.clone().into_inner()))
                    .configure(repository::web::multi::init_repository_handlers)
                    .configure(frontend::init),
            )
    });

    let server = if let Some(tls) = application.tls {
        let mut cert_file = BufReader::new(File::open(tls.certificate_chain)?);
        let mut key_file = BufReader::new(File::open(tls.private_key)?);

        let cert_chain = certs(&mut cert_file)
            .expect("server certificate file error")
            .into_iter()
            .map(Certificate)
            .collect();
        let mut keys: Vec<PrivateKey> = pkcs8_private_keys(&mut key_file)
            .expect("server private key file error")
            .into_iter()
            .map(PrivateKey)
            .collect();

        let config = RustlsServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(cert_chain, keys.remove(0))
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        server.bind_rustls_021(address, config)?
    } else {
        server.bind(address)?
    };
    server.run().await
}

fn convert_error<E: Into<Box<dyn Error + Send + Sync>>>(e: E) -> std::io::Error {
    std::io::Error::new(ErrorKind::Other, e)
}
