//! The artifact query language nitro-repo searches with.
//!
//! Issue #411 asks for artifact searching modelled on Strongbox's and Artifactory's query
//! languages, "pushed as a separate library". This is that crate: a lexer, a parser, a typed AST
//! and a compiler to a parameterized SQL predicate, with no dependency on a web framework or a
//! database driver so it stands on its own.
//!
//! # The language
//!
//! A query is a boolean expression over a closed set of fields:
//!
//! ```text
//! repo == releases and scope == dev.kingtux and version ~= "1.*"
//! name ~= *.jar and not release_type == Snapshot
//! (scope == dev.kingtux or scope == com.example) and created > 2024-01-01
//! ```
//!
//! Operators are `==`, `!=`, `~=` (glob), `!~`, and — on timestamps only — `>`, `>=`, `<`, `<=`.
//! `and`, `or`, `not` and parentheses combine them, and two comparisons side by side mean `and`.
//! Text comparisons are case-insensitive, matching how coordinates are treated everywhere else
//! here.
//!
//! Ordering comparisons are rejected on non-temporal fields: `name > b` would compile to a
//! lexicographic comparison, which is almost never what was meant and never says so.
//!
//! # Safety
//!
//! [`sql::CompiledQuery`] returns a predicate containing only placeholders, plus the values to
//! bind, so caller-supplied text is never part of the statement. Column names come from the
//! [`ast::Field`] enum rather than from the query, so there is nothing to escape there either.
//!
//! ```
//! use nr_aql::{parse, sql::CompiledQuery};
//!
//! let query = parse(r#"name ~= "*.jar" and scope == dev.kingtux"#).unwrap();
//! let compiled = CompiledQuery::compile(&query);
//! assert_eq!(compiled.bindings.len(), 2);
//! ```
use serde::Serialize;

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod sql;

pub use ast::{Field, Query, Value};
pub use parser::{ParseError, parse};

/// A half-open range of character offsets into the query text, so an error can be pointed at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::{Binding, CompiledQuery};

    /// The queries in the module documentation, so the examples cannot rot.
    #[test]
    fn documented_examples_parse_and_compile() {
        for query in [
            "repo == releases and scope == dev.kingtux and version ~= \"1.*\"",
            "name ~= *.jar and not release_type == Snapshot",
            "(scope == dev.kingtux or scope == com.example) and created > 2024-01-01",
        ] {
            let parsed = parse(query).unwrap_or_else(|err| panic!("`{query}`: {err}"));
            let compiled = CompiledQuery::compile(&parsed);
            assert!(!compiled.predicate.is_empty());
            assert!(!compiled.bindings.is_empty());
        }
    }

    /// The AST is serialized to the frontend so it can render a query builder.
    #[test]
    fn the_ast_serializes() {
        let parsed = parse("name == tms and version ~= 1.*").unwrap();
        let json = serde_json::to_value(&parsed).unwrap();
        assert_eq!(json["kind"], "and");
    }

    #[test]
    fn a_query_of_one_comparison_binds_exactly_one_value() {
        let compiled = CompiledQuery::compile(&parse("project == dev.kingtux:tms").unwrap());
        assert_eq!(
            compiled.bindings,
            vec![Binding::Text("dev.kingtux:tms".to_owned())]
        );
    }
}
