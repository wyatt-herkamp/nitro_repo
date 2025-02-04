use digestible::Digestible;
use nr_macros::{NuType, SerdeViaStr};
use sqlx::prelude::Type;
use thiserror::Error;
use tracing::instrument;
pub mod scopes;
pub mod token;
use crate::utils::validations::{
    self, convert_traits_to_new, schema_for_new_type_str, test_validations,
};

pub mod permissions;
#[derive(Debug, Error)]
pub enum InvalidUsername {
    #[error("Username is too short, must be at least 3 got {0} characters")]
    TooShort(usize),
    #[error("Username is too long, must be less than 32 got {0} characters")]
    TooLong(usize),
    #[error(
        "Username contains invalid character `{0}`. Usernames can only contain letters, numbers, `_`, and `-`"
    )]
    InvalidCharacter(char),
}
#[derive(Debug, Type, Clone, Digestible, NuType, SerdeViaStr)]
#[sqlx(transparent)]
pub struct Username(String);
convert_traits_to_new!(Username, InvalidUsername);
schema_for_new_type_str!(Username, pattern = r#"^([a-zA-Z0-9_\-]{3,32}$)"#);
impl Username {
    #[instrument(name = "Username::new")]
    pub fn new(username: String) -> Result<Self, InvalidUsername> {
        if username.len() < 3 {
            return Err(InvalidUsername::TooShort(username.len()));
        }
        if username.len() > 32 {
            return Err(InvalidUsername::TooLong(username.len()));
        }
        if let Some(bad_char) = username.chars().find(|c| !validations::valid_name_char(*c)) {
            return Err(InvalidUsername::InvalidCharacter(bad_char));
        }
        Ok(Self(username))
    }
}
test_validations! {
    mod username_tests for Username {
        valid: [
            "test",
            "test-123",
            "test_123",
            "test-123_",
            "test_123-",
            "test_123-abc",
            "test_123-abc_"
        ],
        invalid: [
            "t e",
            "t"
        ]
    }
}

#[derive(Debug, Error)]
pub enum InvalidEmail {
    #[error("Username is too short, must be at least 3 got {0} characters")]
    TooShort(usize),
    #[error("Username is too long, must be less than 32 got {0} characters")]
    TooLong(usize),
    #[error("Missing @ symbol in email")]
    MissingAt,
}
#[derive(Debug, Type, Clone, NuType, SerdeViaStr)]
#[sqlx(transparent)]

pub struct Email(String);
validations::convert_traits_to_new!(Email, InvalidEmail);
impl Email {
    #[instrument(name = "Email::new")]
    pub fn new(email: String) -> Result<Self, InvalidEmail> {
        if email.len() < 3 {
            return Err(InvalidEmail::TooShort(email.len()));
        }
        if email.len() > 32 {
            return Err(InvalidEmail::TooLong(email.len()));
        }
        if !email.contains('@') {
            return Err(InvalidEmail::MissingAt);
        }
        Ok(Self(email))
    }
}
schema_for_new_type_str!(Email, format = Email);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username() {
        let username = Username::new("test".to_string()).unwrap();
        assert_eq!(username.to_string(), "test");
        assert!(Username::new("te".to_string()).is_err());
        assert!(Username::new("testtesttesttesttesttesttesttesttest".to_string()).is_err());
        assert!(Username::new("test$".to_string()).is_err());
    }
}
