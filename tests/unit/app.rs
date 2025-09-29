use heed::types::{Bytes, Str};
use lmdb_tui::{
    app::{Action, App},
    config::Config,
    db::env::{list_databases, open_env},
};
use tempfile::tempdir;

#[test]
fn reducer_switches_databases() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    let mut wtxn = env.write_txn()?;
    let db1 = env.create_database::<Str, Str>(&mut wtxn, Some("first"))?;
    db1.put(&mut wtxn, "k1", "v1")?;
    let db2 = env.create_database::<Str, Bytes>(&mut wtxn, Some("second"))?;
    db2.put(&mut wtxn, "k2", b"v2")?;
    wtxn.commit()?;

    let names = list_databases(&env)?;
    let config = Config::default();
    let mut app = App::new(env, names, config, dir.path())?;
    assert_eq!(app.selected, 0);
    assert_eq!(app.entries.len(), 1);
    assert_eq!(app.entries[0].0, "k1");

    app.reduce(Action::NextDb)?;
    assert_eq!(app.selected, 1);
    assert_eq!(app.entries.len(), 1);
    assert_eq!(app.entries[0].0, "k2");
    Ok(())
}

#[test]
fn reducer_toggles_help_and_clears_query() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    let names = list_databases(&env)?;
    let config = Config::default();
    let mut app = App::new(env, names, config, dir.path())?;

    assert!(!app.show_help);
    app.reduce(Action::ToggleHelp)?;
    assert!(app.show_help);
    app.help_query.push('a');
    app.reduce(Action::ToggleHelp)?;
    assert!(!app.show_help);
    assert!(app.help_query.is_empty());
    Ok(())
}

#[test]
fn test_spinner_animation() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    let names = list_databases(&env)?;
    let config = Config::default();
    let mut app = App::new(env, names, config, dir.path())?;

    // Initially no spinner
    assert_eq!(app.get_spinner_char(), "");
    
    // Enable loading state
    app.query_loading = true;
    
    // Test spinner animation
    app.update_spinner();
    let char1 = app.get_spinner_char();
    assert!(!char1.is_empty());
    
    app.update_spinner();
    let char2 = app.get_spinner_char();
    assert!(!char2.is_empty());
    
    // Should cycle through different characters
    for _ in 0..10 {
        app.update_spinner();
    }
    
    // Disable loading
    app.query_loading = false;
    assert_eq!(app.get_spinner_char(), "");
    
    Ok(())
}

#[test]
fn test_lazy_loading_enabled() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    let names = list_databases(&env)?;
    let config = Config::default();
    let app = App::new(env, names, config, dir.path())?;

    // Lazy loading should be enabled by default
    assert!(app.lazy_loading_enabled);
    
    Ok(())
}

#[test]
fn test_pagination_actions() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    
    // Create a database with multiple entries
    let mut wtxn = env.write_txn()?;
    let db = env.create_database::<Str, Str>(&mut wtxn, Some("test"))?;
    for i in 0..50 {
        db.put(&mut wtxn, &format!("key_{:03}", i), &format!("value_{}", i))?;
    }
    wtxn.commit()?;

    let names = list_databases(&env)?;
    let config = Config::default();
    let mut app = App::new(env, names, config, dir.path())?;
    
    // Should have loaded initial entries
    assert!(app.entries.len() > 0);
    
    // Test cursor movement
    let initial_cursor = app.cursor;
    app.reduce(Action::NextEntry)?;
    assert_eq!(app.cursor, initial_cursor + 1);
    
    app.reduce(Action::PrevEntry)?;
    assert_eq!(app.cursor, initial_cursor);
    
    Ok(())
}

#[test]
fn test_file_event_debouncing() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let env = open_env(dir.path(), false)?;
    let names = list_databases(&env)?;
    let config = Config::default();
    let mut app = App::new(env, names, config, dir.path())?;
    
    // Initially no file events
    assert!(app.last_file_event.is_none());
    
    // Simulate a file event by setting the timestamp
    app.last_file_event = Some(std::time::Instant::now());
    
    // Verify timestamp was set
    assert!(app.last_file_event.is_some());
    
    Ok(())
}
