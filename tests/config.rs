use lmdb_tui::config::{self, Config};
use tempfile::Builder;

#[test]
fn roundtrip_toml() -> anyhow::Result<()> {
    let cfg = Config {
        theme: Some("dark".into()),
        keymap: [("quit".into(), "q".into())].into_iter().collect(),
    };
    let file = Builder::new().suffix(".toml").tempfile()?;
    config::save(&cfg, file.path())?;
    let loaded = config::load(file.path())?;
    assert_eq!(loaded, cfg);
    Ok(())
}

#[test]
fn roundtrip_yaml() -> anyhow::Result<()> {
    let cfg = Config {
        theme: Some("light".into()),
        keymap: [("open".into(), "o".into())].into_iter().collect(),
    };
    let file = Builder::new().suffix(".yaml").tempfile()?;
    config::save(&cfg, file.path())?;
    let loaded = config::load(file.path())?;
    assert_eq!(loaded, cfg);
    Ok(())
}
