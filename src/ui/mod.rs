pub mod db_view;
pub mod help;
pub mod query;
pub mod stats;
pub mod status;

use ratatui::prelude::{Constraint, Direction, Frame, Layout};

use crate::app::{App, View};

/// Render the application based on the current [`View`].
pub fn render(f: &mut Frame, app: &App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(size);
    match app.current_view() {
        View::Main => db_view::render(
            f,
            chunks[0],
            &app.db_names,
            app.selected,
            &app.entries,
            &app.config,
        ),
        View::Query => query::render(f, chunks[0], &app.query, &app.entries),
    }
    status::render(f, chunks[1], &app.config);
}
