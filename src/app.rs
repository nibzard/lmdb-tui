use std::io::{self};
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use crossterm::cursor::Show;
use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, List, ListItem};

use crate::db::stats::{DbStats, EnvStats};
use crate::jobs::{JobQueue, JobResult};
use ratatui::Terminal;

use crate::db::env::{list_databases, list_entries, open_env};

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

pub fn run(path: &Path, read_only: bool) -> Result<()> {
    let _raw = RawModeGuard::new()?;
    let mut stdout = io::stdout();
    let backend = CrosstermBackend::new(&mut stdout);
    let mut terminal = Terminal::new(backend)?;

    let env = open_env(path, read_only)?;
    let mut db_names = list_databases(&env)?;
    db_names.sort();
    let mut selected = 0usize;
    let mut entries = if let Some(name) = db_names.first() {
        list_entries(&env, name, 100)?
    } else {
        Vec::new()
    };
    let mut queue = JobQueue::new(env.clone());
    queue.request_env_stats()?;
    if let Some(name) = db_names.first() {
        queue.request_db_stats(name.clone())?;
    }
    let mut env_stats: Option<EnvStats> = None;
    let mut db_stats: Option<DbStats> = None;

    loop {
        while let Ok(msg) = queue.receiver.try_recv() {
            match msg {
                JobResult::Env(s) => env_stats = Some(s),
                JobResult::Db(_, s) => db_stats = Some(s),
            }
        }

        terminal.draw(|f| {
            let main = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(f.size());

            let stats_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(main[0]);

            if let Some(ref s) = env_stats {
                crate::ui::stats::render_env(f, stats_chunks[0], s);
            }
            if let Some(ref s) = db_stats {
                crate::ui::stats::render_db(f, stats_chunks[1], s);
            }

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                .split(main[1]);

            let items: Vec<ListItem> = db_names
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    let content = if i == selected {
                        Span::styled(
                            name.clone(),
                            Style::default().add_modifier(Modifier::REVERSED),
                        )
                    } else {
                        Span::raw(name.clone())
                    };
                    ListItem::new(content)
                })
                .collect();
            let list =
                List::new(items).block(Block::default().borders(Borders::ALL).title("Databases"));
            f.render_widget(list, chunks[0]);

            let kv_items: Vec<ListItem> = entries
                .iter()
                .map(|(k, v)| {
                    let val_str = String::from_utf8_lossy(v);
                    let text = format!("{}: {}", k, val_str);
                    ListItem::new(text)
                })
                .collect();
            let kv_list =
                List::new(kv_items).block(Block::default().borders(Borders::ALL).title("Entries"));
            f.render_widget(kv_list, chunks[1]);
        })?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Down => {
                        if !db_names.is_empty() {
                            selected = (selected + 1) % db_names.len();
                            let name = &db_names[selected];
                            entries = list_entries(&env, name, 100)?;
                            queue.request_db_stats(name.clone())?;
                        }
                    }
                    KeyCode::Up => {
                        if !db_names.is_empty() {
                            selected = if selected == 0 {
                                db_names.len() - 1
                            } else {
                                selected - 1
                            };
                            let name = &db_names[selected];
                            entries = list_entries(&env, name, 100)?;
                            queue.request_db_stats(name.clone())?;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
