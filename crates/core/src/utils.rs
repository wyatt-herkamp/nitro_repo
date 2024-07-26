pub mod base64_utils {
    use base64::{engine::general_purpose::STANDARD, DecodeError, Engine};
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
