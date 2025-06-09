use lmdb_tui::config::Config;
use lmdb_tui::ui::query::{self, QueryViewParams};
use ratatui::{backend::TestBackend, Terminal};

#[test]
fn query_view_snapshot() -> anyhow::Result<()> {
    let backend = TestBackend::new(20, 4);
    let mut terminal = Terminal::new(backend)?;
    let entries = vec![("foo".to_string(), b"bar".to_vec())];
    let cfg = Config::default();
    terminal.draw(|f| {
        let size = f.size();
        query::render(f, size, QueryViewParams {
            query: "prefix f",
            entries: &entries,
            selected: 0,
            total_entries: 1,
            page_offset: 0,
            loading: false,
            config: &cfg,
        });
    })?;
    let buffer = terminal.backend().buffer();
    assert_eq!(buffer.get(0, 0).symbol(), "┌");
    assert_eq!(buffer.get(0, 1).symbol(), "│");
    assert_eq!(buffer.get(0, 2).symbol(), "│");
    assert_eq!(buffer.get(0, 3).symbol(), "└");
    
    // Check that the selected item has the arrow indicator and highlighting
    assert_eq!(buffer.get(1, 2).symbol(), "▶"); // Arrow indicator
    assert_eq!(buffer.get(3, 2).symbol(), "f"); // First character of "foo"
    assert_eq!(buffer.get(3, 2).fg, ratatui::style::Color::Black);
    assert_eq!(buffer.get(3, 2).bg, ratatui::style::Color::Yellow);
    Ok(())
}
