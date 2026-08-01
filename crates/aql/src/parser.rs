//! Recursive-descent parser for the query language.
//!
//! Precedence, loosest first: `or`, `and`, `not`, then comparisons and parentheses. That is what
//! makes `a == 1 or b == 2 and c == 3` read as `a == 1 or (b == 2 and c == 3)`, which is what
//! anyone writing it expects.
use crate::{
    Span,
    ast::{Field, Query, Value},
    lexer::{LexError, Operator, SpannedToken, Token, tokenize},
};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    #[error(transparent)]
    Lex(#[from] LexError),
    #[error("The query is empty")]
    Empty,
    #[error("Unknown field `{field}`. Known fields: {}", known_fields())]
    UnknownField { field: String, span: Span },
    #[error("Expected a field name, found `{found}`")]
    ExpectedField { found: String, span: Span },
    #[error("Expected an operator after `{field}`, such as `==` or `~=`")]
    ExpectedOperator { field: String, span: Span },
    #[error("Expected a value after `{operator}`")]
    ExpectedValue { operator: String, span: Span },
    #[error("Unclosed `(`")]
    UnclosedParen { span: Span },
    #[error("Unexpected `)` with no matching `(`")]
    UnmatchedCloseParen { span: Span },
    #[error("Unexpected trailing input after a complete query")]
    TrailingInput { span: Span },
    #[error("`{operator}` cannot be used on `{field}`, which is not an ordered field")]
    OperatorNotOrderable {
        operator: String,
        field: &'static str,
    },
}

fn known_fields() -> String {
    Field::all()
        .iter()
        .map(|field| field.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

impl ParseError {
    /// Where in the query text the problem is, when that is known.
    pub fn span(&self) -> Option<Span> {
        match self {
            ParseError::UnknownField { span, .. }
            | ParseError::ExpectedField { span, .. }
            | ParseError::ExpectedOperator { span, .. }
            | ParseError::ExpectedValue { span, .. }
            | ParseError::UnclosedParen { span }
            | ParseError::UnmatchedCloseParen { span }
            | ParseError::TrailingInput { span } => Some(*span),
            _ => None,
        }
    }
}

pub fn parse(input: &str) -> Result<Query, ParseError> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err(ParseError::Empty);
    }
    let mut parser = Parser { tokens, index: 0 };
    let query = parser.parse_or()?;
    if let Some(remaining) = parser.peek() {
        return Err(ParseError::TrailingInput {
            span: remaining.span,
        });
    }
    Ok(query)
}

struct Parser {
    tokens: Vec<SpannedToken>,
    index: usize,
}

impl Parser {
    fn peek(&self) -> Option<&SpannedToken> {
        self.tokens.get(self.index)
    }
    fn next(&mut self) -> Option<SpannedToken> {
        let token = self.tokens.get(self.index).cloned();
        if token.is_some() {
            self.index += 1;
        }
        token
    }
    /// The span just past the end, for errors about something that is missing rather than wrong.
    fn end_span(&self) -> Span {
        self.tokens
            .last()
            .map(|token| Span {
                start: token.span.end,
                end: token.span.end,
            })
            .unwrap_or(Span { start: 0, end: 0 })
    }

    fn parse_or(&mut self) -> Result<Query, ParseError> {
        let mut left = self.parse_and()?;
        while matches!(self.peek().map(|t| &t.token), Some(Token::Or)) {
            self.next();
            let right = self.parse_and()?;
            left = Query::or(left, right);
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Query, ParseError> {
        let mut left = self.parse_not()?;
        loop {
            match self.peek().map(|t| &t.token) {
                Some(Token::And) => {
                    self.next();
                }
                // Two comparisons side by side mean `and`. Writing it out every time is noise, and
                // both languages this is modelled on allow it.
                Some(Token::Ident(_)) | Some(Token::Not) | Some(Token::OpenParen) => {}
                _ => break,
            }
            let right = self.parse_not()?;
            left = Query::and(left, right);
        }
        Ok(left)
    }

    fn parse_not(&mut self) -> Result<Query, ParseError> {
        if matches!(self.peek().map(|t| &t.token), Some(Token::Not)) {
            self.next();
            let inner = self.parse_not()?;
            return Ok(Query::negate(inner));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Query, ParseError> {
        let Some(token) = self.next() else {
            return Err(ParseError::ExpectedField {
                found: "end of query".to_owned(),
                span: self.end_span(),
            });
        };
        match token.token {
            Token::OpenParen => {
                let inner = self.parse_or()?;
                match self.next() {
                    Some(SpannedToken {
                        token: Token::CloseParen,
                        ..
                    }) => Ok(inner),
                    _ => Err(ParseError::UnclosedParen { span: token.span }),
                }
            }
            Token::CloseParen => Err(ParseError::UnmatchedCloseParen { span: token.span }),
            Token::Ident(name) => {
                let Some(field) = Field::parse(&name) else {
                    return Err(ParseError::UnknownField {
                        field: name,
                        span: token.span,
                    });
                };
                self.parse_comparison(field, &name, token.span)
            }
            other => Err(ParseError::ExpectedField {
                found: describe(&other),
                span: token.span,
            }),
        }
    }

    fn parse_comparison(
        &mut self,
        field: Field,
        name: &str,
        field_span: Span,
    ) -> Result<Query, ParseError> {
        let Some(SpannedToken {
            token: Token::Operator(operator),
            span: operator_span,
        }) = self.next()
        else {
            return Err(ParseError::ExpectedOperator {
                field: name.to_owned(),
                span: field_span,
            });
        };

        // Ordering only means something for a timestamp. Allowing `>` on a name would compile to
        // a lexicographic comparison, which is almost never what was meant and never says so.
        if matches!(
            operator,
            Operator::GreaterThan
                | Operator::GreaterThanOrEqual
                | Operator::LessThan
                | Operator::LessThanOrEqual
        ) && !field.is_temporal()
        {
            return Err(ParseError::OperatorNotOrderable {
                operator: operator.to_string(),
                field: field.as_str(),
            });
        }

        let Some(value_token) = self.next() else {
            return Err(ParseError::ExpectedValue {
                operator: operator.to_string(),
                span: operator_span,
            });
        };
        let value = match value_token.token {
            Token::Ident(text) | Token::String(text) => Value::Text(text),
            Token::Number(number) => Value::Number(number),
            other => {
                return Err(ParseError::ExpectedValue {
                    operator: describe(&other),
                    span: value_token.span,
                });
            }
        };
        Ok(Query::Comparison {
            field,
            operator,
            value,
        })
    }
}

fn describe(token: &Token) -> String {
    match token {
        Token::Ident(value) => value.clone(),
        Token::String(value) => format!("\"{value}\""),
        Token::Number(value) => value.to_string(),
        Token::Operator(operator) => operator.to_string(),
        Token::And => "and".to_owned(),
        Token::Or => "or".to_owned(),
        Token::Not => "not".to_owned(),
        Token::OpenParen => "(".to_owned(),
        Token::CloseParen => ")".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn comparison(field: Field, operator: Operator, value: &str) -> Query {
        Query::Comparison {
            field,
            operator,
            value: Value::Text(value.to_owned()),
        }
    }

    #[test]
    fn parses_a_single_comparison() {
        assert_eq!(
            parse("version == 1.0.0").unwrap(),
            comparison(Field::Version, Operator::Equals, "1.0.0")
        );
    }

    /// `and` binds tighter than `or`, so this must not group left to right.
    #[test]
    fn and_binds_tighter_than_or() {
        let query = parse("name == a or name == b and name == c").unwrap();
        assert_eq!(
            query,
            Query::or(
                comparison(Field::Name, Operator::Equals, "a"),
                Query::and(
                    comparison(Field::Name, Operator::Equals, "b"),
                    comparison(Field::Name, Operator::Equals, "c"),
                )
            )
        );
    }

    #[test]
    fn parentheses_override_precedence() {
        let query = parse("(name == a or name == b) and name == c").unwrap();
        assert_eq!(
            query,
            Query::and(
                Query::or(
                    comparison(Field::Name, Operator::Equals, "a"),
                    comparison(Field::Name, Operator::Equals, "b"),
                ),
                comparison(Field::Name, Operator::Equals, "c"),
            )
        );
    }

    /// Two comparisons side by side mean `and`; writing it every time is noise.
    #[test]
    fn adjacent_comparisons_are_implicitly_and() {
        assert_eq!(
            parse("scope == dev.kingtux name == tms").unwrap(),
            parse("scope == dev.kingtux and name == tms").unwrap()
        );
    }

    #[test]
    fn not_applies_to_what_follows_it() {
        assert_eq!(
            parse("not version ~= *-SNAPSHOT").unwrap(),
            Query::negate(comparison(Field::Version, Operator::Matches, "*-SNAPSHOT"))
        );
        // `not` binds tighter than `and`.
        assert_eq!(
            parse("name == a and not name == b").unwrap(),
            Query::and(
                comparison(Field::Name, Operator::Equals, "a"),
                Query::negate(comparison(Field::Name, Operator::Equals, "b")),
            )
        );
    }

    #[test]
    fn ordering_is_only_allowed_on_timestamps() {
        assert!(parse("created > 2024").is_ok());
        assert!(parse("updated <= 2024").is_ok());
        // Lexicographic ordering on a name is almost never what was meant, and never says so.
        assert_eq!(
            parse("name > b"),
            Err(ParseError::OperatorNotOrderable {
                operator: ">".to_owned(),
                field: "name"
            })
        );
    }

    #[test]
    fn rejects_malformed_queries_with_a_position() {
        assert_eq!(parse(""), Err(ParseError::Empty));
        assert_eq!(parse("   "), Err(ParseError::Empty));

        let err = parse("nonsense == 1").unwrap_err();
        assert!(matches!(err, ParseError::UnknownField { .. }));
        // The message lists what a caller may actually use.
        assert!(err.to_string().contains("version"), "{err}");
        assert_eq!(err.span(), Some(Span { start: 0, end: 8 }));

        assert!(matches!(
            parse("version"),
            Err(ParseError::ExpectedOperator { .. })
        ));
        assert!(matches!(
            parse("version =="),
            Err(ParseError::ExpectedValue { .. })
        ));
        assert!(matches!(
            parse("(version == 1"),
            Err(ParseError::UnclosedParen { .. })
        ));
        assert!(matches!(
            parse("version == 1)"),
            Err(ParseError::TrailingInput { .. })
        ));
    }

    #[test]
    fn a_realistic_query_parses() {
        let query = parse(
            r#"repo == releases and scope == dev.kingtux and version ~= "1.*" and not release_type == Snapshot"#,
        );
        assert!(query.is_ok(), "{:?}", query.unwrap_err());
    }
}
