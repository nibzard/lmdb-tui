use ratatui::{
    prelude::{Constraint, Direction, Frame, Layout, Rect, Style},
    style::Color,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::app::App;
use crate::util::{format_kv_entry, format_size, truncate_with_ellipsis};

/// Render the main database view.
pub fn render(f: &mut Frame, area: Rect, app: &App) {
    // Use three-column layout for large terminals, two-column for smaller ones
    let use_stats_panel = area.width >= 120;

    let chunks = if use_stats_panel {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25), // Databases
                Constraint::Percentage(60), // Entries
                Constraint::Percentage(15), // Stats
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area)
    };

    // Calculate available width for database names (subtract borders and padding)
    let db_width = chunks[0].width.saturating_sub(3);

    let items: Vec<ListItem> = app
        .db_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let truncated_name = truncate_with_ellipsis(name, db_width as usize);
            let content = if i == app.selected {
                Span::styled(truncated_name, app.config.theme.selected_style())
            } else {
                Span::raw(truncated_name)
            };
            ListItem::new(content)
        })
        .collect();
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Databases"));
    f.render_widget(list, chunks[0]);

    // Calculate available width for entries (subtract borders and bookmark indicator)
    let entry_width = chunks[1].width.saturating_sub(5);

    let kv_items: Vec<ListItem> = app
        .entries
        .iter()
        .enumerate()
        .map(|(i, (k, v))| {
            let formatted = format_kv_entry(k, v, entry_width as usize);

            // Add bookmark indicator and highlighting for selected item
            let is_bookmarked = if let Some(db_name) = app.db_names.get(app.selected) {
                app.bookmarks.contains(db_name, k)
            } else {
                false
            };

            let line = if i == app.cursor {
                // Selected item with cursor and optional bookmark
                let bookmark_indicator = if is_bookmarked { "★ " } else { "▶ " };
                Line::from(vec![
                    Span::styled(bookmark_indicator, Style::default().fg(Color::Yellow)),
                    Span::styled(formatted, app.config.theme.selected_style()),
                ])
            } else {
                // Regular item with optional bookmark
                let bookmark_indicator = if is_bookmarked { "★ " } else { "  " };
                Line::from(vec![
                    Span::styled(bookmark_indicator, Style::default().fg(Color::Blue)),
                    Span::raw(formatted),
                ])
            };

            ListItem::new(line)
        })
        .collect();

    let entry_title = format!("Entries ({})", app.entries.len());

    let kv_list =
        List::new(kv_items).block(Block::default().borders(Borders::ALL).title(entry_title));
    f.render_widget(kv_list, chunks[1]);

    // Render stats panel for large terminals
    if use_stats_panel {
        render_stats_panel(f, chunks[2], app);
    }
}

/// Render a stats panel with database information.
fn render_stats_panel(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().borders(Borders::ALL).title("Stats");
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let mut stats_text = vec![];

    // Database info
    stats_text.push(format!("Databases: {}", app.db_names.len()));

    if let Some(db_name) = app.db_names.get(app.selected) {
        stats_text.push(format!("Selected: {}", truncate_with_ellipsis(db_name, 15)));
    }

    // Entry info
    stats_text.push(format!("Entries: {}", app.entries.len()));

    // Bookmark info
    let bookmarked_in_current_db = if let Some(db_name) = app.db_names.get(app.selected) {
        app.entries
            .iter()
            .filter(|(k, _)| app.bookmarks.contains(db_name, k))
            .count()
    } else {
        0
    };

    if bookmarked_in_current_db > 0 {
        stats_text.push(format!("Bookmarks: {}", bookmarked_in_current_db));
    }

    if !app.entries.is_empty() {
        let total_size: usize = app.entries.iter().map(|(k, v)| k.len() + v.len()).sum();
        stats_text.push(format!("Size: {}", format_size(total_size as u64)));

        let avg_key_len: f64 = app.entries.iter().map(|(k, _)| k.len()).sum::<usize>() as f64
            / app.entries.len() as f64;
        let avg_val_len: f64 = app.entries.iter().map(|(_, v)| v.len()).sum::<usize>() as f64
            / app.entries.len() as f64;

        stats_text.push(format!("Avg key: {:.1}", avg_key_len));
        stats_text.push(format!("Avg val: {:.1}", avg_val_len));
    }

    let content = stats_text.join("\n");
    let paragraph = Paragraph::new(content);
    f.render_widget(paragraph, inner);
}
