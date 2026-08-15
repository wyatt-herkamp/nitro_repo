//! Verification of the checksums npm sends with a publish.
//!
//! A publish body carries `dist.integrity` (a Subresource Integrity string, normally
//! `sha512-<base64>`) and the older `dist.shasum` (hex sha1). Both were parsed, stored and then
//! never checked against the tarball actually uploaded — so a corrupted or substituted attachment
//! was accepted and served, and every later `npm install` failed its own integrity check against
//! a value the registry had recorded but never verified.
use base64::{Engine, engine::general_purpose::STANDARD};
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha384, Sha512};
use tracing::instrument;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum IntegrityError {
    #[error("Malformed integrity string `{0}`: expected `<algorithm>-<base64 digest>`")]
    Malformed(String),
    #[error("Unsupported integrity algorithm `{0}`")]
    UnsupportedAlgorithm(String),
    #[error("Integrity mismatch: the uploaded tarball does not match `{algorithm}` in `integrity`")]
    IntegrityMismatch { algorithm: String },
    #[error("Checksum mismatch: the uploaded tarball does not match `shasum`")]
    ShasumMismatch,
}

fn digest(algorithm: &str, data: &[u8]) -> Option<Vec<u8>> {
    let digest = match algorithm {
        "sha512" => Sha512::digest(data).to_vec(),
        "sha384" => Sha384::digest(data).to_vec(),
        "sha256" => Sha256::digest(data).to_vec(),
        "sha1" => Sha1::digest(data).to_vec(),
        _ => return None,
    };
    Some(digest)
}

/// Checks `dist.integrity` against the uploaded bytes.
///
/// npm allows several space-separated entries. Any one of them matching is enough — that is what
/// `ssri` does — but an entry naming an algorithm we cannot compute is not treated as a match.
#[instrument(skip(data))]
pub fn verify_integrity(integrity: &str, data: &[u8]) -> Result<(), IntegrityError> {
    let mut checked_any = false;
    let mut last_algorithm = String::new();
    for entry in integrity.split_whitespace() {
        // An entry may carry `?options` after the digest, which are not part of it.
        let entry = entry.split('?').next().unwrap_or(entry);
        let Some((algorithm, expected)) = entry.split_once('-') else {
            return Err(IntegrityError::Malformed(entry.to_owned()));
        };
        let Some(actual) = digest(algorithm, data) else {
            continue;
        };
        let Ok(expected) = STANDARD.decode(expected) else {
            return Err(IntegrityError::Malformed(entry.to_owned()));
        };
        if actual == expected {
            return Ok(());
        }
        checked_any = true;
        last_algorithm = algorithm.to_owned();
    }
    if checked_any {
        return Err(IntegrityError::IntegrityMismatch {
            algorithm: last_algorithm,
        });
    }
    // Nothing was computable. Refusing beats storing a tarball whose only stated checksum we have
    // no way to confirm.
    Err(IntegrityError::UnsupportedAlgorithm(integrity.to_owned()))
}

/// Checks the legacy `dist.shasum`, a hex sha1 of the tarball.
#[instrument(skip(data))]
pub fn verify_shasum(shasum: &str, data: &[u8]) -> Result<(), IntegrityError> {
    let actual = Sha1::digest(data);
    let actual = actual
        .iter()
        .fold(String::with_capacity(40), |mut hex, byte| {
            use std::fmt::Write;
            let _ = write!(hex, "{byte:02x}");
            hex
        });
    if actual.eq_ignore_ascii_case(shasum) {
        Ok(())
    } else {
        Err(IntegrityError::ShasumMismatch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DATA: &[u8] = b"nitro repo npm attachment";

    fn integrity_for(algorithm: &str) -> String {
        format!(
            "{algorithm}-{}",
            STANDARD.encode(digest(algorithm, DATA).unwrap())
        )
    }
    fn shasum_for(data: &[u8]) -> String {
        Sha1::digest(data)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    }

    #[test]
    fn accepts_a_matching_digest() {
        for algorithm in ["sha512", "sha384", "sha256", "sha1"] {
            verify_integrity(&integrity_for(algorithm), DATA)
                .unwrap_or_else(|err| panic!("{algorithm} rejected a matching tarball: {err}"));
        }
        verify_shasum(&shasum_for(DATA), DATA).unwrap();
    }

    #[test]
    fn rejects_a_tampered_tarball() {
        let integrity = integrity_for("sha512");
        assert_eq!(
            verify_integrity(&integrity, b"something else"),
            Err(IntegrityError::IntegrityMismatch {
                algorithm: "sha512".to_owned()
            })
        );
        assert_eq!(
            verify_shasum(&shasum_for(DATA), b"something else"),
            Err(IntegrityError::ShasumMismatch)
        );
    }

    /// npm may send several; matching any one is enough.
    #[test]
    fn accepts_any_entry_in_a_multi_entry_string() {
        let integrity = format!(
            "sha256-{} {}",
            STANDARD.encode([0u8; 32]),
            integrity_for("sha512")
        );
        verify_integrity(&integrity, DATA).unwrap();
    }

    /// An algorithm we cannot compute must not read as a pass.
    #[test]
    fn refuses_when_nothing_is_verifiable() {
        assert!(matches!(
            verify_integrity("md5-abcd", DATA),
            Err(IntegrityError::UnsupportedAlgorithm(_))
        ));
        assert!(matches!(
            verify_integrity("not-an-integrity-string", DATA),
            Err(IntegrityError::Malformed(_)) | Err(IntegrityError::UnsupportedAlgorithm(_))
        ));
    }
}
