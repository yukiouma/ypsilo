use thiserror::Error;
use std::io;

#[derive(Debug, Error)]
pub enum AlsParseError {
    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    #[error("XML error: {0}")]
    XmlError(String),

    #[error("worksheet not found: {0}")]
    WorksheetNotFound(String),

    #[error("missing required field: {0}")]
    MissingRequiredField(String),

    #[error("invalid field value: {0}")]
    InvalidFieldValue(String),
}