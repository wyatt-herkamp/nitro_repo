use crate::authentication::Authentication;
use crate::error::internal_error::InternalError;
use crate::repository::handler::Repository;
use crate::repository::maven::settings::ProxySettings;
use crate::repository::response::RepoResponse;
use crate::repository::settings::{RepositoryConfig, RepositoryConfigType};
use crate::storage::file::StorageFileResponse;
use crate::storage::models::Storage;
use crate::system::permissions::permissions_checker::CanIDo;

use actix_web::http::header::HeaderMap;

use actix_web::web::Bytes;
use actix_web::{Error, HttpResponse};
use async_trait::async_trait;

use crate::repository::nitro::nitro_repository::NitroRepositoryHandler;
use crate::repository::settings::badge::BadgeSettings;
use crate::repository::settings::frontend::Frontend;

use actix_web::http::StatusCode;
use futures::channel::mpsc::unbounded;
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use log::error;
use schemars::JsonSchema;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::repository::settings::repository_page::RepositoryPage;

#[derive(Debug)]
pub struct ProxyMavenRepository<S: Storage> {
    pub config: RepositoryConfig,
    pub proxy: MavenProxySettings,
    pub badge: BadgeSettings,
    pub frontend: Frontend,
    pub repository_page: RepositoryPage,
    pub storage: Arc<S>,
}

impl<S: Storage> Clone for ProxyMavenRepository<S> {
    fn clone(&self) -> Self {
        ProxyMavenRepository {
            config: self.config.clone(),
            storage: self.storage.clone(),
            proxy: self.proxy.clone(),
            badge: self.badge.clone(),
            frontend: self.frontend.clone(),
            repository_page: self.repository_page.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema, Default)]
pub struct MavenProxySettings {
    proxies: Vec<ProxySettings>,
}

impl RepositoryConfigType for MavenProxySettings {
    fn config_name() -> &'static str {
        "maven_proxy.json"
    }
}
crate::repository::settings::define_configs_on_handler!(
    ProxyMavenRepository<StorageType>,
    badge,
    BadgeSettings,
    frontend,
    Frontend,
    proxy,
    MavenProxySettings,
    repository_page,
    RepositoryPage
);
#[async_trait]
impl<S: Storage> Repository<S> for ProxyMavenRepository<S> {
    fn get_repository(&self) -> &RepositoryConfig {
        &self.config
    }
    fn get_mut_config(&mut self) -> &mut RepositoryConfig {
        &mut self.config
    }
    fn get_storage(&self) -> &S {
        self.storage.as_ref()
    }
    fn features(&self) -> Vec<&'static str> {
        vec!["badge", "frontend", "proxy"]
    }
    async fn handle_get(
        &self,
        path: &str,
        _http: &HeaderMap,
        conn: &DatabaseConnection,
        authentication: Authentication,
    ) -> Result<RepoResponse, Error> {
        crate::helpers::read_check!(authentication, conn, self.config);

        match self
            .storage
            .get_file_as_response(&self.config, path)
            .await
            .map_err(InternalError::from)?
        {
            StorageFileResponse::List(list) => {
                let files = self.process_storage_files(list, path).await?;
                Ok(RepoResponse::try_from((files, StatusCode::OK))?)
            }

            StorageFileResponse::NotFound => {
                let builder = reqwest::ClientBuilder::new()
                    .user_agent("Nitro Repo Proxy Service")
                    .build()
                    .unwrap();
                for proxy in &self.proxy.proxies {
                    let url = format!("{}/{}", proxy.proxy_url, path);
                    let response = builder.get(&url).send().await;
                    if let Ok(response) = response {
                        if response.status().is_success() {
                            let mut stream = response.bytes_stream();
                            let (mut server, client) =
                                unbounded::<Result<actix_web::web::Bytes, InternalError>>();
                            let (mut file_server, file_client) = unbounded::<Bytes>();
                            actix_web::rt::spawn(async move {
                                while let Some(chunk) = stream.next().await {
                                    if let Ok(chunk) = chunk {
                                        file_server.send(chunk.clone()).await.unwrap();
                                        server.send(Ok(chunk)).await.unwrap();
                                    }
                                }
                            });
                            if let Err(error) =
                            self.storage
                                .write_file_stream(&self.config, file_client, path)
                            {
                                error!("Unable to save data: {}", error);
                            }

                            return Ok(RepoResponse::HttpResponse(
                                HttpResponse::Ok().streaming(client),
                            ));
                        }
                    }
                }
                Ok(RepoResponse::FileResponse(StorageFileResponse::NotFound))
            }
            v => Ok(RepoResponse::FileResponse(v)),
        }
    }
}

impl<S: Storage> NitroRepositoryHandler<S> for ProxyMavenRepository<S> {
    #[inline(always)]
    fn parse_project_to_directory<V: Into<String>>(value: V) -> String {
        value.into().replace('.', "/").replace(':', "/")
    }
}

pub mod multi_web {
    crate::repository::maven::settings::macros::define_repository_config_handlers_group!(
        super::MavenProxySettings,
        maven_proxy,
        Proxy
    );
}
