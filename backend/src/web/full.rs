use actix_cors::Cors;
use actix_web::web::{Data, PayloadConfig};
use actix_web::{middleware, App, HttpServer};
use api::authentication::session::SessionManager;
use api::cli::handle_cli;
use api::settings::load_configs;
use api::settings::models::{GeneralSettings, MysqlSettings};
use api::utils::load_logger;
use api::{
    authentication, error, frontend, misc, repository, settings, storage, system, NitroRepo,
};
use log::{error, info, trace};
use sea_orm::Database;
use std::path::Path;
use tokio::fs::read_to_string;
use tokio::sync::RwLock;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if handle_cli().await? {
        // Close Program
        std::process::exit(1);
    }
    let path = Path::new("cfg");
    let main_config = path.join("nitro_repo.toml");
    if !main_config.exists() {
        println!("Nitro Repo Not Installed. Please ReRun nitro launcher with the --install flag");
        std::process::exit(1);
    }
    info!("Loading Main Config");
    let installed_version = semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap();

    let string = read_to_string(&main_config).await?;
    let init_settings: GeneralSettings = toml::from_str(&string)?;

    // Sets the Log Location
    std::env::set_var("LOG_LOCATION", &init_settings.application.log);

    load_logger(&init_settings.application.mode);
    for (key, value) in init_settings.env.iter() {
        trace!("Adding Environment Var {key} set to {value}");
        std::env::set_var(key, value);
    }
    info!("Initializing Database");
    let pool = match init_settings.database.db_type.as_str() {
        "mysql" => {
            let result = MysqlSettings::try_from(init_settings.database.settings.clone());
            if let Err(error) = result {
                error!("Unable to load database Settings {error}");
                std::process::exit(1);
            }
            Database::connect(&result.unwrap().to_string()).await
        }
        _ => {
            error!("Invalid Database Type");
            std::process::exit(1);
        }
    };
    if let Err(error) = pool {
        error!("Unable to load database {error}");
        std::process::exit(1);
    }
    let pool = pool.unwrap();
    info!("Loading Other Configs");
    let settings = load_configs().await;
    if let Err(error) = settings {
        error!("Unable to load Settings {error}");
        std::process::exit(1);
    }
    let settings = settings.unwrap();
    info!("Loading Storages");
    let storages = storage::multi::MultiStorageController::init().await;
    if let Err(error) = storages {
        error!("Unable to load Settings {error}");
        std::process::exit(1);
    }
    let storages = storages.unwrap();
    info!("Initializing Web Server");
    let nitro_repo = NitroRepo {
        settings: RwLock::new(settings),
        core: init_settings,
        current_version: installed_version,
    };
    let application = nitro_repo.core.application.clone();

    let max_upload = application.max_upload;

    let address = application.address.clone();
    let session_manager = SessionManager::try_from(nitro_repo.core.session.clone()).unwrap();
    let session_manager = Data::new(session_manager);
    let site_core = Data::new(nitro_repo);
    let storages = Data::new(storages);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_header()
                    .allow_any_method()
                    .allow_any_origin()
                    .supports_credentials(),
            )
            .wrap(crate::authentication::middleware::HandleSession)
            .wrap(middleware::Logger::default())
            .app_data(Data::new(pool.clone()))
            .app_data(site_core.clone())
            .app_data(storages.clone())
            .app_data(session_manager.clone())
            .app_data(Data::new(PayloadConfig::new(
                (max_upload * 1024 * 1024) as usize,
            )))
            .configure(error::handlers::init)
            .configure(settings::init)
            .configure(repository::web::full::init)
            .configure(storage::admin::init)
            .configure(repository::web::full::admin::init)
            .configure(system::controllers::init)
            .configure(misc::init)
            .configure(authentication::auth_token::controllers::init)
            // DONT REGISTER ANYTHING BELOW FRONTEND!
            .configure(frontend::init)
    })
        .workers(2);

    // I am pretty sure this is correctly working
    // If I am correct this will only be available if the feature ssl is added
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
