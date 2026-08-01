//! The typed syntax tree a query parses into.
//!
//! Values are held as `Value`, never as raw text spliced into a statement. That is what lets the
//! SQL compiler bind every one of them as a parameter — a query language that reaches a database
//! and builds its predicates by string concatenation is an injection hole by construction.
use serde::Serialize;

use crate::lexer::Operator;

/// A field a query can filter on.
///
/// A closed set on purpose. An open one would mean either trusting a caller-supplied column name
/// or maintaining a second allow-list somewhere further from the parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Field {
    /// The repository's name.
    Repository,
    /// The storage the repository lives in.
    Storage,
    /// The full project key — `dev.kingtux:tms` for Maven, `@scope/pkg` for npm.
    Project,
    /// The project's scope: a Maven `groupId`, an npm scope.
    Scope,
    /// The project's name, without its scope.
    Name,
    /// The project's description.
    Description,
    /// A version string.
    Version,
    /// `Stable`, `Snapshot`, `Unknown`.
    ReleaseType,
    /// When a version was first published.
    Created,
    /// When a version was last updated.
    Updated,
}

impl Field {
    pub fn parse(name: &str) -> Option<Self> {
        // Aliases exist because the same concept is called different things by different
        // ecosystems, and a user should not have to know which one this codebase settled on.
        let field = match name.to_ascii_lowercase().as_str() {
            "repository" | "repo" => Field::Repository,
            "storage" => Field::Storage,
            "project" | "project_key" | "key" => Field::Project,
            "scope" | "group" | "groupid" => Field::Scope,
            "name" | "artifact" | "artifactid" => Field::Name,
            "description" => Field::Description,
            "version" => Field::Version,
            "release_type" | "releasetype" | "type" => Field::ReleaseType,
            "created" | "created_at" => Field::Created,
            "updated" | "updated_at" => Field::Updated,
            _ => return None,
        };
        Some(field)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Field::Repository => "repository",
            Field::Storage => "storage",
            Field::Project => "project",
            Field::Scope => "scope",
            Field::Name => "name",
            Field::Description => "description",
            Field::Version => "version",
            Field::ReleaseType => "release_type",
            Field::Created => "created",
            Field::Updated => "updated",
        }
    }

    /// Every field a caller may use, for error messages and for the UI's autocomplete.
    pub fn all() -> &'static [Field] {
        &[
            Field::Repository,
            Field::Storage,
            Field::Project,
            Field::Scope,
            Field::Name,
            Field::Description,
            Field::Version,
            Field::ReleaseType,
            Field::Created,
            Field::Updated,
        ]
    }

    /// Whether the field holds a timestamp, which decides how a comparison is read.
    pub fn is_temporal(&self) -> bool {
        matches!(self, Field::Created | Field::Updated)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Value {
    Text(String),
    Number(i64),
}

impl Value {
    pub fn as_text(&self) -> String {
        match self {
            Value::Text(text) => text.clone(),
            Value::Number(number) => number.to_string(),
        }
    }
}

/// The tree a query parses into.
///
/// Named fields rather than tuple variants throughout, because the whole thing is serialized to
/// the frontend for the query builder to render, and an internally tagged enum cannot carry a
/// tuple variant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Query {
    /// `field <op> value`
    Comparison {
        field: Field,
        operator: Operator,
        value: Value,
    },
    And {
        left: Box<Query>,
        right: Box<Query>,
    },
    Or {
        left: Box<Query>,
        right: Box<Query>,
    },
    Not {
        inner: Box<Query>,
    },
}

impl Query {
    pub fn and(left: Query, right: Query) -> Self {
        Query::And {
            left: Box::new(left),
            right: Box::new(right),
        }
    }
    pub fn or(left: Query, right: Query) -> Self {
        Query::Or {
            left: Box::new(left),
            right: Box::new(right),
        }
    }
    pub fn negate(inner: Query) -> Self {
        Query::Not {
            inner: Box::new(inner),
        }
    }
}

// `Operator` is part of the serialized AST, which the frontend reads to render a query builder.
impl Serialize for Operator {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_names_and_their_aliases_resolve() {
        for (input, expected) in [
            ("repository", Field::Repository),
            ("repo", Field::Repository),
            ("REPO", Field::Repository),
            ("groupId", Field::Scope),
            ("artifactId", Field::Name),
            ("key", Field::Project),
            ("created_at", Field::Created),
        ] {
            assert_eq!(
                Field::parse(input),
                Some(expected),
                "`{input}` did not resolve"
            );
        }
        assert_eq!(Field::parse("not_a_field"), None);
    }

    /// `all()` feeds the "unknown field" error and the UI's field list, so a field missing from it
    /// is invisible to both.
    #[test]
    fn every_field_is_listed_and_round_trips() {
        for field in Field::all() {
            assert_eq!(
                Field::parse(field.as_str()),
                Some(*field),
                "{field:?} does not parse from its own name"
            );
        }
        assert_eq!(Field::all().len(), 10);
    }
}
