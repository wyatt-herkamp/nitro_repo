use chrono::FixedOffset;
use http::HeaderValue;

use crate::utils::bad_request::BadRequestErrors;

pub fn date_time_for_header(date_time: &chrono::DateTime<FixedOffset>) -> HeaderValue {
    let date_time = date_time.with_timezone(&chrono::Utc);
    let date_time = date_time.format("%a, %d %b %Y %H:%M:%S GMT").to_string();
    HeaderValue::from_str(date_time.as_str()).expect("Failed to convert date time to header")
}
pub fn parse_date_time(
    header_value: &HeaderValue,
) -> Result<chrono::DateTime<FixedOffset>, BadRequestErrors> {
    let date_time = header_value.to_str()?;
    chrono::DateTime::parse_from_rfc2822(date_time).map_err(BadRequestErrors::from)
}
