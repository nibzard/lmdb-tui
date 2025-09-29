pub mod command_palette;
pub mod db_view;
pub mod dialogs;
pub mod help;
pub mod preview;
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
        View::Main => db_view::render(f, chunks[0], app),
        View::Query => query::render(
            f,
            chunks[0],
            query::QueryViewParams {
                query: &app.query,
                entries: &app.entries,
                selected: app.query_cursor,
                total_entries: app.total_entries,
                page_offset: app.page_offset,
                loading: app.query_loading,
                config: &app.config,
                spinner_char: app.get_spinner_char(),
            },
        ),
        View::CommandPalette => {
            // Render underlying view first
            match app
                .view
                .get(app.view.len() - 2)
                .copied()
                .unwrap_or(View::Main)
            {
                View::Main => db_view::render(f, chunks[0], app),
                View::Query => query::render(
                    f,
                    chunks[0],
                    query::QueryViewParams {
                        query: &app.query,
                        entries: &app.entries,
                        selected: app.query_cursor,
                        total_entries: app.total_entries,
                        page_offset: app.page_offset,
                        loading: app.query_loading,
                        config: &app.config,
                        spinner_char: app.get_spinner_char(),
                    },
                ),
                _ => {}
            }
            // Then render command palette popup
            command_palette::render(
                f,
                command_palette::CommandPaletteParams {
                    query: &app.command_palette_query,
                    commands: &app.filtered_commands,
                    selected: app.command_palette_selected,
                    config: &app.config,
                },
            );
        }
        View::Preview => {
            // Render underlying view first
            match app
                .view
                .get(app.view.len() - 2)
                .copied()
                .unwrap_or(View::Main)
            {
                View::Main => db_view::render(f, chunks[0], app),
                View::Query => query::render(
                    f,
                    chunks[0],
                    query::QueryViewParams {
                        query: &app.query,
                        entries: &app.entries,
                        selected: app.query_cursor,
                        total_entries: app.total_entries,
                        page_offset: app.page_offset,
                        loading: app.query_loading,
                        config: &app.config,
                        spinner_char: app.get_spinner_char(),
                    },
                ),
                _ => {}
            }
            // Then render preview popup
            if let (Some(key), Some(value)) = (&app.preview_key, &app.preview_value) {
                preview::render(f, key, value);
            }
        }
        View::CreateEntry => {
            // Render underlying view first
            match app
                .view
                .get(app.view.len() - 2)
                .copied()
                .unwrap_or(View::Main)
            {
                View::Main => db_view::render(f, chunks[0], app),
                View::Query => query::render(
                    f,
                    chunks[0],
                    query::QueryViewParams {
                        query: &app.query,
                        entries: &app.entries,
                        selected: app.query_cursor,
                        total_entries: app.total_entries,
                        page_offset: app.page_offset,
                        loading: app.query_loading,
                        config: &app.config,
                        spinner_char: app.get_spinner_char(),
                    },
                ),
                _ => {}
            }
            // Then render create dialog
            dialogs::render_create_dialog(f, app);
        }
        View::EditEntry => {
            // Render underlying view first
            match app
                .view
                .get(app.view.len() - 2)
                .copied()
                .unwrap_or(View::Main)
            {
                View::Main => db_view::render(f, chunks[0], app),
                View::Query => query::render(
                    f,
                    chunks[0],
                    query::QueryViewParams {
                        query: &app.query,
                        entries: &app.entries,
                        selected: app.query_cursor,
                        total_entries: app.total_entries,
                        page_offset: app.page_offset,
                        loading: app.query_loading,
                        config: &app.config,
                        spinner_char: app.get_spinner_char(),
                    },
                ),
                _ => {}
            }
            // Then render edit dialog
            dialogs::render_edit_dialog(f, app);
        }
        View::DeleteConfirm => {
            // Render underlying view first
            match app
                .view
                .get(app.view.len() - 2)
                .copied()
                .unwrap_or(View::Main)
            {
                View::Main => db_view::render(f, chunks[0], app),
                View::Query => query::render(
                    f,
                    chunks[0],
                    query::QueryViewParams {
                        query: &app.query,
                        entries: &app.entries,
                        selected: app.query_cursor,
                        total_entries: app.total_entries,
                        page_offset: app.page_offset,
                        loading: app.query_loading,
                        config: &app.config,
                        spinner_char: app.get_spinner_char(),
                    },
                ),
                _ => {}
            }
            // Then render delete confirmation dialog
            dialogs::render_delete_dialog(f, app);
        }
    }
    status::render(f, chunks[1], app);
}
