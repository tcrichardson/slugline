//! Adapts `slugline_core::theme` tokens (hex strings) into ready-to-render Iced colors.
//! Replaces the old `ui::palette` module (a fixed set of dark-only constants) now that
//! the active theme can change at runtime via `:theme`.

use std::collections::BTreeMap;

use iced::Color;

use slugline_core::theme::{Tokens, resolve_tokens};

/// One `Color` per rendering concern used across `ui/*`. Computed once per theme change
/// (not per-frame) and stored on `App`; every `ui::*::view` function takes `&Palette`
/// instead of reaching for a global constant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub bg: Color,
    pub fg: Color,
    pub muted: Color,
    pub accent: Color,
    pub heading: [Color; 6],
    pub todo_done: Color,
    pub meta: Color,
    pub status_bar: Color,
    pub edit_bar_bg: Color,
    pub rule: Color,
    pub cursor: Color,
    pub blockquote_border: Color,
    pub highlight_bg: Color,
}

/// Parse a `#rrggbb` hex string into a Color. Falls back to opaque black on any
/// malformed input (missing `#`, wrong length, non-hex digits) — config-supplied
/// override strings are untrusted, so this must never panic.
fn parse_hex(s: &str) -> Color {
    let s = s.strip_prefix('#').unwrap_or(s);
    let byte = |i: usize| s.get(i..i + 2).and_then(|h| u8::from_str_radix(h, 16).ok());
    match (s.len() == 6, byte(0), byte(2), byte(4)) {
        (true, Some(r), Some(g), Some(b)) => Color::from_rgb8(r, g, b),
        _ => Color::BLACK,
    }
}

fn token(tokens: &Tokens, key: &str) -> Color {
    tokens
        .get(key)
        .map(|v| parse_hex(v))
        .unwrap_or(Color::BLACK)
}

impl Palette {
    fn from_tokens(tokens: &Tokens) -> Self {
        Self {
            bg: token(tokens, "--bg"),
            fg: token(tokens, "--fg"),
            muted: token(tokens, "--muted"),
            accent: token(tokens, "--accent"),
            heading: [
                token(tokens, "--heading-1"),
                token(tokens, "--heading-2"),
                token(tokens, "--heading-3"),
                token(tokens, "--heading-4"),
                token(tokens, "--heading-5"),
                token(tokens, "--heading-6"),
            ],
            todo_done: token(tokens, "--todo-done"),
            meta: token(tokens, "--meta"),
            status_bar: token(tokens, "--status-bar"),
            edit_bar_bg: token(tokens, "--edit-bar-bg"),
            rule: token(tokens, "--rule"),
            cursor: token(tokens, "--cursor"),
            blockquote_border: token(tokens, "--blockquote-border"),
            highlight_bg: token(tokens, "--highlight-bg"),
        }
    }

    /// Resolve `theme`'s tokens (built-ins + `overrides[theme]`) into a ready-to-render
    /// `Palette`. The one entry point `App` calls on boot and after every theme switch.
    pub fn for_theme(theme: &str, overrides: &BTreeMap<String, Tokens>) -> Self {
        Self::from_tokens(&resolve_tokens(theme, overrides))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_well_formed_hex_color() {
        assert_eq!(parse_hex("#1b2330"), Color::from_rgb8(0x1b, 0x23, 0x30));
        assert_eq!(parse_hex("1b2330"), Color::from_rgb8(0x1b, 0x23, 0x30));
    }

    #[test]
    fn falls_back_to_black_on_malformed_input() {
        assert_eq!(parse_hex("not-a-color"), Color::BLACK);
        assert_eq!(parse_hex("#fff"), Color::BLACK); // 3-digit shorthand unsupported
        assert_eq!(parse_hex(""), Color::BLACK);
    }

    #[test]
    fn for_theme_resolves_light_and_dark_distinctly() {
        let light = Palette::for_theme("light", &BTreeMap::new());
        let dark = Palette::for_theme("dark", &BTreeMap::new());
        assert_ne!(light.bg, dark.bg);
    }

    #[test]
    fn for_theme_applies_a_config_override() {
        let mut overrides = BTreeMap::new();
        let mut dark_overrides = Tokens::new();
        dark_overrides.insert("--bg".to_string(), "#000000".to_string());
        overrides.insert("dark".to_string(), dark_overrides);
        let p = Palette::for_theme("dark", &overrides);
        assert_eq!(p.bg, Color::BLACK);
    }
}
