use crate::app::logging::request_logging::AppTracingLayer;

use super::authentication::layer::AuthenticationLayer;
use super::config::{load_config, WebServer};
use super::{api, config::NitroRepoConfig};
use super::{open_api, NitroRepo};

use anyhow::Context;
use axum::extract::DefaultBodyLimit;
use axum::routing::any;
use axum::{extract::Request, Router};
use axum_extra::routing::RouterExt;
use futures_util::pin_mut;
use http::{HeaderName, HeaderValue};
use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls::ServerConfig;
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::{fs::File, io::BufReader, path::Path, sync::Arc};
use tokio::net::TcpListener;
use tokio::signal;
use tokio_rustls::TlsAcceptor;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_service::Service;
use tracing::{debug, error, info, warn};

const POWERED_BY_HEADER: HeaderName = HeaderName::from_static("x-powered-by");
const POWERED_BY_VALUE: HeaderValue = HeaderValue::from_static("Nitro Repo");
pub(crate) async fn start(config_path: Option<PathBuf>) -> anyhow::Result<()> {
    let NitroRepoConfig {
        web_server,
        database,
        log,
        mode,
        sessions,
        staging: staging_config,
        site,
        security,
        email,
        suggested_local_storage_path,
    } = load_config(config_path)?;
    let WebServer {
        bind_address,
        max_upload,
        tls,
        open_api_routes,
    } = web_server;

    let mode = mode;
    let site = site;
    let logger = super::logging::init(log)?;

    let site = NitroRepo::new(
        mode,
        site,
        security,
        sessions,
        staging_config,
        email,
        database,
        suggested_local_storage_path,
    )
    .await
    .context("Unable to Initialize Website Core")?;

    site.start_session_cleaner();

    let cloned_site = site.clone();
    let auth_layer = AuthenticationLayer::from(site.clone());
    let mut app = Router::new()
        .route(
            "/repositories/{storage}/{repository}/{*path}",
            any(crate::repository::handle_repo_request),
        )
        .route_with_tsr(
            "/repositories/{storage}/{repository}",
            any(crate::repository::handle_repo_request),
        )
        .route(
            "/storages/{storage}/{repository}/{*path}",
            any(crate::repository::handle_repo_request),
        )
        .route_with_tsr(
            "/storages/{storage}/{repository}",
            any(crate::repository::handle_repo_request),
        )
        .nest("/api", api::api_routes())
        .nest("/badge", super::badge::badge_routes())
        .fallback(super::frontend::frontend_request)
        .with_state(site.clone());

    if open_api_routes {
        info!("OpenAPI routes enabled");
        app = app.merge(open_api::build_router())
    }
    let body_limit: DefaultBodyLimit = max_upload.into();
    let app = app
        .layer(auth_layer)
        .layer(SetResponseHeaderLayer::if_not_present(
            POWERED_BY_HEADER,
            POWERED_BY_VALUE,
        ))
        .layer(AppTracingLayer(site.clone()))
        .layer(body_limit);

    if let Some(tls) = tls {
        debug!("Starting TLS server");
        let tls = rustls_server_config(tls.private_key, tls.certificate_chain)
            .context("Failed to create TLS configuration")?;
        start_app_with_tls(tls, app, bind_address).await?;
    } else {
        debug!("Starting non-TLS server");
        start_app(app, bind_address, cloned_site).await?;
    }

    info!("Server shutdown... Goodbye!");
    // TODO: Figure out how to properly shutdown the logger
    drop(logger);
    Ok(())
}
async fn start_app(app: Router, bind: String, site: NitroRepo) -> anyhow::Result<()> {
    let listener = TcpListener::bind(bind).await?;
    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
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
