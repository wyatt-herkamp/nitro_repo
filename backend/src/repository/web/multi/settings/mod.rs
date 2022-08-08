pub mod child;
macro_rules! define_repository_config_get {
    ($config:path, $config_name:ident,$($repository:ident),*) => {
        paste::paste! {
        pub async fn [<get_ $config_name>](
            storage_handler: actix_web::web::Data<crate::storage::multi::MultiStorageController<crate::storage::DynamicStorage>>,
            database: actix_web::web::Data<sea_orm::DatabaseConnection>,
            auth: crate::authentication::Authentication,
            path_params: actix_web::web::Path<(String, String)>,
        ) -> actix_web::Result<actix_web::HttpResponse> {
            use crate::storage::models::Storage;
            use crate::system::permissions::options::CanIDo;
            let user = auth.get_user(&database).await??;
            user.can_i_edit_repos()?;
            let (storage_name, repository_name) = path_params.into_inner();
            let storage = crate::helpers::get_storage!(storage_handler, storage_name);
            let repository = crate::helpers::get_repository!(storage, repository_name);
            match repository.as_ref() {
                $(
                    crate::repository::handler::DynamicRepositoryHandler::$repository(repository) => {
                        let value = crate::repository::settings::RepositoryConfigHandler::<$config>::get(repository);
                        Ok(actix_web::HttpResponse::Ok().json(value))
                    }
                )*
                _ => {
                    return Ok(actix_web::HttpResponse::BadRequest().body("Repository type not supported".to_string()));
                }
            }
        }
        }
    };
}

macro_rules! define_repository_config_set {
    ($config:path, $config_name:ident,$($repository:ident),*) => {
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
                use crate::system::permissions::options::CanIDo;
                let user = auth.get_user(&database).await??;
                user.can_i_edit_repos()?;
                let (storage_name, repository_name) = path_params.into_inner();
                let storage = crate::helpers::get_storage!(storage_handler, storage_name);
                let  (name,mut repository) = crate::helpers::take_repository!(storage, repository_name);
                let body = body.into_inner();
                let result = match repository {
                $(
                    crate::repository::handler::DynamicRepositoryHandler::$repository(ref mut repository) => {
                        crate::repository::settings::RepositoryConfigHandler::<$config>::update( repository, body)
                    }
                )*
                repository => {
                    storage.add_repository_for_updating(name,repository,false).await.expect("Failed to add repository for updating");
                    return Ok(actix_web::HttpResponse::BadRequest().body("Repository type not supported".to_string()));
                }
                };
                storage.save_repository_config(repository.get_repository(),  crate::repository::settings::RepositoryConfigHandler::<$config>::get(&repository)).await;
                storage.add_repository_for_updating(name, repository,false).await.expect("Failed to add repository for updating");
                result?;
                Ok(actix_web::HttpResponse::NoContent().finish())
            }
        }
    };
}
pub(crate) use define_repository_config_get;
pub(crate) use define_repository_config_set;

macro_rules! define_repository_config_handlers_group {
    ($config:path, $config_name:ident,$($repository:ident),*) => {
        paste::paste! {
            crate::repository::web::multi::settings::define_repository_config_get!($config, $config_name, $($repository),*);
            crate::repository::web::multi::settings::define_repository_config_set!($config, $config_name, $($repository),*);
            pub fn init(cfg: &mut actix_web::web::ServiceConfig) {
                cfg.service(actix_web::web::resource([concat!("/repositories/{storage}/{repository}/config/", stringify!($config_name))])
                .route(actix_web::web::get().to([<get_ $config_name>]))
                .route(actix_web::web::put().to([<set_ $config_name>])));
            }
            }
    };
}

pub(crate) use define_repository_config_handlers_group;
