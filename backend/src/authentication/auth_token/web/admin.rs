use crate::authentication::auth_token::AuthTokenEntity;
use crate::authentication::Authentication;
use crate::system::permissions::permissions_checker::CanIDo;
use actix_web::error::ErrorInternalServerError;
use actix_web::{delete, web, HttpResponse};
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::PaginatorTrait;

#[delete("token_system")]
pub async fn delete_token_system(
    authentication: Authentication,
    database: web::Data<DatabaseConnection>,
) -> actix_web::Result<HttpResponse> {
    let login = authentication.get_user(database.as_ref()).await??;
    login.can_i_admin()?;
    AuthTokenEntity::delete_many()
        .exec(database.as_ref())
        .await
        .map_err(ErrorInternalServerError)?;
    if AuthTokenEntity::find()
        .count(database.as_ref())
        .await
        .map_err(ErrorInternalServerError)?
        > 0
    {
        return Ok(HttpResponse::InternalServerError().finish());
    }
    Ok(HttpResponse::NoContent().finish())
}
