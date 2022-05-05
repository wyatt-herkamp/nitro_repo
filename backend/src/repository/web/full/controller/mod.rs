use actix_web::web;

pub mod repository;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg //.service(controller::browse_storage)
        //.service(web::resource(["/storages/", "/storages"]).to(controller::browse))
        .service(
            web::resource([
                "/storages/{storage}/{repository}",
                "/storages/{storage}/{repository}/{file:.*}",
                "/storages/{storage}/{repository}/",
            ])
            .route(web::get().to(repository::get_repository))
            .route(web::put().to(repository::put_repository))
            .route(web::head().to(repository::head_repository))
            .route(web::patch().to(repository::patch_repository))
            .route(web::post().to(repository::post_repository)),
        );
}
