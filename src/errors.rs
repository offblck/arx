pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Bookmark with ID {0} not found")]
    IDNotFound(usize),

    #[error("Bookmark with ID {0} has no URL")]
    NoUrl(usize),

    #[error("Project directories not found")]
    NoProjectDirs,

    #[error("Page {0} not found")]
    PageNotFound(usize),

    #[error("Edit command requires at least one argument")]
    NoEditSpecified,

    #[error("Clipboard not found: {0}")]
    ClipboardNotFound(String),

    #[error("Could not copy to clipboard: {0}")]
    ClipboardCopyError(String),

    #[error("Invalid category: {0}")]
    CategoryParseError(String),

    #[error("JSON parse error: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Toml parse error: {0}")]
    TomlParseError(#[from] toml::de::Error),

    #[error("Internal error when serializing metadata: {0}")]
    TomlSerializeError(#[from] toml::ser::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("No config args provided")]
    NoConfigArgs,

    #[error("Can't paginate by 0")]
    ZeroPagination,
}
