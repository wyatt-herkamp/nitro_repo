use actix_web::{post, web, HttpRequest};
use actix_web::cookie::Cookie;

use crate::api_response::{SiteResponse};

use crate::error::response::unauthorized;

use sea_orm::{DatabaseConnection, InsertResult, NotSet};
use serde::{Deserialize, Serialize};
use crate::system::auth_token;
use crate::system::utils::verify_login;
use crate::utils::get_current_time;
use sea_orm::ActiveValue::Set;
use crate::{APIResponse, SessionManager};
use crate::system::auth_token::TokenType;
use sea_orm::EntityTrait;
use crate::session::{Session, SessionManagerType};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[post("/api/login")]
pub async fn login(connection: web::Data<DatabaseConnection>, session_manager: web::Data<SessionManager>, r: HttpRequest, nc: web::Json<Login>) -> SiteResponse {
    let username = nc.username.clone();
    if let Some(user) = verify_login(nc.username.clone(), nc.password.clone(), &connection).await? {
        let properties = auth_token::AuthProperties {
            description: None,
            token_type: TokenType::SessionToken,
        };
        let token_value = auth_token::generate_token();
        let token = auth_token::ActiveModel {
            id: NotSet,
            token: Set(token_value.clone()),
            expiration: Set(auth_token::token_expiration()),
            user_id: Set(user.id),
            created: Set(get_current_time()),
            properties: Set(properties),
        };
        auth_token::Entity::insert(token).exec(connection.as_ref()).await?;
        let cookie: Cookie = r.cookie("session").unwrap();
        actix_web::rt::spawn(async move {
            let token = token_value.clone();
            let token = auth_token::get_by_token(&token, &connection).await.unwrap().unwrap();
            session_manager.set_auth_token(cookie.value(), token).await;
        });
        APIResponse::respond_new(Some(true), &r)
    } else {
        return unauthorized();
    }
}
