pub mod classify;
pub mod command;
pub mod context;
pub mod render_inline;
pub mod scan;

pub use classify::{Line, classify_line};
pub use command::{
    ArgKind, COMMANDS, CommandName, CommandSpec, ParsedCommand, ValidationResult,
    parse_command_line, spec_for, validate_command,
};
pub use context::{Context, nearest_heading_level, resolve_context};
pub use render_inline::{Span, render_inline};
pub use scan::{Block, DocModel, MetaEntry, Section, SectionKind, scan_document};
