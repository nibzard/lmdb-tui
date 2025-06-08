use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::prelude::*;
use ratatui::Terminal;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;

use lmdb_tui::app::{App, Action};
use lmdb_tui::config::Config;
use lmdb_tui::db::env::{list_databases, open_env};

/// Comprehensive TUI test harness for capturing and analyzing UI state
pub struct TuiTestHarness {
    pub terminal: Terminal<TestBackend>,
    pub app: App,
    snapshots_dir: PathBuf,
    test_name: String,
    snapshot_counter: usize,
}

/// Snapshot data structure for AI analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiSnapshot {
    pub timestamp: DateTime<Utc>,
    pub test_name: String,
    pub snapshot_id: String,
    pub dimensions: (u16, u16),
    pub content: String,
    pub ansi_content: String,
    pub app_state: AppStateSnapshot,
    pub metadata: HashMap<String, String>,
}

/// Simplified app state for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStateSnapshot {
    pub current_view: String,
    pub selected_db: Option<String>,
    pub db_count: usize,
    pub entry_count: usize,
    pub query: String,
    pub query_cursor: usize,
    pub show_help: bool,
}

impl TuiTestHarness {
    /// Create a new test harness with a temporary database
    pub fn new(test_name: &str, width: u16, height: u16) -> anyhow::Result<Self> {
        let backend = TestBackend::new(width, height);
        let terminal = Terminal::new(backend)?;

        // Create temporary test database
        let temp_dir = TempDir::new()?;
        let env = open_env(temp_dir.path(), false)?;
        
        // Add some test data
        Self::populate_test_data(&env)?;
        
        let db_names = list_databases(&env)?;
        let config = Config::default();
        let app = App::new(env, db_names, config)?;

        let snapshots_dir = PathBuf::from("test_snapshots");
        fs::create_dir_all(&snapshots_dir)?;

        Ok(Self {
            terminal,
            app,
            snapshots_dir,
            test_name: test_name.to_string(),
            snapshot_counter: 0,
        })
    }

    /// Create test harness with specific database path
    pub fn with_database(test_name: &str, db_path: &std::path::Path, width: u16, height: u16) -> anyhow::Result<Self> {
        let backend = TestBackend::new(width, height);
        let terminal = Terminal::new(backend)?;

        let env = open_env(db_path, true)?; // Read-only for existing databases
        let db_names = list_databases(&env)?;
        let config = Config::default();
        let app = App::new(env, db_names, config)?;

        let snapshots_dir = PathBuf::from("test_snapshots");
        fs::create_dir_all(&snapshots_dir)?;

        Ok(Self {
            terminal,
            app,
            snapshots_dir,
            test_name: test_name.to_string(),
            snapshot_counter: 0,
        })
    }

    /// Populate test database with sample data
    fn populate_test_data(env: &heed::Env) -> anyhow::Result<()> {
        use heed::types::{Str, Bytes};
        
        let mut wtxn = env.write_txn()?;
        
        // Create a named database
        let test_db = env.create_database::<Str, Bytes>(&mut wtxn, Some("test_data"))?;
        
        // Add sample entries
        test_db.put(&mut wtxn, "user:1", b"{\"name\":\"Alice\",\"age\":30}")?;
        test_db.put(&mut wtxn, "user:2", b"{\"name\":\"Bob\",\"age\":25}")?;
        test_db.put(&mut wtxn, "config:app", b"{\"theme\":\"dark\",\"version\":\"1.0\"}")?;
        test_db.put(&mut wtxn, "session:123", b"{\"started\":\"2024-01-01\",\"active\":true}")?;
        
        // Add some entries to the unnamed database
        let unnamed_db = env.create_database::<Str, Bytes>(&mut wtxn, None)?;
        unnamed_db.put(&mut wtxn, "default:setting", b"default_value")?;
        
        wtxn.commit()?;
        Ok(())
    }

    /// Simulate a key press event
    pub fn send_key(&mut self, key_code: KeyCode) -> anyhow::Result<()> {
        self.send_key_with_modifiers(key_code, KeyModifiers::empty())
    }

    /// Simulate a key press with modifiers
    pub fn send_key_with_modifiers(&mut self, key_code: KeyCode, modifiers: KeyModifiers) -> anyhow::Result<()> {
        let key_event = KeyEvent::new(key_code, modifiers);
        let _event = Event::Key(key_event);
        
        // Simulate the event handling logic from app.rs
        let action = match self.app.current_view() {
            lmdb_tui::app::View::Main => {
                match key_code {
                    k if k == self.app.config.keybindings.quit => Some(Action::Quit),
                    k if k == self.app.config.keybindings.help => Some(Action::ToggleHelp),
                    k if k == self.app.config.keybindings.query => Some(Action::EnterQuery),
                    k if k == self.app.config.keybindings.down => Some(Action::NextDb),
                    k if k == self.app.config.keybindings.up => Some(Action::PrevDb),
                    _ => None,
                }
            }
            lmdb_tui::app::View::Query => {
                match key_code {
                    KeyCode::Esc => Some(Action::ExitView),
                    k if k == self.app.config.keybindings.quit => Some(Action::ExitView),
                    k if k == self.app.config.keybindings.down => {
                        if !self.app.entries.is_empty() {
                            self.app.query_cursor = (self.app.query_cursor + 1) % self.app.entries.len();
                        }
                        None
                    }
                    k if k == self.app.config.keybindings.up => {
                        if !self.app.entries.is_empty() {
                            if self.app.query_cursor == 0 {
                                self.app.query_cursor = self.app.entries.len() - 1;
                            } else {
                                self.app.query_cursor -= 1;
                            }
                        }
                        None
                    }
                    KeyCode::Backspace => {
                        self.app.query.pop();
                        self.app.update_query_results()?;
                        None
                    }
                    KeyCode::Char(c) => {
                        self.app.query.push(c);
                        self.app.update_query_results()?;
                        None
                    }
                    _ => None,
                }
            }
        };

        if let Some(action) = action {
            self.app.reduce(action)?;
        }

        Ok(())
    }

    /// Simulate typing a string
    pub fn type_string(&mut self, text: &str) -> anyhow::Result<()> {
        for c in text.chars() {
            self.send_key(KeyCode::Char(c))?;
        }
        Ok(())
    }

    /// Render the current state to the terminal
    pub fn render(&mut self) -> anyhow::Result<()> {
        self.app.process_background_jobs();
        self.terminal.draw(|f| {
            lmdb_tui::ui::render(f, &self.app);
        })?;
        Ok(())
    }

    /// Capture a snapshot of the current terminal state
    pub fn capture_snapshot(&mut self, description: &str) -> anyhow::Result<TuiSnapshot> {
        self.render()?;
        
        let buffer = self.terminal.backend().buffer();
        let snapshot_id = format!("{}_{}_{:03}", 
            self.test_name, 
            description.replace(' ', "_"), 
            self.snapshot_counter
        );
        
        let dimensions = (buffer.area.width, buffer.area.height);
        let content = self.buffer_to_string(buffer);
        let ansi_content = self.buffer_to_ansi_string(buffer);
        
        let app_state = AppStateSnapshot {
            current_view: format!("{:?}", self.app.current_view()),
            selected_db: self.app.db_names.get(self.app.selected).cloned(),
            db_count: self.app.db_names.len(),
            entry_count: self.app.entries.len(),
            query: self.app.query.clone(),
            query_cursor: self.app.query_cursor,
            show_help: self.app.show_help,
        };

        let mut metadata = HashMap::new();
        metadata.insert("description".to_string(), description.to_string());
        metadata.insert("counter".to_string(), self.snapshot_counter.to_string());

        let snapshot = TuiSnapshot {
            timestamp: Utc::now(),
            test_name: self.test_name.clone(),
            snapshot_id: snapshot_id.clone(),
            dimensions,
            content,
            ansi_content,
            app_state,
            metadata,
        };

        // Save snapshot to files
        self.save_snapshot(&snapshot)?;
        self.snapshot_counter += 1;

        Ok(snapshot)
    }

    /// Convert buffer to plain text string
    fn buffer_to_string(&self, buffer: &Buffer) -> String {
        let mut result = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                let cell = buffer.get(x, y);
                result.push_str(cell.symbol());
            }
            if y < buffer.area.height - 1 {
                result.push('\n');
            }
        }
        result
    }

    /// Convert buffer to ANSI-escaped string with colors
    fn buffer_to_ansi_string(&self, buffer: &Buffer) -> String {
        let mut result = String::new();
        let mut current_style = Style::default();

        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                let cell = buffer.get(x, y);
                
                // Add ANSI codes if style changed
                if cell.style() != current_style {
                    result.push_str(&self.style_to_ansi(cell.style()));
                    current_style = cell.style();
                }
                
                result.push_str(cell.symbol());
            }
            if y < buffer.area.height - 1 {
                result.push_str("\x1b[0m\n"); // Reset style at end of line
                current_style = Style::default();
            }
        }
        result.push_str("\x1b[0m"); // Reset at end
        result
    }

    /// Convert ratatui style to ANSI escape codes
    fn style_to_ansi(&self, style: Style) -> String {
        let mut ansi = String::from("\x1b[0m"); // Reset

        if let Some(fg) = style.fg {
            ansi.push_str(&format!("\x1b[{}m", self.color_to_ansi_fg(fg)));
        }
        if let Some(bg) = style.bg {
            ansi.push_str(&format!("\x1b[{}m", self.color_to_ansi_bg(bg)));
        }

        ansi
    }

    /// Convert ratatui color to ANSI foreground code
    fn color_to_ansi_fg(&self, color: Color) -> u8 {
        match color {
            Color::Black => 30,
            Color::Red => 31,
            Color::Green => 32,
            Color::Yellow => 33,
            Color::Blue => 34,
            Color::Magenta => 35,
            Color::Cyan => 36,
            Color::White => 37,
            Color::Gray => 90,
            _ => 39, // Default
        }
    }

    /// Convert ratatui color to ANSI background code
    fn color_to_ansi_bg(&self, color: Color) -> u8 {
        match color {
            Color::Black => 40,
            Color::Red => 41,
            Color::Green => 42,
            Color::Yellow => 43,
            Color::Blue => 44,
            Color::Magenta => 45,
            Color::Cyan => 46,
            Color::White => 47,
            Color::Gray => 100,
            _ => 49, // Default
        }
    }

    /// Save snapshot to files
    fn save_snapshot(&self, snapshot: &TuiSnapshot) -> anyhow::Result<()> {
        let base_path = self.snapshots_dir.join(&snapshot.snapshot_id);
        
        // Save as plain text
        fs::write(
            base_path.with_extension("txt"),
            &snapshot.content
        )?;
        
        // Save as ANSI
        fs::write(
            base_path.with_extension("ansi"),
            &snapshot.ansi_content
        )?;
        
        // Save as JSON for AI analysis
        fs::write(
            base_path.with_extension("json"),
            serde_json::to_string_pretty(snapshot)?
        )?;

        Ok(())
    }

    /// Generate a comprehensive test report for AI analysis
    pub fn generate_test_report(&self, snapshots: &[TuiSnapshot]) -> anyhow::Result<()> {
        #[derive(Serialize)]
        struct TestReport {
            test_session: TestSession,
        }

        #[derive(Serialize)]
        struct TestSession {
            timestamp: DateTime<Utc>,
            test_name: String,
            total_snapshots: usize,
            snapshots: Vec<TuiSnapshot>,
            instructions: String,
            analysis_hints: Vec<String>,
        }

        let report = TestReport {
            test_session: TestSession {
                timestamp: Utc::now(),
                test_name: self.test_name.clone(),
                total_snapshots: snapshots.len(),
                snapshots: snapshots.to_vec(),
                instructions: "Analyze UI layout, consistency, user experience, and identify potential issues".to_string(),
                analysis_hints: vec![
                    "Check for visual consistency across views".to_string(),
                    "Verify proper highlighting and selection states".to_string(),
                    "Look for text truncation or overflow issues".to_string(),
                    "Assess information density and readability".to_string(),
                    "Evaluate navigation flow and user feedback".to_string(),
                ],
            },
        };

        let report_path = self.snapshots_dir.join(format!("{}_report.json", self.test_name));
        fs::write(report_path, serde_json::to_string_pretty(&report)?)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harness_creation() -> anyhow::Result<()> {
        let harness = TuiTestHarness::new("test_creation", 80, 24)?;
        assert_eq!(harness.app.db_names.len(), 1); // Should have test_data db
        Ok(())
    }

    #[test]
    fn test_snapshot_capture() -> anyhow::Result<()> {
        let mut harness = TuiTestHarness::new("test_snapshot", 40, 10)?;
        let snapshot = harness.capture_snapshot("initial_state")?;
        
        assert_eq!(snapshot.dimensions, (40, 10));
        assert!(!snapshot.content.is_empty());
        assert!(!snapshot.ansi_content.is_empty());
        assert_eq!(snapshot.app_state.current_view, "Main");
        
        Ok(())
    }
}