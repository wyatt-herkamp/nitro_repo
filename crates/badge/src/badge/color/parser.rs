use std::{num::ParseIntError, sync::LazyLock};

use regex::Regex;

use crate::{
    badge::color::{AliasColor::*, NamedColor::*, util::rgb_to_hex},
    error::{Error, Error::BadColorString},
};

pub(crate) fn parse_color(color: &str) -> Result<String, Error> {
    if let Some(color) = is_enum_color(color) {
        return Ok(color.to_string());
    }

    if let Some(color) = is_hex_pound(color) {
        return Ok(color.to_string());
    }

    if let Some(color) = is_hex_no_pound(color) {
        return Ok(format!("#{}", color));
    }

    if let Some((r, g, b)) = is_rgb(color) {
        return Ok(format!("#{}", rgb_to_hex(r, g, b)));
    }

    Err(BadColorString(color.to_string()))
}

fn is_enum_color(color: &str) -> Option<&str> {
    match color {
        "brightgreen" => Some(BrightGreen.hex()),
        "green" => Some(Green.hex()),
        "yellow" => Some(Yellow.hex()),
        "yellowgreen" => Some(YellowGreen.hex()),
        "orange" => Some(Orange.hex()),
        "red" => Some(Red.hex()),
        "blue" => Some(Blue.hex()),
        "grey" => Some(Grey.hex()),
        "lightgrey" => Some(LightGrey.hex()),
        "gray" => Some(Gray.hex()),
        "lightgray" => Some(LightGray.hex()),
        "critical" => Some(Critical.hex()),
        "important" => Some(Important.hex()),
        "success" => Some(Success.hex()),
        "informational" => Some(Informational.hex()),
        "inactive" => Some(Inactive.hex()),
        _ => None,
    }
}

fn is_hex_no_pound(color: &str) -> Option<&str> {
    static HEX_NO_POUND: LazyLock<Regex> =
        LazyLock::new(|| Regex::new("^(?:[0-9a-fA-F]{3}){1,2}$").unwrap());

    if HEX_NO_POUND.is_match(color) {
        Some(color)
    } else {
        None
    }
}
fn is_hex_pound(color: &str) -> Option<&str> {
    static HEX_POUND: LazyLock<Regex> =
        LazyLock::new(|| Regex::new("^#(?:[0-9a-fA-F]{3}){1,2}$").unwrap());

    if HEX_POUND.is_match(color) {
        Some(color)
    } else {
        None
    }
}

fn is_rgb(color: &str) -> Option<(u8, u8, u8)> {
    static RGB: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^rgb\((( *0*([1]?[0-9]?[0-9]|2[0-4][0-9]|25[0-5]) *),){2}( *0*([1]?[0-9]?[0-9]|2[0-4][0-9]|25[0-5]) *)\)$").unwrap()
    });
    static DIGITS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\d+").unwrap());
    if RGB.is_match(color) {
        let digits = DIGITS
            .find_iter(color)
            .map(|m| &color[m.start()..m.end()])
            .map(|m| m.parse::<u8>())
            .collect::<Result<Vec<u8>, ParseIntError>>()
            .ok();
        match digits {
            None => None,
            Some(vec) => {
                if vec.len() != 3 {
                    return None;
                }
                Some((vec[0], vec[1], vec[2]))
            }
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_hex_no_pound_test() {
        assert!(is_hex_no_pound("abcdef").is_some());
        assert!(is_hex_no_pound("abc").is_some());
        assert!(is_hex_no_pound("abcc").is_none());
        assert!(is_hex_no_pound("abccd").is_none());
        assert!(is_hex_no_pound("abccde").is_some());
        assert!(is_hex_no_pound("abcsde").is_none());
        assert!(is_hex_no_pound("#abcdef").is_none());
        assert!(is_hex_no_pound("#abc").is_none());
        assert!(is_hex_no_pound("#abcc").is_none());
        assert!(is_hex_no_pound("#abccd").is_none());
        assert!(is_hex_no_pound("#abccde").is_none());
        assert!(is_hex_no_pound("#abcsde").is_none());
    }

    #[test]
    fn is_hex_pound_test() {
        assert!(is_hex_pound("#abcdef").is_some());
        assert!(is_hex_pound("#abc").is_some());
        assert!(is_hex_pound("#abcc").is_none());
        assert!(is_hex_pound("#abccd").is_none());
        assert!(is_hex_pound("#abccde").is_some());
        assert!(is_hex_pound("#abcsde").is_none());
        assert!(is_hex_pound("abcdef").is_none());
        assert!(is_hex_pound("abc").is_none());
        assert!(is_hex_pound("abcc").is_none());
        assert!(is_hex_pound("abccd").is_none());
        assert!(is_hex_pound("abccde").is_none());
        assert!(is_hex_pound("abcsde").is_none());
    }

    #[test]
    fn is_rgb_test() {
        assert_eq!((10, 200, 50), is_rgb("rgb(10, 200, 50)").unwrap());
        assert_eq!((110, 0, 255), is_rgb("rgb(110, 0, 255)").unwrap());

        assert!(is_rgb("rgb(256, 200, 50)").is_none());
        assert!(is_rgb("rgb255, 200, 50)").is_none());
        assert!(is_rgb("rgb(255, 200, 50").is_none());
        assert!(is_rgb("rrgb(10,200,50)").is_none());
    }

    #[test]
    fn parse_color_test() {
        let oops = "oops".to_string();
        assert_eq!("#0ac832", parse_color("rgb(10, 200, 50)").unwrap());
        assert_eq!("#0ac832", parse_color("0ac832").unwrap());
        assert_eq!("#0ac832", parse_color("#0ac832").unwrap());
        assert_eq!("#4c1", parse_color("brightgreen").unwrap());
        assert_eq!("#007ec6", parse_color("informational").unwrap());
        assert_eq!("oops", parse_color("inrmational").unwrap_or(oops.clone()));
        assert_eq!("oops", parse_color("egb(30,30,30)").unwrap_or(oops.clone()));
        assert_eq!("oops", parse_color("egb(30,30,30)").unwrap_or(oops.clone()));
        assert_eq!("oops", parse_color("0acc").unwrap_or(oops.clone()));
        assert_eq!("oops", parse_color("0a").unwrap_or(oops.clone()));
        assert_eq!("oops", parse_color("0accc").unwrap_or(oops.clone()));
        assert_eq!("oops", parse_color("0acccac").unwrap_or(oops.clone()));
        assert_eq!("oops", parse_color("#0acc").unwrap_or(oops.clone()));
        assert_eq!("oops", parse_color("#0a").unwrap_or(oops.clone()));
        assert_eq!("oops", parse_color("#0accc").unwrap_or(oops.clone()));
        assert_eq!("oops", parse_color("#0acccac").unwrap_or(oops.clone()));
    }
}
