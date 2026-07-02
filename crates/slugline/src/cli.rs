use std::path::PathBuf;

use clap::Parser;
use slugline_core::config::{Config, expand_tilde};

#[derive(Parser, Debug, Default)]
#[command(name = "slugline", version, about = "Keyboard-driven daily notes")]
pub struct Cli {
    /// Override the notes directory.
    #[arg(long)]
    pub notes_dir: Option<PathBuf>,
    /// Use a specific config file instead of the default.
    #[arg(long)]
    pub config: Option<PathBuf>,
}

/// The effective notes directory after applying CLI > file > defaults precedence.
pub struct Resolved {
    pub notes_dir: PathBuf,
}

pub fn resolve(cli: &Cli, config: &Config) -> Resolved {
    Resolved {
        notes_dir: cli
            .notes_dir
            .clone()
            .unwrap_or_else(|| expand_tilde(&config.notes.notes_dir)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_notes_dir_overrides_config() {
        let cli = Cli {
            notes_dir: Some(PathBuf::from("/tmp/notes")),
            config: None,
        };
        let r = resolve(&cli, &Config::default());
        assert_eq!(r.notes_dir, PathBuf::from("/tmp/notes"));
    }

    #[test]
    fn falls_back_to_config_notes_dir() {
        let cli = Cli {
            notes_dir: None,
            config: None,
        };
        let r = resolve(&cli, &Config::default());
        let home = dirs::home_dir().unwrap();
        assert_eq!(r.notes_dir, home.join("Documents/Slugline"));
    }
}
