// formats/mod.rs - Image format handlers
pub mod jpeg;
pub mod raf;

use crate::errors::ExifResult;
use std::io::{Read, Seek};

/// Extract EXIF APP1 segment data from any supported image format
/// Returns the raw APP1 data starting with "Exif\0\0" marker
pub fn extract_exif_segment<R: Read + Seek>(mut reader: R) -> ExifResult<Vec<u8>> {
    // Read first few bytes to detect format
    let mut signature = [0u8; 16];
    reader.read_exact(&mut signature)?;

    // Reset to beginning
    reader.seek(std::io::SeekFrom::Start(0))?;

    // Check for RAF signature
    if signature.starts_with(b"FUJIFILMCCD-RAW") {
        return raf::extract_exif_segment(reader);
    }

    // Check for JPEG signature (FF D8 FF)
    if signature[0] == 0xFF && signature[1] == 0xD8 && signature[2] == 0xFF {
        return jpeg::extract_exif_segment(reader);
    }

    // Unknown format
    Err(crate::errors::ExifError::Format(
        "Unsupported image format".to_string(),
    ))
}
