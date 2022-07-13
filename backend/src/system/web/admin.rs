use crate::authentication::Authentication;
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;
use actix_web::{get, web, HttpResponse};

use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

// struct that derives Serialize and Deserialize contains the number of active storages, number of active repositories, and the number of active users.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SystemStatus {
    pub active_storages: usize,
    pub active_repositories: usize,
    pub active_users: usize,
}

#[get("users/list")]
pub async fn list_users(
    auth: Authentication,
    database: web::Data<DatabaseConnection>,
) -> actix_web::Result<HttpResponse> {
    let user: UserModel = auth.get_user(&database).await??;
    user.can_i_edit_users()?;
    let result: Vec<UserModel> = super::super::user::get_users(&database)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(result))
}
