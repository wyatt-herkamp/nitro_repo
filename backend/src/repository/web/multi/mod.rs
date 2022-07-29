use actix_web::web;

pub mod admin;
pub mod configs;
pub mod public;
pub mod repository_handler;

pub fn init_repository_handlers(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource([
            "/repositories/{storage}/{repository}",
            "/repositories/{storage}/{repository}/{file:.*}",
            "/repositories/{storage}/{repository}/",
            "/storages/{storage}/{repository}",
            "/storages/{storage}/{repository}/{file:.*}",
            "/storages/{storage}/{repository}/",
        ])
        .route(web::get().to(repository_handler::get_repository))
        .route(web::put().to(repository_handler::put_repository))
        .route(web::head().to(repository_handler::head_repository))
        .route(web::patch().to(repository_handler::patch_repository))
        .route(web::post().to(repository_handler::post_repository)),
    )
    .service(
        web::resource("/stage/repositories/{storage}/{repository}/{file:.*}")
            .route(web::put().to(repository_handler::stage_repository)),
    );
}

pub fn init_admin(cfg: &mut web::ServiceConfig) {
    cfg.service(admin::get_repository)
        .service(admin::get_repositories)
        .service(admin::create_repository)
        .service(admin::delete_repository);
    cfg.configure(admin::register_core_updates);
}
