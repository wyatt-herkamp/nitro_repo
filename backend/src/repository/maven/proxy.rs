use crate::authentication::Authentication;
use crate::error::internal_error::InternalError;
use crate::repository::handler::Repository;
use crate::repository::maven::settings::ProxySettings;
use crate::repository::response::RepoResponse;
use crate::repository::settings::{RepositoryConfig, Visibility};
use crate::storage::file::StorageFileResponse;
use crate::storage::models::Storage;
use crate::system::permissions::options::CanIDo;
use crate::system::user::UserModel;
use actix_web::http::header::HeaderMap;

use actix_web::web::Bytes;
use actix_web::{Error, HttpResponse};
use async_trait::async_trait;

use futures::channel::mpsc::unbounded;
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
#[derive(Debug)]
pub struct ProxyMavenRepository<S: Storage> {
    pub config: RepositoryConfig,
    pub proxy: Vec<ProxySettings>,
    pub storage: Arc<S>,
}
impl<S: Storage> Clone for ProxyMavenRepository<S> {
    fn clone(&self) -> Self {
        ProxyMavenRepository {
            config: self.config.clone(),
            storage: self.storage.clone(),
            proxy: self.proxy.clone(),
        }
    }
}
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

    async fn handle_get(
        &self,
        path: &str,
        _http: &HeaderMap,
        conn: &DatabaseConnection,
        authentication: Authentication,
    ) -> Result<RepoResponse, Error> {
        if self.config.visibility == Visibility::Private {
            let caller: UserModel = authentication.get_user(conn).await??;
            if let Some(value) = caller.can_read_from(&self.config)? {
                return Err(value.into());
            }
        }

        match self
            .storage
            .get_file_as_response(&self.config, path)
            .await
            .map_err(InternalError::from)?
        {
            StorageFileResponse::List(_list) => {
                /*                let files = self.process_storage_files(list, path).await?;
                Ok(RepoResponse::try_from((files, StatusCode::OK))?)*/
                panic!("Not implemented")
            }

            StorageFileResponse::NotFound => {
                let builder = reqwest::ClientBuilder::new()
                    .user_agent("Nitro Repo Proxy Service")
                    .build()
                    .unwrap();
                for proxy in &self.proxy {
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
                            self.storage
                                .write_file_stream(&self.config, file_client, path)
                                .expect("Failed to write file stream");

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
