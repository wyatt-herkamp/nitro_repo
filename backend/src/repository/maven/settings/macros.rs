macro_rules! define_repository_config_get {
    ($config:path, $config_name:ident,$maven_type:ident) => {
        paste::paste! {
        pub async fn [<get_ $config_name>](
            storage_handler: actix_web::web::Data<crate::storage::multi::MultiStorageController<crate::storage::DynamicStorage>>,
            database: actix_web::web::Data<sea_orm::DatabaseConnection>,
            auth: crate::authentication::Authentication,
            path_params: actix_web::web::Path<(String, String)>,
        ) -> actix_web::Result<actix_web::HttpResponse> {
            use crate::storage::models::Storage;
            use crate::system::permissions::permissions_checker::CanIDo;
            let user = auth.get_user(&database).await??;
            user.can_i_edit_repos()?;
            let (storage_name, repository_name) = path_params.into_inner();
            let storage = crate::helpers::get_storage!(storage_handler, storage_name);
            let repository = crate::helpers::get_repository!(storage, repository_name);
            if let  crate::repository::handler::DynamicRepositoryHandler::Maven( repository) = repository.as_ref() {
                if let crate::repository::maven::MavenHandler::$maven_type(ref repository) = repository {
                    let value = crate::repository::settings::RepositoryConfigHandler::<$config>::get(repository);
                    return Ok(actix_web::HttpResponse::Ok().json(value));
                }
            }
            return Ok(actix_web::HttpResponse::BadRequest().body("Repository type not supported".to_string()));
        }
        }
    };
}

macro_rules! define_repository_config_set {
    ($config:path, $config_name:ident,$maven_type:ident) => {
        paste::paste! {
            pub async fn [<set_ $config_name>](
                storage_handler: actix_web::web::Data<crate::storage::multi::MultiStorageController<crate::storage::DynamicStorage>>,
                database: actix_web::web::Data<sea_orm::DatabaseConnection>,
                auth: crate::authentication::Authentication,
                path_params: actix_web::web::Path<(String, String)>,
                body: actix_web::web::Json<$config>,
            ) -> actix_web::Result<actix_web::HttpResponse> {
                use crate::storage::models::Storage;
                use crate::repository::handler::Repository;
                use crate::system::permissions::permissions_checker::CanIDo;
                let user = auth.get_user(&database).await??;
                user.can_i_edit_repos()?;
                let (storage_name, repository_name) = path_params.into_inner();
                let storage = crate::helpers::get_storage!(storage_handler, storage_name);
                let  (name,mut repository) = crate::helpers::take_repository!(storage, repository_name);
                let body = body.into_inner();

                let result = if let  crate::repository::handler::DynamicRepositoryHandler::Maven(ref mut repository) = repository {
                    if let crate::repository::maven::MavenHandler::$maven_type(ref mut repository) = repository {
                        let _value = crate::repository::settings::RepositoryConfigHandler::<$config>::get(repository);
                        let value = crate::repository::settings::RepositoryConfigHandler::<$config>::update( repository, body).map(|_| true);
                        if let Err(e) = storage.save_repository_config(repository.get_repository(),  crate::repository::settings::RepositoryConfigHandler::<$config>::get(repository)).await{
                            log::error!("{}", e);
                        }
                        value
                    }else{
                        Ok(false)
                    }
                }else {
                    Ok(false)
                };
                storage.add_repository_for_updating(name, repository,false).await.expect("Failed to add repository for updating");
                if result?{
                    Ok(actix_web::HttpResponse::NoContent().finish())
                }else{
                    Ok(actix_web::HttpResponse::BadRequest().body("Repository type not supported".to_string()))
                }
            }
        }
    };
}
pub(crate) use define_repository_config_get;
pub(crate) use define_repository_config_set;

macro_rules! define_repository_config_handlers_group {
    ($config:path, $config_name:ident,$maven_type:ident) => {
        paste::paste! {
            crate::repository::maven::settings::macros::define_repository_config_get!($config, $config_name, $maven_type);
            crate::repository::maven::settings::macros::define_repository_config_set!($config, $config_name, $maven_type);
            pub fn init(cfg: &mut actix_web::web::ServiceConfig) {
                cfg.service(actix_web::web::resource([concat!("/repositories/{storage}/{repository}/config/", stringify!($config_name))])
                .route(actix_web::web::get().to([<get_ $config_name>]))
                .route(actix_web::web::put().to([<set_ $config_name>])));
            }
            }
    };
}

pub(crate) use define_repository_config_handlers_group;
