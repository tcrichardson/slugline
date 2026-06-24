use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    #[serde(default = "default_notes_dir")]
    pub notes_dir: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_true")]
    pub auto_open: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UiConfig {
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_font")]
    pub font: String,
    #[serde(default = "default_edit_line_position")]
    pub edit_line_position: f32,
    #[serde(default)]
    pub colors: BTreeMap<String, BTreeMap<String, String>>,
}

fn default_notes_dir() -> String {
    "~/Documents/Slugline".to_string()
}
fn default_port() -> u16 {
    4747
}
fn default_true() -> bool {
    true
}
fn default_theme() -> String {
    "light".to_string()
}
fn default_font() -> String {
    "Roboto".to_string()
}
fn default_edit_line_position() -> f32 {
    0.5
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            notes_dir: default_notes_dir(),
            port: default_port(),
            auto_open: default_true(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            font: default_font(),
            edit_line_position: default_edit_line_position(),
            colors: BTreeMap::new(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

impl Config {
    pub fn from_toml(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }
}

/// Expand a leading `~/` to the user's home directory.
pub fn expand_tilde(p: &str) -> PathBuf {
    if let Some(rest) = p.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(p)
}

/// Default config file path: `~/.config/slugline/config.toml` (XDG-style, also on macOS).
pub fn default_config_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".config").join("slugline").join("config.toml")
}

/// Load config from `path`, creating it with defaults if missing.
pub fn load_or_create(path: &Path) -> io::Result<Config> {
    match fs::read_to_string(path) {
        Ok(s) => Config::from_toml(&s).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            let cfg = Config::default();
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let toml = toml::to_string_pretty(&cfg)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            fs::write(path, toml)?;
            Ok(cfg)
        }
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn defaults_apply_when_fields_missing() {
        let cfg = Config::from_toml("").unwrap();
        assert_eq!(cfg.server.port, 4747);
        assert_eq!(cfg.server.auto_open, true);
        assert_eq!(cfg.server.notes_dir, "~/Documents/Slugline");
        assert_eq!(cfg.ui.theme, "light");
        assert_eq!(cfg.ui.font, "Roboto");
        assert!((cfg.ui.edit_line_position - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn parses_overrides() {
        let toml = r##"
            [server]
            port = 9000
            auto_open = false

            [ui]
            theme = "dark"

            [ui.colors.dark]
            "--bg" = "#101018"
        "##;
        let cfg = Config::from_toml(toml).unwrap();
        assert_eq!(cfg.server.port, 9000);
        assert_eq!(cfg.server.auto_open, false);
        assert_eq!(cfg.ui.theme, "dark");
        assert_eq!(cfg.ui.colors["dark"]["--bg"], "#101018");
    }

    #[test]
    fn expands_leading_tilde() {
        let home = dirs::home_dir().unwrap();
        assert_eq!(expand_tilde("~/Documents/Slugline"), home.join("Documents/Slugline"));
        assert_eq!(expand_tilde("/abs/path"), std::path::PathBuf::from("/abs/path"));
    }

    #[test]
    fn load_or_create_writes_default_when_missing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested").join("config.toml");
        let cfg = load_or_create(&path).unwrap();
        assert_eq!(cfg.server.port, 4747);
        assert!(path.exists());
        // Second load reads the file back.
        let again = load_or_create(&path).unwrap();
        assert_eq!(again.server.port, 4747);
    }
}
