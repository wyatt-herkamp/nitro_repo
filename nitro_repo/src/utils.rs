use std::borrow::Cow;
use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;

use actix_web::http::header::HeaderMap;
use rust_embed::RustEmbed;
use tracing::error;

use crate::error::internal_error::InternalError;

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

pub fn get_accept(header_map: &HeaderMap) -> Result<Option<String>, InternalError> {
    let Some(header_value) = header_map.get("accept") else {
        return Ok(None);
    };

    let accept = header_value
        .to_str()
        .map(|x| x.to_string())
        .inspect_err(|err| {
            error!("Failed to convert accept header to string: {}", err);
        })
        .ok();
    Ok(accept)
}

pub mod password {
    use argon2::{
        password_hash::{Error, SaltString},
        Argon2, PasswordHasher, PasswordVerifier,
    };
    use rand::rngs::OsRng;
    use tracing::error;

    pub fn encrypt_password(password: &str) -> Option<String> {
        let salt = SaltString::generate(&mut OsRng);

        let argon2 = Argon2::default();

        let password = argon2.hash_password(password.as_ref(), &salt);
        match password {
            Ok(ok) => Some(ok.to_string()),
            Err(err) => {
                error!("Failed to hash password: {}", err);
                None
            }
        }
    }
}
