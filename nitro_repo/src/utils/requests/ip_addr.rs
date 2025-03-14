use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use axum::{
    extract::{ConnectInfo, FromRef, FromRequestParts, OptionalFromRequestParts, Request},
    response::{IntoResponse, Response},
};
use derive_more::{Deref, From, Into};
use http::{HeaderName, request::Parts};
use thiserror::Error;
use tracing::{Level, error, event, instrument};

use crate::utils::{ErrorReason, ResponseBuilder, header::HeaderMapExt};
#[derive(Debug, Error)]
pub enum ConnectionIpAddrError {
    #[error("Invalid IP address format {0}")]
    ParseError(#[from] std::net::AddrParseError),
    #[error("IP address is not present")]
    IpAddressIsNotPresent,
}
impl IntoResponse for ConnectionIpAddrError {
    fn into_response(self) -> Response {
        match self {
            ConnectionIpAddrError::ParseError(_) => ResponseBuilder::bad_request()
                .extension(ErrorReason::from("Invalid IP address format"))
                .empty(),
            ConnectionIpAddrError::IpAddressIsNotPresent => ResponseBuilder::bad_request()
                .extension(ErrorReason::from("IP address is not present"))
                .empty(),
        }
    }
}
/// Tries to rely on the S::forwarded_header() header to get the client's IP address.
/// This is useful when the server is behind a reverse proxy.
///
/// If the header is not present it will try to get the IP address from the connection info.
/// If the connection info is not present it will return `None`.
#[derive(Debug, Clone, Copy, Deref, From, Into)]
pub struct ConnectionIpAddr(pub IpAddr);

impl<S> FromRequestParts<S> for ConnectionIpAddr
where
    S: FromRef<S>,
    S: HasForwardedHeader + Send + Sync,
{
    type Rejection = ConnectionIpAddrError;
    #[instrument(skip(parts, s))]
    async fn from_request_parts(parts: &mut Parts, s: &S) -> Result<Self, Self::Rejection> {
        let forwarded_ip = if let Some(header) = s.forwarded_header() {
            let forwarded_ip = parts
                .headers
                .get_str_ignore_empty(header)
                .map(|x| IpAddr::from_str(x))
                .transpose()
                .map_err(|e| {
                    error!(?e, "Failed to parse IP address from header");
                    ConnectionIpAddrError::ParseError(e)
                })?;
            if forwarded_ip.is_none() && tracing::enabled!(Level::TRACE) {
                event!(Level::TRACE, "No IP address found in header");
            }
            forwarded_ip
        } else {
            event!(Level::TRACE, "Forwarded header is not configured");
            None
        };
        if s.require_forwarded_header() {
            let Some(forwarded_ip) = forwarded_ip else {
                event!(Level::TRACE, "Forwarded header is required but not present");
                return Err(ConnectionIpAddrError::IpAddressIsNotPresent);
            };
            return Ok(ConnectionIpAddr(forwarded_ip));
        }
        let client_ip = forwarded_ip.or_else(|| {
            parts
                .extensions
                .get::<ConnectInfo<SocketAddr>>()
                .map(|ConnectInfo(c)| c.ip())
        });
        let client_ip = client_ip.ok_or(ConnectionIpAddrError::IpAddressIsNotPresent)?;
        Ok(ConnectionIpAddr(client_ip))
    }
}
impl<S> OptionalFromRequestParts<S> for ConnectionIpAddr
where
    S: FromRef<S>,
    S: HasForwardedHeader + Send + Sync,
{
    type Rejection = ConnectionIpAddrError;
    #[instrument(skip(parts, s))]
    async fn from_request_parts(parts: &mut Parts, s: &S) -> Result<Option<Self>, Self::Rejection> {
        let forwarded_ip = if let Some(header) = s.forwarded_header() {
            let forwarded_ip = parts
                .headers
                .get_str_ignore_empty(header)
                .map(|x| IpAddr::from_str(x))
                .transpose()
                .map_err(|e| {
                    event!(Level::ERROR, ?e, "Failed to parse IP address from header");
                    ConnectionIpAddrError::ParseError(e)
                })?;
            if forwarded_ip.is_none() && tracing::enabled!(Level::TRACE) {
                event!(Level::TRACE, "No IP address found in header");
            }
            forwarded_ip
        } else {
            event!(Level::TRACE, "Forwarded header is not configured");
            None
        };
        if s.require_forwarded_header() {
            let Some(forwarded_ip) = forwarded_ip else {
                event!(Level::TRACE, "Forwarded header is required but not present");
                return Ok(None);
            };
            return Ok(Some(ConnectionIpAddr(forwarded_ip)));
        }
        let client_ip = forwarded_ip.or_else(|| {
            parts
                .extensions
                .get::<ConnectInfo<SocketAddr>>()
                .map(|ConnectInfo(c)| c.ip())
        });
        Ok(client_ip.map(ConnectionIpAddr))
    }
}
pub trait HasForwardedHeader {
    fn forwarded_header(&self) -> Option<&HeaderName>;

    fn require_forwarded_header(&self) -> bool {
        false
    }
}
/// Extract the IP Address but ignore validation of being an IP address
pub fn extract_ip_as_string<S, B>(req: &Request<B>, state: &S) -> Option<String>
where
    S: HasForwardedHeader,
{
    let forwarded_ip = if let Some(header) = state.forwarded_header() {
        req.headers().get_string_ignore_empty(header)
    } else {
        None
    };
    return forwarded_ip.or_else(|| {
        req.extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|ConnectInfo(c)| c.ip().to_string())
    });
}
