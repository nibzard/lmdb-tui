use ratatui::{
    prelude::{Constraint, Direction, Frame, Layout, Rect, Style},
    style::{Color, Modifier},
    text::{Span, Line},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::config::Config;
use crate::util::{format_kv_entry, format_pagination, truncate_with_ellipsis};

/// Detect query type and return appropriate styling
fn detect_query_type(query: &str) -> (&'static str, Style) {
    let trimmed = query.trim();
    if trimmed.starts_with("prefix ") {
        ("PREFIX", Style::default().fg(Color::Green))
    } else if trimmed.starts_with("range ") {
        ("RANGE", Style::default().fg(Color::Blue))
    } else if trimmed.starts_with("regex ") {
        ("REGEX", Style::default().fg(Color::Red))
    } else if trimmed.starts_with("jsonpath ") {
        ("JSON", Style::default().fg(Color::Yellow))
    } else if trimmed.contains("..") {
        ("RANGE", Style::default().fg(Color::Blue))
    } else {
        ("PREFIX", Style::default().fg(Color::Cyan))
    }
}

/// Query view render parameters.
pub struct QueryViewParams<'a> {
    pub query: &'a str,
    pub entries: &'a [(String, Vec<u8>)],
    pub selected: usize,
    pub total_entries: usize,
    pub page_offset: usize,
    pub loading: bool,
    pub config: &'a Config,
    pub spinner_char: &'a str,
}

/// Render the query view with input and result list.
pub fn render(f: &mut Frame, area: Rect, params: QueryViewParams) {
    // Create title with pagination info and loading indicator
    let pagination_info = if params.loading {
        format!(" - {} Loading...", params.spinner_char)
    } else if params.total_entries > 0 {
        let current_result = params.page_offset + params.selected;
        format!(" - {}", format_pagination(current_result, params.total_entries, params.entries.len()))
    } else {
        " - No results".to_string()
    };
    
    let title = format!("Query{}", pagination_info);
    let block = Block::default().borders(Borders::ALL).title(title);
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner);
    
    // Enhanced query input with type detection and cursor
    let query_display = if params.query.is_empty() {
        Span::styled("Enter a query...", Style::default().fg(Color::DarkGray))
    } else {
        // Detect query type and add visual indicators
        let (query_type, style) = detect_query_type(params.query);
        let query_text = params.query.to_string();
        let query_width = chunks[0].width.saturating_sub(12); // Account for prefix and type indicator
        let truncated_query = truncate_with_ellipsis(&query_text, query_width as usize);
        
        Span::styled(
            format!("[{}] {}", query_type, truncated_query),
            style
        )
    };
    
    let query_line = Line::from(vec![
        Span::raw("Query: "),
        query_display,
        Span::styled("█", Style::default().fg(Color::White).add_modifier(Modifier::SLOW_BLINK)) // Cursor
    ]);
    
    let p = Paragraph::new(query_line);
    f.render_widget(p, chunks[0]);
    
    // Enhanced results list with improved formatting and indicators
    if params.entries.is_empty() && !params.loading {
        // Show helpful message when no results
        let no_results_msg = if params.query.is_empty() {
            "Start typing to search..."
        } else {
            "No matches found. Try a different query."
        };
        let p = Paragraph::new(no_results_msg)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::NONE));
        f.render_widget(p, chunks[1]);
    } else {
        let entry_width = chunks[1].width.saturating_sub(4); // Account for borders and indicators
        let items: Vec<ListItem> = params.entries
            .iter()
            .enumerate()
            .map(|(i, (k, v))| {
                let formatted = format_kv_entry(k, v, entry_width as usize);
                let line = if i == params.selected {
                    // Selected item with highlight and arrow indicator
                    Line::from(vec![
                        Span::styled("▶ ", Style::default().fg(Color::Yellow)),
                        Span::styled(formatted, params.config.theme.selected_style())
                    ])
                } else {
                    // Regular item with spacing for alignment
                    Line::from(vec![
                        Span::raw("  "),
                        Span::raw(formatted)
                    ])
                };
                ListItem::new(line)
            })
            .collect();
        
        let list = List::new(items);
        f.render_widget(list, chunks[1]);
        
        // Add progress indicator for large result sets
        if params.total_entries > params.entries.len() {
            let progress = (params.page_offset + params.selected) as f64 / params.total_entries as f64;
            let progress_area = Rect {
                x: chunks[1].right().saturating_sub(1),
                y: chunks[1].y + 1,
                width: 1,
                height: chunks[1].height.saturating_sub(2),
            };
            
            // Simple vertical progress indicator
            let progress_char = if progress < 0.2 { "▁" }
                else if progress < 0.4 { "▂" }
                else if progress < 0.6 { "▃" }
                else if progress < 0.8 { "▄" }
                else { "▅" };
            
            let progress_span = Span::styled(progress_char, Style::default().fg(Color::Blue));
            let p = Paragraph::new(progress_span);
            f.render_widget(p, progress_area);
        }
    }
}
