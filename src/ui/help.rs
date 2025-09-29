use ratatui::{
    prelude::{Constraint, Direction, Frame, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

/// Entry in the help command list.
#[derive(Clone, Debug)]
pub struct HelpEntry {
    pub key: &'static str,
    pub action: &'static str,
}

/// Default help entries derived from the specification.
pub const DEFAULT_ENTRIES: &[HelpEntry] = &[
    HelpEntry {
        key: "o",
        action: "Open env path",
    },
    HelpEntry {
        key: "c",
        action: "Create key/value",
    },
    HelpEntry {
        key: "e",
        action: "Edit selected value",
    },
    HelpEntry {
        key: "d",
        action: "Delete (prompts)",
    },
    HelpEntry {
        key: "/",
        action: "Search in focused pane",
    },
    HelpEntry {
        key: "Ctrl+s",
        action: "Commit transaction",
    },
    HelpEntry {
        key: "Ctrl+z",
        action: "Abort transaction",
    },
    HelpEntry {
        key: "g/G",
        action: "Jump top/bottom",
    },
    HelpEntry {
        key: "?",
        action: "Toggle help",
    },
    HelpEntry {
        key: "F5",
        action: "Refresh database view",
    },
    HelpEntry {
        key: "F6",
        action: "Cycle theme (Dark/Light/High Contrast)",
    },
];

/// Filter help entries by a query string (case-insensitive).
pub fn filter_entries<'a>(entries: &'a [HelpEntry], query: &str) -> Vec<&'a HelpEntry> {
    if query.is_empty() {
        return entries.iter().collect();
    }
    let q = query.to_lowercase();
    entries
        .iter()
        .filter(|e| e.key.to_lowercase().contains(&q) || e.action.to_lowercase().contains(&q))
        .collect()
}

/// Render the help popup widget.
pub fn render(f: &mut Frame, area: Rect, query: &str, entries: &[HelpEntry]) {
    // Clear the background to ensure the popup appears on top
    f.render_widget(Clear, area);
    
    let block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White).bg(Color::Black));
    f.render_widget(block.clone(), area);
    
    let inner = block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);
    
    let query_p = Paragraph::new(format!("Search: {query}"))
        .style(Style::default().fg(Color::White).bg(Color::Black));
    f.render_widget(query_p, chunks[0]);
    
    let filtered = filter_entries(entries, query);
    let items: Vec<ListItem> = filtered
        .into_iter()
        .map(|e| ListItem::new(format!("{} \u{2013} {}", e.key, e.action)))
        .collect();
    let list = List::new(items)
        .style(Style::default().fg(Color::White).bg(Color::Black));
    f.render_widget(list, chunks[1]);
}
