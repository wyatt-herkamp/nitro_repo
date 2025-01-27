use std::sync::Arc;

use nr_core::{
    database::entities::repository::DBRepositoryWithStorageName, repository::browse::BrowseResponse,
};
use reqwest::Response;
use thiserror::Error;
use uuid::Uuid;
pub mod browse;
#[derive(Debug, Error)]
pub enum NrApiError {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}
pub struct NrApiInner {
    client: reqwest::Client,
    base_url: String,
}

impl NrApiInner {
    pub fn new(client: reqwest::Client, base_url: String) -> Self {
        Self { client, base_url }
    }

    pub fn api_route(&self, route: &str) -> String {
        if self.base_url.ends_with('/') {
            format!("{}api/{}", self.base_url, route)
        } else {
            format!("{}/api/{}", self.base_url, route)
        }
    }

    pub fn repository_route(&self, route: &str) -> String {
        if self.base_url.ends_with('/') {
            format!("{}repositories/{}", self.base_url, route)
        } else {
            format!("{}/repositories/{}", self.base_url, route)
        }
    }
    pub fn get(&self, route: &str) -> reqwest::RequestBuilder {
        self.client.get(&self.api_route(route))
    }

    pub fn post(&self, route: &str) -> reqwest::RequestBuilder {
        self.client.post(&self.api_route(route))
    }
}
#[derive(Clone)]
pub struct NrApi(pub Arc<NrApiInner>);

impl NrApi {
    pub fn new(client: reqwest::Client, base_url: String) -> Self {
        Self(Arc::new(NrApiInner::new(client, base_url)))
    }

    pub fn api_route(&self, route: &str) -> String {
        self.0.api_route(route)
    }
    pub async fn get_repositories(&self) -> Result<Vec<DBRepositoryWithStorageName>, NrApiError> {
        let res = self.0.get("repository/list").send().await?;
        let res = res.error_for_status()?;
        let body = res.text().await?;
        Ok(serde_json::from_str(&body)?)
    }
    pub async fn get_repository(
        &self,
        repo_id: Uuid,
    ) -> Result<Option<DBRepositoryWithStorageName>, NrApiError> {
        let res = self
            .0
            .get(&format!("repository/{}", repo_id))
            .send()
            .await?;
        let res = res.error_for_status()?;
        let body = res.text().await?;
        Ok(serde_json::from_str(&body)?)
    }
    pub async fn browse_repository(
        &self,
        repo_id: Uuid,
        path: &str,
    ) -> Result<BrowseResponse, NrApiError> {
        let res = self
            .0
            .get(&format!("repository/browse/{}/{path}", repo_id))
            .send()
            .await?;
        let res = res.error_for_status()?;
        let body = res.text().await?;
        Ok(serde_json::from_str(&body)?)
    }

    pub async fn get_file(&self, repo_id: Uuid, path: &str) -> Result<Response, NrApiError> {
        let repository = self.get_repository(repo_id).await?.unwrap();
        let res = self
            .0
            .client
            .get(&self.0.repository_route(&format!(
                "{}/{}{path}",
                repository.storage_name, repository.name
            )))
            .send()
            .await?;
        let res = res.error_for_status()?;
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use core::num;
    use std::sync::{atomic::AtomicUsize, Arc};

    use nr_core::repository::browse::BrowseFile;
    use tokio::time::sleep;
    use tracing::debug;
    use uuid::Uuid;
    fn init_logger() {
        use tracing::{error, info, level_filters::LevelFilter};
        use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt, Layer};
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| {
            let stdout_log = tracing_subscriber::fmt::layer().pretty().without_time();
            tracing_subscriber::registry()
                .with(
                    stdout_log.with_filter(
                        filter::Targets::new()
                            .with_target("nr_api", LevelFilter::TRACE)
                            .with_target("nr_core", LevelFilter::TRACE)
                            .with_default(LevelFilter::INFO),
                    ),
                )
                .init();
        });
        info!("Logger initialized");
        error!("This is an error message");
    }

    #[tokio::test]
    async fn tests() -> anyhow::Result<()> {
        init_logger();
        let client = reqwest::Client::builder()
            .user_agent("nr-api-test/0.1")
            .build()?;

        let api = super::NrApi::new(client, "http://localhost:6742/".to_string());

        let repos = api.get_repositories().await?;
        sleep(std::time::Duration::from_secs(1)).await;
        let instant = std::time::Instant::now();
        let number_of_requests = Arc::new(AtomicUsize::new(0));
        for repo in repos {
            debug!(?repo, "Browsing Entire Repo");

            recursive_browse(
                api.clone(),
                repo.id,
                String::default(),
                number_of_requests.clone(),
            )
            .await?;
        }
        sleep(std::time::Duration::from_secs(1)).await;
        println!(
            "Number of requests: {}",
            number_of_requests.load(std::sync::atomic::Ordering::Relaxed)
        );
        let elapsed = instant.elapsed();
        println!("Elapsed: {:?}", elapsed);
        Ok(())
    }

    async fn recursive_browse(
        api: super::NrApi,
        repo_id: Uuid,
        path: String,
        number_of_requests: Arc<AtomicUsize>,
    ) -> anyhow::Result<()> {
        let browse = api.browse_repository(repo_id, &path).await?;
        number_of_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        for entry in browse.files {
            debug!(?entry, "Found Entry");

            match entry {
                BrowseFile::Directory {
                    name,
                    number_of_files,
                } => {
                    let next_path = format!("{}/{}", path, name);
                    Box::pin(recursive_browse(
                        api.clone(),
                        repo_id,
                        next_path,
                        number_of_requests.clone(),
                    ))
                    .await?;
                }
                BrowseFile::File { name, .. } => {
                    let next_path = format!("{}/{}", path, name);

                    let file = api.get_file(repo_id, &next_path).await?;
                    number_of_requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    debug!(?file, "Got File");
                }
            }
        }
        Ok(())
    }
}
