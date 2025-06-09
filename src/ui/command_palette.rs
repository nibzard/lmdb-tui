use ratatui::{
    prelude::{Constraint, Direction, Frame, Layout, Rect, Style},
    style::{Color, Modifier},
    text::{Span, Line},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear},
};

use crate::app::Command;
use crate::config::Config;

pub struct CommandPaletteParams<'a> {
    pub query: &'a str,
    pub commands: &'a [Command],
    pub selected: usize,
    pub config: &'a Config,
}

/// Render command palette as a centered popup
pub fn render(f: &mut Frame, params: CommandPaletteParams) {
    // Create centered popup area (similar to help popup)
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(f.size());
    
    let area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Percentage(70),
            Constraint::Percentage(15),
        ])
        .split(popup_layout[1])[1];

    // Clear background
    f.render_widget(Clear, area);
    
    // Main container
    let block = Block::default()
        .title("Command Palette (Ctrl+P)")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Black));
    f.render_widget(block.clone(), area);
    
    let inner = block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);
    
    // Input field with fuzzy search indicator
    render_input(f, chunks[0], params.query);
    
    // Command list with fuzzy matching highlights
    render_command_list(f, chunks[1], params);
}

fn render_input(f: &mut Frame, area: Rect, query: &str) {
    let input_line = Line::from(vec![
        Span::raw("> "),
        Span::styled(query, Style::default().fg(Color::White)),
        Span::styled("█", Style::default().add_modifier(Modifier::SLOW_BLINK)),
    ]);
    
    let paragraph = Paragraph::new(input_line);
    f.render_widget(paragraph, area);
}

fn render_command_list(f: &mut Frame, area: Rect, params: CommandPaletteParams) {
    if params.commands.is_empty() {
        let no_results = Paragraph::new("No matching commands")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(no_results, area);
        return;
    }
    
    let items: Vec<ListItem> = params.commands
        .iter()
        .enumerate()
        .map(|(i, cmd)| {
            let style = if i == params.selected {
                params.config.theme.selected_style()
            } else {
                Style::default()
            };
            
            let indicator = if i == params.selected { "▶ " } else { "  " };
            let keybinding = cmd.keybinding.as_ref()
                .map(|k| format!(" ({})", k))
                .unwrap_or_default();
            
            let line = Line::from(vec![
                Span::raw(indicator),
                Span::styled(&cmd.name, style),
                Span::styled(keybinding, Style::default().fg(Color::DarkGray)),
                Span::raw(" - "),
                Span::styled(&cmd.description, Style::default().fg(Color::Cyan)),
            ]);
            
            ListItem::new(line)
        })
        .collect();
    
    let list = List::new(items);
    f.render_widget(list, area);
}