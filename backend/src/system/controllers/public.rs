use actix_web::{post, web, HttpRequest};

use crate::api_response::{ SiteResponse};

use crate::error::response::unauthorized;

use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use crate::system::utils::verify_login;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[post("/api/login")]
pub async fn login(connection: web::Data<DatabaseConnection>, r: HttpRequest, nc: web::Json<Login>) -> SiteResponse {
    let username = nc.username.clone();
    if let Some(user) = verify_login(nc.username.clone(), nc.password.clone(), &connection).await? {
        todo!("Sadly we no longer have session tokens! Time to break!")

        //APIResponse::respond_new(Some(token), &r)
    } else {
        return unauthorized();
    }
}
