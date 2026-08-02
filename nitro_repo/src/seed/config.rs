//! The seed configuration file.
//!
//! Everything is declarative and idempotent: running the same file twice against the same instance
//! should leave it in the same state, so a seed can be re-run after a change without tearing the
//! instance down first.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedConfig {
    /// The base URL of the running instance, e.g. `http://localhost:6742`.
    pub url: String,
    pub auth: SeedAuth,
    /// Storages to create if they are missing.
    #[serde(default)]
    pub storages: Vec<SeedStorage>,
    /// Repositories to create if they are missing.
    #[serde(default)]
    pub repositories: Vec<SeedRepository>,
    /// Maven artifacts to deploy.
    #[serde(default)]
    pub maven: Vec<MavenProject>,
    /// npm packages to publish.
    #[serde(default)]
    pub npm: Vec<NpmPackage>,
    /// Crates to publish.
    #[serde(default)]
    pub cargo: Vec<CargoCrate>,
    /// Container images to push.
    #[serde(default)]
    pub docker: Vec<DockerImage>,
}

/// How to authenticate.
///
/// A token is preferred — it is what a CI job would use, and it exercises the token path rather than
/// the session one. Basic auth is accepted because it is what `mvn deploy` sends, and seeding a
/// fresh instance with only the installed admin account is the common case.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SeedAuth {
    Token { token: String },
    Basic { username: String, password: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedStorage {
    pub name: String,
    /// Matches the `StorageTypeConfig` tag: `Local`, `FileSystemV2` or `S3`.
    #[serde(rename = "type")]
    pub storage_type: String,
    /// The backend's own settings, passed through untouched.
    pub settings: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedRepository {
    pub storage: String,
    pub name: String,
    /// `maven`, `npm`, `cargo` or `docker`.
    #[serde(rename = "type")]
    pub repository_type: String,
    /// Defaults to public so a seeded instance can be browsed without signing in.
    #[serde(default = "default_visibility")]
    pub visibility: String,
}

fn default_visibility() -> String {
    "Public".to_owned()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MavenProject {
    /// `{storage}/{repository}`.
    pub repository: String,
    #[serde(alias = "group")]
    pub group_id: String,
    #[serde(alias = "artifact")]
    pub artifact_id: String,
    pub versions: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
    /// Extra classified artifacts to deploy beside the main jar, e.g. `["sources", "javadoc"]`.
    #[serde(default = "default_classifiers")]
    pub classifiers: Vec<String>,
    /// Dependencies to write into the generated POM, as `groupId:artifactId:version`.
    #[serde(default)]
    pub dependencies: Vec<String>,
}

fn default_classifiers() -> Vec<String> {
    vec!["sources".to_owned(), "javadoc".to_owned()]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmPackage {
    /// `{storage}/{repository}`.
    pub repository: String,
    /// Including the scope, e.g. `@nitro/example`.
    pub name: String,
    pub versions: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
    /// `{ "latest": "1.2.0", "next": "2.0.0-beta.1" }`. `latest` is set automatically to the last
    /// version published if it is not given here.
    #[serde(default)]
    pub dist_tags: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoCrate {
    /// `{storage}/{repository}`.
    pub repository: String,
    pub name: String,
    pub versions: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerImage {
    /// `{storage}/{repository}`.
    pub repository: String,
    /// The image name *without* the `{storage}/{repository}` prefix — the seeder adds it, because
    /// that is how a client addresses a registry that has no hostname of its own yet.
    pub name: String,
    pub tags: Vec<String>,
}

impl SeedConfig {
    /// The suite written by `seed --write-example`.
    ///
    /// Deliberately covers the shapes that have broken before: a scoped npm package, a snapshot
    /// version, a prerelease, a dotted npm name, and a multi-version artifact so
    /// `maven-metadata.xml` has something to merge.
    pub fn example() -> Self {
        Self {
            url: "http://localhost:6742".to_owned(),
            auth: SeedAuth::Basic {
                username: "admin".to_owned(),
                password: "changeme".to_owned(),
            },
            storages: vec![SeedStorage {
                name: "local".to_owned(),
                storage_type: "Local".to_owned(),
                settings: serde_json::json!({ "path": "./storage/local" }),
            }],
            repositories: vec![
                SeedRepository {
                    storage: "local".to_owned(),
                    name: "releases".to_owned(),
                    repository_type: "maven".to_owned(),
                    visibility: "Public".to_owned(),
                },
                SeedRepository {
                    storage: "local".to_owned(),
                    name: "snapshots".to_owned(),
                    repository_type: "maven".to_owned(),
                    visibility: "Public".to_owned(),
                },
                SeedRepository {
                    storage: "local".to_owned(),
                    name: "npm".to_owned(),
                    repository_type: "npm".to_owned(),
                    visibility: "Public".to_owned(),
                },
                SeedRepository {
                    storage: "local".to_owned(),
                    name: "crates".to_owned(),
                    repository_type: "cargo".to_owned(),
                    visibility: "Public".to_owned(),
                },
                SeedRepository {
                    storage: "local".to_owned(),
                    name: "docker".to_owned(),
                    repository_type: "docker".to_owned(),
                    visibility: "Public".to_owned(),
                },
            ],
            maven: vec![
                MavenProject {
                    repository: "local/releases".to_owned(),
                    group_id: "dev.kingtux".to_owned(),
                    artifact_id: "tms".to_owned(),
                    versions: vec![
                        "1.0.0".to_owned(),
                        "1.0.1".to_owned(),
                        "1.1.0".to_owned(),
                        "2.0.0-beta.1".to_owned(),
                    ],
                    description: Some("A seeded example artifact".to_owned()),
                    classifiers: default_classifiers(),
                    dependencies: vec!["org.slf4j:slf4j-api:2.0.13".to_owned()],
                },
                MavenProject {
                    repository: "local/releases".to_owned(),
                    group_id: "dev.kingtux.tools".to_owned(),
                    artifact_id: "nested-example".to_owned(),
                    versions: vec!["0.1.0".to_owned()],
                    description: Some("A deeper group id, to exercise nested paths".to_owned()),
                    classifiers: vec![],
                    dependencies: vec!["dev.kingtux:tms:1.1.0".to_owned()],
                },
                MavenProject {
                    repository: "local/snapshots".to_owned(),
                    group_id: "dev.kingtux".to_owned(),
                    artifact_id: "tms".to_owned(),
                    versions: vec!["1.2.0-SNAPSHOT".to_owned()],
                    description: Some("Snapshot deploys, for maven-metadata.xml".to_owned()),
                    classifiers: default_classifiers(),
                    dependencies: vec![],
                },
            ],
            npm: vec![
                NpmPackage {
                    repository: "local/npm".to_owned(),
                    name: "@nitro/example".to_owned(),
                    versions: vec!["1.0.0".to_owned(), "1.0.1".to_owned(), "1.1.0".to_owned()],
                    description: Some("A scoped package".to_owned()),
                    dist_tags: Default::default(),
                },
                NpmPackage {
                    repository: "local/npm".to_owned(),
                    // A dotted, unscoped name: `validate_name` used to reject these outright, so
                    // `lodash.merge` could not be published.
                    name: "nitro.helpers".to_owned(),
                    versions: vec!["0.1.0".to_owned(), "0.2.0".to_owned()],
                    description: Some("An unscoped package with a dot in its name".to_owned()),
                    dist_tags: Default::default(),
                },
                NpmPackage {
                    repository: "local/npm".to_owned(),
                    name: "@nitro/prerelease".to_owned(),
                    versions: vec!["1.0.0".to_owned(), "2.0.0-beta.1".to_owned()],
                    description: Some("Prerelease versions, and a non-latest dist-tag".to_owned()),
                    dist_tags: [("next".to_owned(), "2.0.0-beta.1".to_owned())]
                        .into_iter()
                        .collect(),
                },
            ],
            cargo: vec![
                CargoCrate {
                    repository: "local/crates".to_owned(),
                    // Four characters or more, so the index path goes through the
                    // `{c1c2}/{c3c4}/` branch rather than one of the short-name special cases.
                    name: "nitro-example".to_owned(),
                    versions: vec!["0.1.0".to_owned(), "0.2.0".to_owned(), "1.0.0".to_owned()],
                    description: Some("A seeded crate".to_owned()),
                },
                CargoCrate {
                    repository: "local/crates".to_owned(),
                    // Three characters: the `3/{first}/` branch, which is easy to get wrong.
                    name: "nrx".to_owned(),
                    versions: vec!["0.1.0".to_owned()],
                    description: Some("A short crate name, for the index prefix rule".to_owned()),
                },
            ],
            docker: vec![DockerImage {
                repository: "local/docker".to_owned(),
                name: "example".to_owned(),
                tags: vec!["1.0".to_owned(), "latest".to_owned()],
            }],
        }
    }
}
