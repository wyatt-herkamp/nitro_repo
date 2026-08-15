//! Maven checksum files.
//!
//! Maven uploads `.sha1` and `.md5` beside every artifact and expects to be able to fetch them
//! back. Both halves were missing: an uploaded checksum was stored as an opaque blob and never
//! compared against the artifact, and a `GET` for `foo.jar.sha1` was a 404 unless a client had
//! happened to upload one. Hashes were already being computed at the storage layer — they just
//! only ever fed the ETag.
//!
//! Checksums are written and read as lowercase hex. The storage layer keeps them base64-encoded,
//! so this converts rather than recomputing where it can.
use base64::{Engine, engine::general_purpose::STANDARD};
use nr_core::storage::{FileHashes, StoragePath};
use sha2::{Digest, Sha512};

/// The checksum algorithms Maven asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumKind {
    Md5,
    Sha1,
    Sha256,
    Sha512,
}

impl ChecksumKind {
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension {
            "md5" => Some(Self::Md5),
            "sha1" => Some(Self::Sha1),
            "sha256" => Some(Self::Sha256),
            "sha512" => Some(Self::Sha512),
            _ => None,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Md5 => "md5",
            Self::Sha1 => "sha1",
            Self::Sha256 => "sha256",
            Self::Sha512 => "sha512",
        }
    }

    /// Reads the checksum out of what the storage layer already recorded.
    ///
    /// sha512 is not among them, so it is absent here and has to be computed from the file. The
    /// rest never need the artifact to be read back.
    pub fn from_stored(&self, hashes: &FileHashes) -> Option<String> {
        let base64 = match self {
            Self::Md5 => hashes.md5.as_ref(),
            Self::Sha1 => hashes.sha1.as_ref(),
            Self::Sha256 => hashes.sha2_256.as_ref(),
            Self::Sha512 => None,
        }?;
        let bytes = STANDARD.decode(base64).ok()?;
        Some(to_hex(&bytes))
    }

    /// Computes the checksum from the artifact's bytes.
    pub fn compute(&self, data: &[u8]) -> String {
        let digest = match self {
            Self::Md5 => md5_digest(data),
            Self::Sha1 => {
                use sha1::Sha1;
                Sha1::digest(data).to_vec()
            }
            Self::Sha256 => {
                use sha2::Sha256;
                Sha256::digest(data).to_vec()
            }
            Self::Sha512 => Sha512::digest(data).to_vec(),
        };
        to_hex(&digest)
    }
}

fn md5_digest(data: &[u8]) -> Vec<u8> {
    // `md-5` implements the same `Digest` trait as the sha crates.
    use md5::Md5;
    Md5::digest(data).to_vec()
}

fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::new(), |mut hex, byte| {
        let _ = write!(hex, "{byte:02x}");
        hex
    })
}

/// Splits a checksum request into the artifact it describes and the algorithm.
///
/// Returns `None` for anything that is not a checksum file.
pub fn split_checksum_path(path: &StoragePath) -> Option<(StoragePath, ChecksumKind)> {
    let as_string = path.to_string();
    let (artifact, extension) = as_string.rsplit_once('.')?;
    let kind = ChecksumKind::from_extension(extension)?;
    Some((StoragePath::from(artifact), kind))
}

/// Reads the checksum value out of a checksum file's contents.
///
/// Maven writes a bare hex digest, but the GNU tools write `{digest}  {filename}` and some clients
/// add a trailing newline, so the first whitespace-delimited token is what counts.
pub fn parse_checksum_body(body: &str) -> Option<String> {
    let value = body.split_whitespace().next()?;
    if value.is_empty() || !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    Some(value.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_checksum_paths() {
        let (artifact, kind) = split_checksum_path(&StoragePath::from(
            "dev/kingtux/tms/1.0.0/tms-1.0.0.jar.sha1",
        ))
        .expect("not recognised");
        assert_eq!(artifact.to_string(), "dev/kingtux/tms/1.0.0/tms-1.0.0.jar");
        assert_eq!(kind, ChecksumKind::Sha1);

        assert!(
            split_checksum_path(&StoragePath::from("dev/kingtux/tms/1.0.0/tms-1.0.0.jar"))
                .is_none()
        );
        // A signature is not a checksum.
        assert!(
            split_checksum_path(&StoragePath::from(
                "dev/kingtux/tms/1.0.0/tms-1.0.0.jar.asc"
            ))
            .is_none()
        );
    }

    #[test]
    fn reads_checksum_bodies() {
        let digest = "356a192b7913b04c54574d18c28d46e6395428ab";
        assert_eq!(parse_checksum_body(digest).as_deref(), Some(digest));
        assert_eq!(
            parse_checksum_body(&format!("{digest}\n")).as_deref(),
            Some(digest)
        );
        // GNU coreutils style.
        assert_eq!(
            parse_checksum_body(&format!("{digest}  tms-1.0.0.jar")).as_deref(),
            Some(digest)
        );
        // Maven has been seen to upload uppercase.
        assert_eq!(
            parse_checksum_body(&digest.to_uppercase()).as_deref(),
            Some(digest)
        );
        assert_eq!(parse_checksum_body(""), None);
        assert_eq!(parse_checksum_body("not a checksum"), None);
    }

    #[test]
    fn computed_and_stored_checksums_agree() {
        let data = b"a maven artifact";
        // What the storage layer records, in the base64 form it uses.
        let hashes = FileHashes {
            md5: Some(STANDARD.encode(md5_digest(data))),
            sha1: Some(STANDARD.encode(sha1::Sha1::digest(data))),
            sha2_256: Some(STANDARD.encode(sha2::Sha256::digest(data))),
            sha3_256: None,
        };
        for kind in [ChecksumKind::Md5, ChecksumKind::Sha1, ChecksumKind::Sha256] {
            assert_eq!(
                kind.from_stored(&hashes).as_deref(),
                Some(kind.compute(data).as_str()),
                "{kind:?} disagreed between the stored value and a fresh digest"
            );
        }
        // sha512 is not stored, so it can only be computed.
        assert_eq!(ChecksumKind::Sha512.from_stored(&hashes), None);
        assert_eq!(ChecksumKind::Sha512.compute(data).len(), 128);
    }

    #[test]
    fn checksums_are_lowercase_hex() {
        let sha1 = ChecksumKind::Sha1.compute(b"1");
        assert_eq!(sha1, "356a192b7913b04c54574d18c28d46e6395428ab");
    }
}
