use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize, de::Visitor};
use tuxs_config_types::size_config::{ConfigSize, Unit as SizeUnit};

use super::ConfigError;

/// The maximum upload size for the web server.
///
/// If a number is provided it is assumed to be in bytes.
///
/// 'unlimited' will remove the limit.
///
/// Default is 100 MiB.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaxUpload {
    Limit(ConfigSize),
    Unlimited,
}
impl From<MaxUpload> for axum::extract::DefaultBodyLimit {
    fn from(value: MaxUpload) -> Self {
        match value {
            MaxUpload::Limit(size) => axum::extract::DefaultBodyLimit::max(size.size),
            MaxUpload::Unlimited => axum::extract::DefaultBodyLimit::disable(),
        }
    }
}
impl Default for MaxUpload {
    fn default() -> Self {
        MaxUpload::Limit(ConfigSize {
            size: 100,
            unit: SizeUnit::Mebibytes,
        })
    }
}

impl Serialize for MaxUpload {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            super::MaxUpload::Limit(size) => size.serialize(serializer),
            super::MaxUpload::Unlimited => serializer.serialize_str("unlimited"),
        }
    }
}
macro_rules! visit_num {
        (
            $fn_name:ident => $type:ty
        ) => {
            fn $fn_name<E>(self, v: $type) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(MaxUpload::from(v as usize))
            }
        };
        ($( $fn_name:ident => $type:ty ),*) => {
            $(
                visit_num!($fn_name => $type);
            )*
        }
    }
struct MaxUploadVisitor;

impl Visitor<'_> for MaxUploadVisitor {
    type Value = MaxUpload;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid size or 'unlimited'")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        MaxUpload::from_str(value).map_err(serde::de::Error::custom)
    }

    visit_num!(visit_i64 => i64, visit_i32 => i32, visit_i16 => i16, visit_i8 => i8, visit_u64 => u64, visit_u32 => u32, visit_u16 => u16, visit_u8 => u8);
}
impl<'de> Deserialize<'de> for MaxUpload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(MaxUploadVisitor)
    }
}

impl Display for MaxUpload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaxUpload::Limit(size) => write!(f, "{}", size),
            MaxUpload::Unlimited => write!(f, "Unlimited"),
        }
    }
}
impl FromStr for MaxUpload {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.to_lowercase() == "unlimited" {
            Ok(MaxUpload::Unlimited)
        } else {
            let limit_value =
                ConfigSize::from_str(s).map_err(|error| ConfigError::InvalidMaxUpload {
                    error,
                    value: s.to_owned(),
                })?;
            Ok(MaxUpload::Limit(limit_value))
        }
    }
}
impl TryFrom<&str> for MaxUpload {
    type Error = ConfigError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        MaxUpload::from_str(value)
    }
}
impl TryFrom<String> for MaxUpload {
    type Error = ConfigError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        MaxUpload::from_str(value.as_str())
    }
}
impl From<usize> for MaxUpload {
    fn from(value: usize) -> Self {
        MaxUpload::Limit(ConfigSize {
            size: value,
            unit: Default::default(),
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_max_upload_from_str() {
        {
            let max_upload = MaxUpload::from_str("100").unwrap();
            assert_eq!(
                max_upload,
                MaxUpload::Limit(ConfigSize {
                    size: 100,
                    unit: SizeUnit::Bytes,
                })
            );
        }
        {
            let max_upload = MaxUpload::from_str("100b").unwrap();
            assert_eq!(
                max_upload,
                MaxUpload::Limit(ConfigSize {
                    size: 100,
                    unit: SizeUnit::Bytes,
                })
            );
        }
        {
            let max_upload = MaxUpload::from_str("100KiB").unwrap();
            assert_eq!(
                max_upload,
                MaxUpload::Limit(ConfigSize {
                    size: 100,
                    unit: SizeUnit::Kibibytes,
                })
            );
        }
        {
            let max_upload = MaxUpload::from_str("100MiB").unwrap();
            assert_eq!(
                max_upload,
                MaxUpload::Limit(ConfigSize {
                    size: 100,
                    unit: SizeUnit::Mebibytes,
                })
            );
        }
    }
    #[test]
    fn test_unlimited() {
        {
            let max_upload = MaxUpload::from_str("unlimited").unwrap();
            assert_eq!(max_upload, MaxUpload::Unlimited);
        }
        {
            let max_upload = MaxUpload::from_str("Unlimited").unwrap();
            assert_eq!(max_upload, MaxUpload::Unlimited);
        }
    }
}
