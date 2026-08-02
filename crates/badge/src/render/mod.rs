use crate::{badge::Badge, render::font::Font};

mod base;
mod font;
pub(crate) mod renderers;
pub(crate) mod util;

const FONT_FAMILY: &str = r#"font-family="Verdana,Geneva,DejaVu Sans,sans-serif""#;
const BRIGHTNESS_THRESHOLD: f32 = 0.69;

pub trait BadgeRenderer {
    fn font_family(&self) -> &'static str;
    fn font(&self) -> Font;
    fn height(&self) -> usize;
    fn vertical_margin(&self) -> isize;
    fn shadow(&self) -> bool;
    fn render(badge: &Badge) -> String;
}
