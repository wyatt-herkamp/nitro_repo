use std::fmt::Write;

use itoa::Buffer;

use crate::render::util::xml::escape_xml;

pub struct RenderLinkConfig<'a> {
    pub link: &'a str,
    pub height: usize,
    pub text_length: usize,
    pub horizontal_padding: usize,
    pub left_margin: isize,
    pub rendered_text: &'a str,
}

const RAW_STR_LEN: usize = 89;
const NUM_STR_LEN: usize = 14;

pub fn render_link(config: RenderLinkConfig) -> String {
    let rect_height = config.height;
    let rect_width = config.text_length + config.horizontal_padding * 2;
    let rect_x = if config.left_margin > 1 {
        config.left_margin + 1
    } else {
        0
    };

    let escaped_link = escape_xml(config.link);

    let mut buffer = String::with_capacity(
        RAW_STR_LEN + NUM_STR_LEN + config.rendered_text.len() + escaped_link.len(),
    );
    #[cfg(debug_assertions)]
    let start_cap = buffer.capacity();
    let mut itoa_buffer = Buffer::new();
    buffer.push_str(r#"<a target="_blank" xlink:href=""#);
    buffer.push_str(&escaped_link);
    buffer.push_str(r#""><rect width=""#);
    buffer.write_str(itoa_buffer.format(rect_width)).unwrap();
    buffer.push_str(r#"" x=""#);
    buffer.write_str(itoa_buffer.format(rect_x)).unwrap();
    buffer.push_str(r#"" height=""#);
    buffer.write_str(itoa_buffer.format(rect_height)).unwrap();
    buffer.push_str(r#"" fill="rgba(0,0,0,0)"/>"#);
    buffer.push_str(config.rendered_text);
    buffer.push_str(r#"</a>"#);

    #[cfg(debug_assertions)]
    assert_eq!(start_cap, buffer.capacity());

    buffer
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_link_test() {
        assert_eq!(
            render_link(RenderLinkConfig {
                link: "some>link",
                height: 212,
                text_length: 123,
                horizontal_padding: 13,
                left_margin: 15,
                rendered_text: "rendered_text",
            }),
            "<a target=\"_blank\" xlink:href=\"some&gt;link\"><rect width=\"149\" x=\"16\" height=\"212\" fill=\"rgba(0,0,0,0)\"/>rendered_text</a>"
        );
    }
}
