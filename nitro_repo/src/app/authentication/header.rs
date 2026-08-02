use nr_core::utils::base64_utils;
use tracing::{error, instrument};

use crate::utils::bad_request::{BadRequestErrors, InvalidAuthorizationHeader};

#[derive(Debug)]
pub enum AuthorizationHeader {
    Basic { username: String, password: String },
    Bearer { token: String },
    Session { session: String },
    Other { scheme: String, value: String },
}
impl TryFrom<String> for AuthorizationHeader {
    type Error = BadRequestErrors;
    #[instrument(skip(value), name = "AuthorizationHeader::try_from")]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.trim();
        if value.is_empty() {
            return Err(BadRequestErrors::InvalidAuthorizationHeader(
                InvalidAuthorizationHeader::InvalidFormat,
            ));
        }
        // Cargo sends the token on its own, with no scheme at all: `Authorization: <token>`. A
        // value with no space is therefore a bearer token rather than a malformed header — auth
        // tokens are 32 alphanumeric characters (see `auth_token::utils::generate_token`), so
        // there is nothing for it to be confused with.
        let Some((scheme, value)) = value.split_once(' ') else {
            return Ok(AuthorizationHeader::Bearer {
                token: value.to_owned(),
            });
        };
        // The value cannot be empty here: the whole header was trimmed above, so anything after a
        // space has at least one non-whitespace character in it.
        let value = value.trim_start();
        match scheme {
            "Basic" => parse_basic_header(value),
            "Bearer" => Ok(AuthorizationHeader::Bearer {
                token: value.to_owned(),
            }),
            "Session" => Ok(AuthorizationHeader::Session {
                session: value.to_owned(),
            }),
            _ => Ok(AuthorizationHeader::Other {
                scheme: scheme.to_owned(),
                value: value.to_owned(),
            }),
        }
    }
}
#[instrument(skip(header))]
fn parse_basic_header(header: &str) -> Result<AuthorizationHeader, BadRequestErrors> {
    let decoded = base64_utils::decode(header).map_err(|err| {
        error!("Failed to decode base64: {}", err);
        InvalidAuthorizationHeader::InvalidValue
    })?;
    let decoded = String::from_utf8(decoded).map_err(|err| {
        error!("Failed to convert bytes to string: {}", err);
        InvalidAuthorizationHeader::InvalidValue
    })?;
    // `split_once`, not `split` with a length check: only the *first* colon separates the two
    // fields, and everything after it is the password. A password containing a colon used to be
    // rejected outright, which is exactly what happens when a token is pasted into the password
    // field of `docker login`.
    let Some((username, password)) = decoded.split_once(':') else {
        return Err(InvalidAuthorizationHeader::InvalidBasicValue.into());
    };
    Ok(AuthorizationHeader::Basic {
        username: username.to_owned(),
        password: password.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use nr_core::utils::base64_utils;

    use super::AuthorizationHeader;

    fn parse(value: &str) -> Result<AuthorizationHeader, super::BadRequestErrors> {
        AuthorizationHeader::try_from(value.to_owned())
    }

    fn basic(username: &str, password: &str) -> String {
        format!(
            "Basic {}",
            base64_utils::encode(format!("{username}:{password}"))
        )
    }

    #[test]
    fn a_value_with_no_scheme_is_a_bearer_token() {
        // What `cargo` sends.
        let AuthorizationHeader::Bearer { token } = parse("abc123").unwrap() else {
            panic!("expected a bearer token");
        };
        assert_eq!(token, "abc123");
    }

    #[test]
    fn the_known_schemes_are_recognised() {
        assert!(matches!(
            parse("Bearer abc123").unwrap(),
            AuthorizationHeader::Bearer { token } if token == "abc123"
        ));
        assert!(matches!(
            parse("Session abc123").unwrap(),
            AuthorizationHeader::Session { session } if session == "abc123"
        ));
        assert!(matches!(
            parse("Digest abc123").unwrap(),
            AuthorizationHeader::Other { scheme, value } if scheme == "Digest" && value == "abc123"
        ));
    }

    #[test]
    fn a_basic_password_may_contain_a_colon() {
        let AuthorizationHeader::Basic { username, password } =
            parse(&basic("wyatt", "pass:with:colons")).unwrap()
        else {
            panic!("expected basic credentials");
        };
        assert_eq!(username, "wyatt");
        assert_eq!(password, "pass:with:colons");
    }

    #[test]
    fn a_basic_password_may_be_empty() {
        let AuthorizationHeader::Basic { username, password } = parse(&basic("wyatt", "")).unwrap()
        else {
            panic!("expected basic credentials");
        };
        assert_eq!(username, "wyatt");
        assert_eq!(password, "");
    }

    #[test]
    fn a_basic_value_with_no_colon_is_refused() {
        let value = format!("Basic {}", base64_utils::encode("no-colon-here"));
        assert!(parse(&value).is_err());
    }

    #[test]
    fn an_empty_header_is_refused() {
        for raw in ["", "   ", "\t"] {
            assert!(parse(raw).is_err(), "raw: {raw:?}");
        }
    }

    #[test]
    fn surrounding_whitespace_does_not_change_the_result() {
        assert!(matches!(
            parse("  Bearer   abc123  ").unwrap(),
            AuthorizationHeader::Bearer { token } if token == "abc123"
        ));
    }
}
