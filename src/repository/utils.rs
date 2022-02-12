use std::path::PathBuf;

use crate::repository::types::RepositoryRequest;
use crate::utils::get_storage_location;

pub fn build_artifact_directory(request: &RepositoryRequest) -> PathBuf {
    build_directory(request).join(&request.value)
}

pub fn build_directory(request: &RepositoryRequest) -> PathBuf {
    get_storage_location()
        .join("storages")
        .join(&request.storage.name)
        .join(&request.repository.name)
}
