use super::authentication::api_middleware::AuthenticationLayer;
use super::logging::request_tracing::NitroRepoTracing;
use super::NitroRepo;
use super::{api, config::NitroRepoConfig};

use anyhow::Context;
use axum::extract::DefaultBodyLimit;
use axum::routing::any;
use axum::{extract::Request, Router};
use futures_util::pin_mut;
use http::{HeaderName, HeaderValue};
use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls::ServerConfig;
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::net::SocketAddr;
use std::{fs::File, io::BufReader, path::Path, sync::Arc};
use tokio::net::TcpListener;
use tokio::signal;
use tokio_rustls::TlsAcceptor;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_service::Service;
use tracing::{error, info, warn};

const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");
const POWERED_BY_HEADER: HeaderName = HeaderName::from_static("x-powered-by");
const POWERED_BY_VALUE: HeaderValue = HeaderValue::from_static("Nitro Repo");
pub(crate) async fn start(config: NitroRepoConfig) -> anyhow::Result<()> {
    let NitroRepoConfig {
        database,
        log,
        bind_address,
        max_upload,
        mode,
        sessions,
        tls,

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

    let site = NitroRepo::new(mode, site, security, sessions, database)
        .await
        .context("Unable to Initialize Website Core")?;
    let cloned_site = site.clone();
    let auth_layer = AuthenticationLayer::from(site.clone());
    let app = Router::new()
        .route(
            "/repositories/:storage/:repository/*path",
            any(crate::repository::handle_repo_request),
        )
        .route(
            "/storages/:storage/:repository/*path",
            any(crate::repository::handle_repo_request),
        )
        .merge(api::api_routes())
        .merge(super::open_api::build_router())
        .with_state(site);

    let app = app
        .layer(SetResponseHeaderLayer::if_not_present(
            POWERED_BY_HEADER,
            POWERED_BY_VALUE,
        ))
        .layer(NitroRepoTracing::new_trace_layer())
        .layer(PropagateRequestIdLayer::new(REQUEST_ID_HEADER))
        .layer(DefaultBodyLimit::max(max_upload.get_as_bytes()))
        .layer(SetRequestIdLayer::new(REQUEST_ID_HEADER, MakeRequestUuid))
        .layer(CorsLayer::very_permissive())
        .layer(auth_layer);
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
