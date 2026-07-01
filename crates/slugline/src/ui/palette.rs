#![allow(dead_code)]

use iced::Color;

/// Parse a `#rrggbb` hex string into an Iced Color at compile-usage time.
pub const fn hex(rgb: u32) -> Color {
    Color::from_rgb(
        ((rgb >> 16) & 0xff) as f32 / 255.0,
        ((rgb >> 8) & 0xff) as f32 / 255.0,
        (rgb & 0xff) as f32 / 255.0,
    )
}

pub const BG: Color = hex(0x161a26);
pub const FG: Color = hex(0xe7ecf5);
pub const MUTED: Color = hex(0x97a1b3);
pub const CURSOR: Color = hex(0xe7ecf5);
pub const EDIT_BAR_BG: Color = hex(0x2a344c);
pub const RULE: Color = hex(0x2d3650);
pub const HIGHLIGHT_BG: Color = hex(0x713f12);
pub const LINK: Color = hex(0x2f6df6);
pub const TODO_DONE: Color = hex(0x8a93a3);
pub const BLOCKQUOTE_BORDER: Color = hex(0x3b82f6);

/// Heading colors h1..h6.
pub const HEADING: [Color; 6] = [
    hex(0x1d4ed8),
    hex(0x3b82f6),
    hex(0x60a5fa),
    hex(0x7dabfb),
    hex(0x9cc2fc),
    hex(0x9cc2fc),
];
