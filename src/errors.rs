// errors.rs - Error handling for the EXIF library
use thiserror::Error;

/// Represents all possible errors when parsing EXIF data
#[derive(Debug, Error)]
pub enum ExifError {
    /// IO errors when reading/writing files
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid format of the EXIF data
    #[error("Format error: {0}")]
    Format(String),

    /// Unsupported feature or tag
    #[error("Unsupported feature: {0}")]
    Unsupported(String),

    /// Invalid value for a tag
    #[error("Invalid value: {0}")]
    InvalidValue(String),

    /// Missing required data
    #[error("Missing data: {0}")]
    MissingData(String),

    /// Tag not found
    #[error("Tag not found: {0}")]
    TagNotFound(String),
}

// A specialized result type for EXIF operations
pub type ExifResult<T> = Result<T, ExifError>;
