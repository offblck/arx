use crate::commands::CategoryParseError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Bookmark with ID {0} not found")]
    IDNotFound(usize),

    #[error("Bookmark with ID {0} has no URL")]
    NoUrl(usize),

    #[error("Project directories not found")]
    NoProjectDirs,

    #[error("Invalid category: {0}")]
    CategoryParseError(#[from] CategoryParseError),

    #[error("Parse error: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
pub type Result<T> = core::result::Result<T, Error>;
