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

    // Test that the help content is rendered correctly
    // Note: Background color changed from transparent to gray for better visibility
    let buffer = terminal.backend().buffer();

    // Check the content text is correct
    assert_eq!(buffer.get(1, 0).symbol(), "H");
    assert_eq!(buffer.get(2, 0).symbol(), "e");
    assert_eq!(buffer.get(3, 0).symbol(), "l");
    assert_eq!(buffer.get(4, 0).symbol(), "p");

    // Check that the search text is displayed
    assert_eq!(buffer.get(1, 1).symbol(), "S");
    assert_eq!(buffer.get(2, 1).symbol(), "e");
    assert_eq!(buffer.get(3, 1).symbol(), "a");
    assert_eq!(buffer.get(4, 1).symbol(), "r");

    // Check that the help entry is displayed
    assert_eq!(buffer.get(1, 2).symbol(), "o");
    assert_eq!(buffer.get(3, 2).symbol(), "–");

    Ok(())
}
