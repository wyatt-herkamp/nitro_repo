//! Turning a `/v2/...` path into a registry route.
//!
//! The image name is a variable-length prefix (`library/alpine`, `team/project/service`), so unlike
//! npm's leading-segment dispatch this matches from the *end* of the path: the last one to three
//! segments name the operation, and everything before them is the image.

use nr_core::storage::StoragePath;
use uuid::Uuid;

use super::digest::Digest;
use crate::repository::docker::{DockerError, errors::ErrorCode};

/// What a manifest request addresses: a mutable tag, or an immutable digest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reference {
    Tag(String),
    Digest(Digest),
}

impl Reference {
    /// A reference is a digest if it parses as one, and a tag otherwise.
    ///
    /// A tag can never contain `:`, so there is no ambiguity — but the parse has to come first,
    /// because a malformed digest must be an error rather than a tag with a colon in it.
    pub fn parse(value: &str) -> Result<Self, DockerError> {
        if value.contains(':') {
            return Digest::parse(value)
                .map(Reference::Digest)
                .map_err(|error| DockerError::InvalidDigest(error.to_string()));
        }
        if !is_valid_tag(value) {
            return Err(DockerError::Coded {
                code: ErrorCode::ManifestInvalid,
                message: format!("`{value}` is not a valid tag"),
            });
        }
        Ok(Reference::Tag(value.to_owned()))
    }
}

impl std::fmt::Display for Reference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reference::Tag(tag) => f.write_str(tag),
            Reference::Digest(digest) => write!(f, "{digest}"),
        }
    }
}

/// The distribution spec's tag grammar: `[a-zA-Z0-9_][a-zA-Z0-9._-]{0,127}`.
pub fn is_valid_tag(tag: &str) -> bool {
    !tag.is_empty()
        && tag.len() <= 128
        && tag.starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_')
        && tag
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

/// The distribution spec's name grammar: slash-separated components of
/// `[a-z0-9]+(?:(?:\.|_|__|-+)[a-z0-9]+)*`.
///
/// Lowercase only — Docker itself refuses to push an uppercase name, and accepting one here would
/// create an image nobody could pull back.
pub fn is_valid_image_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 255 {
        return false;
    }
    name.split('/').all(is_valid_name_component)
}

fn is_valid_name_component(component: &str) -> bool {
    let bytes = component.as_bytes();
    let alphanumeric = |b: u8| b.is_ascii_lowercase() || b.is_ascii_digit();

    // Must start and end with an alphanumeric; separators only ever sit between them. A
    // single-character component is both the first and the last byte, which is why this is a
    // first/last check rather than a slice of the interior — `bytes[1..len - 1]` panics at len 1.
    let (Some(&first), Some(&last)) = (bytes.first(), bytes.last()) else {
        return false;
    };
    if !alphanumeric(first) || !alphanumeric(last) {
        return false;
    }

    // The grammar allows `.`, `_`, `__` and runs of `-` between alphanumerics. A doubled `.` is
    // not in it, and would also read as a path component this registry does not want to store.
    let mut previous = 0u8;
    for &byte in bytes {
        if !alphanumeric(byte) && !matches!(byte, b'.' | b'_' | b'-') {
            return false;
        }
        if byte == b'.' && previous == b'.' {
            return false;
        }
        previous = byte;
    }
    true
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DockerPath {
    /// `/v2/` — the version check.
    Base,
    /// `/v2/_catalog`
    Catalog,
    /// `/v2/{name}/tags/list`
    TagsList { name: String },
    /// `/v2/{name}/manifests/{reference}`
    Manifest { name: String, reference: Reference },
    /// `/v2/{name}/blobs/{digest}`
    Blob { name: String, digest: Digest },
    /// `/v2/{name}/blobs/uploads/`
    UploadStart { name: String },
    /// `/v2/{name}/blobs/uploads/{uuid}`
    UploadChunk { name: String, uuid: Uuid },
    /// `/v2/{name}/referrers/{digest}`
    Referrers { name: String, digest: Digest },
}

impl DockerPath {
    /// Parses the path *after* `/v2`.
    pub fn parse(path: &StoragePath) -> Result<Self, DockerError> {
        let components: Vec<String> =
            Vec::<nr_core::storage::StoragePathComponent>::from(path.clone())
                .into_iter()
                .map(String::from)
                .collect();
        let parts: Vec<&str> = components.iter().map(String::as_str).collect();

        if parts.is_empty() {
            return Ok(DockerPath::Base);
        }
        if parts == ["_catalog"] {
            return Ok(DockerPath::Catalog);
        }

        // `blobs/uploads` is the only three-segment tail, and it is also a legal two-segment one
        // (`.../blobs/uploads/` starts an upload). Checked before the generic tails so an upload id
        // is never mistaken for a digest.
        if parts.len() >= 3 {
            let tail = &parts[parts.len() - 2..];
            if tail == ["blobs", "uploads"] {
                let name = join_name(&parts[..parts.len() - 2])?;
                return Ok(DockerPath::UploadStart { name });
            }
        }
        if parts.len() >= 4 {
            let tail = &parts[parts.len() - 3..parts.len() - 1];
            if tail == ["blobs", "uploads"] {
                let raw = parts[parts.len() - 1];
                let uuid = Uuid::parse_str(raw).map_err(|_| DockerError::Coded {
                    code: ErrorCode::BlobUploadInvalid,
                    message: format!("`{raw}` is not an upload id this registry issued"),
                })?;
                let name = join_name(&parts[..parts.len() - 3])?;
                return Ok(DockerPath::UploadChunk { name, uuid });
            }
        }

        if parts.len() >= 3 {
            let kind = parts[parts.len() - 2];
            let last = parts[parts.len() - 1];
            let name_parts = &parts[..parts.len() - 2];
            match kind {
                "tags" if last == "list" => {
                    return Ok(DockerPath::TagsList {
                        name: join_name(name_parts)?,
                    });
                }
                "manifests" => {
                    return Ok(DockerPath::Manifest {
                        name: join_name(name_parts)?,
                        reference: Reference::parse(last)?,
                    });
                }
                "blobs" => {
                    let digest = Digest::parse(last)
                        .map_err(|error| DockerError::InvalidDigest(error.to_string()))?;
                    return Ok(DockerPath::Blob {
                        name: join_name(name_parts)?,
                        digest,
                    });
                }
                "referrers" => {
                    let digest = Digest::parse(last)
                        .map_err(|error| DockerError::InvalidDigest(error.to_string()))?;
                    return Ok(DockerPath::Referrers {
                        name: join_name(name_parts)?,
                        digest,
                    });
                }
                _ => {}
            }
        }

        Err(DockerError::Coded {
            code: ErrorCode::NameUnknown,
            message: format!(
                "`/v2/{}` is not a route this registry serves",
                parts.join("/")
            ),
        })
    }
}

fn join_name(parts: &[&str]) -> Result<String, DockerError> {
    if parts.is_empty() {
        return Err(DockerError::Coded {
            code: ErrorCode::NameInvalid,
            message: "the request named no image".to_owned(),
        });
    }
    let name = parts.join("/");
    if !is_valid_image_name(&name) {
        return Err(DockerError::Coded {
            code: ErrorCode::NameInvalid,
            message: format!("`{name}` is not a valid image name"),
        });
    }
    Ok(name)
}

#[cfg(test)]
mod tests {
    use nr_core::storage::StoragePath;

    use super::{DockerPath, Reference, is_valid_image_name, is_valid_tag};
    use crate::repository::docker::types::digest::Digest;

    const EMPTY_SHA256: &str =
        "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

    fn parse(path: &str) -> Result<DockerPath, crate::repository::docker::DockerError> {
        DockerPath::parse(&StoragePath::parse(path).unwrap())
    }

    #[test]
    fn the_base_and_catalog_routes_parse() {
        assert_eq!(parse("").unwrap(), DockerPath::Base);
        assert_eq!(parse("/").unwrap(), DockerPath::Base);
        assert_eq!(parse("_catalog").unwrap(), DockerPath::Catalog);
    }

    #[test]
    fn a_multi_segment_image_name_is_kept_whole() {
        assert_eq!(
            parse("team/project/service/manifests/latest").unwrap(),
            DockerPath::Manifest {
                name: "team/project/service".to_owned(),
                reference: Reference::Tag("latest".to_owned()),
            }
        );
        assert_eq!(
            parse("library/alpine/tags/list").unwrap(),
            DockerPath::TagsList {
                name: "library/alpine".to_owned()
            }
        );
    }

    #[test]
    fn a_manifest_can_be_addressed_by_tag_or_by_digest() {
        assert_eq!(
            parse("alpine/manifests/3.19").unwrap(),
            DockerPath::Manifest {
                name: "alpine".to_owned(),
                reference: Reference::Tag("3.19".to_owned()),
            }
        );
        assert_eq!(
            parse(&format!("alpine/manifests/{EMPTY_SHA256}")).unwrap(),
            DockerPath::Manifest {
                name: "alpine".to_owned(),
                reference: Reference::Digest(Digest::parse(EMPTY_SHA256).unwrap()),
            }
        );
    }

    #[test]
    fn the_blob_and_upload_routes_parse() {
        assert_eq!(
            parse(&format!("alpine/blobs/{EMPTY_SHA256}")).unwrap(),
            DockerPath::Blob {
                name: "alpine".to_owned(),
                digest: Digest::parse(EMPTY_SHA256).unwrap(),
            }
        );
        assert_eq!(
            parse("alpine/blobs/uploads/").unwrap(),
            DockerPath::UploadStart {
                name: "alpine".to_owned()
            }
        );

        let uuid = uuid::Uuid::new_v4();
        assert_eq!(
            parse(&format!("alpine/blobs/uploads/{uuid}")).unwrap(),
            DockerPath::UploadChunk {
                name: "alpine".to_owned(),
                uuid,
            }
        );
    }

    /// An image literally called `blobs` or `manifests` must not be mistaken for the tail that
    /// names the operation.
    #[test]
    fn an_image_named_like_a_route_segment_still_resolves() {
        assert_eq!(
            parse("blobs/manifests/latest").unwrap(),
            DockerPath::Manifest {
                name: "blobs".to_owned(),
                reference: Reference::Tag("latest".to_owned()),
            }
        );
        assert_eq!(
            parse("manifests/tags/list").unwrap(),
            DockerPath::TagsList {
                name: "manifests".to_owned()
            }
        );
    }

    #[test]
    fn the_referrers_route_parses() {
        assert_eq!(
            parse(&format!("alpine/referrers/{EMPTY_SHA256}")).unwrap(),
            DockerPath::Referrers {
                name: "alpine".to_owned(),
                digest: Digest::parse(EMPTY_SHA256).unwrap(),
            }
        );
    }

    #[test]
    fn a_route_with_no_image_name_is_refused() {
        for path in ["manifests/latest", "tags/list", "blobs/uploads/"] {
            assert!(parse(path).is_err(), "{path}");
        }
    }

    #[test]
    fn an_unknown_route_is_refused() {
        for path in ["alpine", "alpine/nonsense/latest", "alpine/manifests"] {
            assert!(parse(path).is_err(), "{path}");
        }
    }

    #[test]
    fn a_bad_upload_id_is_refused() {
        assert!(parse("alpine/blobs/uploads/not-a-uuid").is_err());
    }

    #[test]
    fn image_names_follow_the_distribution_grammar() {
        for good in [
            "alpine",
            "library/alpine",
            "a/b/c",
            "my-app",
            "my_app",
            "app.v2",
            "n1",
        ] {
            assert!(is_valid_image_name(good), "{good}");
        }
        for bad in [
            "",
            "Alpine",
            "UPPER/case",
            "-leading",
            "trailing-",
            "a//b",
            "a b",
            "a/",
            "/a",
            "a..b",
            ".a",
            "a.",
            "_a",
            "a_",
            "a/../b",
        ] {
            assert!(!is_valid_image_name(bad), "{bad}");
        }
    }

    #[test]
    fn tags_follow_the_distribution_grammar() {
        for good in ["latest", "v1.0.0", "1", "_underscore", "a-b.c_d"] {
            assert!(is_valid_tag(good), "{good}");
        }
        for bad in [
            "",
            ".leading-dot",
            "-leading-dash",
            "has space",
            "has/slash",
            &"x".repeat(129),
        ] {
            assert!(!is_valid_tag(bad), "{bad}");
        }
    }
}
