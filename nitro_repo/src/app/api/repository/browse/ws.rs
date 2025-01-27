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
use tokio::select;
use tracing::{debug, debug_span, event, field::Empty, info, instrument, warn, Level, Span};

use crate::{
    error::InternalError,
    repository::{DynRepository, Repository},
};

use super::BrowseStreamPrimaryData;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WebsocketIncomingMessage {
    ListDirectory(StoragePath),
}
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum WebsocketOutgoingMessage {
    DirectoryItem(BrowseFile),
    OpenedDirectory(BrowseStreamPrimaryData),
    EndOfDirectory,
    Error(String),
}
impl From<WebsocketOutgoingMessage> for Message {
    fn from(message: WebsocketOutgoingMessage) -> Self {
        let message = serde_json::to_string(&message).unwrap();
        Message::Text(Utf8Bytes::from(message))
    }
}
async fn handle_message(
    message: Result<Message, axum::Error>,
    active_path: &mut StoragePathStream,
    socket: &mut WebSocket,
) -> Result<bool, InternalError> {
    let span = debug_span!("Handle message", message = debug(&message), "message.type");
    let _guard = span.enter();
    let message = match message {
        Ok(message) => message,
        Err(e) => {
            let message = WebsocketOutgoingMessage::Error(e.to_string());
            socket.send(message.into()).await?;
            return Ok(true);
        }
    };
    let incoming_message = match message {
        Message::Close(_) => {
            span.record("message.type", &"Close");
            return Ok(true);
        }
        Message::Ping(_) | Message::Pong(_) => {
            span.record("message.type", &"Ping/Pong");
            return Ok(false);
        }
        Message::Binary(bytes) => {
            span.record("message.type", &"Binary");
            let message: WebsocketIncomingMessage = match serde_json::from_slice(&bytes) {
                Ok(message) => message,
                Err(e) => {
                    let message = WebsocketOutgoingMessage::Error(e.to_string());
                    socket.send(message.into()).await?;
                    return Ok(false);
                }
            };
            message
        }
        Message::Text(content) => {
            span.record("message.type", &"Text");
            let message: WebsocketIncomingMessage = match serde_json::from_str(&content) {
                Ok(message) => message,
                Err(e) => {
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
            match active_path.change_directory(path).await {
                Ok(ok) => {
                    event!(Level::DEBUG, ?ok, "Opened directory");
                    let message = WebsocketOutgoingMessage::OpenedDirectory(ok);

                    socket.send(message.into()).await?;
                }
                Err(err) => {
                    let message = WebsocketOutgoingMessage::Error(err.to_string());
                    event!(Level::ERROR, ?err, "Failed to open directory");
                    socket.send(message.into()).await?;
                }
            }
            return Ok(false);
        }
    }
}
async fn handle_next_item(
    socket: &mut WebSocket,
    next_item: Result<Option<StorageFileMeta<FileType>>, InternalError>,
) -> Result<bool, InternalError> {
    let span = debug_span!(
        "Handle message",
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
            span.record("exception.message", &e.to_string());
            return Ok(true);
        }
    }
    Ok(false)
}

pub(super) async fn handle_socket(
    mut socket: WebSocket,
    who: SocketAddr,
    repository: DynRepository,
    span: Span,
) {
    let _guard = span.enter();
    info!(?who, "New websocket connection");

    if let Err(socket) = socket.send(Message::Ping(Default::default())).await {
        event!(Level::ERROR, ?socket, "Failed to send ping");
        return;
    }
    let mut active_path =
        StoragePathStream::new_empty(repository.clone(), repository.get_storage());

    loop {
        select! {
             message = socket.recv() => {
                let Some(message) = message else{
                    event!(Level::DEBUG, "End of stream");
                    break;
                };
                 match handle_message(message, &mut active_path, &mut socket).await  {
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
             next_item = active_path.next_item() => {
                debug!(?next_item, "Next item");
                match handle_next_item(&mut socket, next_item).await {
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
    pub fn new_empty(repository: DynRepository, storage: DynStorage) -> Self {
        StoragePathStream {
            path: StoragePath::from("/"),
            repository,
            storage,
            sent_end_of_directory: false,
            current: DynDirectoryListStream::new(EmptyDirectoryListStream),
        }
    }

    #[instrument]
    pub async fn change_directory(
        &mut self,
        path: StoragePath,
    ) -> Result<BrowseStreamPrimaryData, InternalError> {
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
                Ok(ok) => Some(ok),
                Err(err) => {
                    event!(
                        Level::ERROR,
                        ?err,
                        path = ?self.path,
                        "Failed to resolve project and version for path"
                    );
                    Some(ProjectResolution::default())
                }
            }
        };

        let data = BrowseStreamPrimaryData {
            project_resolution,
            number_of_files: self.current.number_of_files() as usize,
        };

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
