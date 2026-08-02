use crate::{
    badge::{
        DEFAULT_LABEL_COLOR, DEFAULT_MESSAGE_COLOR,
        color::{Color, parser::parse_color},
    },
    error::Error,
};

pub(crate) fn get_color_from_option(
    is_label: bool,
    color_string: Option<String>,
    color_enum: Option<&Color>,
) -> Result<String, Error> {
    if let Some(color) = color_enum {
        Ok(match color {
            Color::Named(n) => n.hex().to_string(),
            Color::Alias(a) => a.hex().to_string(),
            &Color::Rgb(r, g, b) => {
                format!("#{}", rgb_to_hex(r, g, b))
            }
        })
    } else if let Some(color) = color_string {
        parse_color(&color)
    } else {
        Ok(if is_label {
            DEFAULT_LABEL_COLOR.hex().to_string()
        } else {
            DEFAULT_MESSAGE_COLOR.hex().to_string()
        })
    }
}

pub(crate) fn brightness(color: &str) -> f32 {
    let (r, g, b) = hex_color_to_rgb(&color[1..]);
    (r * 299 + g * 587 + b * 114) as f32 / 255000f32
}

fn hex_to_u32_one(c: &char) -> u32 {
    match c {
        '0' => 0,
        '1' => 1,
        '2' => 2,
        '3' => 3,
        '4' => 4,
        '5' => 5,
        '6' => 6,
        '7' => 7,
        '8' => 8,
        '9' => 9,
        'a' => 10,
        'b' => 11,
        'c' => 12,
        'd' => 13,
        'e' => 14,
        'f' => 15,
        _ => panic!("unknown digit"),
    }
}

fn u4_to_hexit(u: u8) -> char {
    match u {
        0 => '0',
        1 => '1',
        2 => '2',
        3 => '3',
        4 => '4',
        5 => '5',
        6 => '6',
        7 => '7',
        8 => '8',
        9 => '9',
        10 => 'a',
        11 => 'b',
        12 => 'c',
        13 => 'd',
        14 => 'e',
        15 => 'f',
        _ => panic!("value greater than 15"),
    }
}

pub(crate) fn u64_to_hex(num: u64) -> String {
    let mut hex = String::with_capacity(16); // todo bench capacity

    for i in (0..16).rev() {
        hex.push(u4_to_hexit(((num >> (i * 4)) & 0xf) as u8))
    }

    hex
}

pub(crate) fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
    let mut color = String::with_capacity(10); // todo bench capacity
    color.push(u4_to_hexit(r >> 4));
    color.push(u4_to_hexit(r & 0xf));
    color.push(u4_to_hexit(g >> 4));
    color.push(u4_to_hexit(g & 0xf));
    color.push(u4_to_hexit(b >> 4));
    color.push(u4_to_hexit(b & 0xf));
    color
}

fn hex_to_u32(s: &str) -> u32 {
    let mut num = 0;
    for c in s.chars() {
        num = num << 4 | hex_to_u32_one(&c);
    }
    num
}

pub(crate) fn hex_color_to_rgb(hex: &str) -> (u32, u32, u32) {
    match hex.len() {
        3 => {
            let r = hex_to_u32(&hex[0..1]);
            let g = hex_to_u32(&hex[1..2]);
            let b = hex_to_u32(&hex[2..3]);

            let r = r << 4 | r;
            let g = g << 4 | g;
            let b = b << 4 | b;
            (r, g, b)
        }
        6 => {
            let r = hex_to_u32(&hex[0..2]);
            let g = hex_to_u32(&hex[2..4]);
            let b = hex_to_u32(&hex[4..6]);
            (r, g, b)
        }
        _ => panic!("unknown hex length"),
    }
}

#[cfg(test)]
mod tests {
    use crate::badge::color::util::{
        brightness, hex_color_to_rgb, hex_to_u32_one, rgb_to_hex, u4_to_hexit,
    };

    #[test]
    fn rgb_to_hex_test() {
        assert_eq!(rgb_to_hex(10, 200, 50), "0ac832")
    }

    #[test]
    fn brightness_test() {
        assert_eq!(brightness("#c33"), 0.3794f32);
        assert_eq!(brightness("#afde32"), 0.73858434f32);
    }

    #[test]
    fn hex_color_tests() {
        assert_eq!(hex_color_to_rgb("0ac832"), (10, 200, 50));
    }

    #[test]
    fn rgb_to_hex_string_test() {
        assert_eq!("0ac832", rgb_to_hex(10, 200, 50));
    }

    #[test]
    fn hex_to_u32_one_test() {
        for (char, num) in ('0'..='9').zip(0..=9) {
            assert_eq!(hex_to_u32_one(&char), num);
        }
        for (char, num) in ('a'..='f').zip(10..=15) {
            assert_eq!(hex_to_u32_one(&char), num);
        }
    }

    #[test]
    fn u4_to_hexit_test() {
        for (char, num) in ('0'..='9').zip(0..=9) {
            assert_eq!(u4_to_hexit(num), char);
        }
        for (char, num) in ('a'..='f').zip(10..=15) {
            assert_eq!(u4_to_hexit(num), char);
        }
    }
}
