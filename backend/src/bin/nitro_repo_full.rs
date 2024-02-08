use std::env::{current_dir, set_var};
use std::error::Error;

use std::io::ErrorKind;
use std::path::PathBuf;
use std::process::exit;

use actix_cors::Cors;

use actix_web::middleware::{DefaultHeaders, Logger};
use actix_web::web::{Data, PayloadConfig};
use actix_web::{web, App, HttpServer};
use api::authentication::middleware::HandleSession;
use api::authentication::session::{session_cleaner, SessionManager};

use api::generators::GeneratorCache;
use api::settings::load_configs;
use api::settings::models::GeneralSettings;

use api::storage::multi::MultiStorageController;

use api::utils::load_logger;
use api::{authentication, frontend, repository, storage, system, NitroRepo, Version};
use log::{info, trace};

use tempfile::tempdir;
use tokio::fs::read_to_string;
use tokio::sync::RwLock;

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
    set_var("LOG_LOCATION", &init_settings.application.log);
    load_logger(&init_settings.application.mode);

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
    let session_manager = SessionManager::try_from(init_settings.session.clone()).unwrap();
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
    #[cfg(feature = "unsafe_cookies")]
    {
        #[cfg(not(debug_assertions))]
        {
            compile_error!("You are not in a development environment");
        }
        use api::authentication::session::SessionManagerType;
        use chrono::{Duration, Local};
        use core::str::FromStr;
        use log::{debug, error, warn};

        warn!("Using unsafe cookies");
        warn!("This is not recommended. This is only for development purposes");
        warn!("This is not secure");
        warn!("You have been warned");
        let cookies = load_unsafe_cookies();
        match cookies {
            Ok(ok) => {
                for (key, value) in ok {
                    debug!("Setting unsafe cookie: {key} to user {value}");
                    let session = authentication::session::Session {
                        token: key,
                        user: i64::from_str(&value).ok(),
                        expiration: Local::now() + Duration::days(1),
                    };
                    if let Err(e) = session_manager.push_session(session).await {
                        error!("Error setting unsafe cookie: {:?}", e);
                    }
                }
            }
            Err(err) => {
                error!("{}", err);
            }
        }
    }
    let session_data = Data::new(session_manager);

    actix_web::rt::spawn(session_cleaner(session_data.clone().into_inner()));
    let cache = Data::new(cache);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(storages_data.clone())
            .app_data(site_state.clone())
            .app_data(database_data.clone())
            .app_data(session_data.clone())
            .app_data(max_upload.clone())
            .app_data(cache.clone())
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
                    .wrap(HandleSession(true))
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
                    .wrap(HandleSession(false))
                    .configure(repository::web::multi::init_repository_handlers)
                    .configure(frontend::init),
            )
    });

    #[cfg(feature = "ssl")]
    {
        if let Some(private) = application.ssl_private_key {
            use rustls::ServerConfig as RustlsServerConfig;
            use rustls_pemfile::{certs, pkcs8_private_keys};
            use std::fs::File;
            use std::io::BufReader;
            let mut cert_file = BufReader::new(File::open(
                application
                    .ssl_cert_key
                    .expect("Private key Provided but not public key"),
            )?);
            let mut key_file = BufReader::new(File::open(private)?);

            let cert_chain = certs(&mut cert_file)
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            let mut keys = pkcs8_private_keys(&mut key_file)
                .collect::<Result<Vec<_>, _>>()
                .unwrap();

            let config = RustlsServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(
                    cert_chain,
                    rustls::pki_types::PrivateKeyDer::Pkcs8(keys.remove(0)),
                )
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            server.bind_rustls_0_22(address, config)?.run().await?;
            return Ok(());
        }
    }

    server.bind(address)?.run().await
}

fn convert_error<E: Into<Box<dyn Error + Send + Sync>>>(e: E) -> std::io::Error {
    std::io::Error::new(ErrorKind::Other, e)
}

#[cfg(feature = "unsafe_cookies")]
fn load_unsafe_cookies() -> Result<Vec<(String, String)>, std::io::Error> {
    use std::fs::OpenOptions;
    use std::io::BufRead;
    use std::io::BufReader;
    let buf = PathBuf::new().join("unsafe_cookies.txt");
    if !buf.exists() {
        log::warn!("Unsafe Cookies file not found");
        return Ok(vec![]);
    }
    let file = OpenOptions::new().read(true).open(buf)?;
    let lines = BufReader::new(file).lines();
    let mut cookies = Vec::with_capacity(lines.size_hint().0);
    for x in lines {
        let line = x?;
        let mut parts = line.splitn(2, '=');
        let key = parts.next().unwrap();
        let value = parts.next().unwrap();
        cookies.push((key.to_string(), value.to_string()));
    }

    return Ok(cookies);
}
