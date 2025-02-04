use std::{future::Future, net::SocketAddr, task::Poll};

use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use futures::{SinkExt, Stream};
use nr_core::{
    repository::{browse::BrowseFile, project::ProjectResolution},
    storage::StoragePath,
};
use nr_storage::{
    DirectoryListStream, DynDirectoryListStream, DynStorage, EmptyDirectoryListStream, FileType,
    Storage, StorageFileMeta,
};
use pin_project::pin_project;
use serde::{Deserialize, Serialize};
use strum::EnumIs;
use tokio::select;
use tracing::{
    Level, Span, debug, debug_span, event,
    field::{Empty, debug},
    info, instrument, warn,
};

use crate::{
    app::{
        NitroRepo,
        authentication::ws::{WebSocketAuthentication, WebSocketAuthenticationMessage},
    },
    error::InternalError,
    repository::{DynRepository, Repository, utils::can_read_repository},
};

use super::BrowseStreamPrimaryData;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebsocketIncomingMessage {
    ListDirectory(StoragePath),
    Authentication(WebSocketAuthenticationMessage),
}
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum WebsocketOutgoingMessage {
    DirectoryItem(BrowseFile),
    OpenedDirectory(BrowseStreamPrimaryData),
    EndOfDirectory,
    Error(String),
    Unauthorized,
    Authorized,
}
impl From<WebsocketOutgoingMessage> for Message {
    fn from(message: WebsocketOutgoingMessage) -> Self {
        let message = serde_json::to_string(&message).unwrap();
        Message::Text(Utf8Bytes::from(message))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIs)]
pub enum WSPermissionsStatus {
    Unauthorized,
    Pending,
    Authorized,
}
pub struct BrowseWSState {
    pub repository: DynRepository,
    pub site: NitroRepo,
    pub authentication: Option<WebSocketAuthentication>,
    pub access_status: WSPermissionsStatus,
    pub active_path: StoragePathStream,
}
impl BrowseWSState {
    pub fn new(repository: DynRepository, site: NitroRepo) -> Self {
        let active_path = StoragePathStream::new(repository.clone());
        BrowseWSState {
            repository,
            site,
            authentication: None,
            access_status: WSPermissionsStatus::Pending,
            active_path,
        }
    }

    async fn handle_message(
        &mut self,
        message: Result<Message, axum::Error>,
        socket: &mut WebSocket,
    ) -> Result<bool, InternalError> {
        let span = debug_span!(
            "Handle message",
            message = debug(&message),
            "message.type" = Empty,
            otel.status_code = Empty,
            exception.message = Empty,
        );
        let _guard = span.enter();
        let message = match message {
            Ok(message) => message,
            Err(e) => {
                span.record("otel.status_code", "ERROR");
                span.record("exception.message", e.to_string());
                let message = WebsocketOutgoingMessage::Error(e.to_string());
                socket.send(message.into()).await?;
                return Ok(true);
            }
        };
        let incoming_message = match message {
            Message::Close(_) => {
                span.record("message.type", "Close");
                return Ok(true);
            }
            Message::Ping(_) | Message::Pong(_) => {
                span.record("message.type", "Ping/Pong");
                return Ok(false);
            }
            Message::Binary(bytes) => {
                span.record("message.type", "Binary");
                let message: WebsocketIncomingMessage = match serde_json::from_slice(&bytes) {
                    Ok(message) => message,
                    Err(e) => {
                        span.record("otel.status_code", "ERROR");
                        span.record("exception.message", e.to_string());
                        event!(Level::ERROR, ?e, "Failed to parse message");
                        let message = WebsocketOutgoingMessage::Error(e.to_string());
                        socket.send(message.into()).await?;
                        return Ok(false);
                    }
                };
                message
            }
            Message::Text(content) => {
                span.record("message.type", "Text");
                let message: WebsocketIncomingMessage = match serde_json::from_str(&content) {
                    Ok(message) => message,
                    Err(e) => {
                        span.record("otel.status_code", "ERROR");
                        span.record("exception.message", e.to_string());
                        event!(Level::ERROR, ?e, "Failed to parse message");
                        let message = WebsocketOutgoingMessage::Error(e.to_string());
                        socket.send(message.into()).await?;
                        return Ok(false);
                    }
                };
                message
            }
        };

        debug!(?incoming_message, "Received message");
        match incoming_message {
            WebsocketIncomingMessage::ListDirectory(path) => {
                if self.access_status != WSPermissionsStatus::Authorized {
                    if !can_read_repository(
                        &self.authentication,
                        self.repository.visibility(),
                        self.repository.id(),
                        self.site.as_ref(),
                    )
                    .await?
                    {
                        info!(?self.authentication, "Access denied. Closing connection");
                        self.access_status = WSPermissionsStatus::Unauthorized;
                        let message = WebsocketOutgoingMessage::Unauthorized;
                        socket.send(message.into()).await?;
                        return Ok(true);
                    } else {
                        debug!("Access granted");
                        self.access_status = WSPermissionsStatus::Authorized;
                    }
                }
                match self.active_path.change_directory(path).await {
                    Ok(ok) => {
                        event!(Level::DEBUG, ?ok, "Opened directory");
                        let message = WebsocketOutgoingMessage::OpenedDirectory(ok);
                        socket.send(message.into()).await?;
                    }
                    Err(err) => {
                        span.record("otel.status_code", "ERROR");
                        span.record("exception.message", err.to_string());
                        let message = WebsocketOutgoingMessage::Error(err.to_string());
                        event!(Level::ERROR, ?err, "Failed to open directory");
                        socket.send(message.into()).await?;
                    }
                }
                Ok(false)
            }
            WebsocketIncomingMessage::Authentication(auth) => {
                let auth = auth.attempt_login(&self.site).await;

                match auth {
                    Ok(auth) => {
                        event!(Level::DEBUG, ?auth, "Authenticated");
                        self.authentication = Some(auth);
                        if !can_read_repository(
                            &self.authentication,
                            self.repository.visibility(),
                            self.repository.id(),
                            self.site.as_ref(),
                        )
                        .await?
                        {
                            info!(?self.authentication, "Access denied. Closing connection");

                            self.access_status = WSPermissionsStatus::Unauthorized;
                            let message = WebsocketOutgoingMessage::Unauthorized;
                            socket.send(message.into()).await?;
                            return Ok(true);
                        } else {
                            self.access_status = WSPermissionsStatus::Authorized;
                        }
                        let message = WebsocketOutgoingMessage::Authorized;
                        socket.send(message.into()).await?;
                        Ok(false)
                    }
                    Err(err) => {
                        span.record("otel.status_code", "ERROR");
                        span.record("exception.message", err.to_string());
                        event!(Level::ERROR, ?err, "Failed to authenticate");
                        let message = WebsocketOutgoingMessage::Error(err.to_string());
                        socket.send(message.into()).await?;
                        Ok(true)
                    }
                }
            }
        }
    }
    async fn handle_next_item(
        &mut self,
        socket: &mut WebSocket,
        next_item: Result<Option<StorageFileMeta<FileType>>, InternalError>,
    ) -> Result<bool, InternalError> {
        let span = debug_span!(
            "Handle Next Item",
            next_item = debug(&next_item),
            otel.status_code = Empty,
            exception.message = Empty,
        );
        let _guard = span.enter();
        match next_item {
            Ok(Some(file)) => {
                let message = WebsocketOutgoingMessage::DirectoryItem(file.into());
                debug!(?message, "Sending message");
                socket.send(message.into()).await?;
                span.record("otel.status_code", "OK");
            }
            Ok(None) => {
                let message = WebsocketOutgoingMessage::EndOfDirectory;
                socket.send(message.into()).await?;
                span.record("otel.status_code", "OK");
            }
            Err(e) => {
                event!(Level::ERROR, ?e, "Failed to get next item");
                let message = WebsocketOutgoingMessage::Error(e.to_string());
                socket.send(message.into()).await?;
                span.record("otel.status_code", "ERROR");
                span.record("exception.message", e.to_string());
                return Ok(true);
            }
        }
        Ok(false)
    }
}

pub(super) async fn handle_socket(
    mut socket: WebSocket,
    who: SocketAddr,
    repository: DynRepository,
    site: NitroRepo,
    span: Span,
) {
    let _guard = span.enter();
    info!(?who, "New websocket connection");

    if let Err(socket) = socket.send(Message::Ping(Default::default())).await {
        event!(Level::ERROR, ?socket, "Failed to send ping");
        return;
    }
    let mut state = BrowseWSState::new(repository, site);
    loop {
        select! {
             message = socket.recv() => {
                let Some(message) = message else{
                    event!(Level::DEBUG, "End of stream");
                    break;
                };

                match state.handle_message(message,  &mut socket).await  {
                    Ok(ok) if ok => {
                        break;
                    },
                    Ok(_) => {},
                    Err(err) => {
                        event!(Level::ERROR, ?err, "Failed to handle message");
                        break;
                    },
                }
             }
             next_item = state.active_path.next_item() => {
                debug!(?next_item, "Next item");
                match state.handle_next_item(&mut socket, next_item).await {
                    Ok(ok) if ok => {
                        break;
                    },
                    Ok(_) => {},
                    Err(err) => {
                        event!(Level::ERROR, ?err, "Failed to handle next item");
                        break;
                    },
                }
             }
        }
    }
    if let Err(err) = socket.close().await {
        event!(Level::ERROR, ?err, "Failed to close websocket connection");
    }
    info!("Closing websocket connection");
}
#[derive(Debug)]
pub struct StoragePathStream {
    repository: DynRepository,
    path: StoragePath,
    storage: DynStorage,
    current: DynDirectoryListStream,
    sent_end_of_directory: bool,
}

impl StoragePathStream {
    pub fn new(repository: DynRepository) -> Self {
        let storage = repository.get_storage();
        StoragePathStream {
            path: StoragePath::from("/"),
            repository,
            storage,
            sent_end_of_directory: true,
            current: DynDirectoryListStream::new(EmptyDirectoryListStream),
        }
    }

    #[instrument(skip(self), fields(project, number_of_files))]
    pub async fn change_directory(
        &mut self,
        path: StoragePath,
    ) -> Result<BrowseStreamPrimaryData, InternalError> {
        let span = Span::current();
        self.path = path;
        info!(?self.path, "Changing directory");
        self.sent_end_of_directory = false;
        self.current = self
            .storage
            .stream_directory(self.repository.id(), &self.path)
            .await?
            .unwrap_or_else(|| {
                warn!("Empty directory stream");
                DynDirectoryListStream::new(EmptyDirectoryListStream)
            });

        let project_resolution = {
            event!(Level::DEBUG, "Checking for project and version");
            match self
                .repository
                .resolve_project_and_version_for_path(&self.path)
                .await
            {
                Ok(ok) => ok,
                Err(err) => {
                    event!(
                        Level::ERROR,
                        ?err,
                        path = ?self.path,
                        "Failed to resolve project and version for path"
                    );
                    ProjectResolution::default()
                }
            }
        };
        span.record("project", debug(&project_resolution));
        span.record("number_of_files", self.current.number_of_files());
        let data = BrowseStreamPrimaryData {
            project_resolution: Some(project_resolution),
            number_of_files: self.current.number_of_files() as usize,
        };
        debug!("Opened directory");

        Ok(data)
    }
    pub fn next_item(&mut self) -> NextItem {
        NextItem {
            stream: &mut self.current,
            sent_end_of_dir: &mut self.sent_end_of_directory,
        }
    }
}
#[pin_project]
pub struct NextItem<'a> {
    #[pin]
    stream: &'a mut DynDirectoryListStream,

    sent_end_of_dir: &'a mut bool,
}
impl Future for NextItem<'_> {
    type Output = Result<Option<StorageFileMeta<FileType>>, InternalError>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.project();
        match this.stream.poll_next(cx) {
            std::task::Poll::Ready(Some(file)) => {
                println!("file: {:?}", file);
                std::task::Poll::Ready(file.map_err(InternalError::from))
            }
            std::task::Poll::Ready(None) => {
                // Only Send an end of dictory message once. Otherwise we will keep sending Pending
                if **this.sent_end_of_dir {
                    Poll::Pending
                } else {
                    **this.sent_end_of_dir = true;
                    std::task::Poll::Ready(Ok(None))
                }
            }
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}
