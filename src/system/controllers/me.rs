use crate::DbPool;
use crate::api_response::APIResponse;
use actix_web::{get, HttpRequest, web};
use crate::utils::installed;
use crate::system::utils::get_user_by_header;
use crate::apierror::APIError;
use crate::error::request_error::RequestError;

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