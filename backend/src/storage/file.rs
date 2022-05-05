use crate::error::api_error::APIError;
use crate::error::internal_error::InternalError;
use crate::storage::error::StorageError;
use actix_files::NamedFile;
use actix_web::body::BoxBody;
use actix_web::http::header::ACCEPT;
use actix_web::http::{Method, StatusCode};
use actix_web::{HttpRequest, HttpResponse, Responder, ResponseError};
use log::error;
use serde::Serialize;
use std::fs::{DirEntry, Metadata};
use std::path::PathBuf;
use std::time::SystemTime;
use tokio::fs::OpenOptions;

///Storage Files are just a data container holding the file name, directory relative to the root of nitro_repo and if its a directory
#[derive(Serialize, Clone, Debug)]
pub struct StorageFile {
    pub name: String,
    pub full_path: String,
    pub mime: String,
    pub directory: bool,
    pub file_size: u64,
    pub modified: u128,
    pub created: u128,
}

impl StorageFile {
    fn meta_data(metadata: Metadata) -> (u128, u128, u64, bool) {
        let created = metadata
            .created()
            .unwrap_or(SystemTime::now())
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros();

        let modified = if let Ok(modified) = metadata.modified() {
            modified
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_micros()
        } else {
            created
        };
        let directory = metadata.file_type().is_dir();
        let size = metadata.len();
        (created, modified, size, directory)
    }
    pub async fn create_from_entry<S: Into<String>>(
        relative_path: S,
        entry: &DirEntry,
    ) -> Result<Self, StorageError> {
        let metadata = entry.metadata()?;
        let (created, modified, file_size, directory) = Self::meta_data(metadata);

        let mime = mime_guess::from_path(entry.path())
            .first_or_octet_stream()
            .to_string();
        let name = entry.file_name().to_str().unwrap().to_string();
        let file = StorageFile {
            name,
            full_path: relative_path.into(),
            mime,
            directory,
            file_size,
            modified,
            created,
        };
        Ok(file)
    }
    pub async fn create<S: Into<String>>(
        relative_path: S,
        file_location: &PathBuf,
    ) -> Result<Self, StorageError> {
        let file = OpenOptions::new().read(true).open(&file_location).await?;
        let metadata = file.metadata().await?;
        let (created, modified, file_size, directory) = Self::meta_data(metadata);

        let mime = mime_guess::from_path(file_location)
            .first_or_octet_stream()
            .to_string();
        let name = file_location
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let file = StorageFile {
            name,
            full_path: relative_path.into(),
            mime,
            directory,
            file_size,
            modified,
            created,
        };
        Ok(file)
    }
}
pub struct StorageDirectoryResponse {
    pub files: Vec<StorageFile>,
    pub directory: StorageFile,
}
/// The Types of Storage File web responses it can have
pub enum StorageFileResponse {
    /// A Location to a Local File
    File(PathBuf),
    /// A list of StorageFiles. Usually Responded when a directory
    /// First Value is the Information About the directory
    List(StorageDirectoryResponse),
    /// Not Found
    NotFound,
}
/// A Simple trait for handling file List responses
pub trait FileListResponder {
    /// Parses the Accept the header(badly) to decide the Response Type
    fn listing(self, request: &HttpRequest) -> Result<HttpResponse, actix_web::Error>
    where
        Self: std::marker::Sized,
    {
        if request.method() == Method::HEAD {}
        return if let Some(accept) = request.headers().get(ACCEPT) {
            let x = accept.to_str().map_err(APIError::bad_request)?;
            if x.contains("application/json") {
                self.json_listing(request)
            } else if x.contains("text/html") {
                self.html_listing(request)
            } else {
                Err(Self::invalid_accept_type().into())
            }
        } else {
            self.html_listing(request)
        };
    }
    /// Converts Self into a JSOn based Http Response
    fn json_listing(self, request: &HttpRequest) -> Result<HttpResponse, actix_web::Error>
    where
        Self: std::marker::Sized;
    /// Converts Self Into a HTML based HTTP Response
    fn html_listing(self, request: &HttpRequest) -> Result<HttpResponse, actix_web::Error>
    where
        Self: std::marker::Sized,
    {
        Err(Self::invalid_accept_type().into())
    }
    /// For Internal Use
    /// Invalid Data Type
    fn invalid_accept_type() -> APIError {
        APIError::from(("Invalid Accept Header", StatusCode::BAD_REQUEST))
    }
}
impl FileListResponder for StorageDirectoryResponse {
    fn json_listing(self, request: &HttpRequest) -> Result<HttpResponse, actix_web::Error>
    where
        Self: std::marker::Sized,
    {
        Ok(HttpResponse::Ok().json(self.files).respond_to(request))
    }
}

impl Responder for StorageFileResponse {
    type Body = BoxBody;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        match self {
            StorageFileResponse::File(file) => match NamedFile::open(file) {
                Ok(success) => success.respond_to(req),
                Err(error) => {
                    error!("Unable to Respond with File {}", error);
                    HttpResponse::from_error(error).respond_to(req)
                }
            },
            StorageFileResponse::List(list) => match list.listing(req) {
                Ok(response) => response,
                Err(response) => response.error_response(),
            },
            StorageFileResponse::NotFound => APIError::not_found().error_response(),
        }
    }
}
