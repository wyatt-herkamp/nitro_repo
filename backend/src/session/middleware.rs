use std::future::{ready, Ready};
use std::rc::Rc;
use std::time::SystemTime;

use actix_web::{dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpMessage, web};
use actix_web::cookie::{Cookie, Expiration, SameSite};
use actix_web::http::header::{HeaderValue, SET_COOKIE};
use futures_util::future::LocalBoxFuture;
use log::trace;
use sea_orm::DatabaseConnection;
use crate::{SessionManager, system};
use crate::session::{Session, SessionManagerType};
use crate::system::auth_token;

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
pub struct HandleSession;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for HandleSession
    where
        S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error> + 'static,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SessionMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SessionMiddleware { service: Rc::new(service) }))
    }
}

pub struct SessionMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for SessionMiddleware<S>
    where
        S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error> + 'static,
        S::Future: 'static,
        B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if req.path().starts_with("/storages") {
            trace!("Skipping Authentication Middleware. Repository Must handle it themselves. Debug Path: {} ",req.path());
            let fut = self.service.call(req);

            return Box::pin(async move {
                let res = fut.await?;

                Ok(res)
            });
        }
        let service: Rc<S> = Rc::clone(&self.service);

        Box::pin(async move {
            let (authentication, session): (Option<system::auth_token::Model>, Option<Session>) = if let Some(token) = req.headers().get("Authorization") {
                let database: &web::Data<DatabaseConnection> = req.app_data().unwrap();
                let token = token.to_str().unwrap();
                let option = auth_token::get_by_token(token, &database).await.unwrap();
                if option.is_none() {
                    //TODO return early. Un authenticated
                }

                (option, None)
            } else if let Some(session) = req.cookie("session") {
                let session_manager: &web::Data<SessionManager> = req.app_data().unwrap();
                let session = session_manager.retrieve_session(session.value()).await.unwrap();
                if session.is_none() {
                    //Create a new session and go with it!
                    let session_manager: &web::Data<SessionManager> = req.app_data().unwrap();
                    let session = session_manager.create_session().await.unwrap();
                    (None, Some(session))
                } else {
                    let session = session.unwrap();
                    (session.auth_token, None)
                }
            } else {
                let session_manager: &web::Data<SessionManager> = req.app_data().unwrap();
                let session = session_manager.create_session().await.unwrap();
                (None, Some(session))
            };
            req.extensions_mut().insert(authentication);
            let fut = service.call(req);

            let mut res: Self::Response = fut.await?;
            if let Some(session) = session {
                let mut cookie = Cookie::new("session", &session.token);
                cookie.set_secure(true);
                cookie.set_same_site(SameSite::Lax);

                cookie.set_expires(session.expiration.clone());
                let val = HeaderValue::from_str(&cookie.encoded().to_string()).unwrap();
                res.headers_mut().append(SET_COOKIE, val);
            }
            Ok(res)
        })
    }
}