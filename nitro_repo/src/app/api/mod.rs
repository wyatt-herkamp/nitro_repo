use actix_web::{
    get,
    web::{Data, ServiceConfig},
    HttpResponse,
};

use crate::app::NitroRepo;

pub fn init(service: &mut ServiceConfig) {
    service.service(info);
}
#[get("/info")]
pub async fn info(site: Data<NitroRepo>) -> HttpResponse {
    let site = site.instance.lock().clone();
    HttpResponse::Ok().json(site)
}
