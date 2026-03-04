use thiserror::Error;

#[derive(Error, Debug)]
pub enum I18nError {
    #[error("Translation error: {0}")]
    TranslationError(String),

    #[error("Locale not found: {0}")]
    LocaleNotFound(String),

    #[error("Invalid locale: {0}")]
    InvalidLocale(String),

    #[error("Resource not found: {0}")]
    ResourceNotFound(String),

    #[error("Format error: {0}")]
    FormatError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, I18nError>;
