use ratatui::{
    prelude::{Frame, Rect},
    text::Line,
    widgets::{Block, Borders, Paragraph},
};

use crate::db::stats::{DbStats, EnvStats};

pub fn render_env(f: &mut Frame, area: Rect, stats: &EnvStats) {
    let block = Block::default().borders(Borders::ALL).title("Env Stats");
    let text = vec![
        Line::raw(format!("map size: {}", stats.map_size)),
        Line::raw(format!(
            "readers: {}/{}",
            stats.num_readers, stats.max_readers
        )),
    ];
    let p = Paragraph::new(text).block(block);
    f.render_widget(p, area);
}

pub fn render_db(f: &mut Frame, area: Rect, stats: &DbStats) {
    let block = Block::default().borders(Borders::ALL).title("DB Stats");
    let text = vec![
        Line::raw(format!("entries: {}", stats.entries)),
        Line::raw(format!("depth: {}", stats.depth)),
        Line::raw(format!(
            "pages: b{} l{} o{}",
            stats.branch_pages, stats.leaf_pages, stats.overflow_pages
        )),
    ];
    let p = Paragraph::new(text).block(block);
    f.render_widget(p, area);
}
