// errors.rs - Error handling for the EXIF library
use std::fmt;
use std::io;

/// Represents all possible errors when parsing EXIF data
#[derive(Debug)]
pub enum ExifError {
    /// IO errors when reading/writing files
    Io(io::Error),

    /// Invalid format of the EXIF data
    Format(String),

    /// Unsupported feature or tag
    Unsupported(String),

    /// Invalid value for a tag
    InvalidValue(String),

    /// Missing required data
    MissingData(String),

    /// Tag not found
    TagNotFound(String),
}

impl fmt::Display for ExifError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExifError::Io(err) => write!(f, "I/O error: {}", err),
            ExifError::Format(msg) => write!(f, "Format error: {}", msg),
            ExifError::Unsupported(msg) => write!(f, "Unsupported feature: {}", msg),
            ExifError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            ExifError::MissingData(msg) => write!(f, "Missing data: {}", msg),
            ExifError::TagNotFound(msg) => write!(f, "Tag not found: {}", msg),
        }
    }
}

impl std::error::Error for ExifError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ExifError::Io(err) => Some(err),
            _ => None,
        }
    }
}

// Implement conversions from IO errors
impl From<io::Error> for ExifError {
    fn from(err: io::Error) -> Self {
        ExifError::Io(err)
    }
}

// A specialized result type for EXIF operations
pub type ExifResult<T> = Result<T, ExifError>;
