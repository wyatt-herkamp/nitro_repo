//! Content digests — `sha256:<hex>`.
//!
//! A digest is both an identifier and a path component, so it is parsed into a type rather than
//! passed around as a string. Anything that reaches storage has been through
//! [`Digest::parse`], which is what keeps a crafted `..` or a stray `/` out of a blob path.

use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256, Sha512};

/// The algorithms this registry will accept in a digest.
///
/// Deliberately not open-ended: an unknown algorithm cannot be verified, and accepting one would
/// mean storing content under a digest nobody can check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Algorithm {
    Sha256,
    Sha512,
}

impl Algorithm {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
            Self::Sha512 => "sha512",
        }
    }

    /// How many hex characters a digest of this algorithm has.
    fn hex_length(self) -> usize {
        match self {
            Self::Sha256 => 64,
            Self::Sha512 => 128,
        }
    }

    pub fn digest(self, data: &[u8]) -> String {
        match self {
            Self::Sha256 => to_hex(&Sha256::digest(data)),
            Self::Sha512 => to_hex(&Sha512::digest(data)),
        }
    }
}

impl FromStr for Algorithm {
    type Err = InvalidDigest;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "sha256" => Ok(Self::Sha256),
            "sha512" => Ok(Self::Sha512),
            other => Err(InvalidDigest::UnsupportedAlgorithm(other.to_owned())),
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum InvalidDigest {
    #[error("a digest must be `algorithm:hex`, got `{0}`")]
    Malformed(String),
    #[error("`{0}` is not a digest algorithm this registry supports")]
    UnsupportedAlgorithm(String),
    #[error("`{0}` is not the right length or is not lowercase hex")]
    BadHex(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Digest {
    algorithm: Algorithm,
    hex: String,
}

impl Digest {
    pub fn parse(value: &str) -> Result<Self, InvalidDigest> {
        let Some((algorithm, hex)) = value.split_once(':') else {
            return Err(InvalidDigest::Malformed(value.to_owned()));
        };
        let algorithm: Algorithm = algorithm.parse()?;
        // Lowercase only, and exactly the right length. An uppercase digest would be a different
        // string for the same content, which would store one blob twice and make `HEAD` miss.
        if hex.len() != algorithm.hex_length()
            || !hex
                .chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
        {
            return Err(InvalidDigest::BadHex(hex.to_owned()));
        }
        Ok(Self {
            algorithm,
            hex: hex.to_owned(),
        })
    }

    /// The digest of some content, under the given algorithm.
    pub fn of(algorithm: Algorithm, data: &[u8]) -> Self {
        Self {
            algorithm,
            hex: algorithm.digest(data),
        }
    }

    /// The sha256 digest of some content — what a registry uses unless told otherwise.
    pub fn sha256_of(data: &[u8]) -> Self {
        Self::of(Algorithm::Sha256, data)
    }

    pub fn algorithm(&self) -> Algorithm {
        self.algorithm
    }

    pub fn hex(&self) -> &str {
        &self.hex
    }

    /// Whether this digest is the digest of `data`.
    pub fn matches(&self, data: &[u8]) -> bool {
        self.algorithm.digest(data) == self.hex
    }

    /// The two path components content under this digest is stored at: `{algorithm}/{hex}`.
    ///
    /// Both are constrained by parsing — an algorithm from a fixed set, a hex string of a fixed
    /// length — so neither can be a path traversal.
    pub fn path_components(&self) -> (&'static str, &str) {
        (self.algorithm.as_str(), &self.hex)
    }
}

impl Display for Digest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.algorithm.as_str(), self.hex)
    }
}

impl Serialize for Digest {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        Digest::parse(&raw).map_err(serde::de::Error::custom)
    }
}

fn to_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::new(), |mut hex, byte| {
        let _ = write!(hex, "{byte:02x}");
        hex
    })
}

#[cfg(test)]
mod tests {
    use super::{Algorithm, Digest, InvalidDigest};

    /// The sha256 of the empty string, which is what an empty layer's digest actually is.
    const EMPTY_SHA256: &str =
        "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    #[test]
    fn a_well_formed_digest_parses_and_round_trips() {
        let digest = Digest::parse(EMPTY_SHA256).unwrap();
        assert_eq!(digest.algorithm(), Algorithm::Sha256);
        assert_eq!(digest.to_string(), EMPTY_SHA256);
        assert_eq!(
            digest.path_components(),
            (
                "sha256",
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            )
        );
    }

    #[test]
    fn the_digest_of_content_matches_it() {
        let digest = Digest::sha256_of(b"");
        assert_eq!(digest.to_string(), EMPTY_SHA256);
        assert!(digest.matches(b""));
        assert!(!digest.matches(b"something else"));
    }

    #[test]
    fn a_digest_cannot_carry_a_path_traversal() {
        for bad in [
            "sha256:../../etc/passwd",
            "sha256:..",
            "../sha256:abc",
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b85/",
        ] {
            assert!(Digest::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn a_malformed_digest_is_refused() {
        assert_eq!(
            Digest::parse("nocolon"),
            Err(InvalidDigest::Malformed("nocolon".to_owned()))
        );
        assert_eq!(
            Digest::parse("md5:d41d8cd98f00b204e9800998ecf8427e"),
            Err(InvalidDigest::UnsupportedAlgorithm("md5".to_owned()))
        );
        // Right shape, wrong length.
        assert!(Digest::parse("sha256:abc").is_err());
        // Uppercase hex is a different string for the same bytes.
        assert!(Digest::parse(&EMPTY_SHA256.to_uppercase()).is_err());
    }

    #[test]
    fn sha512_is_accepted_at_its_own_length() {
        let digest = Digest::of(Algorithm::Sha512, b"");
        assert_eq!(digest.hex().len(), 128);
        assert_eq!(Digest::parse(&digest.to_string()).unwrap(), digest);
    }
}
