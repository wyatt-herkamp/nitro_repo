//! Turning a request path into a Cargo registry route.
//!
//! Cargo speaks two things over one URL prefix: the sparse index (under `index/`) and the web API
//! (under `api/v1/`). Both are addressed by path rather than by a header, so — like npm's
//! npm's `GetPath` (`nr_npm::types::request::GetPath`) — this is a `TryFrom<StoragePath>`
//! rather than a set of axum routes.

use nr_core::storage::StoragePath;

use super::super::CargoRegistryError;

/// The index lives under this prefix, keeping `api/` free for the web API.
pub const INDEX_PREFIX: &str = "index";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CargoPath {
    /// `index/config.json` — the sparse index's own descriptor.
    Config,
    /// `index/{prefix}/{name}` — one crate's versions, newline-delimited JSON.
    IndexEntry { name: String },
    /// `api/v1/crates/new`
    Publish,
    /// `api/v1/crates` — with a `?q=` query.
    Search,
    /// `api/v1/crates/{name}/{version}/download`
    Download { name: String, version: String },
    /// `api/v1/crates/{name}/{version}/yank`
    Yank { name: String, version: String },
    /// `api/v1/crates/{name}/{version}/unyank`
    Unyank { name: String, version: String },
    /// `api/v1/crates/{name}/owners`
    Owners { name: String },
}

impl TryFrom<StoragePath> for CargoPath {
    type Error = CargoRegistryError;

    fn try_from(path: StoragePath) -> Result<Self, Self::Error> {
        let components: Vec<String> = Vec::<nr_core::storage::StoragePathComponent>::from(path)
            .into_iter()
            .map(String::from)
            .collect();
        let as_slice: Vec<&str> = components.iter().map(String::as_str).collect();

        match as_slice.as_slice() {
            [INDEX_PREFIX, "config.json"] => Ok(CargoPath::Config),
            // The index prefix directories carry no information the crate name does not already
            // hold, so the name is taken from the last segment and the rest is only checked for
            // plausibility. Serving `ab/cd/serde` as `serde` would let one crate be fetched under
            // another's path and poison a client's cache.
            [INDEX_PREFIX, rest @ ..] if !rest.is_empty() => {
                let name = rest[rest.len() - 1];
                let expected = super::index::index_path_for(name);
                let actual = rest.join("/");
                if actual != expected {
                    return Err(CargoRegistryError::NotFound(format!(
                        "No crate is indexed at `{actual}`"
                    )));
                }
                Ok(CargoPath::IndexEntry {
                    name: name.to_owned(),
                })
            }
            ["api", "v1", "crates"] => Ok(CargoPath::Search),
            ["api", "v1", "crates", "new"] => Ok(CargoPath::Publish),
            ["api", "v1", "crates", name, "owners"] => Ok(CargoPath::Owners {
                name: (*name).to_owned(),
            }),
            ["api", "v1", "crates", name, version, "download"] => Ok(CargoPath::Download {
                name: (*name).to_owned(),
                version: (*version).to_owned(),
            }),
            ["api", "v1", "crates", name, version, "yank"] => Ok(CargoPath::Yank {
                name: (*name).to_owned(),
                version: (*version).to_owned(),
            }),
            ["api", "v1", "crates", name, version, "unyank"] => Ok(CargoPath::Unyank {
                name: (*name).to_owned(),
                version: (*version).to_owned(),
            }),
            other => Err(CargoRegistryError::NotFound(format!(
                "`/{}` is not a route this registry serves",
                other.join("/")
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use nr_core::storage::StoragePath;

    use super::CargoPath;

    fn parse(path: &str) -> Result<CargoPath, super::CargoRegistryError> {
        CargoPath::try_from(StoragePath::parse(path).unwrap())
    }

    #[test]
    fn the_index_routes_parse() {
        assert_eq!(parse("index/config.json").unwrap(), CargoPath::Config);
        assert_eq!(
            parse("index/ca/rg/cargo").unwrap(),
            CargoPath::IndexEntry {
                name: "cargo".to_owned()
            }
        );
        assert_eq!(
            parse("index/1/a").unwrap(),
            CargoPath::IndexEntry {
                name: "a".to_owned()
            }
        );
        assert_eq!(
            parse("index/3/f/foo").unwrap(),
            CargoPath::IndexEntry {
                name: "foo".to_owned()
            }
        );
    }

    #[test]
    fn an_index_path_that_does_not_match_the_name_is_refused() {
        // `serde` belongs at `se/rd/serde`, not here.
        assert!(parse("index/aa/bb/serde").is_err());
        assert!(parse("index/1/serde").is_err());
    }

    #[test]
    fn the_api_routes_parse() {
        assert_eq!(parse("api/v1/crates/new").unwrap(), CargoPath::Publish);
        assert_eq!(parse("api/v1/crates").unwrap(), CargoPath::Search);
        assert_eq!(
            parse("api/v1/crates/serde/1.0.0/download").unwrap(),
            CargoPath::Download {
                name: "serde".to_owned(),
                version: "1.0.0".to_owned()
            }
        );
        assert_eq!(
            parse("api/v1/crates/serde/1.0.0/yank").unwrap(),
            CargoPath::Yank {
                name: "serde".to_owned(),
                version: "1.0.0".to_owned()
            }
        );
        assert_eq!(
            parse("api/v1/crates/serde/1.0.0/unyank").unwrap(),
            CargoPath::Unyank {
                name: "serde".to_owned(),
                version: "1.0.0".to_owned()
            }
        );
        assert_eq!(
            parse("api/v1/crates/serde/owners").unwrap(),
            CargoPath::Owners {
                name: "serde".to_owned()
            }
        );
    }

    #[test]
    fn an_unknown_route_is_refused() {
        for path in [
            "",
            "api",
            "api/v1",
            "api/v2/crates/new",
            "index",
            "nonsense/here",
        ] {
            assert!(parse(path).is_err(), "path: {path:?}");
        }
    }

    #[test]
    fn a_crate_named_new_does_not_shadow_the_publish_route() {
        // `api/v1/crates/new` is publish; `api/v1/crates/new/1.0.0/download` is the crate `new`.
        assert_eq!(
            parse("api/v1/crates/new/1.0.0/download").unwrap(),
            CargoPath::Download {
                name: "new".to_owned(),
                version: "1.0.0".to_owned()
            }
        );
    }
}
