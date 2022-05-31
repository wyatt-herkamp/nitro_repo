use actix_web::{get, web, HttpResponse};

use crate::repository::web::multi::admin::RepositoryConfigTypeEnum;

#[get("/repositories/config/type/{config}")]
pub async fn help_update_type(
    path_params: web::Path<RepositoryConfigTypeEnum>,
) -> actix_web::Result<HttpResponse> {
    let schema = path_params.into_inner().describe();
    Ok(HttpResponse::Ok().json(schema))
}
