use lmdb_tui::db::query::{parse_query, Mode};

#[test]
fn parse_prefix_mode() -> anyhow::Result<()> {
    let mode = parse_query("prefix foo")?;
    if let Mode::Prefix(p) = mode {
        assert_eq!(p, "foo");
    } else {
        panic!("expected prefix");
    }
    Ok(())
}

#[test]
fn parse_range_mode() -> anyhow::Result<()> {
    let mode = parse_query("range a..b")?;
    if let Mode::Range(s, e) = mode {
        assert_eq!((s, e), ("a", "b"));
    } else {
        panic!("expected range");
    }
    Ok(())
}

#[test]
fn parse_regex_mode() -> anyhow::Result<()> {
    let mode = parse_query("regex ^foo$")?;
    match mode {
        Mode::Regex(re) => assert!(re.is_match("foo")),
        _ => panic!("expected regex"),
    }
    Ok(())
}

#[test]
fn parse_default_prefix() -> anyhow::Result<()> {
    let mode = parse_query("bar")?;
    if let Mode::Prefix(p) = mode {
        assert_eq!(p, "bar");
    } else {
        panic!("expected prefix");
    }
    Ok(())
}
