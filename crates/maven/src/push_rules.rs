//! Enforcement of `MavenPushRules` and `ProjectConfig`.
//!
//! `push_policy`, `allow_overwrite` and `must_be_project_member` were defined, persisted, shown in
//! the UI and **never read** — `handle_put` carried a `// TODO: Validate Against Push Rules` and
//! deployed regardless. `ProjectConfig.require_semver` was likewise never checked. A repository
//! configured to accept only releases happily accepted snapshots.
//!
//! POM-vs-path consistency lives here too. Only the presence of `groupId` and `version` was
//! checked before, so a POM claiming `dev.kingtux:tms:1.0.0` could be deployed to
//! `com/example/other/9.9.9/` and the repository would serve it from a coordinate it does not
//! describe.
use maven_rs::pom::Pom;
use nr_core::{repository::Policy, storage::StoragePath};

use super::metadata::is_snapshot_version;

/// Why a deploy was refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PushRejection {
    #[error("This repository does not accept snapshot versions")]
    SnapshotsNotAccepted,
    #[error("This repository only accepts snapshot versions")]
    ReleasesNotAccepted,
    #[error("`{0}` already exists and this repository does not allow overwriting")]
    OverwriteNotAllowed(String),
    #[error("You must be a member of this project to push to it")]
    NotAProjectMember,
    #[error("`{0}` is not a valid semver version, which this repository requires")]
    NotSemver(String),
    #[error(
        "The POM describes `{pom}` but was deployed to `{path}`. \
         The path must match the coordinates in the POM."
    )]
    PomPathMismatch { pom: String, path: String },
}

/// Whether the version directory a file is being deployed into is acceptable under the policy.
///
/// `None` means the path has no version directory to judge — `maven-metadata.xml` at the artifact
/// level, for instance — and the policy does not apply.
pub fn check_policy(policy: Policy, version: Option<&str>) -> Option<PushRejection> {
    let version = version?;
    let is_snapshot = is_snapshot_version(version);
    match policy {
        Policy::Release if is_snapshot => Some(PushRejection::SnapshotsNotAccepted),
        Policy::Snapshot if !is_snapshot => Some(PushRejection::ReleasesNotAccepted),
        _ => None,
    }
}

/// The version directory a deploy path sits in, if it has one.
///
/// A Maven layout is `{group as directories}/{artifact}/{version}/{file}`, so the version is the
/// parent directory. Files that live above that level — `maven-metadata.xml` for the artifact —
/// have no version, and are told apart by the directory not looking like one. A version always
/// starts with a digit in practice; that is the only signal available without asking the database
/// what versions exist, and asking would make every upload wait on a query it does not need.
pub fn version_directory_of(path: &StoragePath) -> Option<String> {
    let parent = path.clone().parent();
    let name = parent.into_iter().next_back()?;
    let name = name.as_ref();
    name.chars()
        .next()
        .is_some_and(|c| c.is_ascii_digit())
        .then(|| name.to_owned())
}

/// Whether a version is acceptable when the repository requires semver.
///
/// A Maven snapshot is `1.0.0-SNAPSHOT`, which is a legal semver prerelease, so it passes as-is.
pub fn check_semver(require_semver: bool, version: &str) -> Option<PushRejection> {
    if !require_semver {
        return None;
    }
    if semver::Version::parse(version).is_ok() {
        return None;
    }
    Some(PushRejection::NotSemver(version.to_owned()))
}

/// Whether a POM's coordinates match the path it was deployed to.
///
/// Maven derives the path from the coordinates, so a mismatch means either a misconfigured client
/// or a deliberate attempt to publish under someone else's coordinates.
pub fn check_pom_matches_path(pom: &Pom, path: &StoragePath) -> Option<PushRejection> {
    let group_id = pom.get_group_id()?;
    let version = pom.get_version()?;
    let artifact_id = &pom.artifact_id;

    let expected = format!("{}/{}/{}", group_id.replace('.', "/"), artifact_id, version);
    let directory = path.clone().parent().to_string();
    // The deploy path is the version directory. A snapshot's files live directly in it too, so
    // this is an equality check rather than a prefix one.
    if directory.trim_matches('/') == expected {
        return None;
    }
    Some(PushRejection::PomPathMismatch {
        pom: format!("{group_id}:{artifact_id}:{version}"),
        path: directory,
    })
}

/// Whether a path is one that is expected to be rewritten on every deploy.
///
/// `maven-metadata.xml` and checksums are republished each time and must not be caught by
/// `allow_overwrite`, or the second deploy of any artifact fails.
pub fn is_rewritable(path: &StoragePath) -> bool {
    let as_string = path.to_string();
    let Some(file_name) = as_string.rsplit('/').next() else {
        return false;
    };
    file_name.starts_with(super::metadata::METADATA_FILE_NAME)
        || matches!(
            file_name.rsplit('.').next(),
            Some("sha1" | "md5" | "sha256" | "sha512" | "asc")
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_rejects_the_wrong_kind_of_version() {
        assert_eq!(
            check_policy(Policy::Release, Some("1.0.0-SNAPSHOT")),
            Some(PushRejection::SnapshotsNotAccepted)
        );
        assert_eq!(check_policy(Policy::Release, Some("1.0.0")), None);
        assert_eq!(
            check_policy(Policy::Snapshot, Some("1.0.0")),
            Some(PushRejection::ReleasesNotAccepted)
        );
        assert_eq!(check_policy(Policy::Snapshot, Some("1.0.0-SNAPSHOT")), None);
        // Mixed accepts both, and a path with no version is not the policy's business.
        assert_eq!(check_policy(Policy::Mixed, Some("1.0.0")), None);
        assert_eq!(check_policy(Policy::Mixed, Some("1.0.0-SNAPSHOT")), None);
        assert_eq!(check_policy(Policy::Release, None), None);
    }

    #[test]
    fn finds_the_version_directory() {
        assert_eq!(
            version_directory_of(&StoragePath::from("dev/kingtux/tms/1.0.0/tms-1.0.0.jar"))
                .as_deref(),
            Some("1.0.0")
        );
        assert_eq!(
            version_directory_of(&StoragePath::from(
                "dev/kingtux/tms/1.0.0-SNAPSHOT/tms-1.0.0-20240101.010101-1.jar"
            ))
            .as_deref(),
            Some("1.0.0-SNAPSHOT")
        );
        // The artifact-level metadata sits above any version.
        assert_eq!(
            version_directory_of(&StoragePath::from("dev/kingtux/tms/maven-metadata.xml")),
            None
        );
    }

    #[test]
    fn semver_is_only_required_when_asked_for() {
        assert_eq!(check_semver(false, "not-a-version"), None);
        assert_eq!(check_semver(true, "1.0.0"), None);
        // A Maven snapshot is a legal semver prerelease.
        assert_eq!(check_semver(true, "1.0.0-SNAPSHOT"), None);
        assert_eq!(
            check_semver(true, "1.0"),
            Some(PushRejection::NotSemver("1.0".to_owned()))
        );
    }

    fn pom(group: &str, artifact: &str, version: &str) -> Pom {
        let xml = format!(
            r#"<project>
                <groupId>{group}</groupId>
                <artifactId>{artifact}</artifactId>
                <version>{version}</version>
            </project>"#
        );
        maven_rs::quick_xml::de::from_str(&xml).expect("test POM did not parse")
    }

    #[test]
    fn pom_coordinates_must_match_the_deploy_path() {
        let pom = pom("dev.kingtux", "tms", "1.0.0");
        assert_eq!(
            check_pom_matches_path(
                &pom,
                &StoragePath::from("dev/kingtux/tms/1.0.0/tms-1.0.0.pom")
            ),
            None
        );
        // Right artifact, wrong version directory.
        assert!(
            check_pom_matches_path(
                &pom,
                &StoragePath::from("dev/kingtux/tms/9.9.9/tms-1.0.0.pom")
            )
            .is_some()
        );
        // Publishing under someone else's group.
        assert!(
            check_pom_matches_path(
                &pom,
                &StoragePath::from("com/example/other/1.0.0/tms-1.0.0.pom")
            )
            .is_some()
        );
    }

    /// A second deploy republishes these, so `allow_overwrite` must not apply to them.
    #[test]
    fn republished_files_are_exempt_from_overwrite_rules() {
        assert!(is_rewritable(&StoragePath::from(
            "dev/kingtux/tms/maven-metadata.xml"
        )));
        assert!(is_rewritable(&StoragePath::from(
            "dev/kingtux/tms/maven-metadata.xml.sha1"
        )));
        assert!(is_rewritable(&StoragePath::from(
            "dev/kingtux/tms/1.0.0/tms-1.0.0.jar.sha1"
        )));
        assert!(!is_rewritable(&StoragePath::from(
            "dev/kingtux/tms/1.0.0/tms-1.0.0.jar"
        )));
        assert!(!is_rewritable(&StoragePath::from(
            "dev/kingtux/tms/1.0.0/tms-1.0.0.pom"
        )));
    }
}
