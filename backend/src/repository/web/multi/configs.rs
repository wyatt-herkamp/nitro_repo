use paste::paste;
use schemars::schema_for;

macro_rules! define_repository_config_get {
    ($value:literal,$ty:tt ) => {
        paste::paste! {
        pub async fn [<get_ $value>](
            storage_handler: actix_web::web::Data<crate::storage::multi::MultiStorageController>,
            database: actix_web::web::Data<sea_orm::DatabaseConnection>,
            auth: crate::authentication::Authentication,
            path_params: actix_web::web::Path<(String, String)>,
        ) -> actix_web::Result<actix_web::HttpResponse> {
            use crate::storage::models::Storage;
use crate::system::permissions::options::CanIDo;
            let user: crate::system::user::UserModel = auth.get_user(&database).await??;
            user.can_i_edit_repos()?;
            let (storage_name, repository_name) = path_params.into_inner();
            let storage = storage_handler
                .get_storage_by_name(&storage_name)
                .await
                .map_err(crate::error::internal_error::InternalError::from)?
                .ok_or_else(|| crate::error::api_error::APIError::storage_not_found())?;
            let repository = storage
                .get_repository(&repository_name)
                .await
                .map_err(crate::error::internal_error::InternalError::from)?
                .ok_or_else(|| crate::error::api_error::APIError::repository_not_found())?;
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
                storage_handler: actix_web::web::Data<crate::storage::multi::MultiStorageController>,
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
        let storage = storage_handler
            .get_storage_by_name(&storage_name)
            .await
            .map_err(crate::error::internal_error::InternalError::from)?
            .ok_or_else(|| crate::error::api_error::APIError::storage_not_found())?;
        let repository = storage
            .get_repository(&repository_name)
            .await
            .map_err(crate::error::internal_error::InternalError::from)?
            .ok_or_else(|| crate::error::api_error::APIError::repository_not_found())?;
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
                storage_handler: actix_web::web::Data<crate::storage::multi::MultiStorageController>,
                database: actix_web::web::Data<sea_orm::DatabaseConnection>,
                auth: crate::authentication::Authentication,
                path_params: actix_web::web::Path<(String, String)>,
            ) -> actix_web::Result<actix_web::HttpResponse> {
                use crate::storage::models::Storage;
use crate::system::permissions::options::CanIDo;
        let user: crate::system::user::UserModel = auth.get_user(&database).await??;
        user.can_i_edit_repos()?;
        let (storage_name, repository_name) = path_params.into_inner();
        let storage = storage_handler
            .get_storage_by_name(&storage_name)
            .await
            .map_err(crate::error::internal_error::InternalError::from)?
            .ok_or_else(|| crate::error::api_error::APIError::storage_not_found())?;
        let repository = storage
            .get_repository(&repository_name)
            .await
            .map_err(crate::error::internal_error::InternalError::from)?
            .ok_or_else(|| crate::error::api_error::APIError::repository_not_found())?;
        repository
            .save_config::<$ty, crate::storage::DynamicStorage>(&storage, None)
            .await
            .map_err(crate::error::internal_error::InternalError::from)?;
        Ok(actix_web::HttpResponse::NoContent().finish())
            }
            }
    };
}

macro_rules! define_repository_config_handlers {
    ($value:literal,$ty:tt ) => {
        crate::repository::web::multi::configs::define_repository_config_get!($value, $ty);
        crate::repository::web::multi::configs::define_repository_config_set!($value, $ty);
        crate::repository::web::multi::configs::define_repository_config_delete!($value, $ty);
        paste::paste! {
                pub fn [<init_ $value>](cfg: &mut actix_web::web::ServiceConfig) {
                        cfg.service(actix_web::web::resource(["/repositories/{storage}/{repository}/config/".to_owned() + $value])
                        .route(actix_web::web::get().to([<get_ $value>]))
                        .route(actix_web::web::put().to([<set_ $value>]))
                        .route(actix_web::web::delete().to([<delete_ $value>]))
                        );
                }
            }
    };
}

pub(crate) use define_repository_config_delete;
pub(crate) use define_repository_config_get;
pub(crate) use define_repository_config_handlers;
pub(crate) use define_repository_config_set;
