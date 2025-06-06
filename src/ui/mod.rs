pub mod db_view;
pub mod help;
pub mod query;
pub mod stats;

use ratatui::prelude::Frame;

use crate::app::{App, View};

/// Render the application based on the current [`View`].
pub fn render(f: &mut Frame, app: &App) {
    let area = f.size();
    match app.current_view() {
        View::Main => db_view::render(f, area, &app.db_names, app.selected, &app.entries),
        View::Query => query::render(f, area, &app.query, &app.entries),
    }
}
