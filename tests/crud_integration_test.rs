use anyhow::Result;
use lmdb_tui::app::{App, Action, View, DialogField};
use lmdb_tui::config::Config;
use lmdb_tui::db::env::open_env;
use tempfile::TempDir;

#[test]
fn test_create_entry_dialog_flow() -> Result<()> {
    // Create a temporary database
    let temp_dir = TempDir::new()?;
    let env = open_env(temp_dir.path(), false)?;
    let config = Config::default();
    
    // Create app with a default database
    let mut app = App::new(env, vec!["data".to_string()], config)?;
    
    // Test opening create dialog
    app.reduce(Action::ExecuteCommand(lmdb_tui::app::CommandId::CreateEntry))?;
    assert_eq!(app.current_view(), View::CreateEntry);
    assert_eq!(app.dialog_field, DialogField::Key);
    assert!(app.dialog_key.is_empty());
    assert!(app.dialog_value.is_empty());
    
    // Simulate typing a key
    app.dialog_key = "test_key".to_string();
    app.dialog_key_cursor = app.dialog_key.len();
    
    // Switch to value field
    app.dialog_field = DialogField::Value;
    
    // Simulate typing a value
    app.dialog_value = "test_value".to_string();
    app.dialog_value_cursor = app.dialog_value.len();
    
    // Confirm creation
    app.reduce(Action::ConfirmCreate)?;
    
    // Should return to main view
    assert_eq!(app.current_view(), View::Main);
    
    // Check that entry was created (should be in entries list)
    let found = app.entries.iter().any(|(k, v)| {
        k == "test_key" && String::from_utf8_lossy(v) == "test_value"
    });
    assert!(found, "Created entry should be found in entries list");
    
    Ok(())
}

#[test]
fn test_edit_entry_dialog_flow() -> Result<()> {
    // Create a temporary database
    let temp_dir = TempDir::new()?;
    let env = open_env(temp_dir.path(), false)?;
    let config = Config::default();
    
    // Create app and add an initial entry
    let mut app = App::new(env, vec!["data".to_string()], config)?;
    
    // First create an entry to edit
    app.dialog_key = "original_key".to_string();
    app.dialog_value = "original_value".to_string();
    app.reduce(Action::ConfirmCreate)?;
    
    // Find and select the created entry
    if let Some(idx) = app.entries.iter().position(|(k, _)| k == "original_key") {
        app.cursor = idx;
    }
    
    // Test opening edit dialog
    app.reduce(Action::ExecuteCommand(lmdb_tui::app::CommandId::EditEntry))?;
    assert_eq!(app.current_view(), View::EditEntry);
    assert_eq!(app.dialog_key, "original_key");
    assert_eq!(app.dialog_value, "original_value");
    
    // Modify the value
    app.dialog_value = "modified_value".to_string();
    app.dialog_value_cursor = app.dialog_value.len();
    
    // Confirm edit
    app.reduce(Action::ConfirmEdit)?;
    
    // Should return to main view
    assert_eq!(app.current_view(), View::Main);
    
    // Check that entry was modified
    let found = app.entries.iter().any(|(k, v)| {
        k == "original_key" && String::from_utf8_lossy(v) == "modified_value"
    });
    assert!(found, "Modified entry should be found in entries list");
    
    Ok(())
}

#[test]
fn test_delete_entry_dialog_flow() -> Result<()> {
    // Create a temporary database
    let temp_dir = TempDir::new()?;
    let env = open_env(temp_dir.path(), false)?;
    let config = Config::default();
    
    // Create app and add an initial entry
    let mut app = App::new(env, vec!["data".to_string()], config)?;
    
    // First create an entry to delete
    app.dialog_key = "delete_me".to_string();
    app.dialog_value = "delete_value".to_string();
    app.reduce(Action::ConfirmCreate)?;
    
    // Find and select the created entry
    if let Some(idx) = app.entries.iter().position(|(k, _)| k == "delete_me") {
        app.cursor = idx;
    }
    
    let initial_count = app.entries.len();
    
    // Test opening delete dialog
    app.reduce(Action::ExecuteCommand(lmdb_tui::app::CommandId::DeleteEntry))?;
    assert_eq!(app.current_view(), View::DeleteConfirm);
    
    // Confirm deletion
    app.reduce(Action::ConfirmDelete)?;
    
    // Should return to main view
    assert_eq!(app.current_view(), View::Main);
    
    // Check that entry was deleted
    let found = app.entries.iter().any(|(k, _)| k == "delete_me");
    assert!(!found, "Deleted entry should not be found in entries list");
    assert_eq!(app.entries.len(), initial_count - 1, "Entry count should decrease by 1");
    
    Ok(())
}

#[test]
fn test_dialog_cancellation() -> Result<()> {
    // Create a temporary database
    let temp_dir = TempDir::new()?;
    let env = open_env(temp_dir.path(), false)?;
    let config = Config::default();
    
    let mut app = App::new(env, vec!["data".to_string()], config)?;
    let initial_count = app.entries.len();
    
    // Test cancelling create dialog
    app.reduce(Action::ExecuteCommand(lmdb_tui::app::CommandId::CreateEntry))?;
    assert_eq!(app.current_view(), View::CreateEntry);
    
    // Cancel dialog
    app.reduce(Action::ExitView)?;
    assert_eq!(app.current_view(), View::Main);
    assert_eq!(app.entries.len(), initial_count, "No entries should be added on cancel");
    
    // Test cancelling delete dialog
    if !app.entries.is_empty() {
        app.reduce(Action::ExecuteCommand(lmdb_tui::app::CommandId::DeleteEntry))?;
        assert_eq!(app.current_view(), View::DeleteConfirm);
        
        // Cancel dialog
        app.reduce(Action::ExitView)?;
        assert_eq!(app.current_view(), View::Main);
        assert_eq!(app.entries.len(), initial_count, "No entries should be deleted on cancel");
    }
    
    Ok(())
}