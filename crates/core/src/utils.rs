pub mod time;
pub mod utopia;
pub mod base64_utils {
    use base64::{DecodeError, Engine, engine::general_purpose::STANDARD};
    use tracing::instrument;
    #[instrument(skip(input), name = "base64_utils::decode")]
    #[inline(always)]
    pub fn decode(input: impl AsRef<[u8]>) -> Result<Vec<u8>, DecodeError> {
        STANDARD.decode(input)
    }

    #[inline(always)]
    pub fn encode(input: impl AsRef<[u8]>) -> String {
        STANDARD.encode(input)
    }
    #[inline(always)]
    pub fn encode_basic_header(username: impl AsRef<str>, password: impl AsRef<str>) -> String {
        STANDARD.encode(format!("{}:{}", username.as_ref(), password.as_ref()))
    }
    pub mod serde_base64 {
        use serde::{Deserialize, Serialize};

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let string = String::deserialize(deserializer)?;
            super::decode(string).map_err(serde::de::Error::custom)
        }
        pub fn serialize<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            super::encode(data).serialize(serializer)
        }
    }
}
pub mod sha256 {
    use sha2::Digest;

    use crate::utils::base64_utils;

    #[inline(always)]
    pub fn encode_to_string(input: impl AsRef<[u8]>) -> String {
        let mut hasher = sha2::Sha256::new();
        hasher.update(input);
        let result = hasher.finalize();
        base64_utils::encode(result)
    }
}
pub mod duration_serde {
    pub mod as_seconds {
        use chrono::Duration;
        use serde::{Deserialize, Serialize, Serializer};
        pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            duration.num_seconds().serialize(serializer)
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let seconds = i64::deserialize(deserializer)?;
            Ok(Duration::seconds(seconds))
        }
    }

    pub mod as_days {
        use chrono::Duration;
        use serde::{Deserialize, Serialize, Serializer};
        pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            duration.num_days().serialize(serializer)
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let days = i64::deserialize(deserializer)?;
            Ok(Duration::days(days))
        }
    }
}
pub mod validations {
    /// Accepted Characters for URL Safe Strings
    ///
    /// - A-Z
    /// - a-z
    /// - 0-9
    /// - _
    /// - -
    ///
    #[inline(always)]
    pub fn valid_name_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '-'
    }
    pub fn valid_name_string(s: &str) -> bool {
        s.chars().all(valid_name_char)
    }

    macro_rules! schema_for_new_type_str {
        ($ty:ty) => {
            impl utoipa::ToSchema for $ty {
                fn name() -> std::borrow::Cow<'static, str> {
                    std::borrow::Cow::Borrowed(stringify!($ty))
                }
            }
        };
        (
            $ty:ty, format = $format:ident
        ) => {
            impl utoipa::PartialSchema for $ty {
                fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
                    utoipa::openapi::ObjectBuilder::new()
                        .schema_type(utoipa::openapi::schema::SchemaType::new(
                            utoipa::openapi::schema::Type::String,
                        ))
                        .format(Some(utoipa::openapi::SchemaFormat::KnownFormat(
                            utoipa::openapi::KnownFormat::$format,
                        )))
                        .into()
                }
            }

            crate::utils::validations::schema_for_new_type_str!($ty);
        };
        (
            $ty:ty, pattern = $pattern:literal
        ) => {
            impl utoipa::PartialSchema for $ty {
                fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
                    utoipa::openapi::ObjectBuilder::new()
                        .schema_type(utoipa::openapi::schema::SchemaType::new(
                            utoipa::openapi::schema::Type::String,
                        ))
                        .pattern(Some($pattern))
                        .into()
                }
            }
            crate::utils::validations::schema_for_new_type_str!($ty);
        };
    }

    macro_rules! convert_traits_to_new {
        ($f:ty, $error:ty) => {
            const _: () = {
                impl std::convert::TryFrom<String> for $f {
                    type Error = $error;
                    fn try_from(value: String) -> Result<Self, Self::Error> {
                        Self::new(value)
                    }
                }
                impl std::convert::TryFrom<&str> for $f {
                    type Error = $error;
                    fn try_from(value: &str) -> Result<Self, Self::Error> {
                        Self::new(value.to_owned())
                    }
                }
                impl std::str::FromStr for $f {
                    type Err = $error;
                    fn from_str(s: &str) -> Result<Self, Self::Err> {
                        Self::new(s.to_owned())
                    }
                }
            };
        };
    }
    pub(crate) use convert_traits_to_new;
    pub(crate) use schema_for_new_type_str;

    macro_rules! test_validations {
        (
            mod $mod_name:ident for $f:ty {
                valid: [
                    $($valid:literal),*
                ],
                invalid: [
                    $($invalid:literal),*
                ]
            }
        ) => {
            #[cfg(test)]
            mod $mod_name {
                use super::*;
                #[test]
                fn test_valid() {
                    $(
                        assert!(<$f>::new($valid.to_owned()).is_ok());
                    )*
                }
                #[test]
                fn test_invalid() {
                    $(
                        assert!(<$f>::new($invalid.to_owned()).is_err());
                    )*
                }
            }
        };
    }
    pub(crate) use test_validations;
}
