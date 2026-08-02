use std::str::FromStr;

use crate::{
    badge::{
        Badge, Links, Logo,
        color::{Color, NamedColor, util::get_color_from_option},
        style::{Style, Style::Flat},
    },
    error::{Error, Error::MessageNotSet},
};

pub(crate) const DEFAULT_LABEL_COLOR: NamedColor = NamedColor::Grey;
pub(crate) const DEFAULT_MESSAGE_COLOR: NamedColor = NamedColor::LightGrey;
pub(crate) const DEFAULT_LOGO_WIDTH: usize = 14;
pub(crate) const DEFAULT_LOGO_PADDING: isize = 3;
pub(crate) const DEFAULT_STYLE: Style = Flat;

/// Constructs a [Badge](Badge) with given options.
/// **Message** is the only required field.
///
/// Each field has a variant that is the `field()` for constructing with
/// absolute rust safety. Those fields also have alternate `field_parse()`
/// methods for 3rd party use that wants to rely on the library for
/// parsing and validation of the fields.
///
/// # Example
///
/// <svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="100" height="20" role="img" aria-label="example: badge"><title>example: badge</title><linearGradient id="bms-058ad8d642fc4c85" x2="0" y2="100%"><stop offset="0" stop-color="#bbb" stop-opacity=".1"/><stop offset="1" stop-opacity=".1"/></linearGradient><clipPath id="bmr-058ad8d642fc4c85"><rect width="100" height="20" rx="3" fill="#fff"/></clipPath><g clip-path="url(#bmr-058ad8d642fc4c85)"><rect width="57" height="20" fill="#555"/><rect x="57" width="43" height="20" fill="#4c1"/><rect width="100" height="20" fill="url(#bms-058ad8d642fc4c85)"/></g><g fill="#fff" text-anchor="middle" font-family="Verdana,Geneva,DejaVu Sans,sans-serif" text-rendering="geometricPrecision" font-size="110"><text aria-hidden="true" x="295" y="150" fill="#010101" fill-opacity=".3" transform="scale(.1)" textLength="470">example</text><text x="295" y="140" transform="scale(.1)" fill="#fff" textLength="470">example</text><text aria-hidden="true" x="775" y="150" fill="#010101" fill-opacity=".3" transform="scale(.1)" textLength="330">badge</text><text x="775" y="140" transform="scale(.1)" fill="#fff" textLength="330">badge</text></g></svg>
/// ```rust
/// # use nr_badge::error::Error;
/// use nr_badge::BadgeBuilder;
///
/// let badge = BadgeBuilder::new()
///   .label("example")
///   .message("badge")
///   .color_parse("success")
///   .build()?;
///
/// println!("{}", badge.svg());
/// # Ok::<(), Error>(())
/// ```
#[derive(Default)]
pub struct BadgeBuilder {
    label: Option<String>,
    message: Option<String>,
    label_color: Option<Color>,
    label_color_parse: Option<String>,
    color: Option<Color>,
    color_parse: Option<String>,
    style: Option<Style>,
    style_parse: Option<String>,
    link: Option<String>,
    links: Option<Links>,
    logo: Option<Logo>,
    logo_url: Option<String>,
    logo_width: Option<usize>,
    logo_padding: Option<isize>,
}

impl BadgeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn label(&mut self, label: &str) -> &mut Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn message(&mut self, message: &str) -> &mut Self {
        self.message = Some(message.to_string());
        self
    }

    pub fn label_color_parse(&mut self, label_color: &str) -> &mut Self {
        self.label_color_parse = Some(label_color.to_lowercase().replace(' ', ""));
        self
    }

    pub fn label_color(&mut self, color: Color) -> &mut Self {
        self.label_color = Some(color);
        self
    }

    pub fn color_parse(&mut self, color: &str) -> &mut Self {
        self.color_parse = Some(color.to_lowercase().replace(' ', ""));
        self
    }

    pub fn color<Col: Into<Color>>(&mut self, color: Col) -> &mut Self {
        self.color = Some(color.into());
        self
    }

    pub fn style_parse(&mut self, style: &str) -> &mut Self {
        self.style_parse = Some(style.trim().to_lowercase());
        self
    }

    pub fn style(&mut self, style: Style) -> &mut Self {
        self.style = Some(style);
        self
    }

    pub fn link(&mut self, link: &str) -> &mut Self {
        self.link = Some(link.trim().to_string());
        self
    }

    pub fn logo(&mut self, logo: Logo) -> &mut Self {
        self.logo = Some(logo);
        self
    }

    #[deprecated(since = "0.2.2", note = "Set the logo instead")]
    pub fn logo_url(&mut self, url: &str) -> &mut Self {
        self.logo_url = Some(url.trim().to_string());
        self
    }

    #[deprecated(since = "0.2.2", note = "Set the logo instead")]
    pub fn logo_width(&mut self, width: usize) -> &mut Self {
        self.logo_width = Some(width);
        self
    }

    #[deprecated(since = "0.2.2", note = "Set the logo instead")]
    pub fn logo_padding(&mut self, padding: isize) -> &mut Self {
        self.logo_padding = Some(padding);
        self
    }

    pub fn link_left(&mut self, link: &str) -> &mut Self {
        let link = link.to_string();
        if let Some(links) = self.links.as_mut() {
            links.left = Some(link);
        } else {
            self.links = Some(Links {
                left: Some(link),
                right: None,
            })
        }
        self
    }

    pub fn link_right(&mut self, link: &str) -> &mut Self {
        let link = link.to_string();
        if let Some(links) = self.links.as_mut() {
            links.right = Some(link);
        } else {
            self.links = Some(Links {
                left: None,
                right: Some(link),
            })
        }
        self
    }

    pub fn build(&self) -> Result<Badge, Error> {
        let message = self
            .message
            .as_ref()
            .ok_or(MessageNotSet)?
            .trim()
            .to_string();

        let label = match self.label.as_ref() {
            Some(label) => {
                let label = label.trim().to_string();
                if label.is_empty() { None } else { Some(label) }
            }
            None => None,
        };

        let style = if let Some(style) = self.style {
            style
        } else if let Some(style) = self.style_parse.as_ref() {
            Style::from_str(style)?
        } else {
            DEFAULT_STYLE
        };

        let links = if let Some(links) = self.links.as_ref() {
            links.clone()
        } else if let Some(link) = self.link.as_ref() {
            Links {
                left: Some(link.to_string()),
                right: None,
            }
        } else {
            Links {
                left: None,
                right: None,
            }
        };

        let logo = if self.logo.is_some() {
            self.logo.clone()
        } else if let Some(url) = self.logo_url.as_ref() {
            let width = self.logo_width.unwrap_or(DEFAULT_LOGO_WIDTH);
            let padding = self.logo_padding.unwrap_or(DEFAULT_LOGO_PADDING);
            Some(Logo::LogoImage {
                url: url.to_string(),
                width,
                padding,
            })
        } else {
            None
        };

        let color = get_color_from_option(false, self.color_parse.clone(), self.color.as_ref())?;
        let label_color = get_color_from_option(
            true,
            self.label_color_parse.clone(),
            self.label_color.as_ref(),
        )?;

        Ok(Badge::new(
            label,
            message,
            label_color,
            color,
            style,
            links,
            logo,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Badge, Links, Style,
        badge::{DEFAULT_LABEL_COLOR, DEFAULT_MESSAGE_COLOR, badge_builder::BadgeBuilder},
    };

    #[test]
    fn badge_builder_text() {
        assert_eq!(
            BadgeBuilder::new()
                .label(" Hello one ")
                .message(" there two  ")
                .build()
                .unwrap(),
            Badge::new(
                Some("Hello one".to_string()),
                "there two".to_string(),
                DEFAULT_LABEL_COLOR.hex().to_string(),
                DEFAULT_MESSAGE_COLOR.hex().to_string(),
                Style::Flat,
                Links {
                    left: None,
                    right: None,
                },
                None,
            )
        );
    }

    #[test]
    fn badge_builder_color_parse() {
        assert_eq!(
            BadgeBuilder::new()
                .label(" Hello one ")
                .message(" there two  ")
                .label_color_parse("#cab")
                .color_parse("#cab")
                .build()
                .unwrap(),
            Badge::new(
                Some("Hello one".to_string()),
                "there two".to_string(),
                "#cab".to_string(),
                "#cab".to_string(),
                Style::Flat,
                Links {
                    left: None,
                    right: None,
                },
                None,
            )
        );

        assert_eq!(
            BadgeBuilder::new()
                .label(" Hello one ")
                .message(" there two  ")
                .label_color_parse("cab")
                .color_parse("CAB")
                .build()
                .unwrap(),
            Badge::new(
                Some("Hello one".to_string()),
                "there two".to_string(),
                "#cab".to_string(),
                "#cab".to_string(),
                Style::Flat,
                Links {
                    left: None,
                    right: None,
                },
                None,
            )
        );

        assert_eq!(
            BadgeBuilder::new()
                .label(" Hello one ")
                .message(" there two  ")
                .label_color_parse("rgb(10, 200, 50)")
                .color_parse("   RGB(10,200,50)   ")
                .build()
                .unwrap(),
            Badge::new(
                Some("Hello one".to_string()),
                "there two".to_string(),
                "#0ac832".to_string(),
                "#0ac832".to_string(),
                Style::Flat,
                Links {
                    left: None,
                    right: None,
                },
                None,
            )
        );
    }
}
