use actix_web::{get, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::api_response::{APIResponse, SiteResponse};
use crate::NitroRepoData;

#[derive(Serialize, Deserialize)]
pub struct SiteInfo {
    pub name: String,
    pub description: String,
}

#[get("/api/info/site")]
pub async fn site_info(site: NitroRepoData, request: HttpRequest) -> SiteResponse {
    let mutex = site.settings.read().await;

    let info = SiteInfo {
        name: mutex.site.name.clone(),
        description: mutex.site.name.clone(),
    };
    APIResponse::respond_new(Some(info), &request)
}
