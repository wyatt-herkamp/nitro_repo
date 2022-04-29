use actix_web::{get, post, web, HttpRequest};
use sea_orm::{DatabaseConnection, IntoActiveModel};

use crate::api_response::{APIResponse, SiteResponse};

use crate::authentication::Authentication;
use crate::system::user;
use crate::system::user::UserModel;
use crate::system::utils::{hash, NewPassword};
pub use sea_orm::{entity::*, query::*, DbErr, FromQueryResult};
#[get("/api/me")]
pub async fn me(
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
    r: HttpRequest,
) -> SiteResponse {
    let user: UserModel = auth.get_user(&database).await??;
    APIResponse::respond_new(Some(user), &r)
}

#[post("/api/me/user/password")]
pub async fn change_my_password(
    database: web::Data<DatabaseConnection>,
    r: HttpRequest,
    auth: Authentication,
    nc: web::Json<NewPassword>,
) -> SiteResponse {
    let user: UserModel = auth.get_user(&database).await??;

    let hashed_password: String = hash(nc.0.password)?;
    let mut user_active: user::database::ActiveModel = user.into_active_model();

    user_active.password = Set(hashed_password);

    let user = user_active.update(database.as_ref()).await?;
    APIResponse::from(Some(user)).respond(&r)
}
