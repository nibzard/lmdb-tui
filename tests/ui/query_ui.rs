use lmdb_tui::ui::query;
use ratatui::{backend::TestBackend, Terminal};

#[test]
fn query_view_snapshot() -> anyhow::Result<()> {
    let backend = TestBackend::new(20, 4);
    let mut terminal = Terminal::new(backend)?;
    let entries = vec![("foo".to_string(), b"bar".to_vec())];
    terminal.draw(|f| {
        let size = f.size();
        query::render(f, size, "prefix f", &entries);
    })?;
    terminal.backend().assert_buffer_lines([
        "┌Query─────────────┐",
        "│Query: prefix f   │",
        "│foo: bar          │",
        "└──────────────────┘",
    ]);
    Ok(())
}
