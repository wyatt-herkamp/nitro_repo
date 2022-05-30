use actix_web::get;
use actix_web::{web, HttpResponse};
use sea_orm::DatabaseConnection;

use crate::authentication::Authentication;
use crate::system::user::UserModel;

#[get("/me")]
pub async fn me(
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
) -> actix_web::Result<HttpResponse> {
    let user: UserModel = auth.get_user(&database).await??;
    Ok(HttpResponse::Ok().json(user))
}
