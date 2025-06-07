use lmdb_tui::ui::help;
use ratatui::{backend::TestBackend, Terminal};

#[test]
fn help_view_snapshot() -> anyhow::Result<()> {
    let backend = TestBackend::new(30, 5);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|f| {
        let size = f.size();
        help::render(f, size, "open", help::DEFAULT_ENTRIES);
    })?;
    terminal.backend().assert_buffer_lines([
        "┌Help────────────────────────┐",
        "│Search: open                │",
        "│o – Open env path           │",
        "│                            │",
        "└────────────────────────────┘",
    ]);
    Ok(())
}
