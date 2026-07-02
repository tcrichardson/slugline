use iced::keyboard::Modifiers;
use iced::keyboard::key::{Key, Named};

/// Apply the US-QWERTY shift layer to a single base character so that `Shift+;`
/// becomes `":"`, `Shift+a` becomes `"A"`, etc. This matches the
/// DOM-`KeyboardEvent.key` behaviour the keymap was originally written against.
fn apply_shift(c: char) -> char {
    match c {
        'a'..='z' => c.to_ascii_uppercase(),
        '1' => '!',
        '2' => '@',
        '3' => '#',
        '4' => '$',
        '5' => '%',
        '6' => '^',
        '7' => '&',
        '8' => '*',
        '9' => '(',
        '0' => ')',
        '-' => '_',
        '=' => '+',
        '[' => '{',
        ']' => '}',
        '\\' => '|',
        ';' => ':',
        '\'' => '"',
        ',' => '<',
        '.' => '>',
        '/' => '?',
        '`' => '~',
        _ => c,
    }
}

/// Map an Iced logical key to the DOM-`KeyboardEvent.key`-style string our keymap expects.
/// Returns `None` for keys we ignore (pure modifiers, unidentified).
///
/// When `mods.shift()` is true, character keys are shifted (e.g. `Shift+;` → `":"`).
pub fn key_string(key: &Key, mods: &Modifiers) -> Option<String> {
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
        Key::Character(s) => {
            let c = s.chars().next()?;
            let shifted = if mods.shift() { apply_shift(c) } else { c };
            Some(shifted.to_string())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::keyboard::key::{Key, Named};
    use smol_str::SmolStr;

    fn no_mods() -> Modifiers {
        Modifiers::default()
    }

    fn shift() -> Modifiers {
        Modifiers::SHIFT
    }

    #[test]
    fn named_and_character_map() {
        assert_eq!(
            key_string(&Key::Named(Named::Enter), &no_mods()).as_deref(),
            Some("Enter")
        );
        assert_eq!(
            key_string(&Key::Named(Named::ArrowLeft), &no_mods()).as_deref(),
            Some("ArrowLeft")
        );
        assert_eq!(
            key_string(&Key::Named(Named::Space), &no_mods()).as_deref(),
            Some(" ")
        );
        assert_eq!(
            key_string(&Key::Character(SmolStr::new("h")), &no_mods()).as_deref(),
            Some("h")
        );
        assert_eq!(key_string(&Key::Named(Named::Shift), &no_mods()), None);
    }

    #[test]
    fn shift_applies_to_letters() {
        assert_eq!(
            key_string(&Key::Character(SmolStr::new("a")), &shift()).as_deref(),
            Some("A")
        );
        assert_eq!(
            key_string(&Key::Character(SmolStr::new("z")), &shift()).as_deref(),
            Some("Z")
        );
    }

    #[test]
    fn shift_applies_to_symbols() {
        assert_eq!(
            key_string(&Key::Character(SmolStr::new(";")), &shift()).as_deref(),
            Some(":")
        );
        assert_eq!(
            key_string(&Key::Character(SmolStr::new("1")), &shift()).as_deref(),
            Some("!")
        );
        assert_eq!(
            key_string(&Key::Character(SmolStr::new("-")), &shift()).as_deref(),
            Some("_")
        );
        assert_eq!(
            key_string(&Key::Character(SmolStr::new("`")), &shift()).as_deref(),
            Some("~")
        );
    }
}
