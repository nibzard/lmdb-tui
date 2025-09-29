use ratatui::{
    prelude::{Constraint, Direction, Frame, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::util::format_bytes;

/// Render the key-value preview popup.
pub fn render(f: &mut Frame, key: &str, value: &[u8]) {
    // Create a centered popup layout (80% of screen)
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(f.size());

    let vertical = popup_layout[1];
    let area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(80),
            Constraint::Percentage(10),
        ])
        .split(vertical)[1];

    // Clear the background to ensure the popup appears on top
    f.render_widget(Clear, area);

    // Create the main container with background
    let block = Block::default()
        .title(format!(
            " Key-Value Preview ({})",
            format_bytes(value.len())
        ))
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));
    f.render_widget(block.clone(), area);

    let inner = block.inner(area);

    // Split into key and value sections
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Key section
            Constraint::Min(0),    // Value section
        ])
        .split(inner);

    // Render key section
    let key_block = Block::default()
        .title(" Key ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));
    f.render_widget(key_block.clone(), sections[0]);

    let key_inner = key_block.inner(sections[0]);
    let key_paragraph = Paragraph::new(key)
        .style(Style::default().bg(Color::DarkGray).fg(Color::Yellow))
        .wrap(Wrap { trim: false });
    f.render_widget(key_paragraph, key_inner);

    // Render value section
    let value_block = Block::default()
        .title(" Value ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));
    f.render_widget(value_block.clone(), sections[1]);

    let value_inner = value_block.inner(sections[1]);

    // Format the value content
    let value_content = format_value_content(value);
    let value_paragraph = Paragraph::new(value_content)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .wrap(Wrap { trim: false });
    f.render_widget(value_paragraph, value_inner);
}

/// Format the value content for display, handling both text and binary data.
fn format_value_content(value: &[u8]) -> Vec<Line> {
    let mut lines = Vec::new();

    // Try to interpret as UTF-8 text first
    if let Ok(text) = std::str::from_utf8(value) {
        // Check if it looks like JSON
        if text.trim_start().starts_with('{') || text.trim_start().starts_with('[') {
            lines.push(Line::from(vec![
                Span::styled("Format: ", Style::default().fg(Color::Green)),
                Span::styled("JSON", Style::default().fg(Color::Cyan)),
            ]));
            lines.push(Line::from(""));

            // Pretty-print JSON if possible
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text) {
                if let Ok(pretty) = serde_json::to_string_pretty(&parsed) {
                    for line in pretty.lines() {
                        lines.push(Line::from(line.to_string()));
                    }
                } else {
                    for line in text.lines() {
                        lines.push(Line::from(line.to_string()));
                    }
                }
            } else {
                for line in text.lines() {
                    lines.push(Line::from(line.to_string()));
                }
            }
        } else {
            // Regular text
            lines.push(Line::from(vec![
                Span::styled("Format: ", Style::default().fg(Color::Green)),
                Span::styled("Text", Style::default().fg(Color::Cyan)),
            ]));
            lines.push(Line::from(""));

            for line in text.lines() {
                lines.push(Line::from(line.to_string()));
            }
        }
    } else {
        // Binary data - show hex dump
        lines.push(Line::from(vec![
            Span::styled("Format: ", Style::default().fg(Color::Green)),
            Span::styled("Binary (hex dump)", Style::default().fg(Color::Cyan)),
        ]));
        lines.push(Line::from(""));

        // Create hex dump with 16 bytes per line
        for (i, chunk) in value.chunks(16).enumerate() {
            let offset = format!("{:08x}: ", i * 16);
            let hex_part: String = chunk
                .iter()
                .map(|b| format!("{:02x}", b))
                .collect::<Vec<_>>()
                .join(" ");

            let ascii_part: String = chunk
                .iter()
                .map(|&b| {
                    if b.is_ascii_graphic() || b == b' ' {
                        b as char
                    } else {
                        '.'
                    }
                })
                .collect();

            let line_content = format!("{}{:<48} |{}|", offset, hex_part, ascii_part);
            lines.push(Line::from(vec![Span::styled(
                line_content,
                Style::default().fg(Color::Gray),
            )]));
        }
    }

    lines
}
