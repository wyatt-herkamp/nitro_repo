use super::authentication::session::SessionManager;
use super::config::NitroRepoConfig;
use super::NitroRepo;

use actix_cors::Cors;
use actix_web::{middleware::DefaultHeaders, web::Data, App, HttpServer};
use anyhow::Context;
use rustls::ServerConfig as RustlsServerConfig;
use rustls_pemfile::{certs, pkcs8_private_keys};
use sea_orm::Database;
use std::fs::File;
use std::io::BufReader;
use tracing_actix_web::TracingLogger;
pub(crate) async fn start(config: NitroRepoConfig) -> anyhow::Result<()> {
    let NitroRepoConfig {
        database,
        log,
        bind_address,
        max_upload,
        mode,
        sessions,
        tls,
        email,
        site,
    } = config;
    log.init(mode)?;
    let database = Database::connect(database)
        .await
        .map(Data::new)
        .with_context(|| "Failed to connect to database")?;

    //  TODO: Run Migrations

    let session_manager = SessionManager::new(sessions).map(Data::new)?;
    let site = Data::new(
        NitroRepo::new(site, database.clone())
            .await
            .context("Unable to Initialize Website Core")?,
    );
    let server = HttpServer::new(move || {
        App::new()
            .wrap(DefaultHeaders::new().add(("X-Powered-By", "By the power of Rust")))
            .wrap(
                Cors::default()
                    .allow_any_header()
                    .allow_any_method()
                    .allow_any_origin()
                    .supports_credentials(),
            )
            .wrap(TracingLogger::default())
            .app_data(session_manager.clone())
            .app_data(site.clone())
            .app_data(database.clone())
    });

    if let Some(tls_config) = tls {
        let mut cert_file = BufReader::new(File::open(tls_config.certificate_chain)?);
        let mut key_file = BufReader::new(File::open(tls_config.private_key)?);

        let cert_chain = certs(&mut cert_file).collect::<Result<Vec<_>, _>>()?;
        let mut keys = pkcs8_private_keys(&mut key_file).collect::<Result<Vec<_>, _>>()?;

        let config = RustlsServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(
                cert_chain,
                rustls::pki_types::PrivateKeyDer::Pkcs8(keys.remove(0)),
            )?;
        server.bind_rustls_0_23(bind_address, config)?.run().await
    } else {
        server.bind(bind_address)?.run().await
    }?;
    Ok(())
}
