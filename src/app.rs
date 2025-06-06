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

use crate::db::stats::{DbStats, EnvStats};
use crate::jobs::{JobQueue, JobResult};

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
    pub job_queue: JobQueue,
    pub env_stats: Option<EnvStats>,
    pub db_stats: Option<DbStats>,
}

impl App {
    pub fn new(env: Env, mut db_names: Vec<String>) -> Result<Self> {
        db_names.sort();
        let entries = if let Some(name) = db_names.first() {
            list_entries(&env, name, 100)?
        } else {
            Vec::new()
        };
        
        let mut job_queue = JobQueue::new(env.clone());
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
            job_queue,
            env_stats: None,
            db_stats: None,
        })
    }

    pub fn current_view(&self) -> View {
        *self.view.last().unwrap_or(&View::Main)
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
                    self.entries = list_entries(&self.env, name, 100)?;
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
                    self.entries = list_entries(&self.env, name, 100)?;
                    self.job_queue.request_db_stats(name.clone())?;
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
        app.process_background_jobs();
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
            if let Some(stdin) = &mut child.stdin {
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
