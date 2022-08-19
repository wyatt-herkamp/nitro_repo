use actix_web::error::ErrorInternalServerError;
use actix_web::{get, put};
use actix_web::{web, HttpResponse};
use sea_orm::ActiveValue::Set;
use sea_orm::{DatabaseConnection, IntoActiveModel};

use crate::authentication::{Authentication, SecureAction};
use crate::system::hash;
use crate::system::user::UserEntity;
use sea_orm::EntityTrait;
#[get("/me")]
pub async fn me(
    database: web::Data<DatabaseConnection>,
    auth: Authentication,
) -> actix_web::Result<HttpResponse> {
    let user = auth.get_user(&database).await??;
    Ok(HttpResponse::Ok().json(user))
}

#[put("/me/password")]
pub async fn update_password(
    database: web::Data<DatabaseConnection>,
    _auth: Authentication,
    nc: web::Json<SecureAction<String>>,
) -> actix_web::Result<HttpResponse> {
    let secure_action: SecureAction<String> = nc.into_inner();
    let model = secure_action.verify(&database).await??;

    let result = hash(secure_action.into_inner())?;
    let mut model = model.into_active_model();
    model.password = Set(result);
    UserEntity::update(model)
        .exec(database.as_ref())
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::NoContent().finish())
}
