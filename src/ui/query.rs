use ratatui::{
    prelude::{Constraint, Direction, Frame, Layout, Rect},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::config::Config;

pub fn render(
    f: &mut Frame,
    area: Rect,
    query: &str,
    entries: &[(String, Vec<u8>)],
    selected: usize,
    cfg: &Config,
) {
    let block = Block::default().borders(Borders::ALL).title("Query");
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);
    let p = Paragraph::new(format!("Query: {query}"));
    f.render_widget(p, chunks[0]);
    let items: Vec<ListItem> = entries
        .iter()
        .enumerate()
        .map(|(i, (k, v))| {
            let text = format!("{}: {}", k, String::from_utf8_lossy(v));
            if i == selected {
                ListItem::new(Span::styled(text, cfg.theme.selected_style()))
            } else {
                ListItem::new(Span::raw(text))
            }
        })
        .collect();
    let list = List::new(items);
    f.render_widget(list, chunks[1]);
}
