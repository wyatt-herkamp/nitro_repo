use std::fmt::Display;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::instrument;

/// npm's own limit, from `validate-npm-package-name`. Applies to the whole name including the
/// scope and the separator.
const MAX_NAME_LENGTH: usize = 214;

#[derive(Debug, Error)]
#[error("Invalid NPM Package Name: {name} - {reason}")]
pub struct InvalidNPMPackageName {
    pub name: String,
    pub reason: &'static str,
}

/// A parsed npm package name.
///
/// `scope` holds the scope **without** the leading `@`. It used to keep the `@`, while `Display`
/// and `Serialize` both re-prepended one — so `@scope/test` round-tripped as `@@scope/test`.
/// Publish stored the doubled key and GET looked up the correct one, so scoped packages could be
/// published but never resolved. The test below asserted the doubled value, which is why it went
/// unnoticed.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct NPMPackageName {
    pub name: String,
    pub scope: Option<String>,
}
impl Display for NPMPackageName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.scope {
            Some(scope) => write!(f, "@{}/{}", scope, self.name),
            None => write!(f, "{}", self.name),
        }
    }
}
impl NPMPackageName {
    /// The name a tarball is published under, which never includes the scope.
    ///
    /// npm names the file `{unscoped}-{version}.tgz` even for a scoped package, so `@nr/mylib`
    /// at 1.0.0 is `mylib-1.0.0.tgz`.
    pub fn unscoped_name(&self) -> &str {
        &self.name
    }

    /// Validates one segment — either the scope or the package name.
    ///
    /// The previous rule was `[a-z0-9_-]` only, which rejects a dot and so rejected a large slice
    /// of the real registry: `lodash.merge` could not be published. These are npm's actual
    /// character rules — the URL-safe set, since the name goes in a registry path unencoded.
    fn validate_segment(
        segment: &str,
        whole: &str,
        what: &'static str,
    ) -> Result<(), InvalidNPMPackageName> {
        let invalid = |reason: &'static str| InvalidNPMPackageName {
            name: whole.to_owned(),
            reason,
        };
        if segment.is_empty() {
            return Err(invalid(if what == "scope" {
                "Scope cannot be empty"
            } else {
                "Name cannot be empty"
            }));
        }
        for character in segment.chars() {
            if character.is_ascii_uppercase() {
                return Err(invalid("Name cannot contain uppercase characters"));
            }
            let is_allowed = character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '-' | '.' | '_' | '~');
            if !is_allowed {
                return Err(invalid(
                    "Name may only contain a-z, 0-9, and the characters `-`, `.`, `_`, `~`",
                ));
            }
        }
        Ok(())
    }

    pub fn validate_name(name: &str) -> Result<(), InvalidNPMPackageName> {
        // npm reserves a leading `.` or `_`; `.` and `..` would also collide with path segments
        // once the name becomes a storage path.
        if name.starts_with('.') || name.starts_with('_') {
            return Err(InvalidNPMPackageName {
                name: name.to_owned(),
                reason: "Name cannot start with a `.` or `_`",
            });
        }
        Self::validate_segment(name, name, "name")
    }
}
impl TryFrom<String> for NPMPackageName {
    type Error = InvalidNPMPackageName;
    #[instrument(name = "NPMPackageName::try_from")]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() > MAX_NAME_LENGTH {
            return Err(InvalidNPMPackageName {
                name: value,
                reason: "Name is longer than 214 characters",
            });
        }
        if let Some(rest) = value.strip_prefix('@') {
            let mut parts = rest.split('/');
            let scope = parts.next().unwrap_or_default();
            let Some(name) = parts.next() else {
                return Err(InvalidNPMPackageName {
                    name: value,
                    reason: "Invalid scope format. Must be @scope/name",
                });
            };
            if parts.next().is_some() {
                return Err(InvalidNPMPackageName {
                    name: value,
                    reason: "Invalid scope format. Must be @scope/name",
                });
            }
            NPMPackageName::validate_segment(scope, &value, "scope")?;
            NPMPackageName::validate_name(name).map_err(|err| InvalidNPMPackageName {
                name: value.clone(),
                reason: err.reason,
            })?;
            Ok(NPMPackageName {
                name: name.to_owned(),
                scope: Some(scope.to_owned()),
            })
        } else {
            if value.contains('/') {
                return Err(InvalidNPMPackageName {
                    name: value,
                    reason: "Only a scoped name (`@scope/name`) may contain a `/`",
                });
            }
            NPMPackageName::validate_name(&value)?;
            Ok(NPMPackageName {
                name: value,
                scope: None,
            })
        }
    }
}
impl TryFrom<&str> for NPMPackageName {
    type Error = InvalidNPMPackageName;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        NPMPackageName::try_from(value.to_owned())
    }
}
impl Serialize for NPMPackageName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
impl<'de> Deserialize<'de> for NPMPackageName {
    fn deserialize<D>(deserializer: D) -> Result<NPMPackageName, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        NPMPackageName::try_from(value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
pub mod tests {
    use pretty_assertions::assert_eq;

    use super::NPMPackageName;

    fn unscoped(name: &str) -> NPMPackageName {
        NPMPackageName {
            name: name.to_owned(),
            scope: None,
        }
    }
    fn scoped(scope: &str, name: &str) -> NPMPackageName {
        NPMPackageName {
            name: name.to_owned(),
            scope: Some(scope.to_owned()),
        }
    }

    #[test]
    pub fn valid_packages() {
        let valid = vec![
            ("test", unscoped("test")),
            ("test-package", unscoped("test-package")),
            ("test_package", unscoped("test_package")),
            // A dot is legal and common. This was rejected outright, so `lodash.merge` and
            // everything like it could not be published.
            ("lodash.merge", unscoped("lodash.merge")),
            ("@scope/test", scoped("scope", "test")),
            ("@scope/test-package", scoped("scope", "test-package")),
            ("@scope/test_package", scoped("scope", "test_package")),
            (
                "@babel/plugin-transform-react-jsx",
                scoped("babel", "plugin-transform-react-jsx"),
            ),
        ];
        for (package, expected) in valid {
            let parsed = NPMPackageName::try_from(package)
                .unwrap_or_else(|err| panic!("Failed to parse `{package}`: {err}"));
            assert_eq!(parsed, expected);
        }
    }

    /// The scope must not keep its `@`, or the name re-serializes with two of them and the
    /// published key stops matching the looked-up one.
    #[test]
    pub fn scoped_name_round_trips() {
        for name in ["@scope/test", "test", "lodash.merge", "@a/b"] {
            let parsed = NPMPackageName::try_from(name).unwrap();
            assert_eq!(parsed.to_string(), name, "`{name}` did not round trip");
            assert_eq!(
                serde_json::to_value(&parsed).unwrap(),
                serde_json::Value::String(name.to_owned())
            );
        }
    }

    #[test]
    pub fn scope_excludes_the_at_sign() {
        let parsed = NPMPackageName::try_from("@scope/test").unwrap();
        assert_eq!(parsed.scope.as_deref(), Some("scope"));
        assert_eq!(parsed.unscoped_name(), "test");
    }

    #[test]
    pub fn invalid_packages() {
        let invalid = [
            "",
            "UPPERCASE",
            ".leading-dot",
            "_leading-underscore",
            "has space",
            "has/slash",
            "@scope",
            "@scope/",
            "@/name",
            "@scope/name/extra",
            "sym*bols",
        ];
        for name in invalid {
            assert!(
                NPMPackageName::try_from(name).is_err(),
                "`{name}` was accepted but is not a legal npm name"
            );
        }
        let too_long = "a".repeat(215);
        assert!(NPMPackageName::try_from(too_long.as_str()).is_err());
    }
}
