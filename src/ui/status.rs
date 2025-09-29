use ratatui::{
    prelude::{Frame, Rect},
    widgets::Paragraph,
};

use crate::app::{App, View};
use crate::util::{key_label, truncate_with_ellipsis};

pub fn render(f: &mut Frame, area: Rect, app: &App) {
    let cfg = &app.config;

    // Generate contextual help based on current view
    let mut text = match app.current_view() {
        View::Main => {
            let mut hints = vec![
                format!("{}: quit", key_label(&cfg.keybindings.quit)),
                format!("{}: help", key_label(&cfg.keybindings.help)),
                format!("{}: query", key_label(&cfg.keybindings.query)),
            ];

            if !app.db_names.is_empty() {
                hints.push(format!(
                    "{}/{}: entries",
                    key_label(&cfg.keybindings.up),
                    key_label(&cfg.keybindings.down)
                ));
                hints.push("←/→: databases".to_string());

                // Add bookmark hints
                if app.is_current_bookmarked() {
                    hints.push("b: remove bookmark".to_string());
                } else {
                    hints.push("b: bookmark".to_string());
                }
                hints.push("B: show history".to_string());
                hints.push("Ctrl+P: commands".to_string());
            }

            // Add undo/redo hints based on availability
            if app.can_undo() {
                hints.push("Ctrl+Z: undo".to_string());
            }
            if app.can_redo() {
                hints.push("Ctrl+Y: redo".to_string());
            }

            hints.join(" | ")
        }
        View::CommandPalette => {
            "Type to search commands • ↑/↓: navigate • Enter: execute • Esc: close".to_string()
        }
        View::Query => {
            let mut hints = vec![
                "Esc: back".to_string(),
                format!("{}: quit", key_label(&cfg.keybindings.quit)),
            ];

            if !app.entries.is_empty() {
                hints.push(format!("{}: up", key_label(&cfg.keybindings.up)));
                hints.push(format!("{}: down", key_label(&cfg.keybindings.down)));
            }

            if app.total_entries > app.entries.len() {
                hints.push("PgUp/PgDn: pages".to_string());
            }

            if !app.entries.is_empty() {
                hints.push("Enter: select".to_string());
            }

            // Add undo/redo hints based on availability
            if app.can_undo() {
                hints.push("Ctrl+Z: undo".to_string());
            }
            if app.can_redo() {
                hints.push("Ctrl+Y: redo".to_string());
            }

            hints.join(" | ")
        }
        View::Preview => "Esc: close preview • q: quit".to_string(),
        View::CreateEntry => "Tab: switch fields • Enter: create • Esc: cancel".to_string(),
        View::EditEntry => "Tab: switch fields • Enter: save • Esc: cancel".to_string(),
        View::DeleteConfirm => "Y/Enter: confirm delete • N/Esc: cancel".to_string(),
    };

    // Add pending changes indicator
    if app.has_pending_changes {
        text = format!("● {} ", text); // Bullet point indicates pending changes
    }

    // Add loading indicator for query operations
    if app.query_loading && app.current_view() == View::Query {
        text = format!("{} {} ", app.get_spinner_char(), text);
    }

    // Truncate to fit available width
    let available_width = area.width as usize;
    let truncated_text = truncate_with_ellipsis(&text, available_width);

    let p = Paragraph::new(truncated_text);
    f.render_widget(p, area);
}
