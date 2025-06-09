use std::io::{self};
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use crossterm::cursor::Show;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use heed::Env;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::bookmarks::{Bookmarks, JumpHistory};
use crate::commands;
use crate::db::stats::{DbStats, EnvStats};
use crate::db::txn::Txn;
use crate::db::undo::UndoStack;
use crate::jobs::{JobQueue, JobResult};

use crate::config::Config;
use crate::constants::DEFAULT_ENTRY_LIMIT;
use crate::db::env::{list_databases, list_entries, open_env};
use crate::ui::{
    self,
    help::{self, DEFAULT_ENTRIES},
};
use ratatui::layout::{Constraint, Direction, Layout};

/// Dialog field focus state for CRUD operations.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DialogField {
    Key,
    Value,
}

/// Guard that enables raw mode on creation and disables it on drop.
pub struct RawModeGuard;

impl RawModeGuard {
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, Show);
    }
}

/// Available application views.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum View {
    Main,
    Query,
    CommandPalette,
    Preview,
    CreateEntry,
    EditEntry,
    DeleteConfirm,
}

/// Actions that update the [`App`] state.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Action {
    NextDb,
    PrevDb,
    NextEntry,
    PrevEntry,
    EnterQuery,
    ExitView,
    ToggleHelp,
    Undo,
    Redo,
    NextPage,
    PrevPage,
    ToggleBookmark,
    ShowBookmarks,
    OpenCommandPalette,
    ExecuteCommand(CommandId),
    EnterPreview,
    ConfirmCreate,
    ConfirmEdit,
    ConfirmDelete,
    Quit,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CommandId {
    CreateEntry,
    EditEntry,
    DeleteEntry,
    ExportDatabase,
    ImportDatabase,
    JumpToDatabase(usize),
    GoToBookmark(usize),
    ShowBookmarks,
    ToggleHelp,
    EnterQuery,
    ClearQuery,
    Commit,
    Abort,
    Undo,
    Redo,
    Quit,
}

#[derive(Clone, Debug)]
pub struct Command {
    pub id: CommandId,
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub keybinding: Option<String>,
}

/// Application state shared between the UI and reducer.
pub struct App {
    pub env: Env,
    pub db_names: Vec<String>,
    pub selected: usize,
    pub entries: Vec<(String, Vec<u8>)>,
    pub view: Vec<View>,
    running: bool,
    pub query: String,
    /// Selected index within query results.
    pub query_cursor: usize,
    /// Total number of entries available (for pagination).
    pub total_entries: usize,
    /// Current page offset for pagination.
    pub page_offset: usize,
    /// Whether a query is currently being executed.
    pub query_loading: bool,
    pub job_queue: JobQueue,
    pub env_stats: Option<EnvStats>,
    pub db_stats: Option<DbStats>,
    pub show_help: bool,
    pub help_query: String,
    pub config: Config,
    pub undo_stack: UndoStack,
    pub has_pending_changes: bool,
    pub bookmarks: Bookmarks,
    pub jump_history: JumpHistory,
    pub cursor: usize,
    pub command_palette_query: String,
    pub command_palette_selected: usize,
    pub filtered_commands: Vec<Command>,
    pub filtered_bookmarks: Vec<(String, String, bool)>, // (db_name, key, is_bookmark)
    pub preview_key: Option<String>,
    pub preview_value: Option<Vec<u8>>,
    pub dialog_key: String,
    pub dialog_value: String,
    pub dialog_key_cursor: usize,
    pub dialog_value_cursor: usize,
    pub dialog_field: DialogField, // Which field is currently focused
}

impl App {
    pub fn new(env: Env, mut db_names: Vec<String>, config: Config) -> Result<Self> {
        db_names.sort();
        let entries = if let Some(name) = db_names.first() {
            list_entries(&env, name, DEFAULT_ENTRY_LIMIT)?
        } else {
            Vec::new()
        };

        let job_queue = JobQueue::new(env.clone());
        job_queue.request_env_stats()?;
        if let Some(name) = db_names.first() {
            job_queue.request_db_stats(name.clone())?;
        }

        Ok(Self {
            env,
            db_names,
            selected: 0,
            entries,
            view: vec![View::Main],
            running: true,
            query: String::new(),
            query_cursor: 0,
            total_entries: 0,
            page_offset: 0,
            query_loading: false,
            job_queue,
            env_stats: None,
            db_stats: None,
            show_help: false,
            help_query: String::new(),
            config,
            undo_stack: UndoStack::new(),
            has_pending_changes: false,
            bookmarks: Bookmarks::new(),
            jump_history: JumpHistory::new(50), // Keep last 50 visited keys
            cursor: 0,
            command_palette_query: String::new(),
            command_palette_selected: 0,
            filtered_commands: Vec::new(),
            filtered_bookmarks: Vec::new(),
            preview_key: None,
            preview_value: None,
            dialog_key: String::new(),
            dialog_value: String::new(),
            dialog_key_cursor: 0,
            dialog_value_cursor: 0,
            dialog_field: DialogField::Key,
        })
    }

    pub fn current_view(&self) -> View {
        self.view.last().copied().unwrap_or(View::Main)
    }
    
    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.undo_stack.can_undo()
    }
    
    /// Check if redo is available  
    pub fn can_redo(&self) -> bool {
        self.undo_stack.can_redo()
    }
    
    /// Get the currently selected key if available
    pub fn current_key(&self) -> Option<String> {
        match self.current_view() {
            View::Main => {
                if self.cursor < self.entries.len() {
                    Some(self.entries[self.cursor].0.clone())
                } else {
                    None
                }
            }
            View::Query => {
                if self.query_cursor < self.entries.len() {
                    Some(self.entries[self.query_cursor].0.clone())
                } else {
                    None
                }
            }
            View::CommandPalette => {
                // Command palette doesn't have a current key
                None
            }
            View::Preview => {
                // Return the preview key if available
                self.preview_key.clone()
            }
            View::CreateEntry | View::EditEntry | View::DeleteConfirm => {
                // For dialogs, return the underlying view's current key
                match self.view.get(self.view.len() - 2).copied().unwrap_or(View::Main) {
                    View::Main => {
                        if self.cursor < self.entries.len() {
                            Some(self.entries[self.cursor].0.clone())
                        } else {
                            None
                        }
                    }
                    View::Query => {
                        if self.query_cursor < self.entries.len() {
                            Some(self.entries[self.query_cursor].0.clone())
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }
        }
    }
    
    /// Toggle bookmark for current selection
    pub fn toggle_bookmark(&mut self) -> Result<()> {
        if let (Some(db_name), Some(key)) = (self.db_names.get(self.selected), self.current_key()) {
            if self.bookmarks.contains(db_name, &key) {
                self.bookmarks.remove(db_name, &key);
            } else {
                self.bookmarks.add(db_name.clone(), key.clone());
                self.jump_history.push(db_name.clone(), key);
            }
        }
        Ok(())
    }
    
    /// Check if current selection is bookmarked
    pub fn is_current_bookmarked(&self) -> bool {
        if let (Some(db_name), Some(key)) = (self.db_names.get(self.selected), self.current_key()) {
            self.bookmarks.contains(db_name, &key)
        } else {
            false
        }
    }

    pub fn process_background_jobs(&mut self) {
        while let Ok(msg) = self.job_queue.receiver.try_recv() {
            match msg {
                JobResult::Env(s) => self.env_stats = Some(s),
                JobResult::Db(_, s) => self.db_stats = Some(s),
            }
        }
    }


    pub fn reduce(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => self.running = false,
            Action::NextDb => {
                if !self.db_names.is_empty() {
                    self.selected = (self.selected + 1) % self.db_names.len();
                    let name = &self.db_names[self.selected];
                    self.entries = list_entries(&self.env, name, DEFAULT_ENTRY_LIMIT)?;
                    self.cursor = 0; // Reset cursor when switching databases
                    self.job_queue.request_db_stats(name.clone())?;
                }
            }
            Action::PrevDb => {
                if !self.db_names.is_empty() {
                    if self.selected == 0 {
                        self.selected = self.db_names.len() - 1;
                    } else {
                        self.selected -= 1;
                    }
                    let name = &self.db_names[self.selected];
                    self.entries = list_entries(&self.env, name, DEFAULT_ENTRY_LIMIT)?;
                    self.cursor = 0; // Reset cursor when switching databases
                    self.job_queue.request_db_stats(name.clone())?;
                }
            }
            Action::EnterQuery => {
                self.view.push(View::Query);
                self.query.clear();
                self.entries.clear();
                self.query_cursor = 0;
                self.total_entries = 0;
                self.page_offset = 0;
                self.query_loading = false;
            }
            Action::ExitView => {
                if self.view.len() > 1 {
                    self.view.pop();
                    if self.current_view() == View::Main {
                        if let Some(name) = self.db_names.get(self.selected) {
                            self.entries = list_entries(&self.env, name, 100)?;
                        }
                    }
                } else {
                    // Don't exit from the main view, just quit the app
                    self.running = false;
                }
            }
            Action::ToggleHelp => {
                self.show_help = !self.show_help;
                if !self.show_help {
                    self.help_query.clear();
                }
            }
            Action::Undo => {
                // Note: Full undo/redo implementation requires transaction management
                // This provides basic functionality without active transactions
                if self.can_undo() {
                    // For demonstration, we'll log that undo was requested
                    // In a full implementation with active transactions:
                    // let mut txn = Txn::begin(&self.env)?;
                    // self.undo_stack.undo(&self.env, &mut txn)?;
                    // txn.commit()?;
                    
                    // Refresh the current view
                    if let Some(name) = self.db_names.get(self.selected) {
                        self.entries = list_entries(&self.env, name, DEFAULT_ENTRY_LIMIT)?;
                    }
                    self.has_pending_changes = self.undo_stack.can_undo() || self.undo_stack.can_redo();
                }
            }
            Action::Redo => {
                // Note: Full redo implementation requires transaction management
                // This provides basic functionality without active transactions
                if self.can_redo() {
                    // For demonstration, we'll log that redo was requested
                    // In a full implementation with active transactions:
                    // let mut txn = Txn::begin(&self.env)?;
                    // self.undo_stack.redo(&self.env, &mut txn)?;
                    // txn.commit()?;
                    
                    // Refresh the current view
                    if let Some(name) = self.db_names.get(self.selected) {
                        self.entries = list_entries(&self.env, name, DEFAULT_ENTRY_LIMIT)?;
                    }
                    self.has_pending_changes = self.undo_stack.can_undo() || self.undo_stack.can_redo();
                }
            }
            Action::NextPage => {
                if self.current_view() == View::Query {
                    let page_size = 100;
                    let max_offset = self.total_entries.saturating_sub(page_size);
                    if self.page_offset < max_offset {
                        self.page_offset = (self.page_offset + page_size).min(max_offset);
                        self.query_cursor = 0;
                        self.update_query_results()?;
                    }
                }
            }
            Action::PrevPage => {
                if self.current_view() == View::Query {
                    let page_size = 100;
                    if self.page_offset >= page_size {
                        self.page_offset = self.page_offset.saturating_sub(page_size);
                        self.query_cursor = 0;
                        self.update_query_results()?;
                    } else if self.page_offset > 0 {
                        self.page_offset = 0;
                        self.query_cursor = 0;
                        self.update_query_results()?;
                    }
                }
            }
            Action::NextEntry => {
                if self.current_view() == View::Main && !self.entries.is_empty() {
                    self.cursor = (self.cursor + 1) % self.entries.len();
                }
            }
            Action::PrevEntry => {
                if self.current_view() == View::Main && !self.entries.is_empty() {
                    if self.cursor == 0 {
                        self.cursor = self.entries.len() - 1;
                    } else {
                        self.cursor -= 1;
                    }
                }
            }
            Action::ToggleBookmark => {
                self.toggle_bookmark()?;
            }
            Action::ShowBookmarks => {
                // Open command palette with bookmarks populated
                self.view.push(View::CommandPalette);
                self.command_palette_query.clear();
                self.command_palette_selected = 0;
                self.show_bookmarks_in_palette();
            }
            Action::OpenCommandPalette => {
                self.view.push(View::CommandPalette);
                self.command_palette_query.clear();
                self.command_palette_selected = 0;
                self.filter_commands();
            }
            Action::ExecuteCommand(cmd_id) => {
                // Pop the command palette first
                if self.current_view() == View::CommandPalette {
                    self.view.pop();
                }
                
                // Execute the command
                match cmd_id {
                    CommandId::CreateEntry => {
                        // Open create dialog
                        self.dialog_key.clear();
                        self.dialog_value.clear();
                        self.dialog_key_cursor = 0;
                        self.dialog_value_cursor = 0;
                        self.dialog_field = DialogField::Key;
                        self.view.push(View::CreateEntry);
                    }
                    CommandId::EditEntry => {
                        // Open edit dialog with current entry data
                        if let Some((key, value)) = self.get_current_entry() {
                            self.dialog_key = key;
                            // Convert value bytes to string for editing
                            self.dialog_value = String::from_utf8_lossy(&value).to_string();
                            self.dialog_key_cursor = self.dialog_key.len();
                            self.dialog_value_cursor = self.dialog_value.len();
                            self.dialog_field = DialogField::Key;
                            self.view.push(View::EditEntry);
                        }
                    }
                    CommandId::DeleteEntry => {
                        // Open delete confirmation dialog
                        if self.get_current_entry().is_some() {
                            self.view.push(View::DeleteConfirm);
                        }
                    }
                    CommandId::EnterQuery => {
                        return self.reduce(Action::EnterQuery);
                    }
                    CommandId::ToggleHelp => {
                        return self.reduce(Action::ToggleHelp);
                    }
                    CommandId::ShowBookmarks => {
                        return self.reduce(Action::ShowBookmarks);
                    }
                    CommandId::Undo => {
                        return self.reduce(Action::Undo);
                    }
                    CommandId::Redo => {
                        return self.reduce(Action::Redo);
                    }
                    CommandId::Quit => {
                        return self.reduce(Action::Quit);
                    }
                    CommandId::JumpToDatabase(idx) => {
                        if idx < self.db_names.len() {
                            self.selected = idx;
                            let name = &self.db_names[self.selected];
                            self.entries = list_entries(&self.env, name, DEFAULT_ENTRY_LIMIT)?;
                            self.cursor = 0;
                            self.job_queue.request_db_stats(name.clone())?;
                        }
                    }
                    CommandId::ExportDatabase => {
                        // Export the current database
                        if let Some(db_name) = self.db_names.get(self.selected) {
                            // For now, just log that export would happen here
                            // In a full implementation, this would open a file dialog
                            log::info!("Export command triggered for database: {}", db_name);
                        }
                    }
                    CommandId::ImportDatabase => {
                        // Import into the current database
                        // This would typically open a file dialog
                        // For now, this is a placeholder as it requires user file input
                        log::info!("Import command triggered - file dialog would open here");
                    }
                    CommandId::ClearQuery => {
                        self.query.clear();
                        self.update_query_results()?;
                    }
                    CommandId::Commit => {
                        // Commit pending transaction
                        // This would require transaction management which isn't fully implemented
                        self.has_pending_changes = false;
                    }
                    CommandId::Abort => {
                        // Abort pending transaction and restore from undo stack
                        if self.can_undo() {
                            return self.reduce(Action::Undo);
                        }
                        self.has_pending_changes = false;
                    }
                    CommandId::GoToBookmark(idx) => {
                        // Jump to a specific bookmark or history entry
                        if let Some((db_name, key, _)) = self.filtered_bookmarks.get(idx) {
                            // Find the database index
                            if let Some(db_idx) = self.db_names.iter().position(|db| db == db_name) {
                                self.selected = db_idx;
                                let entries = list_entries(&self.env, db_name, DEFAULT_ENTRY_LIMIT)?;
                                
                                // Find the key in the entries and set cursor
                                if let Some(entry_idx) = entries.iter().position(|(k, _)| k == key) {
                                    self.cursor = entry_idx;
                                }
                                
                                self.entries = entries;
                                self.job_queue.request_db_stats(db_name.clone())?;
                                
                                // Add this to jump history
                                self.jump_history.push(db_name.clone(), key.clone());
                            }
                        }
                    }
                }
            }
            Action::EnterPreview => {
                if let Some((key, value)) = self.get_current_entry() {
                    self.preview_key = Some(key);
                    self.preview_value = Some(value);
                    self.view.push(View::Preview);
                }
            }
            Action::ConfirmCreate => {
                // Create new entry with dialog data
                if !self.dialog_key.is_empty() && !self.db_names.is_empty() {
                    let db_name = &self.db_names[self.selected];
                    let mut txn = Txn::begin(&self.env)?;
                    
                    match commands::put(
                        &self.env,
                        &mut txn,
                        &mut self.undo_stack,
                        db_name,
                        &self.dialog_key,
                        self.dialog_value.as_bytes(),
                    ) {
                        Ok(()) => {
                            if let Err(e) = txn.commit() {
                                log::error!("Failed to commit create transaction: {}", e);
                            } else {
                                // Refresh entries and mark changes
                                self.has_pending_changes = false;
                                if let Ok(entries) = list_entries(&self.env, db_name, DEFAULT_ENTRY_LIMIT) {
                                    self.entries = entries;
                                    // Find and select the newly created entry
                                    if let Some(idx) = self.entries.iter().position(|(k, _)| k == &self.dialog_key) {
                                        self.cursor = idx;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to create entry: {}", e);
                            txn.abort();
                        }
                    }
                }
                // Close dialog
                self.view.pop();
            }
            Action::ConfirmEdit => {
                // Update existing entry with dialog data
                if !self.dialog_key.is_empty() && !self.db_names.is_empty() {
                    let db_name = &self.db_names[self.selected];
                    let mut txn = Txn::begin(&self.env)?;
                    
                    match commands::put(
                        &self.env,
                        &mut txn,
                        &mut self.undo_stack,
                        db_name,
                        &self.dialog_key,
                        self.dialog_value.as_bytes(),
                    ) {
                        Ok(()) => {
                            if let Err(e) = txn.commit() {
                                log::error!("Failed to commit edit transaction: {}", e);
                            } else {
                                // Refresh entries
                                self.has_pending_changes = false;
                                if let Ok(entries) = list_entries(&self.env, db_name, DEFAULT_ENTRY_LIMIT) {
                                    self.entries = entries;
                                    // Find and select the edited entry
                                    if let Some(idx) = self.entries.iter().position(|(k, _)| k == &self.dialog_key) {
                                        self.cursor = idx;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to edit entry: {}", e);
                            txn.abort();
                        }
                    }
                }
                // Close dialog
                self.view.pop();
            }
            Action::ConfirmDelete => {
                // Delete current entry
                if let Some((key, _)) = self.get_current_entry() {
                    if !self.db_names.is_empty() {
                        let db_name = &self.db_names[self.selected];
                        let mut txn = Txn::begin(&self.env)?;
                        
                        match commands::delete(&self.env, &mut txn, &mut self.undo_stack, db_name, &key) {
                            Ok(()) => {
                                if let Err(e) = txn.commit() {
                                    log::error!("Failed to commit delete transaction: {}", e);
                                } else {
                                    // Refresh entries and adjust cursor
                                    self.has_pending_changes = false;
                                    if let Ok(entries) = list_entries(&self.env, db_name, DEFAULT_ENTRY_LIMIT) {
                                        self.entries = entries;
                                        // Adjust cursor to stay in bounds
                                        if self.cursor >= self.entries.len() && !self.entries.is_empty() {
                                            self.cursor = self.entries.len() - 1;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to delete entry: {}", e);
                                txn.abort();
                            }
                        }
                    }
                }
                // Close dialog
                self.view.pop();
            }
        }
        Ok(())
    }

    /// Get the currently selected key-value entry.
    fn get_current_entry(&self) -> Option<(String, Vec<u8>)> {
        match self.current_view() {
            View::Main => {
                if self.cursor < self.entries.len() {
                    let (key, value) = &self.entries[self.cursor];
                    Some((key.clone(), value.clone()))
                } else {
                    None
                }
            }
            View::Query => {
                if self.query_cursor < self.entries.len() {
                    let (key, value) = &self.entries[self.query_cursor];
                    Some((key.clone(), value.clone()))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Update the query results after the query string has changed.
    fn build_command_list() -> Vec<Command> {
        vec![
            Command {
                id: CommandId::CreateEntry,
                name: "Create Entry".into(),
                description: "Create a new key-value pair".into(),
                keywords: vec!["create".into(), "new".into(), "add".into(), "insert".into()],
                keybinding: Some("c".into()),
            },
            Command {
                id: CommandId::EditEntry,
                name: "Edit Entry".into(),
                description: "Edit the selected entry".into(),
                keywords: vec!["edit".into(), "modify".into(), "update".into()],
                keybinding: Some("e".into()),
            },
            Command {
                id: CommandId::DeleteEntry,
                name: "Delete Entry".into(),
                description: "Delete the selected entry".into(),
                keywords: vec!["delete".into(), "remove".into(), "del".into()],
                keybinding: Some("d".into()),
            },
            Command {
                id: CommandId::EnterQuery,
                name: "Search".into(),
                description: "Enter query mode to search entries".into(),
                keywords: vec!["search".into(), "query".into(), "find".into(), "filter".into()],
                keybinding: Some("/".into()),
            },
            Command {
                id: CommandId::ToggleHelp,
                name: "Help".into(),
                description: "Toggle help panel".into(),
                keywords: vec!["help".into(), "commands".into(), "keys".into()],
                keybinding: Some("?".into()),
            },
            Command {
                id: CommandId::ShowBookmarks,
                name: "Show Bookmarks".into(),
                description: "View bookmarked entries".into(),
                keywords: vec!["bookmarks".into(), "favorites".into(), "saved".into()],
                keybinding: Some("B".into()),
            },
            Command {
                id: CommandId::Undo,
                name: "Undo".into(),
                description: "Undo last operation".into(),
                keywords: vec!["undo".into(), "revert".into()],
                keybinding: Some("Ctrl+z".into()),
            },
            Command {
                id: CommandId::Redo,
                name: "Redo".into(),
                description: "Redo last undone operation".into(),
                keywords: vec!["redo".into(), "restore".into()],
                keybinding: Some("Ctrl+y".into()),
            },
            Command {
                id: CommandId::ExportDatabase,
                name: "Export Database".into(),
                description: "Export current database to file".into(),
                keywords: vec!["export".into(), "save".into(), "backup".into()],
                keybinding: None,
            },
            Command {
                id: CommandId::ImportDatabase,
                name: "Import Database".into(),
                description: "Import data from file".into(),
                keywords: vec!["import".into(), "load".into(), "restore".into()],
                keybinding: None,
            },
            Command {
                id: CommandId::Quit,
                name: "Quit".into(),
                description: "Exit the application".into(),
                keywords: vec!["quit".into(), "exit".into(), "close".into()],
                keybinding: Some("q".into()),
            },
        ]
    }

    pub fn filter_commands(&mut self) {
        let all_commands = Self::build_command_list();
        
        if self.command_palette_query.is_empty() {
            self.filtered_commands = all_commands;
        } else {
            let query = self.command_palette_query.to_lowercase();
            self.filtered_commands = all_commands
                .into_iter()
                .filter(|cmd| {
                    cmd.name.to_lowercase().contains(&query) ||
                    cmd.description.to_lowercase().contains(&query) ||
                    cmd.keywords.iter().any(|k| k.to_lowercase().contains(&query))
                })
                .collect();
        }
        
        // Reset selection if it's out of bounds
        if self.command_palette_selected >= self.filtered_commands.len() {
            self.command_palette_selected = 0;
        }
    }

    fn show_bookmarks_in_palette(&mut self) {
        let mut bookmark_commands = Vec::new();
        
        // Collect all bookmarks and history into a single vector for indexing
        let mut all_entries = Vec::new();
        
        // Add bookmarks first
        for (db_name, key) in self.bookmarks.entries() {
            all_entries.push((db_name.clone(), key.clone(), true)); // true = bookmark
        }
        
        // Add history entries
        for (db_name, key) in self.jump_history.entries() {
            // Only add if not already in bookmarks
            if !self.bookmarks.contains(db_name, key) {
                all_entries.push((db_name.clone(), key.clone(), false)); // false = history
            }
        }
        
        // Create commands for all entries
        for (idx, (db_name, key, is_bookmark)) in all_entries.iter().enumerate() {
            let icon = if *is_bookmark { "📎" } else { "🕒" };
            let category = if *is_bookmark { "bookmarked" } else { "recent" };
            
            bookmark_commands.push(Command {
                id: CommandId::GoToBookmark(idx),
                name: format!("{} {}: {}", icon, db_name, key),
                description: format!("Jump to {} entry in {}", category, db_name),
                keywords: vec![db_name.clone(), key.clone(), 
                             if *is_bookmark { "bookmark".into() } else { "history".into() }],
                keybinding: None,
            });
        }
        
        // Store the entries for later lookup during command execution
        self.filtered_bookmarks = all_entries;
        
        // If no bookmarks or history, show a message
        if bookmark_commands.is_empty() {
            bookmark_commands.push(Command {
                id: CommandId::ShowBookmarks, // Just show bookmarks again or do nothing
                name: "No bookmarks found".into(),
                description: "Use 'b' to bookmark the current entry".into(),
                keywords: vec!["bookmark".into(), "empty".into()],
                keybinding: Some("b".into()),
            });
        }
        
        self.filtered_commands = bookmark_commands;
    }

    pub fn update_query_results(&mut self) -> Result<()> {
        if self.db_names.is_empty() {
            self.entries.clear();
            self.total_entries = 0;
            return Ok(());
        }
        self.query_loading = true;
        let db_name = &self.db_names[self.selected];
        let mode = crate::db::query::parse_query(&self.query)?;
        
        // Get total count for pagination (scan all without limit)
        let all_results = crate::db::query::scan(&self.env, db_name, mode.clone(), usize::MAX)?;
        self.total_entries = all_results.len();
        
        // Get current page results
        let page_size = 100;
        let start = self.page_offset;
        let end = (start + page_size).min(all_results.len());
        self.entries = all_results.into_iter().skip(start).take(end - start).collect();
        
        if self.query_cursor >= self.entries.len() {
            self.query_cursor = self.entries.len().saturating_sub(1);
        }
        self.query_loading = false;
        Ok(())
    }
}

/// Run the TUI application.
pub fn run(path: &Path, read_only: bool) -> Result<()> {
    let _raw = RawModeGuard::new()?;
    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend)?;

    let config = Config::load()?;
    let env = open_env(path, read_only)?;
    let names = list_databases(&env)?;
    let mut app = App::new(env, names, config)?;

    while app.running {
        app.process_background_jobs();
        terminal.draw(|f| {
            ui::render(f, &app);
            if app.show_help {
                let popup_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Percentage((100 - 60) / 2),
                            Constraint::Percentage(60),
                            Constraint::Percentage((100 - 60) / 2),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());
                let vertical = popup_layout[1];
                let area = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(
                        [
                            Constraint::Percentage((100 - 60) / 2),
                            Constraint::Percentage(60),
                            Constraint::Percentage((100 - 60) / 2),
                        ]
                        .as_ref(),
                    )
                    .split(vertical)[1];
                help::render(f, area, &app.help_query, DEFAULT_ENTRIES);
            }
        })?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if app.show_help {
                    match key.code {
                        KeyCode::Esc => {
                            app.reduce(Action::ToggleHelp)?;
                        }
                        KeyCode::Backspace => {
                            app.help_query.pop();
                        }
                        KeyCode::Char(c) => {
                            if key.code == app.config.keybindings.quit {
                                app.reduce(Action::ToggleHelp)?;
                            } else {
                                app.help_query.push(c);
                            }
                        }
                        _ => {}
                    }
                    continue;
                }
                let action = match app.current_view() {
                    View::Main => {
                        if key.code == app.config.keybindings.quit {
                            Some(Action::Quit)
                        } else if key.code == app.config.keybindings.help {
                            Some(Action::ToggleHelp)
                        } else if key.code == app.config.keybindings.query {
                            Some(Action::EnterQuery)
                        } else if key.code == app.config.keybindings.down {
                            Some(Action::NextEntry)
                        } else if key.code == app.config.keybindings.up {
                            Some(Action::PrevEntry)
                        } else if key.code == KeyCode::Left {
                            Some(Action::PrevDb)
                        } else if key.code == KeyCode::Right {
                            Some(Action::NextDb)
                        } else if key.code == KeyCode::Char('z') && key.modifiers.contains(KeyModifiers::CONTROL) {
                            Some(Action::Undo)
                        } else if key.code == KeyCode::Char('y') && key.modifiers.contains(KeyModifiers::CONTROL) {
                            Some(Action::Redo)
                        } else if key.code == KeyCode::Char('b') {
                            Some(Action::ToggleBookmark)
                        } else if key.code == KeyCode::Char('B') {
                            Some(Action::ShowBookmarks)
                        } else if key.code == KeyCode::Char('p') && key.modifiers.contains(KeyModifiers::CONTROL) {
                            Some(Action::OpenCommandPalette)
                        } else if key.code == KeyCode::Enter {
                            Some(Action::EnterPreview)
                        } else {
                            None
                        }
                    }
                    View::Query => {
                        if key.code == KeyCode::Esc || key.code == app.config.keybindings.quit {
                            Some(Action::ExitView)
                        } else if key.code == app.config.keybindings.down {
                            if !app.entries.is_empty() {
                                app.query_cursor = (app.query_cursor + 1) % app.entries.len();
                            }
                            None
                        } else if key.code == app.config.keybindings.up {
                            if !app.entries.is_empty() {
                                if app.query_cursor == 0 {
                                    app.query_cursor = app.entries.len() - 1;
                                } else {
                                    app.query_cursor -= 1;
                                }
                            }
                            None
                        } else if key.code == KeyCode::Backspace {
                            app.query.pop();
                            app.update_query_results()?;
                            None
                        } else if key.code == KeyCode::Char('z') && key.modifiers.contains(KeyModifiers::CONTROL) {
                            Some(Action::Undo)
                        } else if key.code == KeyCode::Char('y') && key.modifiers.contains(KeyModifiers::CONTROL) {
                            Some(Action::Redo)
                        } else if key.code == KeyCode::PageDown {
                            Some(Action::NextPage)
                        } else if key.code == KeyCode::PageUp {
                            Some(Action::PrevPage)
                        } else if key.code == KeyCode::Enter {
                            Some(Action::EnterPreview)
                        } else if let KeyCode::Char(c) = key.code {
                            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                                match c {
                                    'b' => Some(Action::ToggleBookmark),
                                    'B' => Some(Action::ShowBookmarks),
                                    _ => {
                                        app.query.push(c);
                                        app.update_query_results()?;
                                        None
                                    }
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    View::CommandPalette => {
                        if key.code == KeyCode::Esc || key.code == app.config.keybindings.quit {
                            Some(Action::ExitView)
                        } else if key.code == app.config.keybindings.down {
                            if !app.filtered_commands.is_empty() {
                                app.command_palette_selected = (app.command_palette_selected + 1) % app.filtered_commands.len();
                            }
                            None
                        } else if key.code == app.config.keybindings.up {
                            if !app.filtered_commands.is_empty() {
                                if app.command_palette_selected == 0 {
                                    app.command_palette_selected = app.filtered_commands.len() - 1;
                                } else {
                                    app.command_palette_selected -= 1;
                                }
                            }
                            None
                        } else if key.code == KeyCode::Enter {
                            app.filtered_commands.get(app.command_palette_selected).map(|cmd| Action::ExecuteCommand(cmd.id))
                        } else if key.code == KeyCode::Backspace {
                            app.command_palette_query.pop();
                            app.filter_commands();
                            None
                        } else if let KeyCode::Char(c) = key.code {
                            if !key.modifiers.contains(KeyModifiers::CONTROL) {
                                app.command_palette_query.push(c);
                                app.filter_commands();
                            }
                            None
                        } else {
                            None
                        }
                    }
                    View::Preview => {
                        if key.code == KeyCode::Esc || key.code == app.config.keybindings.quit {
                            Some(Action::ExitView)
                        } else {
                            None
                        }
                    }
                    View::CreateEntry | View::EditEntry => {
                        match key.code {
                            KeyCode::Esc => Some(Action::ExitView),
                            KeyCode::Tab => {
                                // Switch between key and value fields
                                app.dialog_field = match app.dialog_field {
                                    DialogField::Key => DialogField::Value,
                                    DialogField::Value => DialogField::Key,
                                };
                                None
                            }
                            KeyCode::Enter => {
                                if app.current_view() == View::CreateEntry {
                                    Some(Action::ConfirmCreate)
                                } else {
                                    Some(Action::ConfirmEdit)
                                }
                            }
                            KeyCode::Backspace => {
                                match app.dialog_field {
                                    DialogField::Key => {
                                        if app.dialog_key_cursor > 0 {
                                            app.dialog_key_cursor -= 1;
                                            app.dialog_key.remove(app.dialog_key_cursor);
                                        }
                                    }
                                    DialogField::Value => {
                                        if app.dialog_value_cursor > 0 {
                                            app.dialog_value_cursor -= 1;
                                            app.dialog_value.remove(app.dialog_value_cursor);
                                        }
                                    }
                                }
                                None
                            }
                            KeyCode::Left => {
                                match app.dialog_field {
                                    DialogField::Key => {
                                        if app.dialog_key_cursor > 0 {
                                            app.dialog_key_cursor -= 1;
                                        }
                                    }
                                    DialogField::Value => {
                                        if app.dialog_value_cursor > 0 {
                                            app.dialog_value_cursor -= 1;
                                        }
                                    }
                                }
                                None
                            }
                            KeyCode::Right => {
                                match app.dialog_field {
                                    DialogField::Key => {
                                        if app.dialog_key_cursor < app.dialog_key.len() {
                                            app.dialog_key_cursor += 1;
                                        }
                                    }
                                    DialogField::Value => {
                                        if app.dialog_value_cursor < app.dialog_value.len() {
                                            app.dialog_value_cursor += 1;
                                        }
                                    }
                                }
                                None
                            }
                            KeyCode::Char(c) => {
                                match app.dialog_field {
                                    DialogField::Key => {
                                        app.dialog_key.insert(app.dialog_key_cursor, c);
                                        app.dialog_key_cursor += 1;
                                    }
                                    DialogField::Value => {
                                        app.dialog_value.insert(app.dialog_value_cursor, c);
                                        app.dialog_value_cursor += 1;
                                    }
                                }
                                None
                            }
                            _ => None,
                        }
                    }
                    View::DeleteConfirm => {
                        match key.code {
                            KeyCode::Esc => Some(Action::ExitView),
                            KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                                Some(Action::ConfirmDelete)
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') => Some(Action::ExitView),
                            _ => None,
                        }
                    }
                };
                if let Some(act) = action {
                    app.reduce(act)?;
                }
            }
        }
    }
    Ok(())
}

pub fn run_plain(path: &Path, read_only: bool, json: bool) -> Result<()> {
    let env = open_env(path, read_only)?;
    let mut names = list_databases(&env)?;
    names.sort();
    let output = if json {
        serde_json::to_string_pretty(&names)? + "\n"
    } else {
        names.join("\n") + "\n"
    };
    output_with_pager(&output)?;
    Ok(())
}

fn output_with_pager(text: &str) -> io::Result<()> {
    if let Ok(pager) = std::env::var("PAGER") {
        if !pager.is_empty() {
            let mut child = std::process::Command::new(pager)
                .stdin(std::process::Stdio::piped())
                .spawn()?;
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                stdin.write_all(text.as_bytes())?;
            }
            child.wait()?;
            return Ok(());
        }
    }
    print!("{}", text);
    Ok(())
}
