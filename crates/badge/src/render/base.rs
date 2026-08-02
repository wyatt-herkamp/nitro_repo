use crate::{
    badge::{Badge, Links},
    render::{
        BadgeRenderer,
        renderers::{
            RenderLogoConfig, RenderLogoReturn, RenderTextConfig, RenderTextReturn, render_logo,
            render_text,
        },
        util::text::create_accessible_text,
    },
};

#[derive(Debug, Eq, PartialEq)]
pub struct BaseConfig {
    pub left_width: usize,
    pub right_width: usize,
    pub width: usize,
    pub label_color: String,
    pub color: String,
    pub label: Option<String>,
    pub message: String,
    pub accessible_text: String,
    pub rendered_label: String,
    pub rendered_message: String,
    pub links: Links,
    pub rendered_logo: String,
}

impl BaseConfig {
    pub fn new(badge: &Badge, renderer: &impl BadgeRenderer) -> Self {
        let horizontal_padding = 5;

        let RenderLogoReturn {
            rendered_logo,
            logo_width,
        } = render_logo(RenderLogoConfig {
            logo: badge.logo(),
            badge_height: renderer.height(),
            horizontal_padding,
        });

        // todo figure out what
        //     labelColor = hasLabel || hasLogo ? labelColor : color
        // is trying to do. JS sucks. Rust <3

        let label_margin = logo_width as isize + 1;

        let RenderTextReturn {
            text: rendered_label,
            width: label_width,
        } = render_text(RenderTextConfig {
            left_margin: label_margin,
            horizontal_padding,
            content: badge.label().as_ref().unwrap_or(&"".to_string()),
            height: renderer.height(),
            vertical_margin: renderer.vertical_margin(),
            shadow: renderer.shadow(),
            color: badge.label_color(),
            link: if !badge.links().is_single() {
                badge.links().left()
            } else {
                &None
            },
            font: &renderer.font(),
        });

        let left_width = if badge.label().is_some() {
            label_width + 2 * horizontal_padding + logo_width
        } else {
            0
        };

        let mut message_margin = left_width as isize - 1;
        if badge.label().is_none() {
            if badge.logo().is_some() {
                message_margin = message_margin + logo_width as isize + horizontal_padding as isize;
            } else {
                message_margin += 1;
            }
        }

        let RenderTextReturn {
            text: rendered_message,
            width: message_width,
        } = render_text(RenderTextConfig {
            left_margin: message_margin,
            horizontal_padding,
            content: badge.message(),
            height: renderer.height(),
            vertical_margin: renderer.vertical_margin(),
            shadow: renderer.shadow(),
            color: badge.color(),
            link: badge.links().right(),
            font: &renderer.font(),
        });

        let mut right_width = message_width + 2 * horizontal_padding;
        if badge.logo().is_some() && badge.label().is_none() {
            right_width += logo_width + horizontal_padding - 1;
        }

        let width = left_width + right_width;

        let accessible_text = create_accessible_text(badge.label(), badge.message());

        BaseConfig {
            left_width,
            right_width,
            width,
            label_color: badge.label_color().to_string(), // todo can these be removed
            color: badge.color().to_string(),
            label: badge.label().clone(),
            message: badge.message().to_string(),
            accessible_text,
            rendered_label,
            rendered_message,
            rendered_logo,
            links: badge.links().clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BadgeBuilder, render::renderers::styles::Flat};

    #[test]
    fn base_config_test() {
        let badge = BadgeBuilder::new()
            .label("hello")
            .message("message")
            .build()
            .unwrap();
        let config = BaseConfig::new(&badge, &(Flat {}));

        assert_eq!(config.width, 96);
        assert_eq!(config.left_width, 37);
        assert_eq!(config.right_width, 59);
    }
}
