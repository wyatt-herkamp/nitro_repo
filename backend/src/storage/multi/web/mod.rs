use actix_web::web::ServiceConfig;

pub mod admin;
pub mod public;

pub fn init_admin_routes(cfg: &mut ServiceConfig) {
    cfg.service(admin::get_storages)
        .service(admin::new_storage)
        .service(admin::delete_storage)
        .service(admin::get_storage);
}
pub fn init_public_routes(cfg: &mut ServiceConfig) {
    cfg.service(public::get_storages_multi);
}
