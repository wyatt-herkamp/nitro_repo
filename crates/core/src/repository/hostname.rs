use nr_macros::{NuType, SerdeViaStr};
use sqlx::prelude::Type;
use thiserror::Error;
use tracing::instrument;

use crate::utils::validations::{self, schema_for_new_type_str, test_validations};

/// The maximum length of a fully qualified domain name, minus the root label.
const MAX_HOSTNAME_LENGTH: usize = 253;
/// The maximum length of a single DNS label.
const MAX_LABEL_LENGTH: usize = 63;

#[derive(Debug, Error)]
pub enum InvalidHostname {
    #[error("Hostname is empty")]
    Empty,
    #[error("Hostname is too long, must be at most 253 characters, got {0}")]
    TooLong(usize),
    #[error("Hostname label `{0}` is empty or longer than 63 characters")]
    InvalidLabel(String),
    #[error(
        "Hostname contains invalid character `{0}`. Hostnames can only contain letters, numbers, `-`, and `.`"
    )]
    InvalidCharacter(char),
    #[error("Hostname label `{0}` may not start or end with `-`")]
    LabelBoundaryHyphen(String),
    #[error("A hostname may not contain a scheme, port, path or userinfo")]
    NotBareHostname,
}

/// A bare hostname, as it appears in a request's `Host` header once the port is stripped.
#[derive(Debug, Type, Clone, Default, SerdeViaStr, NuType)]
#[sqlx(transparent)]
pub struct Hostname(String);
schema_for_new_type_str!(
    Hostname,
    pattern = r#"^[a-z0-9]([a-z0-9\-]{0,61}[a-z0-9])?(\.[a-z0-9]([a-z0-9\-]{0,61}[a-z0-9])?)*$"#
);
validations::convert_traits_to_new!(Hostname, InvalidHostname);

impl Hostname {
    /// Normalises then validates a hostname.
    ///
    /// Mixed case and a trailing root dot are normalised away rather than rejected: DNS is
    /// case-insensitive, the `hostname` column's collation is case-insensitive, and
    /// `MAVEN.Example.com.` and `maven.example.com` are the same host. Everything the
    /// normalisation cannot fix — a scheme, a port, a path, an underscore — is an error, so a
    /// pasted URL fails loudly instead of being silently mangled into something that never matches.
    #[instrument(name = "Hostname::new")]
    pub fn new(hostname: String) -> Result<Self, InvalidHostname> {
        let hostname = hostname.trim().to_ascii_lowercase();
        // A fully qualified name may end in the root label. It is the same host either way, and
        // the `Host` header never carries it, so drop it rather than store a name that can never
        // be matched.
        let hostname = hostname.strip_suffix('.').unwrap_or(&hostname);

        if hostname.is_empty() {
            return Err(InvalidHostname::Empty);
        }
        if hostname.len() > MAX_HOSTNAME_LENGTH {
            return Err(InvalidHostname::TooLong(hostname.len()));
        }
        // These are the characters you get by pasting a URL rather than a hostname, so they get
        // the message that says so rather than the generic invalid-character one.
        if hostname
            .chars()
            .any(|c| matches!(c, ':' | '/' | '@' | '?' | '#') || c.is_whitespace())
        {
            return Err(InvalidHostname::NotBareHostname);
        }
        for label in hostname.split('.') {
            if label.is_empty() || label.len() > MAX_LABEL_LENGTH {
                return Err(InvalidHostname::InvalidLabel(label.to_owned()));
            }
            if let Some(bad_char) = label
                .chars()
                .find(|c| !c.is_ascii_alphanumeric() && *c != '-')
            {
                return Err(InvalidHostname::InvalidCharacter(bad_char));
            }
            if label.starts_with('-') || label.ends_with('-') {
                return Err(InvalidHostname::LabelBoundaryHyphen(label.to_owned()));
            }
        }
        Ok(Self(hostname.to_owned()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

test_validations! {
    mod hostname_tests for Hostname {
        valid: [
            "maven.example.com",
            "MAVEN.Example.COM",
            "maven.example.com.",
            "example.com",
            "localhost",
            "a-b.example.com",
            "1.2.3.4",
            "  maven.example.com  "
        ],
        invalid: [
            "",
            "   ",
            "maven.example.com:8443",
            "https://maven.example.com",
            "maven.example.com/path",
            "user@maven.example.com",
            "maven..example.com",
            ".example.com",
            "-maven.example.com",
            "maven-.example.com",
            "maven_repo.example.com",
            "maven repo.example.com"
        ]
    }
}

#[cfg(test)]
mod normalization_tests {
    use super::Hostname;

    #[test]
    fn normalizes_case_trailing_dot_and_whitespace() {
        for raw in [
            "MAVEN.Example.COM",
            "maven.example.com.",
            "  MAVEN.example.com. ",
        ] {
            let hostname = Hostname::new(raw.to_owned()).expect(raw);
            assert_eq!(hostname.as_str(), "maven.example.com", "raw: {raw}");
        }
    }

    #[test]
    fn a_label_of_exactly_63_characters_is_allowed() {
        let label = "a".repeat(63);
        assert!(Hostname::new(format!("{label}.example.com")).is_ok());
        assert!(Hostname::new(format!("{label}a.example.com")).is_err());
    }
}
