use thiserror::Error;

/// Errors that can occur during ADIF parsing
#[derive(Error, Debug)]
pub enum AdifError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid data specifier at position {position}: {message}")]
    InvalidDataSpecifier { position: usize, message: String },

    #[error("Invalid field length at position {position}: expected {expected}, found {found}")]
    InvalidFieldLength {
        position: usize,
        expected: usize,
        found: usize,
    },

    #[error("Unexpected end of file at position {0}")]
    UnexpectedEof(usize),

    #[error("Invalid data type indicator '{0}'")]
    InvalidDataType(char),

    #[error("Parse error at position {position}: {message}")]
    ParseError { position: usize, message: String },
}

pub type Result<T> = std::result::Result<T, AdifError>;
