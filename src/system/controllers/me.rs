use crate::api_response::APIResponse;
use crate::apierror::APIError;
use crate::error::request_error::RequestError;
use crate::error::request_error::RequestError::{NotAuthorized};
use crate::system::action::{update_user};
use crate::system::models::User;
use crate::system::utils::{get_user_by_header, NewPassword};
use crate::utils::installed;
use crate::DbPool;
use actix_web::{get,post, web, HttpRequest};

#[get("/api/me")]
pub async fn me(
    pool: web::Data<DbPool>,
    r: HttpRequest,
) -> Result<APIResponse<User>, RequestError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let user =
        get_user_by_header(r.headers(), &connection)?.ok_or_else(|| APIError::NotAuthorized)?;

    return Ok(APIResponse::new(true, Some(user)));
}
#[post("/api/admin/user/password")]
pub async fn change_my_password(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    nc: web::Json<NewPassword>,
) -> Result<APIResponse<User>, RequestError> {
    let connection = pool.get()?;
    installed(&connection)?;
    let mut user = get_user_by_header(r.headers(), &connection)?.ok_or_else(|| NotAuthorized)?;
    let string = nc.0.hash().unwrap();
    user.set_password(string);
    update_user(&user, &connection)?;
    return Ok(APIResponse::new(true, Some(user)));
}
