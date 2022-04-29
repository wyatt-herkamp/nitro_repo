use actix_web::web::Data;
use actix_web::{get, web, HttpRequest};
use sea_orm::DatabaseConnection;
use std::ops::Deref;

use crate::api_response::{APIResponse, SiteResponse};
use crate::system::permissions::options::CanIDo;
use crate::NitroRepo;
use crate::authentication::Authentication;
use crate::system::user::UserModel;
#[get("/api/settings/report")]
pub async fn setting_report(
    site: Data<NitroRepo>,
    database: web::Data<DatabaseConnection>,
    r: HttpRequest, auth: Authentication,
) -> SiteResponse {
    let caller: UserModel = auth.get_user(&database).await??;
    caller.can_i_admin()?;
    let settings = site.settings.read().await;
    APIResponse::from(Some(settings.deref())).respond(&r)
}
