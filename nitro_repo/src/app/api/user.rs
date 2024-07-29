use actix_web::{
    cookie::Cookie,
    get, post,
    web::{self, Data, ServiceConfig},
    HttpResponse,
};
use nr_core::database::user::UserSafeData;
use serde::{Deserialize, Serialize};

use crate::{
    app::{
        authentication::{
            session::{Session, SessionManager},
            verify_login, Authentication, MeWithSession,
        },
        DatabaseConnection,
    },
    error::internal_error::InternalError,
};

pub fn init(service: &mut ServiceConfig) {
    service.service(me).service(login);
}

#[get("/me")]
pub async fn me(auth: Authentication) -> HttpResponse {
    match auth {
        Authentication::AuthToken(_, user) | Authentication::Basic(user) => {
            HttpResponse::Ok().json(user)
        }
        Authentication::Session(session, user) => {
            HttpResponse::Ok().json(MeWithSession::from((session, user)))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginRequest {
    pub email_or_username: String,
    pub password: String,
}

#[post("/login")]
pub async fn login(
    login: web::Json<LoginRequest>,
    database: DatabaseConnection,
    session: Data<SessionManager>,
) -> Result<HttpResponse, InternalError> {
    let LoginRequest {
        email_or_username,
        password,
    } = login.into_inner();
    let user = match verify_login(email_or_username, password, &database).await {
        Ok(ok) => ok,
        Err(err) => {
            return Ok(HttpResponse::Unauthorized().json(err.to_string()));
        }
    };
    let duration = chrono::Duration::days(1);

    let session = session.create_session(user.id, duration)?;

    let cookie = Cookie::build("session", session.session_id.clone())
        .http_only(true)
        .secure(true)
        .finish();
    let user_with_session = MeWithSession::from((session, user));

    let response = HttpResponse::Ok().cookie(cookie).json(user_with_session);
    Ok(response)
}
