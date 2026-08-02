use crate::color::Color;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Logo {
    LogoImage {
        url: String,
        width: usize,
        padding: isize,
    },
    /// SVG Based Logo
    /// If the color attribute is set it will try to add one. This allows for changing of the fill color of the SVG.
    ///
    /// Simple Icons provides a list of SVG based logos: https://simpleicons.org/
    /// ```rust
    ///  use nr_badge::error::Error;
    /// use nr_badge::{BadgeBuilder, Logo};
    /// use nr_badge::color::{Color, NamedColor};
    /// let badge = BadgeBuilder::new().message("Chrome").color(Color::Rgb(0,0,0)).logo(Logo::SVGLogo {
    ///     svg: r#"<svg role="img" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><title>Google Chrome</title><path d="M12 0C8.21 0 4.831 1.757 2.632 4.501l3.953 6.848A5.454 5.454 0 0 1 12 6.545h10.691A12 12 0 0 0 12 0zM1.931 5.47A11.943 11.943 0 0 0 0 12c0 6.012 4.42 10.991 10.189 11.864l3.953-6.847a5.45 5.45 0 0 1-6.865-2.29zm13.342 2.166a5.446 5.446 0 0 1 1.45 7.09l.002.001h-.002l-5.344 9.257c.206.01.413.016.621.016 6.627 0 12-5.373 12-12 0-1.54-.29-3.011-.818-4.364zM12 16.364a4.364 4.364 0 1 1 0-8.728 4.364 4.364 0 0 1 0 8.728Z"/></svg>"#.to_string(),
    ///     color: Some(NamedColor::Yellow.into()),
    ///     width: 100,
    ///    padding: 10,
    /// }).build().unwrap();
    ///
    /// println!("{}", badge.svg());
    ///
    /// ```
    SVGLogo {
        svg: String,
        color: Option<Color>,
        width: usize,
        padding: isize,
    },
}

impl Logo {
    #[deprecated(since = "0.2.2", note = "Use the Enum Type `Logo::LogoImage` instead")]
    pub fn new(url: String, width: usize, padding: isize) -> Self {
        Self::LogoImage {
            url,
            width,
            padding,
        }
    }
    #[deprecated(since = "0.2.2", note = "New Logo Type Makes this function obsolete. ")]
    pub fn url(&self) -> &str {
        match self {
            Self::LogoImage { url, .. } => url,
            Self::SVGLogo { .. } => panic!("SVGLogo::url() called on SVGLogo"),
        }
    }

    pub fn width(&self) -> usize {
        match self {
            Self::LogoImage { width, .. } => *width,
            Self::SVGLogo { width, .. } => *width,
        }
    }

    pub fn padding(&self) -> isize {
        match self {
            Self::LogoImage { padding, .. } => *padding,
            Self::SVGLogo { padding, .. } => *padding,
        }
    }
}
