use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use crossterm::event::KeyCode;
use ratatui::style::{Color, Style};
use serde::Deserialize;

#[derive(Debug, Default)]
pub struct Config {
    pub keybindings: KeyBindings,
    pub theme: Theme,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path();
        if path.exists() {
            let data = fs::read_to_string(&path)
                .with_context(|| format!("failed to read config: {}", path.display()))?;
            let raw: RawConfig = toml::from_str(&data).with_context(|| "invalid config")?;
            Ok(raw.into())
        } else {
            Ok(Self::default())
        }
    }
}

#[derive(Debug)]
pub struct KeyBindings {
    pub quit: KeyCode,
    pub up: KeyCode,
    pub down: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            quit: KeyCode::Char('q'),
            up: KeyCode::Up,
            down: KeyCode::Down,
        }
    }
}

#[derive(Debug)]
pub struct Theme {
    pub selected_fg: Color,
    pub selected_bg: Color,
}

impl Theme {
    pub fn selected_style(&self) -> Style {
        Style::default().fg(self.selected_fg).bg(self.selected_bg)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            selected_fg: Color::Black,
            selected_bg: Color::Yellow,
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    #[serde(default)]
    keybindings: RawKeyBindings,
    #[serde(default)]
    theme: RawTheme,
}

#[derive(Debug, Deserialize, Default)]
struct RawKeyBindings {
    #[serde(default = "default_quit")]
    quit: String,
    #[serde(default = "default_up")]
    up: String,
    #[serde(default = "default_down")]
    down: String,
}

#[derive(Debug, Deserialize, Default)]
struct RawTheme {
    #[serde(default = "default_fg")]
    selected_fg: String,
    #[serde(default = "default_bg")]
    selected_bg: String,
}

fn default_quit() -> String {
    "q".into()
}
fn default_up() -> String {
    "Up".into()
}
fn default_down() -> String {
    "Down".into()
}
fn default_fg() -> String {
    "Black".into()
}
fn default_bg() -> String {
    "Yellow".into()
}

impl From<RawConfig> for Config {
    fn from(raw: RawConfig) -> Self {
        Self {
            keybindings: KeyBindings {
                quit: parse_key(&raw.keybindings.quit),
                up: parse_key(&raw.keybindings.up),
                down: parse_key(&raw.keybindings.down),
            },
            theme: Theme {
                selected_fg: parse_color(&raw.theme.selected_fg),
                selected_bg: parse_color(&raw.theme.selected_bg),
            },
        }
    }
}

fn parse_key(name: &str) -> KeyCode {
    match name.to_lowercase().as_str() {
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        c if c.len() == 1 => KeyCode::Char(c.chars().next().unwrap()),
        _ => KeyCode::Null,
    }
}

fn parse_color(name: &str) -> Color {
    match name.to_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        _ => Color::Reset,
    }
}

fn config_path() -> PathBuf {
    let dir = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    dir.join("lmdb-tui").join("config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn defaults_when_missing() -> anyhow::Result<()> {
        let cfg = Config::load()?;
        assert_eq!(cfg.keybindings.quit, KeyCode::Char('q'));
        Ok(())
    }

    #[test]
    fn parses_custom_file() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("config.toml");
        fs::write(
            &path,
            r#"[keybindings]
quit = "x"
up = "k"
down = "j"

[theme]
selected_fg = "White"
selected_bg = "Blue"
"#,
        )?;
        let data = fs::read_to_string(&path)?;
        let raw: RawConfig = toml::from_str(&data)?;
        let cfg: Config = raw.into();
        assert_eq!(cfg.keybindings.quit, KeyCode::Char('x'));
        assert_eq!(cfg.theme.selected_bg, Color::Blue);
        Ok(())
    }
}
