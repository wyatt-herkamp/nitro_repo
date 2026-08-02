use std::fmt::Write;

use itoa::Buffer;

use crate::{
    badge::Links,
    render::{
        renderers::{render_attributes, render_title},
        util::xml::strip_xml_whitespace,
    },
};

pub struct RenderBadgeConfig<'a> {
    pub left_width: usize,
    pub right_width: usize,
    pub height: usize,
    pub accessible_text: &'a str,
    pub links: &'a Links,
}

// pub fn render_badge_old(config: RenderBadgeConfig, main: &str) -> String {
//   let width = config.left_width + config.right_width;
//
//   let main = if let Some(left_link) = config.links.single() {
//     format!(
//       r#"<a target="_blank" xlink:href="{left_link}">{main}</a>"#,
//       main = main,
//       left_link = left_link
//     )
//   } else {
//     main.to_string()
//   };
//
//   let badge = format!(
//     r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{width}" height="{height}" {attributes}>
//       {title}{main}
//       </svg>"#,
//     width = width,
//     height = config.height,
//     attributes = render_attributes(config.links, config.accessible_text),
//     title = render_title(config.links, config.accessible_text),
//     main = main
//   );
//
//   strip_xml_whitespace(&badge)
// }

const NO_LINK_LEN: usize = 114;
const HEIGHT_AND_WIDTH_LEN: usize = 10;
const LINK_LEN: usize = 37;

pub fn render_badge(config: RenderBadgeConfig, main: &str) -> String {
    let width = config.left_width + config.right_width;
    let link = config.links.single().unwrap_or("");
    let link_len = if !link.is_empty() {
        LINK_LEN + link.len()
    } else {
        0
    };

    let attributes = render_attributes(config.links, config.accessible_text);
    let title = render_title(config.links, config.accessible_text);

    let mut buffer = String::with_capacity(
        HEIGHT_AND_WIDTH_LEN + NO_LINK_LEN + link_len + main.len() + attributes.len() + title.len(),
    );

    let mut itoa_buffer = Buffer::new();
    #[cfg(debug_assertions)]
    let start_cap = buffer.capacity();
    buffer.push_str(r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width=""##);

    buffer.write_str(itoa_buffer.format(width)).unwrap();
    buffer.push_str(r#"" height=""#);
    buffer.write_str(itoa_buffer.format(config.height)).unwrap();
    buffer.push_str(r#"" "#);

    buffer.push_str(&attributes);
    buffer.push('>');
    buffer.push_str(&title);

    if link_len != 0 {
        buffer.push_str(r##"<a target="_blank" xlink:href=""##);
        buffer.push_str(link);
        buffer.push_str(r#"">"#);
        buffer.push_str(main);
        buffer.push_str(r#"</a>"#);
    } else {
        buffer.push_str(main);
    };

    buffer.push_str(r#"</svg>"#);

    #[cfg(debug_assertions)]
    assert_eq!(start_cap, buffer.capacity());

    strip_xml_whitespace(&buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_badge_test() {
        assert_eq!(
            render_badge(
                RenderBadgeConfig {
                    left_width: 100,
                    right_width: 200,
                    height: 20,
                    accessible_text: "access&text",
                    links: &Links {
                        left: None,
                        right: None
                    }
                },
                "main"
            ),
            r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="300" height="20" role="img" aria-label="access&amp;text"><title>access&amp;text</title>main</svg>"##
        );
    }
    #[test]
    fn render_badge_link_test() {
        assert_eq!(
            render_badge(
                RenderBadgeConfig {
                    left_width: 100,
                    right_width: 200,
                    height: 20,
                    accessible_text: "access&text",
                    links: &Links {
                        left: Some("Link".to_string()),
                        right: None
                    }
                },
                "main"
            ),
            r##"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="300" height="20" ><a target="_blank" xlink:href="Link">main</a></svg>"##
        );
    }
}
