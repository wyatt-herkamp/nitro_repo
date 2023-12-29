use actix_web::{delete, error::ErrorInternalServerError, web, HttpResponse};
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait};

use crate::{
    authentication::{auth_token::AuthTokenEntity, Authentication, TrulyAuthenticated},
    system::permissions::permissions_checker::CanIDo,
};

#[delete("token_system")]
pub async fn delete_token_system(
    authentication: TrulyAuthenticated,
    database: web::Data<DatabaseConnection>,
) -> actix_web::Result<HttpResponse> {
    let login = authentication.into_user();
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
