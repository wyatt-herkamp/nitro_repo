pub use badge_builder::*;
pub use links::*;
pub use logo::*;
/// Style doc_svgs
pub use style::Style;

mod badge_builder;
pub mod color;
mod links;
mod logo;
mod style;

// `Style`, `Links`, and `Logo` are already in scope here through this module's own re-exports
// above; importing them again would shadow those and make them private.
use std::hash::{Hash, Hasher};

use crate::{
    badge::color::util::u64_to_hex,
    render::{BadgeRenderer, renderers::styles::*},
};

/// The purpose of a unique id is to make the svg compatible with direct website embedding.
///
/// Hashing is done as a escaping mechanism.
fn gen_id(
    label: &Option<String>,
    message: &str,
    label_color: &str,
    color: &str,
    style: &Style,
    links: &Links,
    logo: &Option<Logo>,
) -> String {
    let mut hasher = seahash::SeaHasher::new();
    logo.hash(&mut hasher);
    label.hash(&mut hasher);
    label_color.hash(&mut hasher);
    color.hash(&mut hasher);
    style.hash(&mut hasher);
    message.hash(&mut hasher);
    links.left.hash(&mut hasher);
    links.right.hash(&mut hasher);
    u64_to_hex(hasher.finish())
}

/// Badges are valid and have all the necessary
/// fields to construct an SVG without error.
/// Use the [BadgeBuilder](crate::badge::BadgeBuilder) to construct.
///
/// # Example
///
/// ```rust
/// use nr_badge::BadgeBuilder;
/// # use nr_badge::error::Error;
///
/// let badge = BadgeBuilder::new()
///   .message("Example")
///   .build()?;
///
/// # Ok::<(), Error>(())
/// ```
///
#[derive(Debug, Eq, PartialEq)]
pub struct Badge {
    label: Option<String>,
    message: String,
    label_color: String,
    color: String,
    style: Style,
    links: Links,
    logo: Option<Logo>,
    id: String,
    ids: String,
    idr: String,
}

impl Badge {
    pub(crate) fn new(
        label: Option<String>,
        message: String,
        label_color: String,
        color: String,
        style: Style,
        links: Links,
        logo: Option<Logo>,
    ) -> Self {
        let id = gen_id(
            &label,
            &message,
            &label_color,
            &color,
            &style,
            &links,
            &logo,
        );
        let ids = format!("bms-{}", id);
        let idr = format!("bmr-{}", id);

        Self {
            label,
            message,
            label_color,
            color,
            style,
            links,
            logo,
            id,
            ids,
            idr,
        }
    }

    pub fn svg(&self) -> String {
        match self.style {
            Style::Flat => Flat::render(self),
            Style::Plastic => Plastic::render(self),
            Style::FlatSquare => FlatSquare::render(self),
        }
    }

    pub fn label(&self) -> &Option<String> {
        &self.label
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn label_color(&self) -> &str {
        &self.label_color
    }

    pub fn color(&self) -> &str {
        &self.color
    }

    pub fn style(&self) -> Style {
        self.style
    }

    pub fn links(&self) -> &Links {
        &self.links
    }

    pub fn logo(&self) -> &Option<Logo> {
        &self.logo
    }

    pub fn ids(&self) -> &str {
        &self.ids
    }

    pub fn idr(&self) -> &str {
        &self.idr
    }
    pub fn id(&self) -> &str {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::badge::style::Style::Flat;

    #[test]
    fn gen_id_test() {
        assert_eq!(
            gen_id(
                &Some("hello".to_string()),
                "e",
                "e",
                "3",
                &Flat,
                &Links {
                    left: Option::from("hello".to_string()),
                    right: None,
                },
                &None,
            ),
            "95efd3465e2fbf49"
        );
        assert_eq!(
            gen_id(
                &Some("hello".to_string()),
                "message",
                "label_color",
                "#ec3d",
                &Flat,
                &Links {
                    left: None,
                    right: None,
                },
                &None,
            ),
            "81df87a2241d1ad9"
        );
    }
}
