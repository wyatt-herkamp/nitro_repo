use actix_web::web::ServiceConfig;

pub mod admin;

pub fn init_admin_routes(cfg: &mut ServiceConfig) {
    cfg.service(admin::get_storages).service(admin::new_storage);
}