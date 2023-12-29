use std::{
    fmt,
    future::{ready, Ready},
    rc::Rc,
    sync::Arc,
};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::{header::AUTHORIZATION, Method},
    Error, HttpMessage,
};
use chrono::Local;
use futures_util::future::LocalBoxFuture;
use log::trace;

use super::RawAuthentication;
use crate::authentication::session::SessionManager;

pub struct HandleSession(pub Arc<SessionManager>);

impl<S, B> Transform<S, ServiceRequest> for HandleSession
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = SessionMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SessionMiddleware {
            service: Rc::new(service),
            session_manager: self.0.clone(),
        }))
    }
}

pub struct SessionMiddleware<S> {
    service: Rc<S>,
    session_manager: Arc<SessionManager>,
}

impl<S, B> SessionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    pub fn get_authentication(
        &self,
        req: &ServiceRequest,
    ) -> Result<Option<RawAuthentication>, Error> {
        if let Some(session_cookie) = req.cookie("session") {
            let session = self
                .session_manager
                .get_session(session_cookie.value())
                .unwrap();
            if let Some(session) = session {
                if session.expires <= Local::now() {
                    return Ok(None);
                }
                return Ok(Some(RawAuthentication::Session(session)));
            } else {
                return Ok(None);
            }
        } else if let Some(auth) = req.headers().get(AUTHORIZATION) {
            let auth = auth.to_str().unwrap().to_owned();
            let split = auth.splitn(1, ' ').collect::<Vec<&str>>();
            if split.len() < 2 {
                return Ok(None);
            }
            let auth_type = split.first().unwrap();
            let value: String = (*split.get(1).unwrap()).trim().to_owned();

            match *auth_type {
                "Bearer" => Ok(Some(RawAuthentication::AuthToken(value))),
                "Basic" => Ok(Some(RawAuthentication::Basic(value))),
                "Session" => {
                    if self.session_manager.config.allow_in_header {
                        let session = self.session_manager.get_session(&value).unwrap();
                        if let Some(session) = session {
                            if session.expires <= Local::now() {
                                return Ok(None);
                            }
                            return Ok(Some(RawAuthentication::Session(session)));
                        } else {
                            return Ok(None);
                        }
                    } else {
                        Ok(None)
                    }
                }
                other => Ok(Some(RawAuthentication::AuthorizationHeaderUnknown(
                    other.to_string(),
                    value,
                ))),
            }
        } else {
            return Ok(None);
        }
    }
}

impl<S, B> Service<ServiceRequest> for SessionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if req.method() == Method::OPTIONS {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            });
        }
        let service: Rc<S> = Rc::clone(&self.service);
        trace!("Request at {}", req.path());
        let authentication = self.get_authentication(&req).unwrap();
        // Move all into an Async Box.
        Box::pin(async move {
            req.extensions_mut().insert(authentication);
            // Finish the request
            let fut = service.call(req);
            // Get the response
            let res: Self::Response = fut.await?;
            Ok(res)
        })
    }
}
