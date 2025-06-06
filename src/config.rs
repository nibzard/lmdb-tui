use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct Config {
    #[serde(default)]
    pub keymap: BTreeMap<String, String>,
    pub theme: Option<String>,
}

pub fn load(path: &Path) -> Result<Config> {
    let data = std::fs::read_to_string(path)?;
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let cfg = if ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml") {
        serde_yaml::from_str(&data)?
    } else {
        toml::from_str(&data)?
    };
    Ok(cfg)
}

pub fn save(config: &Config, path: &Path) -> Result<()> {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
    let data = if ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml") {
        serde_yaml::to_string(config)?
    } else {
        toml::to_string_pretty(config)?
    };
    std::fs::write(path, data)?;
    Ok(())
}
