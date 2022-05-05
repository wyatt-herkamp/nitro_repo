use actix_web::{get, post, web, HttpRequest, Responder};
use sea_orm::{DatabaseConnection, IntoActiveModel};

use crate::api_response::{APIResponse, NRResponse};

use crate::authentication::Authentication;
use crate::system::user;
use crate::system::user::UserModel;
use crate::system::utils::{hash, NewPassword};
pub use sea_orm::{entity::*, query::*, DbErr, FromQueryResult};
use crate::error::internal_error::InternalError;

#[get("/api/me")]
pub async fn me(
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
    r: HttpRequest,
) -> NRResponse {
    let user: UserModel = auth.get_user(&database).await??;
    Ok(APIResponse::from(Some(user)))
}

#[post("/api/me/user/password")]
pub async fn change_my_password(
    database: web::Data<DatabaseConnection>,
    r: HttpRequest,
    auth: Authentication,
    nc: web::Json<NewPassword>,
) -> NRResponse {
    let user: UserModel = auth.get_user(&database).await??;

    let hashed_password: String = hash(nc.0.password).into::<InternalError>()?;
    let mut user_active: user::database::ActiveModel = user.into_active_model();

    user_active.password = Set(hashed_password);

    let user = user_active.update(database.as_ref()).await.into::<InternalError>()?;
    Ok(APIResponse::ok())
}
