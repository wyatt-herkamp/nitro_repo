use super::authentication::api_middleware::HandleSession;
use super::authentication::session::SessionManager;
use super::config::NitroRepoConfig;
use super::NitroRepo;

use actix_cors::Cors;
use actix_web::Scope;
use actix_web::{middleware::DefaultHeaders, web::Data, App, HttpServer};
use anyhow::Context;
use rustls::ServerConfig as RustlsServerConfig;
use rustls_pemfile::{certs, pkcs8_private_keys};
use sqlx::PgPool;
use std::fs::File;
use std::io::BufReader;
use tracing::trace;
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
        server_workers,
    } = config;
    log.init(mode)?;

    let database = PgPool::connect_with(database.into())
        .await
        .map(Data::new)
        .context("Could not connec to database")?;
    //  TODO: Run Migrations
    sqlx::migrate!()
        .run(database.as_ref())
        .await
        .context("Failed to run Migrations")?;
    let session_manager = SessionManager::new(sessions).map(Data::new)?;
    let site = Data::new(
        NitroRepo::new(site, database.clone())
            .await
            .context("Unable to Initialize Website Core")?,
    );
    let cloned_site = site.clone();
    tokio::spawn(async move {
        if let Err(err) = handle_signals(cloned_site).await {
            tracing::error!("Failed to handle signals: {}", err);
        }
    });
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
            .wrap(HandleSession {
                session_manager: session_manager.clone(),
                nitro_repo: site.clone(),
            })
            .service(Scope::new("/api").configure(crate::app::api::init))
    });
    let server = if let Some(workers) = server_workers {
        server.workers(workers)
    } else {
        server
    };

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
async fn handle_signals(website: Data<NitroRepo>) -> anyhow::Result<()> {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for SIGINT");
    website.close().await;
    Ok(())
}
