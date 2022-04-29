use std::future::{ready, Ready};
use std::rc::Rc;
use std::time::SystemTime;
use actix_service::ServiceFactoryExt;

use actix_web::{dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpMessage, web};
use actix_web::body::None;
use actix_web::cookie::{Cookie, Expiration, SameSite};
use actix_web::http::header::{HeaderValue, SET_COOKIE};
use futures_util::future::LocalBoxFuture;
use log::{trace, warn};
use sea_orm::{DatabaseConnection, EntityTrait};
use crate::{SessionManager, system};
use crate::error::internal_error::InternalError;
use crate::session::{Authentication, Session, SessionManagerType};
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
        let service: Rc<S> = Rc::clone(&self.service);

        Box::pin(async move {
            let (authentication, session): (Authentication, Option<Session>) = if let Some(token) = req.headers().get("Authorization") {
                let database: &web::Data<DatabaseConnection> = req.app_data().unwrap();
                let token = token.to_str().unwrap();

                let split = token.split(' ').collect::<Vec<&str>>();
                if split.len() != 2 {
                    (Authentication::None, Option::None)
                } else {
                    let value = split.get(0).unwrap();
                    let auth_type = split.get(1).unwrap();
                    if auth_type.eq(&"Bearer") {
                        let auth_token = auth_token::get_by_token(value, &database).await.unwrap();
                        if let Some(token) = auth_token {
                            (Authentication::AuthToken(token), Option::None)
                        } else {
                            (Authentication::None, Option::None)
                        }
                    } else if auth_type.eq(&"Basic") {
                        let result = base64::decode(value).unwrap();
                        let string = String::from_utf8(result).unwrap();
                        let split = string.split(':').collect::<Vec<&str>>();

                        if !split.len().eq(&2) {
                            (Authentication::None, Option::None)
                        } else {
                            let username = split.get(0).unwrap().to_string();
                            let password = split.get(1).unwrap().to_string();
                            if username.eq("token") {
                                let auth_token = auth_token::get_by_token(&password, &database).await.unwrap();
                                if let Some(token) = auth_token {
                                    (Authentication::AuthToken(token), Option::None)
                                } else {
                                    (Authentication::None, Option::None)
                                }
                            } else {
                                let user = system::utils::verify_login(username, password, &database).await.unwrap();
                                if let Some(user) = user {
                                    (Authentication::Basic(user), Option::None)
                                } else {
                                    (Authentication::None, Option::None)
                                }
                            }
                        }
                    } else {
                        warn!("Unsupported Authorization Type: {}", auth_type);
                        (Authentication::None, Option::None)
                    }
                }
            } else if let Some(session) = req.cookie("session") {
                let session_manager: &web::Data<SessionManager> = req.app_data().unwrap();
                let session = session_manager.retrieve_session(session.value()).await.unwrap();
                if session.is_none() {
                    //Create a new session and go with it!
                    let session_manager: &web::Data<SessionManager> = req.app_data().unwrap();
                    let session = session_manager.create_session().await.unwrap();
                    (Authentication::Session(session.clone()), Some(session))
                } else {
                    let mut session = session.unwrap();
                    if session.expiration <= SystemTime::UNIX_EPOCH {
                        if let Some(auth_token) = session.auth_token {
                            let database: &web::Data<DatabaseConnection> = req.app_data().unwrap();
                            let connection = database.clone();
                            actix_web::rt::spawn(async move {
                                // Move this database call into the thread pool.
                                auth_token::Entity::delete_by_id(auth_token.id).exec(connection.as_ref()).await.unwrap();
                            });
                        }
                        session = session_manager.re_create_session(&session.token).await.unwrap();
                    }
                    (Authentication::Session(session.clone()), Option::None)
                }
            } else {
                let session_manager: &web::Data<SessionManager> = req.app_data().unwrap();
                let session = session_manager.create_session().await.unwrap();
                (Authentication::Session(session.clone()), Some(session))
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