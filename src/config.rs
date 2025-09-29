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
    pub help: KeyCode,
    pub query: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            quit: KeyCode::Char('q'),
            up: KeyCode::Up,
            down: KeyCode::Down,
            help: KeyCode::Char('?'),
            query: KeyCode::Char('/'),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub selected_fg: Color,
    pub selected_bg: Color,
    pub background: Color,
    pub foreground: Color,
    pub border: Color,
    pub highlight: Color,
    pub dim: Color,
}

impl Theme {
    pub fn selected_style(&self) -> Style {
        Style::default().fg(self.selected_fg).bg(self.selected_bg)
    }
    
    pub fn background_style(&self) -> Style {
        Style::default().fg(self.foreground).bg(self.background)
    }
    
    pub fn border_style(&self) -> Style {
        Style::default().fg(self.border)
    }
    
    pub fn highlight_style(&self) -> Style {
        Style::default().fg(self.highlight)
    }
    
    pub fn dim_style(&self) -> Style {
        Style::default().fg(self.dim)
    }
    
    pub fn dark() -> Self {
        Self {
            name: "Dark".into(),
            selected_fg: Color::Black,
            selected_bg: Color::Yellow,
            background: Color::Black,
            foreground: Color::White,
            border: Color::Gray,
            highlight: Color::Cyan,
            dim: Color::DarkGray,
        }
    }
    
    pub fn light() -> Self {
        Self {
            name: "Light".into(),
            selected_fg: Color::White,
            selected_bg: Color::Blue,
            background: Color::White,
            foreground: Color::Black,
            border: Color::DarkGray,
            highlight: Color::Blue,
            dim: Color::Gray,
        }
    }
    
    pub fn high_contrast() -> Self {
        Self {
            name: "High Contrast".into(),
            selected_fg: Color::Black,
            selected_bg: Color::White,
            background: Color::Black,
            foreground: Color::White,
            border: Color::White,
            highlight: Color::Yellow,
            dim: Color::Gray,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
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
    #[serde(default = "default_help")]
    help: String,
    #[serde(default = "default_query")]
    query: String,
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
fn default_help() -> String {
    "?".into()
}
fn default_query() -> String {
    "/".into()
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
                help: parse_key(&raw.keybindings.help),
                query: parse_key(&raw.keybindings.query),
            },
            theme: {
                let mut theme = Theme::default();
                theme.selected_fg = parse_color(&raw.theme.selected_fg);
                theme.selected_bg = parse_color(&raw.theme.selected_bg);
                theme
            },
        }
    }
}

fn parse_key(name: &str) -> KeyCode {
    match name.to_lowercase().as_str() {
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "enter" => KeyCode::Enter,
        "space" => KeyCode::Char(' '),
        "tab" => KeyCode::Tab,
        "backspace" => KeyCode::Backspace,
        "delete" => KeyCode::Delete,
        "esc" | "escape" => KeyCode::Esc,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        c if c.len() == 1 => KeyCode::Char(c.chars().next().unwrap()),
        _ => {
            log::warn!("Unknown key '{}', falling back to Null", name);
            KeyCode::Null
        }
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
        "gray" | "grey" => Color::Gray,
        "reset" => Color::Reset,
        _ => {
            log::warn!("Unknown color '{}', falling back to Reset", name);
            Color::Reset
        }
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

    #[test]
    fn parse_key_supports_special_keys() {
        assert_eq!(parse_key("enter"), KeyCode::Enter);
        assert_eq!(parse_key("Enter"), KeyCode::Enter);
        assert_eq!(parse_key("ENTER"), KeyCode::Enter);
        assert_eq!(parse_key("space"), KeyCode::Char(' '));
        assert_eq!(parse_key("esc"), KeyCode::Esc);
        assert_eq!(parse_key("escape"), KeyCode::Esc);
        assert_eq!(parse_key("pageup"), KeyCode::PageUp);
        assert_eq!(parse_key("pagedown"), KeyCode::PageDown);
        assert_eq!(parse_key("invalid"), KeyCode::Null);
    }

    #[test]
    fn parse_color_supports_common_colors() {
        assert_eq!(parse_color("red"), Color::Red);
        assert_eq!(parse_color("RED"), Color::Red);
        assert_eq!(parse_color("Red"), Color::Red);
        assert_eq!(parse_color("gray"), Color::Gray);
        assert_eq!(parse_color("grey"), Color::Gray);
        assert_eq!(parse_color("reset"), Color::Reset);
        assert_eq!(parse_color("invalid"), Color::Reset);
    }
}
