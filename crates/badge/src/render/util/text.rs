use crate::{
    badge::color::util::brightness,
    render::{BRIGHTNESS_THRESHOLD, font::Font, util::round::round_up_to_odd},
};

pub struct TextColoring {
    pub text: &'static str,
    pub shadow: &'static str,
}

pub fn colors_for_background(color: &str) -> TextColoring {
    if brightness(color) <= BRIGHTNESS_THRESHOLD {
        TextColoring {
            text: "#fff",
            shadow: "#010101",
        }
    } else {
        TextColoring {
            text: "#333",
            shadow: "#ccc",
        }
    }
}

pub fn preferred_width(text: &str, font: &Font) -> usize {
    round_up_to_odd(font.width_of_str(text) as usize)
}

pub fn create_accessible_text(label: &Option<String>, message: &str) -> String {
    if let Some(label) = label {
        let mut buffer = String::with_capacity(label.len() + message.len() + 2);
        #[cfg(debug_assertions)]
        let start_cap = buffer.capacity();
        buffer.push_str(label);
        buffer.push_str(": ");
        buffer.push_str(message);
        #[cfg(debug_assertions)]
        assert_eq!(start_cap, buffer.capacity());
        buffer
    } else {
        message.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::font::Font::Verdana11Px;

    #[test]
    fn create_accessible_text_test() {
        assert_eq!(
            create_accessible_text(&Some("hello".to_string()), "message"),
            "hello: message".to_string()
        );

        assert_eq!(
            create_accessible_text(&Some("Some Message".to_string()), "Hello There😏"),
            "Some Message: Hello There😏".to_string()
        );

        assert_eq!(
            create_accessible_text(&None, "message"),
            "message".to_string()
        );
    }

    #[test]
    fn preferred_width_test() {
        assert_eq!(preferred_width("Hello", &Verdana11Px), 27);
        assert_eq!(preferred_width("&&lllasef   ", &Verdana11Px), 59);
        assert_eq!(preferred_width("He fe esfllo", &Verdana11Px), 61);
    }
}
