use actix_web::{error::ErrorInternalServerError, get, put, web, HttpResponse};
use sea_orm::{ActiveValue::Set, DatabaseConnection, EntityTrait, IntoActiveModel};

use crate::{
    authentication::{Authentication, SecureAction, TrulyAuthenticated},
    system::{hash, user::UserEntity},
};
#[get("/me")]
pub async fn me(
    database: web::Data<DatabaseConnection>,
    auth: TrulyAuthenticated,
) -> actix_web::Result<HttpResponse> {
    let user = auth.into_user();
    Ok(HttpResponse::Ok().json(user))
}

#[put("/me/password")]
pub async fn update_password(
    database: web::Data<DatabaseConnection>,
    _auth: TrulyAuthenticated,
    nc: web::Json<SecureAction<String>>,
) -> actix_web::Result<HttpResponse> {
    let secure_action: SecureAction<String> = nc.into_inner();
    let model = secure_action
        .verify(&database)
        .await?
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("Invalid login"))?;

    let result = hash(secure_action.into_inner())?;
    let mut model = model.into_active_model();
    model.password = Set(result);
    UserEntity::update(model)
        .exec(database.as_ref())
        .await
        .map_err(ErrorInternalServerError)?;
    Ok(HttpResponse::NoContent().finish())
}
