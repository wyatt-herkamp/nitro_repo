use crate::repository::settings::RepositoryConfig;
use crate::storage::models::Storage;
use crate::storage::DynamicStorage;
use crate::system::user::database::UserSafeData;

use log::{trace, warn};
use reqwest::header::{HeaderMap, USER_AGENT};

use std::sync::Arc;

pub async fn stage_to_external(
    username: String,
    password: String,
    url: String,
    directory: String,
    storage: Arc<DynamicStorage>,
    repository: RepositoryConfig,
    _model: UserSafeData,
) -> anyhow::Result<()> {
    let string = base64::encode(format!("{}:{}", username, password));
    let mut map = HeaderMap::new();
    map.insert(
        "Authorization",
        format!("Basic {}", string).parse().unwrap(),
    );
    map.insert(USER_AGENT, "Nitro Repository Staging Service".parse()?);
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
    let mut v = directory
        .split('/')
        .map(|v| v.to_string())
        .collect::<Vec<_>>();
    v.pop();
    v.push("maven-metadata.xml".to_string());
    let path = v.join("/");
    let full_url = format!("{}/{}", url, path);
    let data = storage.get_file(&repository, &path).await?;
    if let Some(v) = data {
        result.put(&full_url).body(v).send().await?;
    }
    Ok(())
}
