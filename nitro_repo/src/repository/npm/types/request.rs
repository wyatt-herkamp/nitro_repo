use std::{borrow::Cow, str::FromStr};

use ahash::HashMap;
use axum::response::{IntoResponse, Response};
use http::{HeaderValue, header::ToStrError};
use nr_core::{
    database::entities::project::{NewProject, versions::NewVersion},
    repository::project::VersionData,
    storage::{StoragePath, StoragePathComponent},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::{Display, EnumString};
use tracing::{debug, info};
use uuid::Uuid;

use super::NPMPackageName;
use crate::repository::{maven::get_release_type, npm::NPMRegistryError};

/// The value of the `npm-command` header.
///
/// Only `publish` existed, so every other client command fell through to a 400 — `npm deprecate`,
/// `npm dist-tag`, `npm unpublish`, `npm access` and friends all failed with "Invalid command"
/// regardless of whether the route behind them worked.
#[derive(Debug, Display, EnumString, PartialEq, Eq, Clone, Copy)]
pub enum NPMCommand {
    #[strum(serialize = "publish")]
    Publish,
    #[strum(serialize = "unpublish")]
    Unpublish,
    #[strum(serialize = "deprecate")]
    Deprecate,
    #[strum(serialize = "dist-tag")]
    DistTag,
    #[strum(serialize = "access")]
    Access,
    #[strum(serialize = "owner")]
    Owner,
    #[strum(serialize = "star")]
    Star,
    #[strum(serialize = "adduser", serialize = "login")]
    AddUser,
}
impl TryFrom<&HeaderValue> for NPMCommand {
    type Error = InvalidNPMCommand;
    fn try_from(value: &HeaderValue) -> Result<Self, Self::Error> {
        let value = value.to_str()?;
        NPMCommand::from_str(value)
            .map_err(|_| InvalidNPMCommand::InvalidCommand(value.to_string()))
    }
}
#[derive(Debug, thiserror::Error)]
pub enum InvalidNPMCommand {
    #[error("Invalid command {0}")]
    InvalidCommand(String),
    #[error("Unparsable command {0}")]
    UnparsableCommand(#[from] ToStrError),
    #[error("No header found")]
    NoHeaderFound,
}
impl IntoResponse for InvalidNPMCommand {
    fn into_response(self) -> Response {
        Response::builder()
            .status(http::StatusCode::BAD_REQUEST)
            .body(self.to_string().into())
            .unwrap()
    }
}
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct PublishVersion {
    pub name: NPMPackageName,
    pub version: String,
    pub dist: PublishDist,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
    #[serde(rename = "_id")]
    pub hidden_id: String,
    #[serde(default)]
    pub readme: String,
    #[serde(default, rename = "readmeFilename")]
    pub readme_file_name: String,
    #[serde(rename = "_nodeVersion")]
    pub secret_node_version: String,
    #[serde(rename = "_npmVersion")]
    pub hidden_npm_version: String,
}
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct PublishDist {
    pub integrity: String,
    pub shasum: String,
    pub tarball: String,
    #[serde(flatten)]
    pub other: HashMap<String, Value>,
}
impl PublishDist {
    #[tracing::instrument]
    pub fn validate_tarball(
        &self,
        storage_name: &str,
        repository_name: &str,
    ) -> Result<(), NPMRegistryError> {
        let url = url::Url::from_str(&self.tarball).map_err(|error| {
            info!(?error, "Invalid tarball");
            NPMRegistryError::InvalidTarball {
                tarball_route: self.tarball.clone(),
                error: Cow::Owned(format!("Invalid URL: {}", error)),
            }
        })?;
        let mut path = url
            .path_segments()
            .ok_or(NPMRegistryError::InvalidTarball {
                tarball_route: self.tarball.clone(),
                error: Cow::Borrowed("No Path"),
            })?;
        if path.next().is_none() {
            info!(?url, "Invalid tarball (Missing Base Path for tarball)");
            return Err(NPMRegistryError::InvalidTarball {
                tarball_route: self.tarball.clone(),
                error: Cow::Borrowed("Missing base path"),
            });
        }
        if path.next() != Some(storage_name) {
            info!(?url, "Invalid tarball (Missing storage name)");
            return Err(NPMRegistryError::InvalidTarball {
                tarball_route: self.tarball.clone(),
                error: Cow::Borrowed("Missing storage name"),
            });
        }
        if path.next() != Some(repository_name) {
            info!(?url, "Invalid tarball (Missing repository name)");
            return Err(NPMRegistryError::InvalidTarball {
                tarball_route: self.tarball.clone(),
                error: Cow::Borrowed("Missing repository name"),
            });
        }
        Ok(())
    }
}
impl PublishVersion {
    /// `description` from the published `package.json`.
    ///
    /// It arrives in `extra` because everything outside the named fields is flattened there.
    pub fn description(&self) -> Option<&str> {
        self.extra.get("description").and_then(Value::as_str)
    }
    /// A field from the published `package.json` that this type does not name explicitly.
    pub fn extra_field(&self, key: &str) -> Option<&Value> {
        self.extra.get(key)
    }
    pub fn new_project(
        &self,
        save_path: String,
        repository_id: Uuid,
    ) -> Result<NewProject, NPMRegistryError> {
        let project_key = self.name.to_string();
        let description = self.description().map(str::to_owned);
        let NPMPackageName { name, scope } = self.name.clone();
        Ok(NewProject {
            scope,
            project_key,
            name,
            storage_path: save_path,
            repository: repository_id,
            // Was hardcoded `None`, so a package's description never reached the database and the
            // project page and packument both showed nothing.
            description,
        })
    }
    pub fn new_version(
        &self,
        project_id: Uuid,
        save_path: String,
        publisher: i32,
    ) -> Result<NewVersion, NPMRegistryError> {
        let release_type = get_release_type(&self.version);
        let extra = VersionData {
            extra: Some(serde_json::to_value(self).unwrap()),
            ..Default::default()
        };
        Ok(NewVersion {
            project_id,
            version: self.version.clone(),
            release_type,
            version_path: save_path,
            publisher: Some(publisher),
            version_page: None,
            extra,
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GetPath {
    /// `GET /` — the registry root. npm probes this to decide whether a registry is alive.
    RegistryBase,
    /// `GET /-/ping`
    Ping,
    /// `GET /-/whoami`
    Whoami,
    /// `GET /-/v1/search`. The query itself arrives in the URI query string, not the path.
    Search,
    /// `GET /-/v1/done/{session}` — the `doneUrl` npm polls during a browser login.
    LoginDone {
        session: String,
    },
    /// `GET /-/package/{name}/dist-tags`
    DistTags {
        name: String,
    },
    GetPackageInfo {
        name: String,
    },
    VersionInfo {
        name: String,
        version: String,
    },
    GetTar {
        name: String,
        version: String,
        file: String,
    },
}
impl GetPath {
    /// Path Types
    ///
    /// - `@{scope}/{package}` - Get package info
    /// - `@{scope}/{package}/{version}` - Get version info
    /// - `@{scope}/{package}/-/{scope}/{package}-{version}.tgz` - Get file
    pub fn scoped_package_call(
        components: Vec<StoragePathComponent>,
    ) -> Result<Self, NPMRegistryError> {
        let length = components.len();
        if length < 2 {
            // `GET /{storage}/{repository}/@scope` — a scope with no package after it. This used
            // to `panic!`, which took the handler task down on a request anyone could make.
            info!(?components, "Scoped path is missing a package name");
            return Err(NPMRegistryError::InvalidGetRequest);
        }
        let name = format!("{}/{}", components[0], components[1]);
        if length == 2 {
            return Ok(GetPath::GetPackageInfo { name });
        }
        if length == 3 {
            let version = components[2].to_string();
            debug!(?name, ?version, "Version info");
            return Ok(GetPath::VersionInfo { name, version });
        }
        if length == 5 {
            let file = components[4].to_string();
            let version = extract_version_from_file(&file, &name)
                .ok_or(NPMRegistryError::InvalidGetRequest)?;
            return Ok(GetPath::GetTar {
                name,
                version,
                file,
            });
        }
        info!(?components, "Invalid path");
        Err(NPMRegistryError::InvalidGetRequest)
    }
    /// Path Types
    ///
    /// - `{package}` - Get package info
    /// - `{package}/{version}` - Get version info
    /// - `{package}/-/{package}-{version}.tgz` - Get file
    pub fn unscoped_package_call(
        components: Vec<StoragePathComponent>,
    ) -> Result<Self, NPMRegistryError> {
        let length = components.len();

        let name = components[0].to_string();
        if length == 1 {
            return Ok(GetPath::GetPackageInfo { name });
        }
        if length == 2 {
            let version = components[1].to_string();
            debug!(?name, ?version, "Version info");
            return Ok(GetPath::VersionInfo { name, version });
        }
        if length == 3 {
            let file = components[2].to_string();
            let version = extract_version_from_file(&file, &name)
                .ok_or(NPMRegistryError::InvalidGetRequest)?;
            return Ok(GetPath::GetTar {
                name,
                version,
                file,
            });
        }
        info!(?components, "Invalid path");
        Err(NPMRegistryError::InvalidGetRequest)
    }

    /// Routes under `/-/`, which npm reserves for registry endpoints rather than packages.
    ///
    /// A name can appear here in either form: npm percent-encodes the separator in a scoped name
    /// for these routes, and axum decodes it back, so `@scope/pkg` arrives as two components.
    fn registry_call(components: &[StoragePathComponent]) -> Result<Self, NPMRegistryError> {
        let rest: Vec<String> = components[1..].iter().map(|c| c.to_string()).collect();
        let as_str: Vec<&str> = rest.iter().map(String::as_str).collect();
        match as_str.as_slice() {
            ["ping"] => Ok(GetPath::Ping),
            ["whoami"] => Ok(GetPath::Whoami),
            ["v1", "search"] => Ok(GetPath::Search),
            ["v1", "done", session] => Ok(GetPath::LoginDone {
                session: (*session).to_owned(),
            }),
            ["package", rest @ .., "dist-tags"] if !rest.is_empty() => Ok(GetPath::DistTags {
                name: rest.join("/"),
            }),
            _ => {
                info!(?components, "Unhandled registry route");
                Err(NPMRegistryError::InvalidGetRequest)
            }
        }
    }
}
impl TryFrom<StoragePath> for GetPath {
    type Error = NPMRegistryError;

    fn try_from(value: StoragePath) -> Result<Self, Self::Error> {
        let as_string = value.to_string();
        let components: Vec<_> = value.into();
        if components.is_empty() {
            return Ok(GetPath::RegistryBase);
        }
        if components[0].as_ref() == "-" {
            return GetPath::registry_call(&components);
        }
        if as_string.starts_with('@') {
            GetPath::scoped_package_call(components)
        } else {
            GetPath::unscoped_package_call(components)
        }
    }
}

/// Pulls the version out of a tarball filename.
///
/// This used to be `file.split('-').last()`, which is only right when neither the package name nor
/// the version contains a hyphen. `mylib-1.0.0-beta.1.tgz` yielded `beta.1`, so every prerelease
/// resolved to a version that does not exist — and a hyphenated name like `npm-check-updates` only
/// worked by luck, because the version happened to be the final segment.
///
/// The filename is `{unscoped name}-{version}.tgz`, so the name is what tells us where the version
/// starts. A filename that does not match the package it was requested under is rejected rather
/// than guessed at.
pub fn extract_version_from_file(file: &str, package_name: &str) -> Option<String> {
    let unscoped = package_name.rsplit('/').next()?;
    let version = file
        .strip_suffix(".tgz")?
        .strip_prefix(unscoped)?
        .strip_prefix('-')?;
    if version.is_empty() {
        return None;
    }
    Some(version.to_owned())
}

#[cfg(test)]
pub mod tests {
    use nr_core::storage::StoragePath;

    use super::{GetPath, extract_version_from_file};

    /// A hyphen in the version is the common case for a prerelease, and a hyphen in the name is
    /// the common case for everything else.
    #[test]
    pub fn versions_with_hyphens() {
        let cases = [
            ("mylib-1.0.0.tgz", "mylib", Some("1.0.0")),
            // `split('-').last()` gave `beta.1` here.
            ("mylib-1.0.0-beta.1.tgz", "mylib", Some("1.0.0-beta.1")),
            (
                "npm-check-updates-11.0.3.tgz",
                "npm-check-updates",
                Some("11.0.3"),
            ),
            (
                "npm-check-updates-11.0.3-rc.2.tgz",
                "npm-check-updates",
                Some("11.0.3-rc.2"),
            ),
            // The tarball for a scoped package is named without the scope.
            ("mylib-1.0.0.tgz", "@nr/mylib", Some("1.0.0")),
            // Belongs to a different package — not ours to guess at.
            ("other-1.0.0.tgz", "mylib", None),
            ("mylib-1.0.0.tar.gz", "mylib", None),
            ("mylib-.tgz", "mylib", None),
        ];
        for (file, name, expected) in cases {
            assert_eq!(
                extract_version_from_file(file, name).as_deref(),
                expected,
                "{file} under {name}"
            );
        }
    }

    /// The bug that made scoped packages unusable: publish stored the key that
    /// `NPMPackageName` produced, GET looked up the key that `GetPath` produced, and for a scoped
    /// package the two were `@@nr/mylib` and `@nr/mylib`. They never matched, so a scoped package
    /// could be published and then never resolved.
    #[test]
    pub fn published_and_requested_keys_agree() {
        use crate::repository::npm::types::NPMPackageName;

        for name in ["@nr/mylib", "mylib", "@babel/core"] {
            let published = NPMPackageName::try_from(name).unwrap().to_string();
            let requested = match GetPath::try_from(StoragePath::from(name)).unwrap() {
                GetPath::GetPackageInfo { name } => name,
                other => panic!("`{name}` parsed as {other:?}"),
            };
            assert_eq!(
                published, requested,
                "publish and fetch disagree for {name}"
            );
        }
    }

    #[test]
    pub fn registry_routes() {
        let cases = [
            ("", GetPath::RegistryBase),
            ("-/ping", GetPath::Ping),
            ("-/whoami", GetPath::Whoami),
            // Declared but never constructed, so a search was parsed as a tarball fetch and 404d
            // with a message about a missing package.
            ("-/v1/search", GetPath::Search),
            (
                "-/package/mylib/dist-tags",
                GetPath::DistTags {
                    name: "mylib".to_owned(),
                },
            ),
            (
                "-/package/@nr/mylib/dist-tags",
                GetPath::DistTags {
                    name: "@nr/mylib".to_owned(),
                },
            ),
        ];
        for (path, expected) in cases {
            let parsed = GetPath::try_from(StoragePath::from(path))
                .unwrap_or_else(|err| panic!("`{path}` failed to parse: {err}"));
            assert_eq!(parsed, expected, "for path `{path}`");
        }
    }

    #[test]
    pub fn tests() {
        let tests = vec![
            (
                StoragePath::from("@nr/mylib/-/@nr/mylib-1.0.0.tgz"),
                GetPath::GetTar {
                    name: "@nr/mylib".to_string(),
                    version: "1.0.0".to_string(),
                    file: "mylib-1.0.0.tgz".to_string(),
                },
            ),
            (
                StoragePath::from("mylib/-/mylib-1.0.0.tgz"),
                GetPath::GetTar {
                    name: "mylib".to_string(),
                    version: "1.0.0".to_string(),
                    file: "mylib-1.0.0.tgz".to_string(),
                },
            ),
            (
                StoragePath::from("mylib/1.0.0"),
                GetPath::VersionInfo {
                    name: "mylib".to_string(),
                    version: "1.0.0".to_string(),
                },
            ),
            (
                StoragePath::from("mylib"),
                GetPath::GetPackageInfo {
                    name: "mylib".to_string(),
                },
            ),
            (
                StoragePath::from("npm-check-updates/-/npm-check-updates-11.0.3.tgz"),
                GetPath::GetTar {
                    name: "npm-check-updates".to_string(),
                    version: "11.0.3".to_string(),
                    file: "npm-check-updates-11.0.3.tgz".to_string(),
                },
            ),
        ];
        for (path, expected) in tests {
            let get_path = GetPath::try_from(path).unwrap();
            assert_eq!(get_path, expected);
        }
    }
}
