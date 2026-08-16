//! `maven-metadata.xml`.
//!
//! This did not exist anywhere in the codebase — zero occurrences across `nitro_repo/src` and
//! `crates/`. Maven reads it to resolve version ranges, `LATEST` and `RELEASE`, and, for a
//! snapshot, to find out which timestamped build a bare `1.0.0-SNAPSHOT` actually means. Without
//! it none of those work, and snapshot deploy and resolution do not work at all.
//!
//! Client-uploaded metadata was stored verbatim and never merged, so a second machine deploying
//! the same artifact overwrote the first one's version list. This generates the document from what
//! the repository actually holds instead, which is the only source that stays correct when two
//! clients deploy concurrently.
//!
//! Two documents, at two levels:
//!
//! - `{group}/{artifact}/maven-metadata.xml` — every version of an artifact, built from the
//!   database.
//! - `{group}/{artifact}/{version}-SNAPSHOT/maven-metadata.xml` — the timestamped builds within
//!   one snapshot version, built by listing the directory, since the individual builds are not
//!   rows in the database.
use chrono::NaiveDateTime;
use maven_rs::meta::{
    DeployMetadata, Snapshot, SnapshotMetadata, SnapshotVersion, SnapshotVersioning,
    SnapshotVersions, StableVersioning, StableVersions,
};
use nr_core::{
    database::entities::project::{DBProject, versions::DBProjectVersion},
    repository::project::ReleaseType,
    storage::StoragePath,
};
use tracing::instrument;

use super::MavenError;

pub const METADATA_FILE_NAME: &str = "maven-metadata.xml";

/// What a request for a metadata document resolved to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataRequest {
    /// The version list for an artifact. The path is the artifact directory.
    Artifact(StoragePath),
    /// The timestamped builds inside one snapshot version. The path is the version directory.
    Snapshot(StoragePath),
}

impl MetadataRequest {
    /// Recognises a request for a metadata document, and says which of the two it is.
    ///
    /// The suffix is whatever followed `maven-metadata.xml` — empty for the document itself,
    /// `.sha1` and friends for its checksums, which are generated from the same content so they
    /// cannot disagree with it.
    pub fn parse(path: &StoragePath) -> Option<(Self, &'static str)> {
        let as_string = path.to_string();
        let file_name = as_string.rsplit('/').next()?;
        let suffix = match file_name.strip_prefix(METADATA_FILE_NAME)? {
            "" => "",
            ".sha1" => ".sha1",
            ".md5" => ".md5",
            ".sha256" => ".sha256",
            ".sha512" => ".sha512",
            // `maven-metadata.xml.asc` and anything else is a real stored file, not something to
            // generate.
            _ => return None,
        };
        let directory = path.clone().parent();
        let last = directory.clone().into_iter().next_back()?;
        let request = if is_snapshot_version(last.as_ref()) {
            MetadataRequest::Snapshot(directory)
        } else {
            MetadataRequest::Artifact(directory)
        };
        Some((request, suffix))
    }
}

pub fn is_snapshot_version(version: &str) -> bool {
    version.to_uppercase().ends_with("-SNAPSHOT")
}

/// Builds the artifact-level document from the database.
///
/// `latest` is the most recently published version of any kind; `release` is the most recent
/// non-snapshot. Maven treats them differently — `RELEASE` must never resolve to a snapshot — and
/// conflating them is what makes a repository hand a snapshot to a build that asked for a release.
#[instrument(skip(versions))]
pub fn artifact_metadata(
    project: &DBProject,
    versions: &[DBProjectVersion],
) -> Result<DeployMetadata, MavenError> {
    let (group_id, artifact_id) = split_project_key(&project.key)?;

    // `get_all_versions` returns newest first. Maven writes the version list oldest first, and
    // some clients take the last entry as the newest.
    let mut ordered: Vec<&DBProjectVersion> = versions.iter().collect();
    ordered.reverse();

    let latest = versions.first().map(|version| version.version.clone());
    let release = versions
        .iter()
        .find(|version| !matches!(version.release_type, ReleaseType::Snapshot))
        .map(|version| version.version.clone());

    Ok(DeployMetadata {
        group_id,
        artifact_id,
        versioning: StableVersioning {
            latest,
            release,
            versions: StableVersions {
                version: ordered
                    .into_iter()
                    .map(|version| version.version.clone())
                    .collect(),
            },
            last_updated: Some(project.updated_at.naive_utc()),
        },
    })
}

/// One timestamped build inside a snapshot version directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotBuild {
    /// The version string that replaces `-SNAPSHOT`, e.g. `1.0.0-20240101.010101-1`.
    pub value: String,
    pub timestamp: NaiveDateTime,
    pub build_number: u32,
    pub classifier: Option<String>,
    pub extension: String,
}

/// Reads a timestamped snapshot filename.
///
/// Maven names these `{artifact}-{base}-{yyyyMMdd.HHmmss}-{build}[-{classifier}].{ext}`, where
/// `base` is the version with `-SNAPSHOT` removed — so `1.0.0-SNAPSHOT` deploys as
/// `tms-1.0.0-20240101.010101-1.jar`. Splitting on `-` does not work here: the timestamp, the
/// build number and the classifier are all hyphen-separated and the artifact id may contain
/// hyphens too, which is why this matches the prefix it knows rather than guessing.
pub fn parse_snapshot_file(
    file_name: &str,
    artifact_id: &str,
    base_version: &str,
) -> Option<SnapshotBuild> {
    let prefix = format!("{artifact_id}-{base_version}-");
    let rest = file_name.strip_prefix(&prefix)?;

    // `yyyyMMdd.HHmmss` is a fixed 15 characters, and it is the one part that cannot be found by
    // splitting because it contains its own separator.
    if rest.len() < 16 {
        return None;
    }
    let (timestamp, rest) = rest.split_at(15);
    let timestamp = NaiveDateTime::parse_from_str(timestamp, "%Y%m%d.%H%M%S").ok()?;
    let rest = rest.strip_prefix('-')?;

    // What remains is `{build}[-{classifier}].{ext}`. The extension is everything after the first
    // `.`, so `.jar` and `.tar.gz` both work, but a checksum sitting beside the artifact is left
    // for the checksum path rather than being reported as a build of its own.
    let (name, extension) = rest.split_once('.')?;
    if matches!(extension, "sha1" | "md5" | "sha256" | "sha512" | "asc")
        || extension.contains('.') && extension.rsplit('.').next().is_some_and(is_checksum_suffix)
    {
        return None;
    }
    let (build_number, classifier) = match name.split_once('-') {
        Some((build, classifier)) => (build, Some(classifier.to_owned())),
        None => (name, None),
    };
    let build_number: u32 = build_number.parse().ok()?;

    Some(SnapshotBuild {
        value: format!(
            "{base_version}-{}-{build_number}",
            timestamp.format("%Y%m%d.%H%M%S")
        ),
        timestamp,
        build_number,
        classifier,
        extension: extension.to_owned(),
    })
}

fn is_checksum_suffix(suffix: &str) -> bool {
    matches!(suffix, "sha1" | "md5" | "sha256" | "sha512" | "asc")
}

/// Builds the snapshot document from the builds found in a version directory.
///
/// Returns `None` when the directory holds no timestamped build, which is the normal state for a
/// snapshot deployed by a client that uses non-unique names — there is nothing to describe, and
/// answering with an empty document would tell Maven a build exists that does not.
pub fn snapshot_metadata(
    group_id: String,
    artifact_id: String,
    version: String,
    builds: &[SnapshotBuild],
) -> Option<SnapshotMetadata> {
    let newest = builds.iter().max_by_key(|build| build.build_number)?;

    let snapshot_versions: Vec<SnapshotVersion> = builds
        .iter()
        // Only the newest build is what a bare `-SNAPSHOT` resolves to; older ones stay
        // addressable by their full name but must not appear here, or Maven may pick one.
        .filter(|build| build.build_number == newest.build_number)
        .map(|build| SnapshotVersion {
            classifier: build.classifier.clone(),
            extension: build.extension.clone(),
            value: build.value.clone(),
            updated: Some(build.timestamp),
        })
        .collect();

    Some(SnapshotMetadata {
        group_id,
        artifact_id,
        version,
        versioning: SnapshotVersioning {
            snapshot: Some(Snapshot {
                timestamp: Some(newest.timestamp),
                build_number: newest.build_number.to_string(),
            }),
            snapshot_versions: Some(SnapshotVersions {
                snapshot_version: snapshot_versions,
            }),
            last_updated: Some(newest.timestamp),
        },
    })
}

/// Serializes a metadata document with the root element Maven expects.
///
/// `quick_xml::se::to_string` would name the root after the Rust type, which Maven does not read.
pub fn to_xml<T: serde::Serialize>(value: &T) -> Result<String, MavenError> {
    let body = maven_rs::quick_xml::se::to_string_with_root("metadata", value)?;
    Ok(format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{body}"
    ))
}

/// Splits a `groupId:artifactId` project key.
fn split_project_key(key: &str) -> Result<(String, String), MavenError> {
    key.split_once(':')
        .map(|(group, artifact)| (group.to_owned(), artifact.to_owned()))
        .ok_or(MavenError::MissingFromPom("groupId"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_metadata_requests() {
        let (request, suffix) =
            MetadataRequest::parse(&StoragePath::from("dev/kingtux/tms/maven-metadata.xml"))
                .expect("not recognised");
        // The trailing slash is what `StoragePath::parent` produces, and it is also the form the
        // project's `storage_path` was written in — the two have to agree or the lookup misses.
        let MetadataRequest::Artifact(directory) = &request else {
            panic!("expected an artifact request, got {request:?}");
        };
        assert_eq!(directory.to_string(), "dev/kingtux/tms/");
        assert_eq!(suffix, "");

        let (request, suffix) = MetadataRequest::parse(&StoragePath::from(
            "dev/kingtux/tms/1.0.0-SNAPSHOT/maven-metadata.xml.sha1",
        ))
        .expect("not recognised");
        let MetadataRequest::Snapshot(directory) = &request else {
            panic!("expected a snapshot request, got {request:?}");
        };
        assert_eq!(directory.to_string(), "dev/kingtux/tms/1.0.0-SNAPSHOT/");
        assert_eq!(suffix, ".sha1");

        // A real stored file that happens to sit next to one.
        assert!(
            MetadataRequest::parse(&StoragePath::from("dev/kingtux/tms/maven-metadata.xml.asc"))
                .is_none()
        );
        assert!(
            MetadataRequest::parse(&StoragePath::from("dev/kingtux/tms/tms-1.0.0.jar")).is_none()
        );
    }

    #[test]
    fn reads_timestamped_snapshot_names() {
        let build = parse_snapshot_file("tms-1.0.0-20240101.010101-3.jar", "tms", "1.0.0")
            .expect("plain jar not parsed");
        assert_eq!(build.build_number, 3);
        assert_eq!(build.classifier, None);
        assert_eq!(build.extension, "jar");
        assert_eq!(build.value, "1.0.0-20240101.010101-3");

        let sources =
            parse_snapshot_file("tms-1.0.0-20240101.010101-3-sources.jar", "tms", "1.0.0")
                .expect("classified jar not parsed");
        assert_eq!(sources.classifier.as_deref(), Some("sources"));
        assert_eq!(sources.extension, "jar");

        // An artifact id with hyphens in it — `split('-')` would fall apart here.
        let hyphenated = parse_snapshot_file(
            "my-cool-lib-2.1.0-20240101.010101-1.pom",
            "my-cool-lib",
            "2.1.0",
        )
        .expect("hyphenated artifact id not parsed");
        assert_eq!(hyphenated.extension, "pom");
        assert_eq!(hyphenated.build_number, 1);
    }

    #[test]
    fn ignores_things_that_are_not_builds() {
        // Checksums sit beside the artifacts and are not builds of their own.
        assert!(
            parse_snapshot_file("tms-1.0.0-20240101.010101-3.jar.sha1", "tms", "1.0.0").is_none()
        );
        // Belongs to a different artifact.
        assert!(parse_snapshot_file("other-1.0.0-20240101.010101-3.jar", "tms", "1.0.0").is_none());
        // Not timestamped at all.
        assert!(parse_snapshot_file("tms-1.0.0-SNAPSHOT.jar", "tms", "1.0.0").is_none());
        assert!(parse_snapshot_file("tms-1.0.0-notatimestamp-1.jar", "tms", "1.0.0").is_none());
    }

    /// Only the newest build may appear, or Maven can resolve a bare `-SNAPSHOT` to a stale one.
    #[test]
    fn snapshot_metadata_describes_only_the_newest_build() {
        let builds = vec![
            parse_snapshot_file("tms-1.0.0-20240101.010101-1.jar", "tms", "1.0.0").unwrap(),
            parse_snapshot_file("tms-1.0.0-20240102.010101-2.jar", "tms", "1.0.0").unwrap(),
            parse_snapshot_file("tms-1.0.0-20240102.010101-2-sources.jar", "tms", "1.0.0").unwrap(),
        ];
        let metadata = snapshot_metadata(
            "dev.kingtux".to_owned(),
            "tms".to_owned(),
            "1.0.0-SNAPSHOT".to_owned(),
            &builds,
        )
        .expect("no metadata built");

        assert_eq!(metadata.versioning.snapshot.unwrap().build_number, "2");
        let versions = metadata
            .versioning
            .snapshot_versions
            .unwrap()
            .snapshot_version;
        assert_eq!(versions.len(), 2, "expected the jar and its sources");
        assert!(
            versions
                .iter()
                .all(|version| version.value == "1.0.0-20240102.010101-2")
        );
    }

    #[test]
    fn no_builds_means_no_document() {
        assert!(
            snapshot_metadata(
                "dev.kingtux".to_owned(),
                "tms".to_owned(),
                "1.0.0-SNAPSHOT".to_owned(),
                &[]
            )
            .is_none()
        );
    }

    #[test]
    fn serializes_with_a_metadata_root() {
        let metadata = DeployMetadata {
            group_id: "dev.kingtux".to_owned(),
            artifact_id: "tms".to_owned(),
            versioning: StableVersioning {
                latest: Some("1.0.1".to_owned()),
                release: Some("1.0.1".to_owned()),
                versions: StableVersions {
                    version: vec!["1.0.0".to_owned(), "1.0.1".to_owned()],
                },
                // Always set by the generators. A `None` here serializes to an empty
                // `<lastUpdated/>`, which the same reader then refuses to parse — so the document
                // would not round trip, and Maven would see a field it cannot read.
                last_updated: Some(
                    chrono::NaiveDate::from_ymd_opt(2024, 1, 1)
                        .unwrap()
                        .and_hms_opt(1, 1, 1)
                        .unwrap(),
                ),
            },
        };
        let xml = to_xml(&metadata).unwrap();
        assert!(xml.starts_with("<?xml version=\"1.0\""));
        assert!(xml.contains("<metadata>"), "wrong root element: {xml}");
        assert!(xml.contains("<groupId>dev.kingtux</groupId>"));
        assert!(xml.contains("<version>1.0.0</version>"));
        // Round trips through the same reader Maven's own documents go through.
        let parsed: DeployMetadata = maven_rs::quick_xml::de::from_str(&xml).unwrap();
        assert_eq!(parsed.versioning.versions.version.len(), 2);
    }

    #[test]
    fn snapshot_versions_are_recognised() {
        assert!(is_snapshot_version("1.0.0-SNAPSHOT"));
        assert!(is_snapshot_version("1.0.0-snapshot"));
        assert!(!is_snapshot_version("1.0.0"));
        assert!(!is_snapshot_version("SNAPSHOT-1.0.0"));
    }
}
