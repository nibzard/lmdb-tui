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

use crate::config::Config;
use crate::db::env::{list_databases, list_entries, open_env};
use crate::ui::{self, help::{self, DEFAULT_ENTRIES}};
use ratatui::layout::{Constraint, Direction, Layout};

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);
    let vertical = popup_layout[1];
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(vertical)[1]
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
}

/// Actions that update the [`App`] state.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Action {
    NextDb,
    PrevDb,
    EnterQuery,
    ExitView,
    ToggleHelp,
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
    pub show_help: bool,
    pub help_query: String,
    pub config: Config,
}

impl App {
    pub fn new(env: Env, mut db_names: Vec<String>, config: Config) -> Result<Self> {
        db_names.sort();
        let entries = if let Some(name) = db_names.first() {
            list_entries(&env, name, 100)?
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
            job_queue,
            env_stats: None,
            db_stats: None,
            show_help: false,
            help_query: String::new(),
            config,
        })
    }

    pub fn current_view(&self) -> View {
        self.view.last().copied().unwrap_or(View::Main)
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

    let config = Config::load()?;
    let env = open_env(path, read_only)?;
    let names = list_databases(&env)?;
    let mut app = App::new(env, names, config)?;

    while app.running {
        app.process_background_jobs();
        terminal.draw(|f| {
            ui::render(f, &app);
            if app.show_help {
                let area = centered_rect(60, 60, f.size());
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
                            Some(Action::NextDb)
                        } else if key.code == app.config.keybindings.up {
                            Some(Action::PrevDb)
                        } else {
                            None
                        }
                    }
                    View::Query => {
                        if key.code == KeyCode::Esc || key.code == app.config.keybindings.quit {
                            Some(Action::ExitView)
                        } else {
                            None
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
