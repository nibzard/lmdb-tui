use std::io::{self};
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use crossterm::cursor::Show;
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use heed::Env;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::db::env::{list_databases, list_entries, open_env};
use crate::ui;

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
}

/// Actions that update the [`App`] state.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Action {
    NextDb,
    PrevDb,
    EnterQuery,
    ExitView,
    Quit,
}

/// Application state shared between the UI and reducer.
pub struct App {
    pub env: Env,
    pub db_names: Vec<String>,
    pub selected: usize,
    pub entries: Vec<(String, Vec<u8>)>,
    view: Vec<View>,
    running: bool,
    pub query: String,
}

impl App {
    pub fn new(env: Env, mut db_names: Vec<String>) -> Result<Self> {
        db_names.sort();
        let entries = if let Some(name) = db_names.first() {
            list_entries(&env, name, 100)?
        } else {
            Vec::new()
        };
        Ok(Self {
            env,
            db_names,
            selected: 0,
            entries,
            view: vec![View::Main],
            running: true,
            query: String::new(),
        })
    }

    pub fn current_view(&self) -> View {
        *self.view.last().unwrap_or(&View::Main)
    }

    pub fn reduce(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => self.running = false,
            Action::NextDb => {
                if !self.db_names.is_empty() {
                    self.selected = (self.selected + 1) % self.db_names.len();
                    let name = &self.db_names[self.selected];
                    self.entries = list_entries(&self.env, name, 100)?;
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
                    self.entries = list_entries(&self.env, name, 100)?;
                }
            }
            Action::EnterQuery => self.view.push(View::Query),
            Action::ExitView => {
                if self.view.len() > 1 {
                    self.view.pop();
                } else {
                    self.running = false;
                }
            }
        }
        Ok(())
    }
}

/// Run the TUI application.
pub fn run(path: &Path, read_only: bool) -> Result<()> {
    let _raw = RawModeGuard::new()?;
    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend)?;

    let env = open_env(path, read_only)?;
    let names = list_databases(&env)?;
    let mut app = App::new(env, names)?;

    while app.running {
        terminal.draw(|f| ui::render(f, &app))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                let action = match app.current_view() {
                    View::Main => match key.code {
                        KeyCode::Char('q') => Some(Action::Quit),
                        KeyCode::Char('/') => Some(Action::EnterQuery),
                        KeyCode::Down => Some(Action::NextDb),
                        KeyCode::Up => Some(Action::PrevDb),
                        _ => None,
                    },
                    View::Query => match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => Some(Action::ExitView),
                        _ => None,
                    },
                };
                if let Some(act) = action {
                    app.reduce(act)?;
                }
            }
        }
    }
    Ok(())
}
