use actix_web::{get, HttpRequest, post, web};

use crate::api_response::{APIResponse, SiteResponse};
use crate::DbPool;
use crate::error::response::{mismatching_passwords, unauthorized};
use crate::system::action::update_user_password;
use crate::system::utils::{get_user_by_header, hash, NewPassword};

#[get("/api/me")]
pub async fn me(pool: web::Data<DbPool>, r: HttpRequest) -> SiteResponse {
    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() {
        return unauthorized();
    }

    APIResponse::respond_new(user, &r)
}

#[post("/api/me/user/password")]
pub async fn change_my_password(
    pool: web::Data<DbPool>,
    r: HttpRequest,
    nc: web::Json<NewPassword>,
) -> SiteResponse {
    let connection = pool.get()?;

    let user = get_user_by_header(r.headers(), &connection)?;
    if user.is_none() {
        return unauthorized();
    }
    let user = user.unwrap();
    let string = hash(nc.0.password)?;
    update_user_password(&user.id, string, &connection)?;
    APIResponse::from(Some(user)).respond(&r)
}
