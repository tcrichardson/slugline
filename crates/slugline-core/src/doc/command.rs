use crate::date::is_valid_date;

/// The text typed after the leading `:`, split into a lowercased name and the rest of
/// the line (trimmed). Port of `web/src/lib/doc/command.ts` `parseCommandLine`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCommand {
    pub name: String,
    pub arg: String,
}

/// Parse the text typed after the leading `:` (the colon is not included).
pub fn parse_command_line(input: &str) -> ParsedCommand {
    let trimmed = input.trim_start();
    match trimmed.find(' ') {
        None => ParsedCommand {
            name: trimmed.to_lowercase(),
            arg: String::new(),
        },
        Some(sp) => ParsedCommand {
            name: trimmed[..sp].to_lowercase(),
            arg: trimmed[sp + 1..].trim().to_string(),
        },
    }
}

/// Every recognized `:` command. Mirrors `web/src/lib/doc/command.ts`'s `CommandName` union.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandName {
    Meeting,
    Note,
    Section,
    Todo,
    Start,
    End,
    Scheduled,
    Purpose,
    Topic,
    People,
    Goto,
    Today,
    Tab,
    Close,
    W,
    Theme,
}

impl CommandName {
    /// The typed name that resolves to this command, lowercase, before alias resolution.
    pub fn canonical(self) -> &'static str {
        match self {
            CommandName::Meeting => "meeting",
            CommandName::Note => "note",
            CommandName::Section => "section",
            CommandName::Todo => "todo",
            CommandName::Start => "start",
            CommandName::End => "end",
            CommandName::Scheduled => "scheduled",
            CommandName::Purpose => "purpose",
            CommandName::Topic => "topic",
            CommandName::People => "people",
            CommandName::Goto => "goto",
            CommandName::Today => "today",
            CommandName::Tab => "tab",
            CommandName::Close => "close",
            CommandName::W => "w",
            CommandName::Theme => "theme",
        }
    }
}

/// The kind of argument a command expects, and how it is validated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgKind {
    None,
    Text,
    Time,
    Date,
    Theme,
}

/// A command's shape: which argument kind it takes and whether that argument is required.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommandSpec {
    pub name: CommandName,
    pub arg_kind: ArgKind,
    pub arg_required: bool,
}

/// Every command, in the canonical order the command palette lists them. Mirrors
/// `web/src/lib/doc/command.ts`'s `COMMANDS` (a `Record`, whose `Object.keys` order is
/// insertion order — preserved here as array order).
pub const COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        name: CommandName::Meeting,
        arg_kind: ArgKind::Text,
        arg_required: true,
    },
    CommandSpec {
        name: CommandName::Note,
        arg_kind: ArgKind::Text,
        arg_required: true,
    },
    CommandSpec {
        name: CommandName::Section,
        arg_kind: ArgKind::Text,
        arg_required: true,
    },
    CommandSpec {
        name: CommandName::Todo,
        arg_kind: ArgKind::Text,
        arg_required: true,
    },
    CommandSpec {
        name: CommandName::Start,
        arg_kind: ArgKind::None,
        arg_required: false,
    },
    CommandSpec {
        name: CommandName::End,
        arg_kind: ArgKind::None,
        arg_required: false,
    },
    CommandSpec {
        name: CommandName::Scheduled,
        arg_kind: ArgKind::Time,
        arg_required: true,
    },
    CommandSpec {
        name: CommandName::Purpose,
        arg_kind: ArgKind::Text,
        arg_required: true,
    },
    CommandSpec {
        name: CommandName::Topic,
        arg_kind: ArgKind::Text,
        arg_required: true,
    },
    CommandSpec {
        name: CommandName::People,
        arg_kind: ArgKind::Text,
        arg_required: true,
    },
    CommandSpec {
        name: CommandName::Goto,
        arg_kind: ArgKind::Date,
        arg_required: true,
    },
    CommandSpec {
        name: CommandName::Today,
        arg_kind: ArgKind::None,
        arg_required: false,
    },
    CommandSpec {
        name: CommandName::Tab,
        arg_kind: ArgKind::Date,
        arg_required: true,
    },
    CommandSpec {
        name: CommandName::Close,
        arg_kind: ArgKind::None,
        arg_required: false,
    },
    CommandSpec {
        name: CommandName::W,
        arg_kind: ArgKind::None,
        arg_required: false,
    },
    CommandSpec {
        name: CommandName::Theme,
        arg_kind: ArgKind::Theme,
        arg_required: false,
    },
];

/// Look up a command's spec by its `CommandName` (not the typed/aliased string — see
/// `lookup` for that). Used by the command palette to show each command's argument kind.
pub fn spec_for(name: CommandName) -> &'static CommandSpec {
    COMMANDS
        .iter()
        .find(|s| s.name.canonical() == name.canonical())
        .expect("every CommandName has a COMMANDS entry")
}

fn lookup(typed_name: &str) -> Option<&'static CommandSpec> {
    COMMANDS.iter().find(|s| s.name.canonical() == typed_name)
}

/// Short aliases resolved before `COMMANDS` lookup. Add future shortcuts here.
fn resolve_alias(typed_name: &str) -> &str {
    match typed_name {
        "p" => "people",
        other => other,
    }
}

/// The outcome of validating a typed command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    Ok { command: CommandName, arg: String },
    Err { error: String },
}

fn validate_arg(kind: ArgKind, arg: &str) -> Option<&'static str> {
    match kind {
        ArgKind::None | ArgKind::Text => None,
        ArgKind::Time => {
            let ok = TIME_RE.is_match(arg);
            if ok { None } else { Some("Expected HH:MM") }
        }
        ArgKind::Date => {
            if is_valid_date(arg) {
                None
            } else {
                Some("Expected YYYY-MM-DD")
            }
        }
        ArgKind::Theme => {
            if arg.is_empty() || arg == "light" || arg == "dark" {
                None
            } else {
                Some("Expected light or dark")
            }
        }
    }
}

use std::sync::LazyLock;

use regex::Regex;

static TIME_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([01]\d|2[0-3]):[0-5]\d$").unwrap());

/// Validate a full typed command line (the text after `:`, colon excluded). Port of
/// `web/src/lib/doc/command.ts` `validateCommand`.
pub fn validate_command(input: &str) -> ValidationResult {
    let ParsedCommand { name, arg } = parse_command_line(input);
    let resolved = resolve_alias(&name);
    let Some(spec) = lookup(resolved) else {
        return ValidationResult::Err {
            error: format!("Unknown command: :{name}"),
        };
    };

    if spec.arg_required && arg.is_empty() {
        return ValidationResult::Err {
            error: format!(":{name} requires an argument"),
        };
    }
    if let Some(error) = validate_arg(spec.arg_kind, &arg) {
        return ValidationResult::Err {
            error: error.to_string(),
        };
    }

    ValidationResult::Ok {
        command: spec.name,
        arg,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_name_and_rest_of_line_argument() {
        let p = parse_command_line("meeting Daily Standup");
        assert_eq!(p.name, "meeting");
        assert_eq!(p.arg, "Daily Standup");
    }

    #[test]
    fn lowercases_the_name_and_handles_no_arg_commands() {
        let p = parse_command_line("Today");
        assert_eq!(p.name, "today");
        assert_eq!(p.arg, "");
    }

    #[test]
    fn accepts_a_valid_text_command() {
        let r = validate_command("meeting Weekly Sync");
        assert_eq!(
            r,
            ValidationResult::Ok {
                command: CommandName::Meeting,
                arg: "Weekly Sync".to_string(),
            }
        );
    }

    #[test]
    fn rejects_unknown_commands() {
        match validate_command("meetng x") {
            ValidationResult::Err { error } => assert!(error.contains("Unknown command")),
            other => panic!("expected Err, got {other:?}"),
        }
    }

    #[test]
    fn requires_arguments_where_mandated() {
        assert!(matches!(
            validate_command("todo"),
            ValidationResult::Err { .. }
        ));
    }

    #[test]
    fn validates_hh_mm_for_scheduled() {
        assert!(matches!(
            validate_command("scheduled 14:30"),
            ValidationResult::Ok { .. }
        ));
        assert!(matches!(
            validate_command("scheduled 25:00"),
            ValidationResult::Err { .. }
        ));
    }

    #[test]
    fn validates_yyyy_mm_dd_for_goto() {
        assert!(matches!(
            validate_command("goto 2026-06-23"),
            ValidationResult::Ok { .. }
        ));
        assert!(matches!(
            validate_command("goto 2026-13-01"),
            ValidationResult::Err { .. }
        ));
    }

    #[test]
    fn validates_theme_values() {
        assert!(matches!(
            validate_command("theme dark"),
            ValidationResult::Ok { .. }
        ));
        assert!(matches!(
            validate_command("theme neon"),
            ValidationResult::Err { .. }
        ));
    }

    #[test]
    fn accepts_no_arg_commands() {
        assert!(matches!(
            validate_command("start"),
            ValidationResult::Ok { .. }
        ));
        assert!(matches!(
            validate_command("close"),
            ValidationResult::Ok { .. }
        ));
    }

    #[test]
    fn allows_theme_with_no_argument_toggle() {
        match validate_command("theme") {
            ValidationResult::Ok { arg, .. } => assert_eq!(arg, ""),
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    #[test]
    fn p_alice_resolves_to_command_people_via_validate_command() {
        match validate_command("p Alice Smith") {
            ValidationResult::Ok { command, arg } => {
                assert_eq!(command.canonical(), "people");
                assert_eq!(arg, "Alice Smith");
            }
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    #[test]
    fn p_with_no_argument_fails_validation() {
        assert!(matches!(
            validate_command("p"),
            ValidationResult::Err { .. }
        ));
    }

    #[test]
    fn people_resolves_directly() {
        match validate_command("people Bob Jones") {
            ValidationResult::Ok { command, arg } => {
                assert_eq!(command.canonical(), "people");
                assert_eq!(arg, "Bob Jones");
            }
            other => panic!("expected Ok, got {other:?}"),
        }
    }

    #[test]
    fn every_command_name_has_a_spec() {
        for spec in COMMANDS {
            assert_eq!(spec_for(spec.name).name.canonical(), spec.name.canonical());
        }
    }
}
