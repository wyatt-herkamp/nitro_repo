//! `PUT /api/v1/crates/new` — the publish request.
//!
//! The body is not JSON. It is a pair of length-prefixed frames:
//!
//! ```text
//! u32 LE  metadata length
//! ..      metadata JSON
//! u32 LE  crate file length
//! ..      the .crate file
//! ```
//!
//! See <https://doc.rust-lang.org/cargo/reference/registry-web-api.html#publish>.

use std::collections::BTreeMap;

use bytes::Bytes;
use serde::{Deserialize, Serialize};

use super::super::CargoRegistryError;

/// Refuses a declared frame length that could not possibly be satisfied before allocating for it.
/// The length is four attacker-controlled bytes; taken at face value it is a request to reserve up
/// to 4 GiB.
fn take_frame(
    body: &Bytes,
    offset: &mut usize,
    what: &'static str,
) -> Result<Bytes, CargoRegistryError> {
    let header_end = offset
        .checked_add(4)
        .ok_or(CargoRegistryError::MalformedPublishBody(what))?;
    if body.len() < header_end {
        return Err(CargoRegistryError::MalformedPublishBody(what));
    }
    let length = u32::from_le_bytes([
        body[*offset],
        body[*offset + 1],
        body[*offset + 2],
        body[*offset + 3],
    ]) as usize;
    let end = header_end
        .checked_add(length)
        .ok_or(CargoRegistryError::MalformedPublishBody(what))?;
    if body.len() < end {
        return Err(CargoRegistryError::MalformedPublishBody(what));
    }
    *offset = end;
    Ok(body.slice(header_end..end))
}

/// Splits a publish body into its metadata and its `.crate` file.
pub fn split_publish_body(body: Bytes) -> Result<(PublishMetadata, Bytes), CargoRegistryError> {
    let mut offset = 0usize;
    let metadata = take_frame(&body, &mut offset, "metadata")?;
    let crate_file = take_frame(&body, &mut offset, "crate file")?;
    let metadata: PublishMetadata = serde_json::from_slice(&metadata)?;
    Ok((metadata, crate_file))
}

/// The metadata frame. Cargo adds fields over time, so unknown ones are ignored rather than
/// refused — a newer toolchain must still be able to publish here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishMetadata {
    pub name: String,
    pub vers: String,
    #[serde(default)]
    pub deps: Vec<PublishDependency>,
    #[serde(default)]
    pub features: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub documentation: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub readme: Option<String>,
    #[serde(default)]
    pub readme_file: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub license_file: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
    #[serde(default)]
    pub links: Option<String>,
    #[serde(default)]
    pub rust_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishDependency {
    pub name: String,
    pub version_req: String,
    #[serde(default)]
    pub features: Vec<String>,
    #[serde(default)]
    pub optional: bool,
    #[serde(default = "default_true")]
    pub default_features: bool,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub registry: Option<String>,
    #[serde(default)]
    pub explicit_name_in_toml: Option<String>,
}

fn default_true() -> bool {
    true
}

/// A crate name Cargo would accept: alphanumeric, `-` and `_`, starting with a letter.
///
/// Enforced here rather than only at the storage layer because the name becomes a path component
/// in both the index and the `.crate` location, and because a name Cargo cannot express is a
/// package nobody could ever depend on.
pub fn is_valid_crate_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && name.starts_with(|c: char| c.is_ascii_alphabetic())
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

#[cfg(test)]
mod tests {
    use bytes::{BufMut, Bytes, BytesMut};

    use super::{is_valid_crate_name, split_publish_body};

    fn body(metadata: &[u8], crate_file: &[u8]) -> Bytes {
        let mut buffer = BytesMut::new();
        buffer.put_u32_le(metadata.len() as u32);
        buffer.put_slice(metadata);
        buffer.put_u32_le(crate_file.len() as u32);
        buffer.put_slice(crate_file);
        buffer.freeze()
    }

    #[test]
    fn a_well_formed_body_splits_into_metadata_and_crate() {
        let metadata = br#"{"name":"example","vers":"1.0.0","deps":[],"features":{}}"#;
        let (parsed, file) = split_publish_body(body(metadata, b"tarball bytes")).unwrap();
        assert_eq!(parsed.name, "example");
        assert_eq!(parsed.vers, "1.0.0");
        assert_eq!(&file[..], b"tarball bytes");
    }

    #[test]
    fn a_length_longer_than_the_body_is_refused() {
        let mut buffer = BytesMut::new();
        buffer.put_u32_le(u32::MAX);
        buffer.put_slice(b"{}");
        assert!(split_publish_body(buffer.freeze()).is_err());
    }

    #[test]
    fn a_truncated_body_is_refused() {
        for truncated in [&b""[..], &b"\x01"[..], &b"\x02\x00\x00\x00{"[..]] {
            assert!(split_publish_body(Bytes::from_static(truncated)).is_err());
        }
    }

    #[test]
    fn a_body_missing_its_crate_frame_is_refused() {
        let metadata = br#"{"name":"example","vers":"1.0.0"}"#;
        let mut buffer = BytesMut::new();
        buffer.put_u32_le(metadata.len() as u32);
        buffer.put_slice(metadata);
        assert!(split_publish_body(buffer.freeze()).is_err());
    }

    #[test]
    fn crate_names_follow_cargos_rules() {
        for good in ["serde", "nitro-repo", "nitro_repo", "a", "a1"] {
            assert!(is_valid_crate_name(good), "{good}");
        }
        for bad in [
            "",
            "1crate",
            "-leading",
            "_leading",
            "has space",
            "has/slash",
            "has.dot",
        ] {
            assert!(!is_valid_crate_name(bad), "{bad}");
        }
    }
}
