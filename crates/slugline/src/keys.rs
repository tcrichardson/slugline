use iced::keyboard::key::{Key, Named};

/// Map an Iced logical key to the DOM-`KeyboardEvent.key`-style string our keymap expects.
/// Returns `None` for keys we ignore (pure modifiers, unidentified).
pub fn key_string(key: &Key) -> Option<String> {
    match key {
        Key::Named(named) => Some(
            match named {
                Named::Enter => "Enter",
                Named::Backspace => "Backspace",
                Named::Tab => "Tab",
                Named::Escape => "Escape",
                Named::ArrowLeft => "ArrowLeft",
                Named::ArrowRight => "ArrowRight",
                Named::ArrowUp => "ArrowUp",
                Named::ArrowDown => "ArrowDown",
                Named::Space => " ",
                _ => return None,
            }
            .to_string(),
        ),
        Key::Character(s) => Some(s.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::keyboard::key::{Key, Named};
    use smol_str::SmolStr;

    #[test]
    fn named_and_character_map() {
        assert_eq!(
            key_string(&Key::Named(Named::Enter)).as_deref(),
            Some("Enter")
        );
        assert_eq!(
            key_string(&Key::Named(Named::ArrowLeft)).as_deref(),
            Some("ArrowLeft")
        );
        assert_eq!(key_string(&Key::Named(Named::Space)).as_deref(), Some(" "));
        assert_eq!(
            key_string(&Key::Character(SmolStr::new("h"))).as_deref(),
            Some("h")
        );
        assert_eq!(key_string(&Key::Named(Named::Shift)), None);
    }
}
