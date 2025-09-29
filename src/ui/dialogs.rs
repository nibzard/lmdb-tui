use ratatui::{
    prelude::{Constraint, Direction, Frame, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::{App, DialogField};

/// Render the create entry dialog.
pub fn render_create_dialog(f: &mut Frame, app: &App) {
    render_entry_dialog(f, app, "Create Entry", "Enter key and value for new entry:");
}

/// Render the edit entry dialog.
pub fn render_edit_dialog(f: &mut Frame, app: &App) {
    render_entry_dialog(f, app, "Edit Entry", "Modify key and value:");
}

/// Common function to render create/edit dialogs.
fn render_entry_dialog(f: &mut Frame, app: &App, title: &str, description: &str) {
    // Create a centered popup layout (70% of screen)
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Percentage(70),
            Constraint::Percentage(15),
        ])
        .split(f.size());

    let vertical = popup_layout[1];
    let area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Percentage(70),
            Constraint::Percentage(15),
        ])
        .split(vertical)[1];

    // Clear the background to ensure the popup appears on top
    f.render_widget(Clear, area);

    // Create the main container with background
    let block = Block::default()
        .title(format!(" {} ", title))
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));
    f.render_widget(block.clone(), area);

    let inner = block.inner(area);

    // Split into sections: description, key, value
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Description
            Constraint::Length(3), // Key field
            Constraint::Length(3), // Value field
            Constraint::Min(0),    // Spacer
        ])
        .split(inner);

    // Render description
    let desc_paragraph = Paragraph::new(description)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .wrap(Wrap { trim: false });
    f.render_widget(desc_paragraph, sections[0]);

    // Render key field
    let key_focused = app.dialog_field == DialogField::Key;
    let key_block = Block::default().title(" Key ").borders(Borders::ALL).style(
        Style::default().bg(Color::DarkGray).fg(if key_focused {
            Color::Yellow
        } else {
            Color::White
        }),
    );
    f.render_widget(key_block.clone(), sections[1]);

    let key_inner = key_block.inner(sections[1]);
    let key_content = if key_focused {
        // Show cursor in key field
        let (before, after) = app.dialog_key.split_at(app.dialog_key_cursor);
        vec![
            Span::styled(
                before,
                Style::default().bg(Color::DarkGray).fg(Color::White),
            ),
            Span::styled("█", Style::default().bg(Color::Yellow).fg(Color::Black)),
            Span::styled(after, Style::default().bg(Color::DarkGray).fg(Color::White)),
        ]
    } else {
        vec![Span::styled(
            &app.dialog_key,
            Style::default().bg(Color::DarkGray).fg(Color::White),
        )]
    };
    let key_paragraph =
        Paragraph::new(Line::from(key_content)).style(Style::default().bg(Color::DarkGray));
    f.render_widget(key_paragraph, key_inner);

    // Render value field
    let value_focused = app.dialog_field == DialogField::Value;
    let value_block = Block::default()
        .title(" Value ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray).fg(if value_focused {
            Color::Yellow
        } else {
            Color::White
        }));
    f.render_widget(value_block.clone(), sections[2]);

    let value_inner = value_block.inner(sections[2]);
    let value_content = if value_focused {
        // Show cursor in value field
        let (before, after) = app.dialog_value.split_at(app.dialog_value_cursor);
        vec![
            Span::styled(
                before,
                Style::default().bg(Color::DarkGray).fg(Color::White),
            ),
            Span::styled("█", Style::default().bg(Color::Yellow).fg(Color::Black)),
            Span::styled(after, Style::default().bg(Color::DarkGray).fg(Color::White)),
        ]
    } else {
        vec![Span::styled(
            &app.dialog_value,
            Style::default().bg(Color::DarkGray).fg(Color::White),
        )]
    };
    let value_paragraph = Paragraph::new(Line::from(value_content))
        .style(Style::default().bg(Color::DarkGray))
        .wrap(Wrap { trim: false });
    f.render_widget(value_paragraph, value_inner);
}

/// Render the delete confirmation dialog.
pub fn render_delete_dialog(f: &mut Frame, app: &App) {
    // Create a smaller centered popup layout (50% of screen)
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(f.size());

    let vertical = popup_layout[1];
    let area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(vertical)[1];

    // Clear the background to ensure the popup appears on top
    f.render_widget(Clear, area);

    // Create the main container with background
    let block = Block::default()
        .title(" Delete Entry ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Red));
    f.render_widget(block.clone(), area);

    let inner = block.inner(area);

    // Get the current entry key for confirmation
    let key = app.current_key().unwrap_or_else(|| "Unknown".to_string());

    let content = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "Are you sure you want to delete:",
            Style::default().bg(Color::Red).fg(Color::White),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            &key,
            Style::default().bg(Color::Red).fg(Color::Yellow),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Y/Enter: Yes    N/Esc: No",
            Style::default().bg(Color::Red).fg(Color::White),
        )]),
    ];

    let paragraph = Paragraph::new(content)
        .style(Style::default().bg(Color::Red))
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner);
}
