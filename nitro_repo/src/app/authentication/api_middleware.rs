use std::future::{ready, Ready};
use std::rc::Rc;

use actix_web::body::{BoxBody, EitherBody};
use actix_web::http::header::{self};
use actix_web::http::Method;
use actix_web::web::Data;
use actix_web::HttpResponse;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use tracing::warn;

use crate::request_error;

use super::session::{Session, SessionManager};
use super::AuthenticationRaw;

pub struct HandleSession {
    session_manager: Data<SessionManager>,
}

impl<S, B> Transform<S, ServiceRequest> for HandleSession
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Transform = SessionMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SessionMiddleware {
            service: Rc::new(service),
            session_manager: self.session_manager.clone(),
        }))
    }
}

pub struct SessionMiddleware<S> {
    service: Rc<S>,
    session_manager: Data<SessionManager>,
}
impl<S, B> SessionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    async fn handle_session(
        session_manager: Data<SessionManager>,
        req: &ServiceRequest,
        cookie: impl AsRef<str>,
    ) -> Result<(), HttpResponse> {
        let session: Option<Session> = match session_manager.get_session(cookie.as_ref()) {
            Ok(ok) => ok,
            Err(e) => {
                warn!("Session Manager Error: {}", e);
                return Err(HttpResponse::InternalServerError().body("Session Manager Error"));
            }
        };
        if let Some(session) = session {
            let raw = AuthenticationRaw::Session(session);
            req.extensions_mut().insert(raw);
        }
        Ok(())
    }

    ///
    ///
    /// # Arguments
    ///
    /// * `service`: A clone of the service
    /// * `req`: The request
    /// * `session_manager`:  The session manager
    ///
    /// returns: Result<ServiceResponse<EitherBody<B, BoxBody>>, Error>
    ///    - Ok: The response  - Will just be the call to the next handler
    ///   - Err: The error - Will be an error response
    async fn handle_authentication(
        service: Rc<S>,
        req: ServiceRequest,
        session_manager: Data<SessionManager>,
    ) -> Result<ServiceResponse<EitherBody<B, BoxBody>>, Error> {
        if let Some(auth) = req.headers().get(header::AUTHORIZATION) {
            let auth_as_str = auth.to_str().map_err(|e| {
                warn!("Failed to convert auth header to string: {}", e);
                request_error::bad_request("Not a valid String in Authorization Header")
            })?;

            let split = auth_as_str.split(' ').collect::<Vec<&str>>();

            if split.len() != 2 {
                return Err(request_error::bad_request("Could not parse auth header").into());
            }
            let auth_type = split[0];
            match auth_type {
                "session" => {
                    if let Err(e) = Self::handle_session(session_manager, &req, split[1]).await {
                        return Ok(req.into_response(e.map_into_right_body()));
                    }
                }
                _ => {
                    return Err(request_error::bad_request(
                        "Unsupported Authorization Header Type ",
                    )
                    .into());
                }
            }
        } else if let Some(cookie) = req.cookie("session") {
            if let Err(e) = Self::handle_session(session_manager, &req, cookie.value()).await {
                return Ok(req.into_response(e.map_into_right_body()));
            }
        }
        let fut = service.call(req);

        let res = fut.await?;
        Ok(res.map_into_left_body())
    }
}
impl<S, B> Service<ServiceRequest> for SessionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Check if its an OPTIONS request. If so exit early and let the request pass through
        if req.method() == Method::OPTIONS {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res.map_into_left_body())
            });
        }
        // Grab required data from the service
        let session_manager = self.session_manager.clone();
        // Move into an async block
        let session = Self::handle_authentication(self.service.clone(), req, session_manager);
        Box::pin(async move {
            let res = session.await?;
            Ok(res)
        })
    }
}
