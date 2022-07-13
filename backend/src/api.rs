use crate::storage::models::StorageSaver;
use crate::storage::multi::web::admin;
use utoipa::OpenApi;
#[derive(OpenApi)]
#[openapi(
    handlers(admin::get_storage, admin::get_storages),
    components(StorageSaver)
)]
pub struct ApiDoc;
