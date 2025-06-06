use ratatui::{
    prelude::{Constraint, Direction, Frame, Layout, Rect},
    style::{Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem},
};

/// Render the main database view.
pub fn render(
    f: &mut Frame,
    area: Rect,
    db_names: &[String],
    selected: usize,
    entries: &[(String, Vec<u8>)],
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    let items: Vec<ListItem> = db_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let content = if i == selected {
                Span::styled(
                    name.clone(),
                    Style::default().add_modifier(Modifier::REVERSED),
                )
            } else {
                Span::raw(name.clone())
            };
            ListItem::new(content)
        })
        .collect();
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Databases"));
    f.render_widget(list, chunks[0]);

    let kv_items: Vec<ListItem> = entries
        .iter()
        .map(|(k, v)| ListItem::new(format!("{}: {}", k, String::from_utf8_lossy(v))))
        .collect();
    let kv_list =
        List::new(kv_items).block(Block::default().borders(Borders::ALL).title("Entries"));
    f.render_widget(kv_list, chunks[1]);
}
