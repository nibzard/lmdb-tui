use anyhow::{anyhow, Result};
use crossterm::event::KeyCode;
use std::time::Duration;

/// Convert bytes to a lowercase hex string.
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

/// Convert a hex string back to bytes.
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return Err(anyhow!("hex string length must be even"));
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|e| anyhow!(e)))
        .collect()
}

/// Attempt to decode bytes as UTF-8. Returns None if invalid.
pub fn bytes_to_utf8(bytes: &[u8]) -> Option<String> {
    std::str::from_utf8(bytes).map(|s| s.to_string()).ok()
}

/// Format a byte size as human readable string.
pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.1} {}", size, UNITS[unit])
    }
}

/// Format bytes length as human readable string.
pub fn format_bytes(len: usize) -> String {
    format_size(len as u64)
}

/// Format a duration as `h m s` string.
pub fn format_duration(dur: Duration) -> String {
    let secs = dur.as_secs();
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{}h {}m {}s", h, m, s)
    } else if m > 0 {
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", s)
    }
}

/// Truncate text to fit within a given width, adding ellipsis if needed.
pub fn truncate_with_ellipsis(text: &str, max_width: usize) -> String {
    if text.len() <= max_width {
        text.to_string()
    } else if max_width <= 3 {
        "...".to_string()
    } else {
        let truncated = &text[..max_width - 3];
        format!("{}...", truncated)
    }
}

/// Format a key-value pair with proper truncation for display.
pub fn format_kv_entry(key: &str, value: &[u8], max_width: usize) -> String {
    let value_str = String::from_utf8_lossy(value);
    let entry = format!("{}: {}", key, value_str);
    truncate_with_ellipsis(&entry, max_width)
}

/// Format pagination info (e.g., "1/5 results").
pub fn format_pagination(current: usize, total: usize, result_count: usize) -> String {
    if total == 0 {
        "No results".to_string()
    } else if result_count == 1 {
        format!("{}/{} result", current + 1, total)
    } else {
        format!("{}/{} results", current + 1, total)
    }
}

/// Convert a [`KeyCode`] to a readable label.
pub fn key_label(code: &KeyCode) -> String {
    match code {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".into(),
        KeyCode::Tab => "Tab".into(),
        KeyCode::Backspace => "Backspace".into(),
        KeyCode::Delete => "Delete".into(),
        KeyCode::Esc => "Esc".into(),
        KeyCode::Up => "Up".into(),
        KeyCode::Down => "Down".into(),
        KeyCode::Left => "Left".into(),
        KeyCode::Right => "Right".into(),
        KeyCode::Home => "Home".into(),
        KeyCode::End => "End".into(),
        KeyCode::PageUp => "PageUp".into(),
        KeyCode::PageDown => "PageDown".into(),
        _ => format!("{:?}", code),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hex_roundtrip() {
        let data = b"hello";
        let hex = bytes_to_hex(data);
        assert_eq!(hex, "68656c6c6f");
        let back = hex_to_bytes(&hex).unwrap();
        assert_eq!(back, data);
    }

    #[test]
    fn utf8_conversion() {
        let s = "test";
        assert_eq!(bytes_to_utf8(s.as_bytes()), Some(s.to_string()));
        assert!(bytes_to_utf8(&[0xff]).is_none());
    }

    #[test]
    fn size_formatting() {
        assert_eq!(format_size(1023), "1023 B");
        assert_eq!(format_size(2048), "2.0 KB");
    }

    #[test]
    fn duration_formatting() {
        assert_eq!(format_duration(Duration::from_secs(45)), "45s");
        assert_eq!(format_duration(Duration::from_secs(125)), "2m 5s");
    }

    #[test]
    fn key_labels() {
        use crossterm::event::KeyCode;
        assert_eq!(key_label(&KeyCode::Char('q')), "q");
        assert_eq!(key_label(&KeyCode::Enter), "Enter");
    }

    #[test]
    fn text_truncation() {
        assert_eq!(truncate_with_ellipsis("hello", 10), "hello");
        assert_eq!(truncate_with_ellipsis("hello world", 8), "hello...");
        assert_eq!(truncate_with_ellipsis("hi", 2), "hi");
        assert_eq!(truncate_with_ellipsis("hello", 3), "...");
    }

    #[test]
    fn pagination_formatting() {
        assert_eq!(format_pagination(0, 5, 5), "1/5 results");
        assert_eq!(format_pagination(2, 5, 1), "3/5 result");
        assert_eq!(format_pagination(0, 0, 0), "No results");
    }
}
