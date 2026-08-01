//! Turns query text into tokens.
//!
//! Kept separate from the parser so a malformed query can be reported against the exact byte it
//! went wrong at, rather than "syntax error" for the whole string.
use std::fmt;

use crate::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// A bare word: a field name, or an unquoted value.
    Ident(String),
    /// A quoted string. Quoting is what lets a value contain spaces, operators, or a leading
    /// digit without being mistaken for something else.
    String(String),
    Number(i64),
    Operator(Operator),
    And,
    Or,
    Not,
    OpenParen,
    CloseParen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Equals,
    NotEquals,
    /// Glob match, `~=`. `*` stands for any run of characters and `?` for one.
    Matches,
    NotMatches,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Operator::Equals => "==",
            Operator::NotEquals => "!=",
            Operator::Matches => "~=",
            Operator::NotMatches => "!~",
            Operator::GreaterThan => ">",
            Operator::GreaterThanOrEqual => ">=",
            Operator::LessThan => "<",
            Operator::LessThanOrEqual => "<=",
        };
        f.write_str(text)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LexError {
    #[error("Unterminated string starting at character {0}")]
    UnterminatedString(usize),
    #[error("Unexpected character `{character}` at {position}")]
    UnexpectedCharacter { character: char, position: usize },
    #[error("`{found}` is not an operator; did you mean `{suggestion}`?")]
    IncompleteOperator {
        found: String,
        suggestion: &'static str,
    },
}

/// Splits query text into tokens.
pub fn tokenize(input: &str) -> Result<Vec<SpannedToken>, LexError> {
    let characters: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < characters.len() {
        let start = index;
        let character = characters[index];

        if character.is_whitespace() {
            index += 1;
            continue;
        }

        let token = match character {
            '(' => {
                index += 1;
                Token::OpenParen
            }
            ')' => {
                index += 1;
                Token::CloseParen
            }
            '\'' | '"' => {
                let quote = character;
                index += 1;
                let mut value = String::new();
                loop {
                    let Some(&next) = characters.get(index) else {
                        return Err(LexError::UnterminatedString(start));
                    };
                    index += 1;
                    if next == '\\' {
                        // An escape lets a quote appear inside a quoted value. Anything else after
                        // a backslash is taken literally, which keeps Windows paths usable.
                        match characters.get(index) {
                            Some(&escaped) => {
                                value.push(escaped);
                                index += 1;
                            }
                            None => return Err(LexError::UnterminatedString(start)),
                        }
                        continue;
                    }
                    if next == quote {
                        break;
                    }
                    value.push(next);
                }
                Token::String(value)
            }
            '=' | '!' | '~' | '>' | '<' => {
                let next = characters.get(index + 1).copied();
                let (operator, length) = match (character, next) {
                    ('=', Some('=')) => (Operator::Equals, 2),
                    ('!', Some('=')) => (Operator::NotEquals, 2),
                    ('~', Some('=')) => (Operator::Matches, 2),
                    ('!', Some('~')) => (Operator::NotMatches, 2),
                    ('>', Some('=')) => (Operator::GreaterThanOrEqual, 2),
                    ('<', Some('=')) => (Operator::LessThanOrEqual, 2),
                    ('>', _) => (Operator::GreaterThan, 1),
                    ('<', _) => (Operator::LessThan, 1),
                    // A single `=` is the most common way to get this wrong, so it is worth
                    // naming rather than reporting as an unexpected character.
                    ('=', _) => {
                        return Err(LexError::IncompleteOperator {
                            found: "=".to_owned(),
                            suggestion: "==",
                        });
                    }
                    ('!', _) => {
                        return Err(LexError::IncompleteOperator {
                            found: "!".to_owned(),
                            suggestion: "!=",
                        });
                    }
                    ('~', _) => {
                        return Err(LexError::IncompleteOperator {
                            found: "~".to_owned(),
                            suggestion: "~=",
                        });
                    }
                    _ => unreachable!("outer match restricts the character"),
                };
                index += length;
                Token::Operator(operator)
            }
            character if is_ident_start(character) => {
                let mut value = String::new();
                while let Some(&next) = characters.get(index) {
                    if !is_ident_char(next) {
                        break;
                    }
                    value.push(next);
                    index += 1;
                }
                // Keywords are matched case-insensitively so `AND` and `and` both read naturally.
                match value.to_ascii_lowercase().as_str() {
                    "and" => Token::And,
                    "or" => Token::Or,
                    "not" => Token::Not,
                    _ => Token::Ident(value),
                }
            }
            // A value starting with a digit is far more often a version (`1.0.0`) or a date
            // (`2024-01-01`) than a number. Consuming only digits here would stop at the first
            // `.` and report it as an unexpected character, so the whole run is taken and only
            // then judged: if all of it is an integer it is a number, otherwise it is text.
            character if character.is_ascii_digit() || character == '-' => {
                let mut value = String::new();
                if character == '-' {
                    value.push('-');
                    index += 1;
                }
                while let Some(&next) = characters.get(index) {
                    if !is_ident_char(next) {
                        break;
                    }
                    value.push(next);
                    index += 1;
                }
                match value.parse::<i64>() {
                    Ok(number) => Token::Number(number),
                    Err(_) => Token::Ident(value),
                }
            }
            character => {
                return Err(LexError::UnexpectedCharacter {
                    character,
                    position: index,
                });
            }
        };
        tokens.push(SpannedToken {
            token,
            span: Span { start, end: index },
        });
    }
    Ok(tokens)
}

fn is_ident_start(character: char) -> bool {
    character.is_alphabetic() || character == '_' || character == '*'
}

/// What may appear inside an unquoted word.
///
/// Deliberately wide: an unquoted value is often a version (`1.0.0-SNAPSHOT`), a coordinate
/// (`dev.kingtux:tms`) or a glob (`*.jar`), and forcing quotes around all of those would make the
/// common query the awkward one.
fn is_ident_char(character: char) -> bool {
    character.is_alphanumeric()
        || matches!(
            character,
            '_' | '-' | '.' | '*' | '?' | '/' | ':' | '@' | '+' | '~'
        )
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn tokens(input: &str) -> Vec<Token> {
        tokenize(input)
            .unwrap_or_else(|err| panic!("`{input}` failed to tokenize: {err}"))
            .into_iter()
            .map(|spanned| spanned.token)
            .collect()
    }

    #[test]
    fn reads_a_simple_comparison() {
        assert_eq!(
            tokens("version == 1.0.0"),
            vec![
                Token::Ident("version".to_owned()),
                Token::Operator(Operator::Equals),
                Token::Ident("1.0.0".to_owned()),
            ]
        );
    }

    #[test]
    fn reads_every_operator() {
        for (text, expected) in [
            ("==", Operator::Equals),
            ("!=", Operator::NotEquals),
            ("~=", Operator::Matches),
            ("!~", Operator::NotMatches),
            (">", Operator::GreaterThan),
            (">=", Operator::GreaterThanOrEqual),
            ("<", Operator::LessThan),
            ("<=", Operator::LessThanOrEqual),
        ] {
            assert_eq!(
                tokens(&format!("a {text} b"))[1],
                Token::Operator(expected),
                "`{text}` did not lex"
            );
        }
    }

    #[test]
    fn keywords_are_case_insensitive() {
        assert_eq!(
            tokens("a == b AND c == d Or Not e == f"),
            vec![
                Token::Ident("a".to_owned()),
                Token::Operator(Operator::Equals),
                Token::Ident("b".to_owned()),
                Token::And,
                Token::Ident("c".to_owned()),
                Token::Operator(Operator::Equals),
                Token::Ident("d".to_owned()),
                Token::Or,
                Token::Not,
                Token::Ident("e".to_owned()),
                Token::Operator(Operator::Equals),
                Token::Ident("f".to_owned()),
            ]
        );
    }

    /// A coordinate, a version and a glob all have to survive without quotes, or the common query
    /// becomes the awkward one.
    #[test]
    fn unquoted_values_keep_their_punctuation() {
        assert_eq!(
            tokens("project == dev.kingtux:tms"),
            vec![
                Token::Ident("project".to_owned()),
                Token::Operator(Operator::Equals),
                Token::Ident("dev.kingtux:tms".to_owned()),
            ]
        );
        assert_eq!(tokens("name ~= *.jar")[2], Token::Ident("*.jar".to_owned()));
        assert_eq!(
            tokens("version == 1.0.0-SNAPSHOT")[2],
            Token::Ident("1.0.0-SNAPSHOT".to_owned())
        );
    }

    #[test]
    fn quoted_strings_may_contain_anything() {
        assert_eq!(
            tokens(r#"name == "a value with spaces and == in it""#)[2],
            Token::String("a value with spaces and == in it".to_owned())
        );
        assert_eq!(
            tokens(r#"name == "escaped \" quote""#)[2],
            Token::String("escaped \" quote".to_owned())
        );
        assert_eq!(
            tokens("name == 'single'")[2],
            Token::String("single".to_owned())
        );
    }

    #[test]
    fn numbers_are_numbers() {
        assert_eq!(tokens("downloads > 100")[2], Token::Number(100));
        assert_eq!(tokens("value == -5")[2], Token::Number(-5));
    }

    #[test]
    fn reports_where_a_query_went_wrong() {
        assert_eq!(
            tokenize(r#"name == "unterminated"#),
            Err(LexError::UnterminatedString(8))
        );
        assert_eq!(
            tokenize("name = value"),
            Err(LexError::IncompleteOperator {
                found: "=".to_owned(),
                suggestion: "=="
            })
        );
        assert_eq!(
            tokenize("name == value & other"),
            Err(LexError::UnexpectedCharacter {
                character: '&',
                position: 14
            })
        );
    }

    #[test]
    fn spans_point_at_the_source() {
        let tokens = tokenize("version == 1.0.0").unwrap();
        assert_eq!(tokens[0].span, Span { start: 0, end: 7 });
        assert_eq!(tokens[1].span, Span { start: 8, end: 10 });
        assert_eq!(tokens[2].span, Span { start: 11, end: 16 });
    }
}
