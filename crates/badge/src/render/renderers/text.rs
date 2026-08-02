use std::fmt::Write;

use itoa::Buffer;

use crate::render::{
    font::Font,
    renderers::{RenderLinkConfig, render_link},
    util::{
        text::{TextColoring, colors_for_background, preferred_width},
        xml::escape_xml,
    },
};

pub struct RenderTextConfig<'a> {
    pub left_margin: isize,
    pub horizontal_padding: usize,
    pub content: &'a str,
    pub height: usize,
    pub vertical_margin: isize,
    pub shadow: bool,
    pub color: &'a str,
    pub link: &'a Option<String>,
    pub font: &'a Font,
}

pub struct RenderTextReturn {
    pub text: String,
    pub width: usize,
}

// pub fn render_text_new(config: RenderTextConfig) -> RenderTextReturn {
//   if config.content.is_empty() {
//     return RenderTextReturn {
//       text: "".to_string(),
//       width: 0,
//     };
//   }
//
//   let text_length = preferred_width(&config.content, config.font);
//   let escaped_content = escape_xml(&config.content);
//
//   let shadow_margin = 150 + config.vertical_margin;
//   let text_margin = 140 + config.vertical_margin;
//   let out_text_length = 10 * text_length;
//
//   let x = 10.0
//     * (config.left_margin as f32 + 0.5 * text_length as f32 + config.horizontal_padding as f32);
//
//   let TextColoring { text, shadow } = colors_for_background(&config.color);
//
//   let shadow_text = if config.shadow {
//     format!(
//       r#"<text aria-hidden="true" x="{x}" y="{shadow_margin}" fill="{shadow_color}" fill-opacity=".3" transform="scale(.1)" textLength="{out_text_length}">{escaped_content}</text>"#,
//       x = x,
//       shadow_margin = shadow_margin,
//       shadow_color = shadow,
//       out_text_length = out_text_length,
//       escaped_content = escaped_content
//     )
//   } else {
//     "".to_string()
//   };
//   let rendered_text = format!(
//     r#"{shadow}<text x="{x}" y="{text_margin}" transform="scale(.1)" fill="{text_color}" textLength="{out_text_length}">{escaped_content}</text>"#,
//     escaped_content = escaped_content,
//     out_text_length = out_text_length,
//     x = x,
//     text_margin = text_margin,
//     text_color = text,
//     shadow = shadow_text
//   );
//
//   let rendered_text = if let Some(link) = config.link.as_ref() {
//     render_link(RenderLinkConfig {
//       link,
//       height: config.height,
//       text_length,
//       horizontal_padding: config.horizontal_padding,
//       left_margin: config.left_margin,
//       rendered_text,
//     })
//   } else {
//     rendered_text
//   };
//
//   RenderTextReturn {
//     text: rendered_text,
//     width: text_length,
//   }
// }

pub fn render_text(config: RenderTextConfig) -> RenderTextReturn {
    if config.content.is_empty() {
        return RenderTextReturn {
            text: "".to_string(),
            width: 0,
        };
    }

    let text_length = preferred_width(config.content, config.font);
    let escaped_content = escape_xml(config.content);

    let shadow_margin = 150 + config.vertical_margin;
    let text_margin = 140 + config.vertical_margin;
    let out_text_length = 10 * text_length;

    let x = 10.0
        * (config.left_margin as f32 + 0.5 * text_length as f32 + config.horizontal_padding as f32);

    let TextColoring { text, shadow } = colors_for_background(config.color);

    let buffer_capacity = if config.shadow {
        160 + 150 + escaped_content.len() * 2
    } else {
        150 + escaped_content.len()
    };

    let mut buffer = String::with_capacity(buffer_capacity);

    #[cfg(debug_assertions)]
    let start_cap = buffer.capacity();
    let mut itoa_buffer = Buffer::new();

    if config.shadow {
        buffer.push_str(r#"<text aria-hidden="true" x=""#);
        buffer.write_str(itoa_buffer.format(x as u32)).unwrap_or(());
        buffer.push_str(r#"" y=""#);
        buffer
            .write_str(itoa_buffer.format(shadow_margin as u32))
            .unwrap_or(());
        buffer.push_str(r#"" fill=""#);
        buffer.push_str(shadow);
        buffer.push_str(r#"" fill-opacity=".3" transform="scale(.1)" textLength=""#);
        buffer
            .write_str(itoa_buffer.format(out_text_length as u32))
            .unwrap_or(());
        buffer.push_str(r#"">"#);
        buffer.push_str(&escaped_content);
        buffer.push_str("</text>");
    };

    buffer.push_str(r#"<text x=""#);
    buffer.write_str(itoa_buffer.format(x as u32)).unwrap_or(());
    buffer.push_str(r#"" y=""#);
    buffer
        .write_str(itoa_buffer.format(text_margin as u32))
        .unwrap_or(());
    buffer.push_str(r#"" transform="scale(.1)" fill=""#);
    buffer.push_str(text);
    buffer.push_str(r#"" textLength=""#);
    buffer
        .write_str(itoa_buffer.format(out_text_length as u32))
        .unwrap_or(());
    buffer.push_str(r#"">"#);
    buffer.push_str(&escaped_content);
    buffer.push_str(r#"</text>"#);

    #[cfg(debug_assertions)]
    assert_eq!(start_cap, buffer.capacity());

    let rendered_text = if let Some(link) = config.link.as_ref() {
        render_link(RenderLinkConfig {
            link,
            height: config.height,
            text_length,
            horizontal_padding: config.horizontal_padding,
            left_margin: config.left_margin,
            rendered_text: &buffer,
        })
    } else {
        buffer
    };

    RenderTextReturn {
        text: rendered_text,
        width: text_length,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_text_test() {
        let s = render_text(RenderTextConfig {
            left_margin: 12,
            horizontal_padding: 13,
            content: "content<>",
            height: 20,
            vertical_margin: 2,
            shadow: true,
            color: "#ccc",
            link: &None,
            font: &Font::Verdana11Px,
        });

        assert_eq!(
            s.text,
            r##"<text aria-hidden="true" x="545" y="152" fill="#ccc" fill-opacity=".3" transform="scale(.1)" textLength="590">content&lt;&gt;</text><text x="545" y="142" transform="scale(.1)" fill="#333" textLength="590">content&lt;&gt;</text>"##
        );
        assert_eq!(s.width, 59);
    }

    #[test]
    fn render_text_no_shadow_test() {
        let s = render_text(RenderTextConfig {
            left_margin: 12,
            horizontal_padding: 13,
            content: "content<>",
            height: 20,
            vertical_margin: 2,
            shadow: false,
            color: "#ccc",
            link: &None,
            font: &Font::Verdana11Px,
        });

        assert_eq!(
            s.text,
            r##"<text x="545" y="142" transform="scale(.1)" fill="#333" textLength="590">content&lt;&gt;</text>"##
        );
        assert_eq!(s.width, 59);
    }
}
