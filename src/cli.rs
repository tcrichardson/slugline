use std::path::PathBuf;

use clap::Parser;

use crate::config::{expand_tilde, Config};

#[derive(Parser, Debug, Default)]
#[command(name = "slugline", version, about = "Keyboard-driven daily notes")]
pub struct Cli {
    /// Override the notes directory.
    #[arg(long)]
    pub notes_dir: Option<PathBuf>,
    /// Override the listen port.
    #[arg(long)]
    pub port: Option<u16>,
    /// Do not auto-open the browser on launch.
    #[arg(long)]
    pub no_open: bool,
    /// Use a specific config file instead of the default.
    #[arg(long)]
    pub config: Option<PathBuf>,
}

/// The effective runtime settings after applying CLI > file > defaults precedence.
pub struct Resolved {
    pub notes_dir: PathBuf,
    pub port: u16,
    pub auto_open: bool,
}

pub fn resolve(cli: &Cli, config: &Config) -> Resolved {
    Resolved {
        notes_dir: cli
            .notes_dir
            .clone()
            .unwrap_or_else(|| expand_tilde(&config.server.notes_dir)),
        port: cli.port.unwrap_or(config.server.port),
        auto_open: if cli.no_open {
            false
        } else {
            config.server.auto_open
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::path::PathBuf;

    #[test]
    fn cli_overrides_take_precedence_over_config() {
        let cli = Cli {
            notes_dir: Some(PathBuf::from("/tmp/notes")),
            port: Some(9999),
            no_open: true,
            config: None,
        };
        let cfg = Config::default();
        let r = resolve(&cli, &cfg);
        assert_eq!(r.notes_dir, PathBuf::from("/tmp/notes"));
        assert_eq!(r.port, 9999);
        assert_eq!(r.auto_open, false);
    }

    #[test]
    fn falls_back_to_config_then_defaults() {
        let cli = Cli {
            notes_dir: None,
            port: None,
            no_open: false,
            config: None,
        };
        let cfg = Config::default();
        let r = resolve(&cli, &cfg);
        let home = dirs::home_dir().unwrap();
        assert_eq!(r.notes_dir, home.join("Documents/Slugline"));
        assert_eq!(r.port, 4747);
        assert_eq!(r.auto_open, true);
    }
}
