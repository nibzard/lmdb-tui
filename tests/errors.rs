use lmdb_tui::errors::AppError;

#[test]
fn formats_database_not_found() {
    let err = AppError::DatabaseNotFound("foo".into());
    assert_eq!(format!("{err}"), "database not found: foo");
}

#[test]
fn converts_to_anyhow_error() -> anyhow::Result<()> {
    let err = AppError::DatabaseNotFound("bar".into());
    let ah: anyhow::Error = err.into();
    assert!(ah.to_string().contains("database not found: bar"));
    Ok(())
}
