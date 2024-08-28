pub mod iso_8601 {
    use chrono::{DateTime, FixedOffset};
    use serde::{Deserialize, Serialize};

    pub static ISO_8601: &str = "%Y-%m-%dT%H:%M:%S.%f";
    pub fn serialize<S>(time: &DateTime<FixedOffset>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        to_string(time).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<FixedOffset>, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        DateTime::parse_from_str(&s, ISO_8601).map_err(serde::de::Error::custom)
    }
    pub fn to_string(time: &DateTime<FixedOffset>) -> String {
        time.format(ISO_8601).to_string()
    }
    pub fn from_string(s: &str) -> Result<DateTime<FixedOffset>, chrono::ParseError> {
        DateTime::<FixedOffset>::parse_from_rfc3339(s)
    }

    #[cfg(test)]
    mod tests {
        #[test]
        pub fn test() {
            let from = super::from_string("2024-08-28T00:09:11.230Z").unwrap();
            println!("{:?}", from);
        }
    }
}
