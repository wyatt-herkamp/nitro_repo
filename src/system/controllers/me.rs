use crate::api_response::{APIResponse, SiteResponse};


use crate::system::action::update_user;

use crate::system::utils::{get_user_by_header, NewPassword};

use crate::DbPool;
use actix_web::{get, post, web, HttpRequest};
use crate::error::response::{mismatching_passwords, unauthorized};

#[get("/api/me")]
pub async fn me(
    pool: web::Data<DbPool>,
    r: HttpRequest,
) -> SiteResponse {
    let connection = pool.get()?;

    let user =
        get_user_by_header(r.headers(), &connection)?;
    if user.is_none() {
        return unauthorized();
    }

    return APIResponse::respond_new(user, &r);
}

#[post("/api/admin/user/password")]
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
    let mut user = user.unwrap();
    let string = nc.0.hash()?;
    if string.is_none() {
        return mismatching_passwords();
    }
    user.set_password(string.unwrap());
    update_user(&user, &connection)?;
    return APIResponse::from(Some(user)).respond(&r);
}
