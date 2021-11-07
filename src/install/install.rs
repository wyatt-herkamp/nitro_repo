use std::pin::Pin;
use std::task::{Context, Poll};

use crate::api_response::{APIResponse, RequestErrorResponse};
use crate::utils::installed;
use crate::DbPool;
use actix_service::{Service, Transform};
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use futures::future::{ok, Ready};
use futures::Future;

pub struct Installed;

impl<S, B> Transform<S> for Installed
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = InstallMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(InstallMiddleware { service })
    }
}

pub struct InstallMiddleware<S> {
    service: S,
}

impl<S, B> Service for InstallMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let option = req.app_data::<Data<DbPool>>().unwrap();
        let connection = option.get().unwrap();
        if req.path().contains("install") {
            let fut = self.service.call(req);

            return Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            });
        }
        //TODO Return an error??????
        let result = installed(&connection).unwrap();
        if !result {
            let response = APIResponse::new(
                false,
                Some(RequestErrorResponse {
                    user_friendly_message: None,
                    error_code: Some("UNINSTALLED".to_string()),
                }),
            )
            .error(StatusCode::BAD_GATEWAY);
            return Box::pin(async move { Ok(req.into_response(response.unwrap().into_body())) });
        }
        let fut = self.service.call(req);

        return Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        });
    }
}
