use actix_web::{get, HttpRequest};
use serde::{Deserialize, Serialize};


use crate::{APIResponse, NitroRepoData, SiteResponse};
#[derive(Serialize, Deserialize)]
pub struct SiteInfo {
    pub name: String,
    pub description: String,
}


#[get("/api/info/site")]
pub async fn site_info(site: NitroRepoData, request: HttpRequest) -> SiteResponse {
    let mutex = site.settings.lock().unwrap();

    let info = SiteInfo { name: mutex.site.name.clone(), description: mutex.site.name.clone() };
    APIResponse::respond_new(Some(info), &request)
}
