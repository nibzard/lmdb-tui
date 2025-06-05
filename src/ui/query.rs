use ratatui::{
    prelude::{Constraint, Direction, Frame, Layout, Rect},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};
pub fn render(f: &mut Frame, area: Rect, query: &str, entries: &[(String, Vec<u8>)]) {
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
        .map(|(k, v)| ListItem::new(format!("{}: {}", k, String::from_utf8_lossy(v))))
        .collect();
    let list = List::new(items);
    f.render_widget(list, chunks[1]);
}
