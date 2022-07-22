macro_rules! define_repository_config_get {
    ($value:literal,$ty:path, $($repo:ident),*) => {
        paste::paste! {
        pub async fn [<get_ $value>](
            storage_handler: actix_web::web::Data<crate::storage::multi::MultiStorageController<crate::storage::DynamicStorage>>,
            database: actix_web::web::Data<sea_orm::DatabaseConnection>,
            auth: crate::authentication::Authentication,
            path_params: actix_web::web::Path<(String, String)>,
        ) -> actix_web::Result<actix_web::HttpResponse> {
            use crate::storage::models::Storage;
            use crate::system::permissions::options::CanIDo;
            let user: crate::system::user::UserModel = auth.get_user(&database).await??;
            user.can_i_edit_repos()?;
            let (storage_name, repository_name) = path_params.into_inner();
            let storage = crate::helpers::get_storage!(storage_handler, storage_name);
            let repository = crate::helpers::get_repository!(storage, repository_name);
            match repository.as_ref() {
                $(
                    crate::repository::handler::DynamicRepositoryHandler::$repo(repository) => {
                        let value = crate::repository::settings::RepositoryConfigHandler::<$ty>::get(repository);
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
    ($value:literal,$ty:path, $($repo:ident),* ) => {
        paste::paste! {
            pub async fn [<set_ $value>](
                storage_handler: actix_web::web::Data<crate::storage::multi::MultiStorageController<crate::storage::DynamicStorage>>,
                database: actix_web::web::Data<sea_orm::DatabaseConnection>,
                auth: crate::authentication::Authentication,
                path_params: actix_web::web::Path<(String, String)>,
                body: actix_web::web::Json<$ty>,
            ) -> actix_web::Result<actix_web::HttpResponse> {
                use crate::storage::models::Storage;
use crate::system::permissions::options::CanIDo;
        let user: crate::system::user::UserModel = auth.get_user(&database).await??;
        user.can_i_edit_repos()?;
        let (storage_name, repository_name) = path_params.into_inner();
            let storage = crate::helpers::get_storage!(storage_handler, storage_name);
            let  (name,mut repository) = crate::helpers::take_repository!(storage, repository_name);
            let body = body.into_inner();
            match repository {
                $(
                    crate::repository::handler::DynamicRepositoryHandler::$repo(ref mut repository) => {
                        crate::repository::settings::RepositoryConfigHandler::<$ty>::update( repository, body);
                    }
                )*
                repository => {
                    storage.add_repository_for_updating(name, repository).expect("Failed to add repository for updating");
                    return Ok(actix_web::HttpResponse::BadRequest().body("Repository type not supported".to_string()));
                }
            }
            storage.add_repository_for_updating(name, repository).expect("Failed to add repository for updating");
            Ok(actix_web::HttpResponse::NoContent().finish())
            }
        }
    };
}
pub(crate) use define_repository_config_get;
pub(crate) use define_repository_config_set;
macro_rules! define_repository_config_handlers_group {
    ($value:literal, $ty:path, $($repo:ident),* ) => {
            crate::repository::web::multi::configs::define_repository_config_get!($value, $ty, $($repo),*);
            crate::repository::web::multi::configs::define_repository_config_set!($value, $ty, $($repo),*);
        paste::paste! {
    pub fn [<init_repository_config_ $value>](cfg: &mut actix_web::web::ServiceConfig) {
            cfg.service(actix_web::web::resource([concat!("/repositories/{storage}/{repository}/config/", $value)])
                .route(actix_web::web::get().to([<get_ $value>]))
                .route(actix_web::web::put().to([<set_ $value>])));
    }
    }
    };
}

pub(crate) use define_repository_config_handlers_group;
