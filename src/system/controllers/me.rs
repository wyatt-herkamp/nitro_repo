use crate::DbPool;
use crate::api_response::APIResponse;
use actix_web::{get, HttpRequest, web};
use crate::utils::installed;
use crate::system::utils::{get_user_by_header, NewPassword};
use crate::apierror::APIError;
use crate::error::request_error::RequestError;
use crate::system::action::{update_user, get_user_by_username};
use crate::error::request_error::RequestError::{NotAuthorized, NotFound};
use crate::system::models::User;

#[get("/api/me")]
pub async fn me(
    pool: web::Data<DbPool>,
    r: HttpRequest,
) -> Result<APIResponse<bool>, RequestError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let user =
        get_user_by_header(r.headers(), &connection)?.ok_or_else(|| APIError::NotAuthorized)?;

    return Ok(APIResponse::new(true, Some(true)));
}
#[post("/api/admin/user/password")]
pub async fn change_my_password(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    nc: web::Json<NewPassword>,
) -> Result<APIResponse<User>, RequestError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let mut user =
        get_user_by_header(r.headers(), &connection)?.ok_or_else(|| NotAuthorized)?;
    let string = nc.0.hash().unwrap();
    user.set_password(string);
    update_user(&user, &connection)?;
    return Ok(APIResponse::new(true, Some(user)));
}
