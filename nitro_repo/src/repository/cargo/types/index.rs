//! The registry index record.
//!
//! One JSON object per published version, serialised one-per-line into the sparse index. The shape
//! is fixed by Cargo and documented at
//! <https://doc.rust-lang.org/cargo/reference/registry-index.html>.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::publish::{PublishDependency, PublishMetadata};

/// The index schema version this registry writes.
///
/// `2` is what unlocks `features2` — a feature whose value contains a `dep:` or `?/` entry can only
/// be expressed there, and an older Cargo that does not understand `v: 2` is told to upgrade rather
/// than being handed a record it would misread.
pub const INDEX_SCHEMA_VERSION: u32 = 2;

/// A single line of the sparse index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexEntry {
    pub name: String,
    pub vers: String,
    pub deps: Vec<IndexDependency>,
    /// SHA-256 of the `.crate` file, lowercase hex. Cargo verifies every download against this.
    pub cksum: String,
    pub features: BTreeMap<String, Vec<String>>,
    pub yanked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub links: Option<String>,
    pub v: u32,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub features2: BTreeMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rust_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IndexDependency {
    /// The name the dependent crate refers to this dependency by. With `explicit_name_in_toml` this
    /// is the rename, and `package` carries the real crate name.
    pub name: String,
    pub req: String,
    pub features: Vec<String>,
    pub optional: bool,
    pub default_features: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    pub kind: String,
    /// `None` means "this registry". Cargo writes the upstream's index URL here otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registry: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
}

impl From<PublishDependency> for IndexDependency {
    fn from(dependency: PublishDependency) -> Self {
        let PublishDependency {
            name,
            version_req,
            features,
            optional,
            default_features,
            target,
            kind,
            registry,
            explicit_name_in_toml,
        } = dependency;
        // A renamed dependency is published as `explicit_name_in_toml = "new", name = "real"`, but
        // the index inverts that: `name` is what the crate calls it and `package` is what to fetch.
        // Getting this backwards makes a renamed dependency resolve to the wrong crate.
        let (name, package) = match explicit_name_in_toml {
            Some(renamed) => (renamed, Some(name)),
            None => (name, None),
        };
        Self {
            name,
            req: version_req,
            features,
            optional,
            default_features,
            target,
            kind: kind.unwrap_or_else(|| "normal".to_owned()),
            registry,
            package,
        }
    }
}

impl IndexEntry {
    /// Builds the index record for a version being published.
    ///
    /// Cargo sends one `features` map; the entries that need index schema 2 are split out into
    /// `features2` here, because an older client reading them from `features` would fail to parse
    /// the `dep:`/`?/` syntax rather than ignoring it.
    pub fn from_publish(metadata: PublishMetadata, cksum: String) -> Self {
        let PublishMetadata {
            name,
            vers,
            deps,
            features,
            links,
            rust_version,
            ..
        } = metadata;

        let mut plain = BTreeMap::new();
        let mut v2 = BTreeMap::new();
        for (feature, values) in features {
            if values
                .iter()
                .any(|value| value.starts_with("dep:") || value.contains("?/"))
            {
                v2.insert(feature, values);
            } else {
                plain.insert(feature, values);
            }
        }

        Self {
            name,
            vers,
            deps: deps.into_iter().map(IndexDependency::from).collect(),
            cksum,
            features: plain,
            yanked: false,
            links,
            v: INDEX_SCHEMA_VERSION,
            features2: v2,
            rust_version,
        }
    }
}

/// Where a crate's index record lives, relative to the index root.
///
/// Cargo derives the path from the name's length so the index never has one enormous directory:
/// one- and two-character names live under `1/` and `2/`, three-character names under `3/{first}/`,
/// and everything else under the first two characters then the next two. The name is lowercased —
/// the index is case-insensitive even though the crate name is not.
///
/// Getting this wrong does not fail loudly; every lookup simply 404s.
pub fn index_path_for(name: &str) -> String {
    let lower = name.to_lowercase();
    let chars: Vec<char> = lower.chars().collect();
    match chars.len() {
        0 => lower,
        1 => format!("1/{lower}"),
        2 => format!("2/{lower}"),
        3 => format!("3/{}/{}", chars[0], lower),
        _ => {
            let first: String = chars[0..2].iter().collect();
            let second: String = chars[2..4].iter().collect();
            format!("{first}/{second}/{lower}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{IndexEntry, index_path_for};
    use crate::repository::cargo::types::publish::PublishMetadata;

    #[test]
    fn the_index_path_depends_on_the_name_length() {
        assert_eq!(index_path_for("a"), "1/a");
        assert_eq!(index_path_for("ab"), "2/ab");
        assert_eq!(index_path_for("abc"), "3/a/abc");
        assert_eq!(index_path_for("abcd"), "ab/cd/abcd");
        assert_eq!(index_path_for("serde"), "se/rd/serde");
        assert_eq!(index_path_for("cargo"), "ca/rg/cargo");
    }

    #[test]
    fn the_index_path_is_lowercased() {
        assert_eq!(index_path_for("Serde"), "se/rd/serde");
        assert_eq!(index_path_for("A"), "1/a");
        assert_eq!(index_path_for("Nitro_Repo"), "ni/tr/nitro_repo");
    }

    #[test]
    fn features_needing_schema_two_are_split_out() {
        let metadata: PublishMetadata = serde_json::from_value(serde_json::json!({
            "name": "example",
            "vers": "1.0.0",
            "deps": [],
            "features": {
                "plain": ["other"],
                "uses-dep": ["dep:serde"],
                "uses-weak": ["serde?/derive"],
            },
        }))
        .unwrap();

        let entry = IndexEntry::from_publish(metadata, "abc".to_owned());
        assert_eq!(entry.v, 2);
        assert!(entry.features.contains_key("plain"));
        assert!(entry.features2.contains_key("uses-dep"));
        assert!(entry.features2.contains_key("uses-weak"));
        assert!(!entry.features.contains_key("uses-dep"));
    }

    #[test]
    fn a_renamed_dependency_keeps_the_real_name_in_package() {
        let metadata: PublishMetadata = serde_json::from_value(serde_json::json!({
            "name": "example",
            "vers": "1.0.0",
            "deps": [{
                "name": "serde_json",
                "version_req": "^1",
                "features": [],
                "optional": false,
                "default_features": true,
                "target": null,
                "kind": "normal",
                "registry": null,
                "explicit_name_in_toml": "json",
            }],
            "features": {},
        }))
        .unwrap();

        let entry = IndexEntry::from_publish(metadata, "abc".to_owned());
        let dependency = &entry.deps[0];
        assert_eq!(dependency.name, "json");
        assert_eq!(dependency.package.as_deref(), Some("serde_json"));
        assert_eq!(dependency.req, "^1");
    }
}
