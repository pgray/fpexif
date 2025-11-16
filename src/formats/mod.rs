// formats/mod.rs - Image format handlers
pub mod cr3;
pub mod jpeg;
pub mod raf;
pub mod tiff;

use crate::errors::ExifResult;
use std::io::{Read, Seek};

/// Extract EXIF APP1 segment data from any supported image format
/// Returns the raw APP1 data starting with "Exif\0\0" marker
///
/// Supported formats:
/// - JPEG (.jpg, .jpeg)
/// - RAF (.raf) - Fujifilm RAW
/// - CR2 (.cr2) - Canon RAW 2
/// - CR3 (.cr3) - Canon RAW 3
/// - NEF (.nef) - Nikon Electronic Format
/// - DNG (.dng) - Adobe Digital Negative
/// - TIFF (.tif, .tiff) - Tagged Image File Format
/// - And other TIFF-based RAW formats (ORF, SRW, RW2, etc.)
pub fn extract_exif_segment<R: Read + Seek>(mut reader: R) -> ExifResult<Vec<u8>> {
    // Read first few bytes to detect format
    let mut signature = [0u8; 16];
    reader.read_exact(&mut signature)?;

    // Reset to beginning
    reader.seek(std::io::SeekFrom::Start(0))?;

    // Check for RAF signature (Fujifilm RAW)
    if signature.starts_with(b"FUJIFILMCCD-RAW") {
        return raf::extract_exif_segment(reader);
    }

    // Check for JPEG signature (FF D8 FF)
    if signature[0] == 0xFF && signature[1] == 0xD8 && signature[2] == 0xFF {
        return jpeg::extract_exif_segment(reader);
    }

    // Check for TIFF signature (II or MM) - covers CR2, NEF, DNG, and other TIFF-based RAW formats
    if (signature[0] == b'I' && signature[1] == b'I')
        || (signature[0] == b'M' && signature[1] == b'M')
    {
        return tiff::extract_exif_segment(reader);
    }

    // Check for CR3 signature (Canon RAW 3) - ISO Base Media File Format
    // CR3 files start with: [size:4 bytes][ftyp][crx ]
    if signature[4..8] == *b"ftyp" && signature.len() >= 12 {
        // Check for CR3 brand
        if &signature[8..12] == b"crx " {
            return cr3::extract_exif_segment(reader);
        }
    }

    // Unknown format
    Err(crate::errors::ExifError::Format(
        "Unsupported image format".to_string(),
    ))
}
