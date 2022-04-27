use actix_web::{get, post, web, HttpRequest};
use sea_orm::{DatabaseConnection, IntoActiveModel};

use crate::api_response::{APIResponse, SiteResponse};
use crate::error::response::unauthorized;
use crate::system::user;
use crate::system::utils::{get_user_by_header, hash, NewPassword};
pub use sea_orm::{entity::*, query::*, DbErr, FromQueryResult};

#[get("/api/me")]
pub async fn me(database: web::Data<DatabaseConnection>, r: HttpRequest) -> SiteResponse {
    let user = get_user_by_header(r.headers(), &database).await?;
    if user.is_none() {
        return unauthorized();
    }

    APIResponse::respond_new(user, &r)
}

#[post("/api/me/user/password")]
pub async fn change_my_password(
    database: web::Data<DatabaseConnection>,
    r: HttpRequest,
    nc: web::Json<NewPassword>,
) -> SiteResponse {
    let user = get_user_by_header(r.headers(), &database).await?;
    if user.is_none() {
        return unauthorized();
    }

    let user: user::Model = user.unwrap();
    let hashed_password: String = hash(nc.0.password)?;
    let mut user_active: user::ActiveModel = user.into_active_model();

    user_active.password = Set(hashed_password);

    let user = user_active.update(database.as_ref()).await?;
    APIResponse::from(Some(user)).respond(&r)
}
