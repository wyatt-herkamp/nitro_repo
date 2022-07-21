macro_rules! define_repository_config_get {
    ($value:literal,$ty:tt ) => {
        paste::paste! {
        pub async fn [<get_ $value>](
            storage_handler: actix_web::web::Data<crate::storage::multi::MultiStorageController<DynamicStorage>>,
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
            let config = repository
                .get_config::<$ty, crate::storage::DynamicStorage>(&storage)
                .await
                .map_err(crate::error::internal_error::InternalError::from)?;
            if let Some(config) = config {
                Ok(actix_web::HttpResponse::Ok().json(config))
            } else {
                Ok(actix_web::HttpResponse::NotFound().finish())
            }
        }
        }
    };
}
macro_rules! define_repository_config_set {
    ($value:literal,$ty:tt ) => {
        paste::paste! {
            pub async fn [<set_ $value>](
                storage_handler: actix_web::web::Data<crate::storage::multi::MultiStorageController<DynamicStorage>>,
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
            let repository = crate::helpers::get_repository!(storage, repository_name);
        let body = body.into_inner();
        repository
            .save_config::<$ty, crate::storage::DynamicStorage>(&storage,Some(&body))
            .await
            .map_err(crate::error::internal_error::InternalError::from)?;
        Ok(actix_web::HttpResponse::NoContent().finish())
            }
            }
    };
}
macro_rules! define_repository_config_delete {
    ($value:literal,$ty:tt ) => {
        paste::paste! {
            pub async fn [<delete_ $value>](
                storage_handler: actix_web::web::Data<crate::storage::multi::MultiStorageController<DynamicStorage>>,
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
        repository
            .save_config::<$ty, crate::storage::DynamicStorage>(&storage, None)
            .await
            .map_err(crate::error::internal_error::InternalError::from)?;
        Ok(actix_web::HttpResponse::NoContent().finish())
            }
            }
    };
}

macro_rules! define_repository_config_handlers_group {
    ($($value:literal, $ty:tt),*) => {
        $(
            crate::repository::web::multi::configs::define_repository_config_get!($value, $ty);
            crate::repository::web::multi::configs::define_repository_config_set!($value, $ty);
            crate::repository::web::multi::configs::define_repository_config_delete!($value, $ty);
        )*
    pub fn init_repository_configs(cfg: &mut actix_web::web::ServiceConfig) {
        $(
        paste::paste! {
            cfg.service(actix_web::web::resource([concat!("/repositories/{storage}/{repository}/config/", $value)])
                .route(actix_web::web::get().to([<get_ $value>]))
                .route(actix_web::web::put().to([<set_ $value>]))
                .route(actix_web::web::delete().to([<delete_ $value>])));
    }
        )*
    }
    };
}





