use crate::repository::settings::RepositoryConfig;
use crate::storage::models::Storage;
use crate::storage::DynamicStorage;
use crate::system::user::database::UserSafeData;
use std::io;

use log::{trace, warn};
use reqwest::header::{HeaderMap, USER_AGENT};

use crate::storage::error::StorageError;
use std::sync::Arc;
use thiserror::Error;
use crate::utils::base64_utils;

#[derive(Error, Debug)]
pub enum ExternalStageError {
    #[error("{0}")]
    IoError(#[from] io::Error),
    #[error("{0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("{0}")]
    StorageError(#[from] StorageError),
}
pub async fn stage_to_external(
    username: String,
    password: String,
    url: String,
    directory: String,
    storage: Arc<DynamicStorage>,
    repository: RepositoryConfig,
    _model: UserSafeData,
) -> Result<(), ExternalStageError> {
    let string = base64_utils::encode(format!("{}:{}", username, password));
    let mut map = HeaderMap::new();
    map.insert(
        "Authorization",
        format!("Basic {}", string).parse().unwrap(),
    );
    map.insert(
        USER_AGENT,
        "Nitro Repository Staging Service".parse().unwrap(),
    );
    let result = reqwest::ClientBuilder::new().default_headers(map).build()?;
    for x in storage
        .list_files(&repository.name, directory.clone())
        .await?
    {
        if x.is_dir {
            warn!("Skipping directory {}", x.name);
            continue;
        }
        let file_path = format!("{}/{}", directory, x.name);
        let data = storage.get_file(&repository, &file_path).await?;
        let full_url = format!("{}/{}", url, file_path);
        trace!("Staging {:?} to {}", x, full_url);
        if let Some(v) = data {
            result.put(&full_url).body(v).send().await?;
        }
    }
    let mut v: Vec<_> = directory.split('/').collect();
    v.pop();
    v.push("maven-metadata.xml");
    let path = v.join("/");
    let full_url = format!("{}/{}", url, path);
    let data = storage.get_file(&repository, &path).await?;
    if let Some(v) = data {
        result.put(&full_url).body(v).send().await?;
    }
    Ok(())
}
