use actix_web::{get, post, web, HttpRequest};

use crate::api_response::APIResponse;

use crate::error::request_error::RequestError;
use crate::{utils, DbPool};

#[get("/api/installed")]
pub async fn installed(pool: web::Data<DbPool>) -> Result<APIResponse<bool>, RequestError> {
    let connection = pool.get()?;
    let result = utils::installed(&connection);
    if result.is_err() {
        return Ok(APIResponse::new(true, Some(false)));
    }
    Ok(APIResponse::new(true, Some(true)))
}
