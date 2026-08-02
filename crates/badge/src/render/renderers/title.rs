use crate::{badge::Links, render::util::xml::escape_xml};

const RAW_STRING_LEN: usize = 15;

pub fn render_title(links: &Links, accessible_text: &str) -> String {
    if links.any() {
        "".to_string()
    } else {
        let accessible_text = escape_xml(accessible_text);
        let mut buffer = String::with_capacity(RAW_STRING_LEN + accessible_text.len());

        #[cfg(debug_assertions)]
        let start_cap = buffer.capacity();

        buffer.push_str(r#"<title>"#);
        buffer.push_str(&accessible_text);
        buffer.push_str(r#"</title>"#);

        #[cfg(debug_assertions)]
        assert_eq!(start_cap, buffer.capacity());

        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_title_test() {
        assert_eq!(
            render_title(
                &Links {
                    left: None,
                    right: None,
                },
                "hel&lo!",
            ),
            "<title>hel&amp;lo!</title>"
        );

        assert_eq!(
            render_title(
                &Links {
                    left: Some("link".to_string()),
                    right: None,
                },
                "hel&lo!",
            ),
            ""
        );
    }
}
