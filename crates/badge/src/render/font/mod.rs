mod char_datum;
mod util;

use std::sync::LazyLock;

use crate::render::font::{char_datum::CharDatum, util::is_control_char};

/// Only Verdana 11px is carried, because it is the only font the implemented styles use. Upstream
/// shipped tables for Helvetica 11px bold and Verdana 10px too; they were already dead when this
/// was vendored, and the unused ones are not in the tree.
static FONTS: LazyLock<[Vec<CharDatum>; 1]> = LazyLock::new(|| {
    let verd11 = include_bytes!("../../../fonts/verdana-11px-normal.bincode");
    [bincode::deserialize(verd11).expect("Bundled font table is not valid bincode")]
});
pub enum Font {
    Verdana11Px,
}

impl Font {
    pub fn width_of_str(&self, s: &str) -> f32 {
        let dict = self.char_dict();

        s.chars()
            .map(|c| Font::width_of_char_code(c as u32, dict))
            .sum::<f32>()
    }

    fn width_of_char_code(c: u32, dict: &[CharDatum]) -> f32 {
        if is_control_char(c) {
            return 0.0;
        }
        match dict.binary_search_by_key(&c, |datum| datum.low) {
            Ok(idx) => dict[idx].width,
            Err(idx) => {
                let datum = &dict[idx - 1];
                if datum.contains(c) {
                    datum.width
                } else {
                    Font::width_of_char_code('m' as u32, dict)
                }
            }
        }
    }

    fn char_dict(&self) -> &[CharDatum] {
        match self {
            Font::Verdana11Px => &FONTS[0],
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::render::font::Font::Verdana11Px;

    #[test]
    fn font_test() {
        assert_eq!(Verdana11Px.width_of_str("crates"), 33.64);
        assert_eq!(Verdana11Px.width_of_str(&(207 as char).to_string()), 4.63);
    }
}
