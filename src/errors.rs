use thiserror::Error;

/// Application level errors used throughout the crate.
#[derive(Debug, Error)]
pub enum AppError {
    /// Requested database does not exist in the environment.
    #[error("database not found: {0}")]
    DatabaseNotFound(String),

    /// Wrapper around IO errors.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Errors originating from the underlying LMDB library.
    #[error(transparent)]
    Lmdb(#[from] heed::Error),
}
