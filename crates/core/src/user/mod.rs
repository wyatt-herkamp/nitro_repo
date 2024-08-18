use std::{fmt::Display, str::FromStr};

use derive_more::derive::{AsRef, Deref, Into};
use serde::Serialize;
use sqlx::prelude::Type;
use thiserror::Error;
use tracing::instrument;

use crate::utils::validations;

pub mod permissions;
#[derive(Debug, Error)]
pub enum InvalidUsername {
    #[error("Username is too short, must be at least 3 got {0} characters")]
    TooShort(usize),
    #[error("Username is too long, must be less than 32 got {0} characters")]
    TooLong(usize),
    #[error("Username contains invalid character `{0}`. Usernames can only contain letters, numbers, `_`, and `-`")]
    InvalidCharacter(char),
}
#[derive(Debug, Type, Deref, AsRef, Clone, PartialEq, Eq, Into, Default)]
#[sqlx(transparent)]
#[as_ref(forward)]

pub struct Username(String);
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
impl Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

validations::from_impls!(Username, InvalidUsername);
#[derive(Debug, Error)]
pub enum InvalidEmail {
    #[error("Username is too short, must be at least 3 got {0} characters")]
    TooShort(usize),
    #[error("Username is too long, must be less than 32 got {0} characters")]
    TooLong(usize),
    #[error("Missing @ symbol in email")]
    MissingAt,
}
#[derive(Debug, Type, Deref, AsRef, Clone, PartialEq, Eq, Into, Default)]
#[as_ref(forward)]
#[sqlx(transparent)]

pub struct Email(String);
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

validations::from_impls!(Email, InvalidEmail);

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
