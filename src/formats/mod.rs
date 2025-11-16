// formats/mod.rs - Image format handlers
pub mod cr3;
pub mod jpeg;
pub mod png;
pub mod raf;
pub mod tiff;

use crate::errors::ExifResult;
use std::io::{Read, Seek};

/// Extract EXIF APP1 segment data from any supported image format
/// Returns the raw APP1 data starting with "Exif\0\0" marker
///
/// Supported formats:
/// - JPEG (.jpg, .jpeg)
/// - PNG (.png) - Portable Network Graphics (with eXIf chunk)
/// - TIFF (.tif, .tiff) - Tagged Image File Format
/// - RAF (.raf) - Fujifilm RAW
/// - CR2 (.cr2) - Canon RAW 2
/// - CR3 (.cr3) - Canon RAW 3
/// - NEF (.nef) - Nikon Electronic Format (DSLR)
/// - NRW (.nrw) - Nikon RAW (Coolpix)
/// - DNG (.dng) - Adobe Digital Negative
/// - ARW (.arw) - Sony Alpha RAW
/// - PEF (.pef) - Pentax Electronic File
/// - RWL (.rwl) - Leica RAW
/// - ORF (.orf) - Olympus RAW Format
/// - SRW (.srw) - Samsung RAW Format
/// - RW2 (.rw2) - Panasonic RAW Format
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

    // Check for PNG signature (0x89 0x50 0x4E 0x47 0x0D 0x0A 0x1A 0x0A)
    if signature[0] == 0x89
        && signature[1] == 0x50
        && signature[2] == 0x4E
        && signature[3] == 0x47
        && signature[4] == 0x0D
        && signature[5] == 0x0A
        && signature[6] == 0x1A
        && signature[7] == 0x0A
    {
        return png::extract_exif_segment(reader);
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
