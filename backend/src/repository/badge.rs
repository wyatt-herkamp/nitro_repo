use crate::api_response::SiteResponse;
use crate::NitroRepoData;
use actix_web::{get, web, HttpRequest, HttpResponse};
use badge_maker::BadgeBuilder;
use sea_orm::DatabaseConnection;

use crate::repository::controller::to_request;

#[get("/badge/{storage}/{repository}/{file:.*}/badge")]
pub async fn badge(
    connection: web::Data<DatabaseConnection>,
    site: NitroRepoData,
    r: HttpRequest,
    path: web::Path<(String, String, String)>,
) -> SiteResponse {
    let (storage, repository, file) = path.into_inner();

    let request = to_request(storage, repository, file, site).await?;

    let (label, message) = if request.value.eq("nitro_repo_example") {
        (request.repository.name.clone(), "example".to_string())
    } else if request.value.eq("nitro_repo_status") {
        (
            request.repository.name.clone(),
            request.repository.settings.active.to_string(),
        )
    } else if request.value.eq("nitro_repo_info") {
        (
            format!("{} Repository", &request.repository.repo_type.to_string()),
            request.repository.name.clone(),
        )
    } else {
        let version = request
            .repository
            .repo_type
            .latest_version(&request, &r, &connection)
            .await?;
        (
            request.repository.name.clone(),
            version.unwrap_or_else(|| "404".to_string()),
        )
    };
    let b_s = request.repository.settings.badge;

    let svg: String = BadgeBuilder::new()
        .label(&label)
        .message(message.as_str())
        .style(b_s.style.to_badge_maker_style())
        .color_parse(b_s.color.as_str())
        .label_color_parse(b_s.label_color.as_str())
        .build()
        .unwrap()
        .svg();
    return Ok(HttpResponse::Ok().content_type("image/svg+xml").body(svg));
}
