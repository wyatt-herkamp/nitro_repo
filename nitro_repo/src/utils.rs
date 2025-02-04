use std::borrow::Cow;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;

use digestible::{Digester, Digestible, IntoBase64, byteorder::NativeEndian};
use http::HeaderValue;
use rust_embed::RustEmbed;
use sha2::Digest;
use tracing::error;
pub mod response_builder;
pub mod responses;
use crate::error::{InternalError, ResponseBuildError};
pub const JSON_MEDIA_TYPE: HeaderValue = HeaderValue::from_static("application/json");
pub const TEXT_MEDIA_TYPE: HeaderValue = HeaderValue::from_static("text/plain");
#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/resources"]
pub struct Resources;

impl Resources {
    ///  Gets the file from the resources file if it exists or defaults to the embedded file.
    ///
    /// # Arguments
    ///
    /// * `file`: Relative path to the file.
    ///
    /// returns: Result<Cow<[u8]>, InternalError>
    /// Errors are returned if the IO operation fails.
    /// # Panics
    /// If the embedded resource is not found.
    /// This should never happen.
    /// If it does, it is a bug.
    /// Please report it.
    pub fn file_get(file: &str) -> Result<Cow<'_, [u8]>, InternalError> {
        let buf = Path::new("resources").join(file);
        if buf.exists() {
            let mut file = match OpenOptions::new().read(true).open(buf) {
                Ok(ok) => ok,
                Err(err) => {
                    error!("Unable to open the {file:?}: {}", err);
                    return Err(InternalError::from(err));
                }
            };
            let mut buffer = Vec::with_capacity(file.metadata()?.len() as usize);
            file.read_to_end(&mut buffer)?;
            Ok(Cow::Owned(buffer))
        } else {
            Ok(Resources::get(file)
                .expect("Embedded Resource was not found")
                .data)
        }
    }
}

pub mod headers;

pub fn generate_etag(data: &impl Digestible) -> Result<HeaderValue, ResponseBuildError> {
    let hasher = sha2::Sha256::new().into_base64();
    let result = hasher.digest::<NativeEndian>(data);

    Ok(HeaderValue::try_from(result)?)
}
