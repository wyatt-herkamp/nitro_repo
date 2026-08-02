//! Compiles a parsed query into a SQL predicate and its bind parameters.
//!
//! **No caller-supplied text ever reaches the statement.** Values come back as a separate
//! `Vec<Binding>` and the predicate refers to them positionally, so a value containing a quote, a
//! semicolon or a comment marker is data and cannot become syntax. Column names are not
//! caller-supplied either — they come from the closed `Field` enum, so there is nothing to escape.
//!
//! This crate does not depend on a database driver, which is what keeps it publishable on its own
//! (#411 asks for that). The caller binds the parameters with whatever driver it uses.
use crate::{
    ast::{Field, Query, Value},
    lexer::Operator,
};

/// A value to bind, in the order the placeholders refer to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Binding {
    Text(String),
    Number(i64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledQuery {
    /// A SQL boolean expression with `$1`-style placeholders.
    pub predicate: String,
    pub bindings: Vec<Binding>,
}

/// Which column each field maps to.
///
/// Written against the join in [`CompiledQuery::FROM_CLAUSE`], so the two move together.
fn column_for(field: Field) -> &'static str {
    match field {
        Field::Repository => "repositories.name",
        Field::Storage => "storages.name",
        Field::Project => "projects.key",
        Field::Scope => "projects.scope",
        Field::Name => "projects.name",
        Field::Description => "projects.description",
        Field::Version => "project_versions.version",
        Field::ReleaseType => "project_versions.release_type",
        Field::Created => "project_versions.created_at",
        Field::Updated => "project_versions.updated_at",
    }
}

impl CompiledQuery {
    /// The joins every compiled predicate is written against.
    pub const FROM_CLAUSE: &'static str = "FROM project_versions \
         INNER JOIN projects ON projects.id = project_versions.project_id \
         INNER JOIN repositories ON repositories.id = projects.repository_id \
         INNER JOIN storages ON storages.id = repositories.storage_id";

    pub fn compile(query: &Query) -> Self {
        let mut compiler = Compiler {
            bindings: Vec::new(),
        };
        let predicate = compiler.compile(query);
        Self {
            predicate,
            bindings: compiler.bindings,
        }
    }
}

struct Compiler {
    bindings: Vec<Binding>,
}

impl Compiler {
    fn bind(&mut self, binding: Binding) -> String {
        self.bindings.push(binding);
        // Postgres placeholders are 1-based.
        format!("${}", self.bindings.len())
    }

    fn compile(&mut self, query: &Query) -> String {
        match query {
            Query::And { left, right } => {
                format!("({} AND {})", self.compile(left), self.compile(right))
            }
            Query::Or { left, right } => {
                format!("({} OR {})", self.compile(left), self.compile(right))
            }
            // A NULL column makes `NOT (col = x)` NULL rather than true, so a row with no
            // description would vanish from `not description == "x"`. Being explicit about it is
            // what makes negation mean what a reader expects.
            Query::Not { inner } => format!("(NOT COALESCE({}, FALSE))", self.compile(inner)),
            Query::Comparison {
                field,
                operator,
                value,
            } => self.compile_comparison(*field, *operator, value),
        }
    }

    fn compile_comparison(&mut self, field: Field, operator: Operator, value: &Value) -> String {
        let column = column_for(field);

        match operator {
            Operator::Matches | Operator::NotMatches => {
                let placeholder = self.bind(Binding::Text(glob_to_like(&value.as_text())));
                // ILIKE, because artifact coordinates are matched case-insensitively everywhere
                // else in this codebase. ESCAPE names the character `glob_to_like` used, so a
                // literal `%` or `_` in a query stays literal.
                //
                // `COLLATE "C"` is not cosmetic: `projects.key` carries a nondeterministic ICU
                // collation (`ignorecase`), and Postgres refuses LIKE and ILIKE against those with
                // `nondeterministic collations are not supported for ILIKE`. Forcing a
                // deterministic collation on the operand is what makes the pattern match legal;
                // ILIKE still supplies the case-insensitivity the collation was there for.
                let comparison = format!("{column} COLLATE \"C\" ILIKE {placeholder} ESCAPE '\\'");
                if operator == Operator::NotMatches {
                    format!("(NOT COALESCE({comparison}, FALSE))")
                } else {
                    comparison
                }
            }
            Operator::Equals | Operator::NotEquals => {
                let placeholder = self.bind(binding_for(value));
                let sql_operator = if operator == Operator::Equals {
                    "="
                } else {
                    "!="
                };
                // Timestamps are compared as timestamps; everything else as case-insensitive text.
                if field.is_temporal() {
                    format!("{column} {sql_operator} {placeholder}::timestamptz")
                } else if matches!(value, Value::Number(_)) {
                    format!("{column} {sql_operator} {placeholder}")
                } else {
                    let comparison = format!("LOWER({column}) = LOWER({placeholder})");
                    if operator == Operator::NotEquals {
                        format!("(NOT COALESCE({comparison}, FALSE))")
                    } else {
                        comparison
                    }
                }
            }
            Operator::GreaterThan
            | Operator::GreaterThanOrEqual
            | Operator::LessThan
            | Operator::LessThanOrEqual => {
                let placeholder = self.bind(binding_for(value));
                let sql_operator = match operator {
                    Operator::GreaterThan => ">",
                    Operator::GreaterThanOrEqual => ">=",
                    Operator::LessThan => "<",
                    Operator::LessThanOrEqual => "<=",
                    _ => unreachable!("outer match restricts the operator"),
                };
                // The parser only allows ordering on a temporal field, so the cast is always
                // right here.
                format!("{column} {sql_operator} {placeholder}::timestamptz")
            }
        }
    }
}

fn binding_for(value: &Value) -> Binding {
    match value {
        Value::Text(text) => Binding::Text(text.clone()),
        Value::Number(number) => Binding::Number(*number),
    }
}

/// Rewrites a glob into a `LIKE` pattern.
///
/// `*` and `?` are what people type; `%` and `_` are what SQL wants. A `%` or `_` already in the
/// query is escaped so it stays a literal — without that, searching for `my_lib` would quietly
/// also match `myXlib`.
fn glob_to_like(glob: &str) -> String {
    let mut pattern = String::with_capacity(glob.len() + 2);
    for character in glob.chars() {
        match character {
            '*' => pattern.push('%'),
            '?' => pattern.push('_'),
            '%' | '_' | '\\' => {
                pattern.push('\\');
                pattern.push(character);
            }
            other => pattern.push(other),
        }
    }
    pattern
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::parse;

    fn compile(input: &str) -> CompiledQuery {
        CompiledQuery::compile(&parse(input).unwrap_or_else(|err| panic!("`{input}`: {err}")))
    }

    #[test]
    fn a_comparison_binds_its_value() {
        let compiled = compile("version == 1.0.0");
        assert_eq!(
            compiled.predicate,
            "LOWER(project_versions.version) = LOWER($1)"
        );
        assert_eq!(compiled.bindings, vec![Binding::Text("1.0.0".to_owned())]);
    }

    #[test]
    fn placeholders_are_numbered_in_binding_order() {
        let compiled = compile("scope == a and name == b or version == c");
        assert_eq!(
            compiled.bindings,
            vec![
                Binding::Text("a".to_owned()),
                Binding::Text("b".to_owned()),
                Binding::Text("c".to_owned()),
            ]
        );
        assert!(compiled.predicate.contains("$1"));
        assert!(compiled.predicate.contains("$2"));
        assert!(compiled.predicate.contains("$3"));
    }

    /// The whole point of the design: a value is data, never syntax.
    #[test]
    fn hostile_values_never_reach_the_statement() {
        let nasty = r#"'; DROP TABLE projects; --"#;
        let compiled = CompiledQuery::compile(&parse(&format!("name == \"{nasty}\"")).unwrap());
        assert!(
            !compiled.predicate.contains("DROP"),
            "a value reached the SQL text: {}",
            compiled.predicate
        );
        assert_eq!(compiled.bindings, vec![Binding::Text(nasty.to_owned())]);

        // And through the glob path, which does rewrite its input.
        let compiled = CompiledQuery::compile(&parse(&format!("name ~= \"{nasty}\"")).unwrap());
        assert!(!compiled.predicate.contains("DROP"));
        assert_eq!(compiled.bindings.len(), 1);
    }

    #[test]
    fn globs_become_like_patterns() {
        assert_eq!(glob_to_like("*.jar"), "%.jar");
        assert_eq!(glob_to_like("tms-1.?.0"), "tms-1._.0");
        // A literal `%` or `_` in the query must not become a wildcard.
        assert_eq!(glob_to_like("my_lib"), r"my\_lib");
        assert_eq!(glob_to_like("100%"), r"100\%");

        let compiled = compile("name ~= *.jar");
        assert_eq!(compiled.bindings, vec![Binding::Text("%.jar".to_owned())]);
        assert!(compiled.predicate.contains("ILIKE"));
        // `projects.key` carries a nondeterministic collation in the real schema, and Postgres
        // rejects LIKE/ILIKE against those outright. Without this the query compiles, passes every
        // unit test here, and then fails at runtime against an actual database.
        assert!(
            compiled.predicate.contains(r#"COLLATE "C""#),
            "glob comparisons must force a deterministic collation: {}",
            compiled.predicate
        );
        assert!(compiled.predicate.contains(r"ESCAPE '\'"));
    }

    /// `NOT (col = x)` is NULL, not true, when the column is NULL — so a project with no
    /// description would silently disappear from a negated query.
    #[test]
    fn negation_keeps_rows_with_null_columns() {
        for query in [
            "not description == x",
            "description != x",
            "description !~ x",
        ] {
            let compiled = compile(query);
            assert!(
                compiled.predicate.contains("COALESCE"),
                "`{query}` would drop rows with a NULL description: {}",
                compiled.predicate
            );
        }
    }

    #[test]
    fn temporal_comparisons_are_cast() {
        let compiled = compile("created > 2024-01-01");
        assert!(
            compiled.predicate.contains("::timestamptz"),
            "{}",
            compiled.predicate
        );
        assert_eq!(
            compiled.bindings,
            vec![Binding::Text("2024-01-01".to_owned())]
        );
    }

    #[test]
    fn grouping_survives_compilation() {
        let compiled = compile("(scope == a or scope == b) and name == c");
        assert!(compiled.predicate.starts_with("(("));
        assert!(compiled.predicate.contains(" OR "));
        assert!(compiled.predicate.contains(" AND "));
    }

    /// Every field has to map to a column, or a query using it compiles to something invalid.
    #[test]
    fn every_field_maps_to_a_column() {
        for field in Field::all() {
            let column = column_for(*field);
            assert!(column.contains('.'), "{field:?} has no table qualifier");
            let (table, _) = column.split_once('.').unwrap();
            assert!(
                CompiledQuery::FROM_CLAUSE.contains(table),
                "{field:?} refers to `{table}`, which the FROM clause does not join"
            );
        }
    }
}
