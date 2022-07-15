use crate::repository::web::multi::public::repositories::{self, PublicRepositoryResponse};
use crate::storage::models::StorageSaver;
use crate::storage::multi::web::admin;
use crate::storage::multi::web::public::{self, PublicStorageResponse};
use utoipa::OpenApi;
#[derive(OpenApi)]
#[openapi(
    handlers(
        admin::get_storage,
        admin::get_storages,
        public::get_storages_multi,
        repositories::get_repositories
    ),
    components(StorageSaver, PublicStorageResponse, PublicRepositoryResponse)
)]
pub struct ApiDoc;
