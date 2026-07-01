pub mod edits;
pub mod insert;
pub mod keymap;
pub mod motions;
pub mod state;

pub use keymap::{AppEffect, KeyInput, KeyResult, handle_key};
pub use state::{Cursor, EditorState, Mode, Pending, clamp_cursor, create_editor_state};
