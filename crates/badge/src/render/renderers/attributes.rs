use crate::{badge::Links, render::util::xml::escape_xml};

const RAW_STRING_LEN: usize = 30;

pub fn render_attributes(links: &Links, accessible_text: &str) -> String {
    if !links.any() {
        let escaped = escape_xml(accessible_text);
        let mut buffer = String::with_capacity(RAW_STRING_LEN + escaped.len());

        #[cfg(debug_assertions)]
        let start_cap = buffer.capacity();

        buffer.push_str(r##"role="img" aria-label=""##);
        buffer.push_str(&escaped);
        buffer.push('"');

        #[cfg(debug_assertions)]
        assert_eq!(start_cap, buffer.capacity());

        buffer
    } else {
        "".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_attributes_test() {
        assert_eq!(
            render_attributes(
                &Links {
                    left: None,
                    right: None,
                },
                "access",
            ),
            r##"role="img" aria-label="access""##
        );

        assert_eq!(
            render_attributes(
                &Links {
                    left: None,
                    right: None,
                },
                r##"&'"<> he,lo"##,
            ),
            r##"role="img" aria-label="&amp;&apos;&quot;&lt;&gt; he,lo""##
        );
    }
}
