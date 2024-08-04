use super::authentication::api_middleware::AuthenticationLayer;
use super::NitroRepo;
use super::{api, config::NitroRepoConfig};

use anyhow::Context;
use axum::extract::DefaultBodyLimit;
use axum::routing::post;
use axum::{extract::Request, routing::get, Router};
use futures_util::pin_mut;
use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls::ServerConfig;
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::net::TcpListener;
use tokio::signal;
use tokio_rustls::TlsAcceptor;
use tower_http::cors::{Any, CorsLayer};
use tower_service::Service;
use tracing::{error, info, warn};
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
        security,
        ..
    } = config;
    log.init(mode)?;
    let tls = tls
        .map(|tls| {
            rustls_server_config(tls.private_key, tls.certificate_chain)
                .context("Failed to create TLS configuration")
        })
        .transpose()?;

    let site = NitroRepo::new(site, security, sessions, database)
        .await
        .context("Unable to Initialize Website Core")?;
    let cloned_site = site.clone();
    let auth_layer = AuthenticationLayer::from(site.clone());
    let app = Router::new()
        .route("/api/info", get(api::info))
        .route("/api/install", post(api::install))
        .route("/api/user/me", get(api::user::me))
        .route("/api/user/login", post(api::user::login))
        .layer(DefaultBodyLimit::max(max_upload.get_as_bytes()))
        .layer(CorsLayer::new().allow_origin(Any).allow_credentials(true))
        .layer(auth_layer)
        .with_state(site);
    if let Some(tls) = tls {
        start_app_with_tls(tls, app, bind_address).await?;
    } else {
        start_app(app, bind_address, cloned_site).await?;
    }

    Ok(())
}
async fn start_app(app: Router, bind: String, site: NitroRepo) -> anyhow::Result<()> {
    let listener = TcpListener::bind(bind).await?;
    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(site))
        .await?;
    Ok(())
}
async fn start_app_with_tls(
    tls: Arc<ServerConfig>,
    app: Router,
    bind: String,
) -> anyhow::Result<()> {
    let tls_acceptor = TlsAcceptor::from(tls);
    let tcp_listener = TcpListener::bind(bind).await.unwrap();

    pin_mut!(tcp_listener);
    loop {
        let tower_service = app.clone();
        let tls_acceptor = tls_acceptor.clone();

        // Wait for new tcp connection
        let (cnx, addr) = tcp_listener.accept().await.unwrap();

        tokio::spawn(async move {
            // Wait for tls handshake to happen
            let Ok(stream) = tls_acceptor.accept(cnx).await else {
                error!("error during tls handshake connection from {}", addr);
                return;
            };
            let stream = TokioIo::new(stream);
            let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
                tower_service.clone().call(request)
            });

            let ret = hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                .serve_connection_with_upgrades(stream, hyper_service)
                .await;

            if let Err(err) = ret {
                warn!("error serving connection from {}: {}", addr, err);
            }
        });
    }
    Ok(())
}

async fn shutdown_signal(website: NitroRepo) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
    info!("Shutting down");
    website.close().await;
}

fn rustls_server_config(
    key: impl AsRef<Path>,
    cert: impl AsRef<Path>,
) -> anyhow::Result<Arc<ServerConfig>> {
    let mut key_reader = BufReader::new(File::open(key).unwrap());
    let mut cert_reader = BufReader::new(File::open(cert).unwrap());

    let cert_chain = certs(&mut cert_reader).collect::<Result<Vec<_>, _>>()?;
    let mut keys = pkcs8_private_keys(&mut key_reader).collect::<Result<Vec<_>, _>>()?;

    let mut config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            cert_chain,
            rustls::pki_types::PrivateKeyDer::Pkcs8(keys.remove(0)),
        )?;

    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Ok(Arc::new(config))
}
