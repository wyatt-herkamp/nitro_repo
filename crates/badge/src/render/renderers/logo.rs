use std::fmt::Write;

use base64::{Engine, engine::general_purpose::STANDARD};
use itoa::Buffer;

use crate::{
    badge::Logo,
    render::util::xml::{escape_xml, replace_fill_attribute},
};

const RAW_STR_LEN: usize = 47;
const NUM_STR_LEN: usize = 14;
const BASE_64_SVG_IMAGE: &str = "data:image/svg+xml;base64,";
pub struct RenderLogoConfig<'logo> {
    pub logo: &'logo Option<Logo>,
    pub badge_height: usize,
    pub horizontal_padding: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RenderLogoReturn {
    pub rendered_logo: String,
    pub logo_width: usize,
}

pub fn render_logo(config: RenderLogoConfig) -> RenderLogoReturn {
    if config.logo.is_none() {
        return RenderLogoReturn {
            rendered_logo: "".to_string(),
            logo_width: 0,
        };
    }

    let logo = config.logo.as_ref().unwrap();

    let logo_height = 14;
    let y = (config.badge_height - logo_height) / 2;
    let x = config.horizontal_padding;
    let escaped_logo = match logo {
        Logo::LogoImage { url, .. } => escape_xml(url),
        Logo::SVGLogo { svg, color, .. } => {
            if let Some(color) = color {
                let value = replace_fill_attribute(svg, &format!("fill=\"{}\"", color));
                format!("{}{}", BASE_64_SVG_IMAGE, STANDARD.encode(value))
            } else {
                format!("{}{}", BASE_64_SVG_IMAGE, STANDARD.encode(svg))
            }
        }
    };

    let mut buffer = String::with_capacity(RAW_STR_LEN + NUM_STR_LEN + escaped_logo.len());

    #[cfg(debug_assertions)]
    let start_cap = buffer.capacity();
    let mut itoa_buffer = Buffer::new();

    buffer.push_str(r#"<image x=""#);
    buffer.write_str(itoa_buffer.format(x)).unwrap();
    buffer.push_str(r#"" y=""#);
    buffer.write_str(itoa_buffer.format(y)).unwrap();
    buffer.push_str(r#"" width=""#);
    buffer.write_str(itoa_buffer.format(logo.width())).unwrap();
    buffer.push_str(r#"" height=""#);
    buffer.write_str(itoa_buffer.format(logo_height)).unwrap();
    buffer.push_str(r#"" xlink:href=""#);
    buffer.push_str(&escaped_logo);
    buffer.push_str(r#""/>"#);

    #[cfg(debug_assertions)]
    assert_eq!(start_cap, buffer.capacity());

    RenderLogoReturn {
        rendered_logo: buffer,
        logo_width: (logo.width() as isize + logo.padding()) as usize,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_logo_test() {
        assert_eq!(
            render_logo(RenderLogoConfig {
                logo: &None,
                badge_height: 20,
                horizontal_padding: 30
            }),
            RenderLogoReturn {
                rendered_logo: "".to_string(),
                logo_width: 0
            }
        );
        assert_eq!(
            render_logo(RenderLogoConfig {
                logo: &Some(Logo::LogoImage {
                    url: "some_<>url".to_string(),
                    width: 20,
                    padding: 25
                }),
                badge_height: 20,
                horizontal_padding: 30
            }),
            RenderLogoReturn {
                rendered_logo:
                    r#"<image x="30" y="3" width="20" height="14" xlink:href="some_&lt;&gt;url"/>"#
                        .to_string(),
                logo_width: 45
            }
        );
    }
}
