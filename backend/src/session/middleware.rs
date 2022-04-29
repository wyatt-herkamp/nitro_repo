use actix_service::ServiceFactoryExt;
use std::fmt;
use std::future::{ready, Ready};
use std::rc::Rc;

use std::time::SystemTime;


use crate::session::{Authentication, Session, SessionManagerType};
use crate::system::auth_token;
use crate::{system, SessionManager};

use actix_web::cookie::{Cookie, SameSite};
use actix_web::http::header::{HeaderValue, ORIGIN, SET_COOKIE};
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpMessage,
};
use actix_web::http::Method;

use futures_util::future::LocalBoxFuture;
use log::{trace, warn};
use sea_orm::{DatabaseConnection, EntityTrait};

pub struct HandleSession;

impl<S, B> Transform<S, ServiceRequest> for HandleSession
    where
        S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error> + 'static,
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
        }))
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
        if req.method() == Method::OPTIONS {
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            });
        }
        let service: Rc<S> = Rc::clone(&self.service);
        trace!("Request at {}", req.path());
        // Move all into an Async Box.
        Box::pin(async move {
            //Step One Find the Authorization
            let (authentication, session): (Authentication, Option<Session>) = if let Some(header) =
            req.headers().get("Authorization")
            {
                //If it is an Authorization Header pull Database from App Data
                let database: &web::Data<DatabaseConnection> = req.app_data().unwrap();
                // Convert Header to String
                let header_value = header.to_str().unwrap();

                let split = header_value.split(' ').collect::<Vec<&str>>();
                if split.len() != 2 {
                    // If the length is not correct. It is an invalid authorization. But let request continue
                    (Authentication::NoIdentification, Option::None)
                } else {
                    let value = split.get(0).unwrap();
                    let auth_type = split.get(1).unwrap();
                    // If its a Bearer use the token System
                    if auth_type.eq(&"Bearer") {
                        let auth_token = auth_token::get_by_token(value, &database)
                            .await
                            .map_err(internal_server_error)?;
                        if let Some(token) = auth_token {
                            (Authentication::AuthToken(token), Option::None)
                        } else {
                            (Authentication::NoIdentification, Option::None)
                        }
                    } else if auth_type.eq(&"Basic") {
                        //If its a Basic header. Parse from base64
                        let base64 = base64::decode(value).map_err(request_error)?;
                        let string = String::from_utf8(base64).map_err(request_error)?;
                        let split = string.split(':').collect::<Vec<&str>>();

                        if !split.len().eq(&2) {
                            (Authentication::NoIdentification, Option::None)
                        } else {
                            let username = split.get(0).unwrap().to_string();
                            let password = split.get(1).unwrap().to_string();
                            // Maven will pass everything as a Basic. Setting the username as Token lets you use the token system
                            if username.eq("token") {
                                // Treat the password as a token
                                let auth_token = auth_token::get_by_token(&password, &database)
                                    .await
                                    .map_err(internal_server_error)?;
                                if let Some(token) = auth_token {
                                    (Authentication::AuthToken(token), Option::None)
                                } else {
                                    (Authentication::NoIdentification, Option::None)
                                }
                            } else {
                                // Treat authorization as normal login
                                let user =
                                    system::utils::verify_login(username, password, &database)
                                        .await
                                        .map_err(internal_server_error)?;
                                if let Some(user) = user {
                                    (Authentication::Basic(user), None)
                                } else {
                                    (Authentication::NoIdentification, None)
                                }
                            }
                        }
                    } else {
                        // Invalid Authoriaztzation Header type
                        warn!("Unsupported Authorization Type: {}", auth_type);
                        (Authentication::NoIdentification, None)
                    }
                }
            } else if let Some(cookie) = req.cookie("session") {
                //Check for the Session Cookie
                let session_manager: &web::Data<SessionManager> = req.app_data().unwrap();
                trace!("Cookie sent {}", cookie.encoded().to_string());
                let session = session_manager
                    .retrieve_session(cookie.value())
                    .await
                    .unwrap();
                if session.is_none() {
                    //Create a new session and go with it!
                    let session_manager: &web::Data<SessionManager> = req.app_data().unwrap();
                    if let Some(origin) = req.headers().get(ORIGIN) {
                        trace!("Cookie {} not found. Creating a new Session for {}",cookie.value(),origin.to_str().unwrap_or("Bad Origin"));
                        let session_manager: &web::Data<SessionManager> = req.app_data().unwrap();
                        let session = session_manager.create_session().await.unwrap();
                        (Authentication::Session(session.clone()), Some(session))
                    } else {
                        (Authentication::NoIdentification, Option::None)
                    }
                } else {
                    let mut session = session.unwrap();
                    if session.expiration <= SystemTime::UNIX_EPOCH {
                        if let Some(auth_token) = session.auth_token {
                            let database: &web::Data<DatabaseConnection> = req.app_data().unwrap();
                            let connection = database.clone();
                            actix_web::rt::spawn(async move {
                                // Move this database call into the thread pool.
                                if let Err(error) = auth_token::Entity::delete_by_id(auth_token.id)
                                    .exec(connection.as_ref())
                                    .await
                                {
                                    log::error!("Unable to delete Auth Token {}", error);
                                }
                            });
                        }
                        session = session_manager
                            .re_create_session(&session.token)
                            .await
                            .unwrap();
                    }
                    (Authentication::Session(session.clone()), Option::None)
                }
            } else {
                // Try to create a new Session for the user. Could be a first request
                // Require a Origin Header for request
                if let Some(origin) = req.headers().get(ORIGIN) {
                    trace!("Creating a new Session for {}. ",origin.to_str().unwrap_or("Bad Origin"));
                    let session_manager: &web::Data<SessionManager> = req.app_data().unwrap();
                    let session = session_manager.create_session().await.unwrap();
                    (Authentication::Session(session.clone()), Some(session))
                } else {
                    (Authentication::NoIdentification, Option::None)
                }
            };
            // Add the authentication Information for the data
            req.extensions_mut().insert(authentication);
            // Finish the request
            let fut = service.call(req);
            // Get the response
            let mut res: Self::Response = fut.await?;
            // If a new cookie needs to be added. Do it
            if let Some(session) = session {
                let mut cookie = Cookie::new("session", &session.token);
                cookie.set_secure(false);
                cookie.set_same_site(SameSite::Lax);
                cookie.set_path("/");
                cookie.set_expires(session.expiration.clone());
                let cookie_encoded = cookie.encoded().to_string();
                trace!("Sending Cookie Response {}",&cookie_encoded);
                let val = HeaderValue::from_str(&cookie_encoded).unwrap();

                res.headers_mut().append(SET_COOKIE, val);
            }
            Ok(res)
        })
    }
}

fn internal_server_error<E: fmt::Debug + fmt::Display + 'static>(err: E) -> Error {
    actix_web::error::InternalError::from_response(
        err,
        actix_web::HttpResponse::InternalServerError().finish(),
    )
        .into()
}

fn request_error<E: fmt::Debug + fmt::Display + 'static>(err: E) -> Error {
    actix_web::error::InternalError::from_response(
        err,
        actix_web::HttpResponse::BadRequest().finish(),
    )
        .into()
}
