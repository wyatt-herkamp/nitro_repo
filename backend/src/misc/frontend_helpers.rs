use crate::api_response::{APIResponse, NRResponse};
use crate::NitroRepoData;
use actix_web::{get, HttpRequest};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SiteInfo {
    pub name: String,
    pub description: String,
}

#[get("/api/info/site")]
pub async fn site_info(site: NitroRepoData, request: HttpRequest) -> NRResponse {
    let mutex = site.settings.read().await;

    let info = SiteInfo {
        name: mutex.site.name.clone(),
        description: mutex.site.name.clone(),
    };
    Ok(Some(info).into())
}
