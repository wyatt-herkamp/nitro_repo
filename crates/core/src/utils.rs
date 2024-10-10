pub mod time;
pub mod utopia;
pub mod base64_utils {
    use base64::{engine::general_purpose::STANDARD, DecodeError, Engine};
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
    #[inline(always)]
    pub fn valid_name_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '-'
    }
    pub fn valid_name_string(s: &str) -> bool {
        s.chars().all(valid_name_char)
    }
    macro_rules! from_impls {
        ($f:ty, $error:ty) => {
            impl serde::Serialize for $f {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    self.0.serialize(serializer)
                }
            }
            impl<'de> serde::Deserialize<'de> for $f {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    String::deserialize(deserializer)
                        .and_then(|s| Self::new(s).map_err(|e| serde::de::Error::custom(e)))
                }
            }
            impl TryFrom<String> for $f {
                type Error = $error;
                fn try_from(value: String) -> Result<Self, Self::Error> {
                    Self::new(value)
                }
            }
            impl TryFrom<&str> for $f {
                type Error = $error;
                fn try_from(value: &str) -> Result<Self, Self::Error> {
                    Self::new(value.to_string())
                }
            }
            impl std::str::FromStr for $f {
                type Err = $error;
                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    Self::new(s.to_string())
                }
            }
        };
    }
    pub(crate) use from_impls;
}
