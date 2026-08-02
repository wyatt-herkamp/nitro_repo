use crate::{
    badge::Badge,
    render::{
        BadgeRenderer, FONT_FAMILY,
        base::BaseConfig,
        font::Font,
        renderers::{RenderBadgeConfig, render_badge},
    },
};

pub struct FlatSquare {}

impl BadgeRenderer for FlatSquare {
    fn font_family(&self) -> &'static str {
        FONT_FAMILY
    }

    fn font(&self) -> Font {
        Font::Verdana11Px
    }

    fn height(&self) -> usize {
        20
    }

    fn vertical_margin(&self) -> isize {
        0
    }

    fn shadow(&self) -> bool {
        false
    }

    fn render(badge: &Badge) -> String {
        let renderer = Self {};

        let config = BaseConfig::new(badge, &renderer);

        let body = format!(
            r##"<g shape-rendering="crispEdges">
        <rect width="{left_width}" height="{height}" fill="{label_color}"/>
        <rect x="{left_width}" width="{right_width}" height="{height}" fill="{color}"/>
      </g>
      <g fill="#fff" text-anchor="middle" {font_family} text-rendering="geometricPrecision" font-size="110">
        {rendered_logo}{rendered_label}{rendered_message}
      </g>"##,
            left_width = config.left_width,
            right_width = config.right_width,
            rendered_label = config.rendered_label,
            rendered_message = config.rendered_message,
            font_family = renderer.font_family(),
            height = renderer.height(),
            label_color = config.label_color,
            color = config.color,
            rendered_logo = config.rendered_logo
        );
        render_badge(
            RenderBadgeConfig {
                left_width: config.left_width,
                right_width: config.right_width,
                height: renderer.height(),
                accessible_text: &config.accessible_text,
                links: &config.links,
            },
            &body,
        )
    }
}
