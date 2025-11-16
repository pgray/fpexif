// formats/tiff.rs - TIFF-based RAW format EXIF extraction
// Handles CR2 (Canon), NEF (Nikon), DNG (Adobe/Ricoh), and other TIFF-based RAW formats
use crate::errors::{ExifError, ExifResult};
use std::io::{Read, Seek, SeekFrom};

/// Extract EXIF APP1 segment from a TIFF-based file
/// TIFF files already contain EXIF data in their IFD structure,
/// so we wrap it in the APP1 format expected by the parser
pub fn extract_exif_segment<R: Read + Seek>(mut reader: R) -> ExifResult<Vec<u8>> {
    // Read TIFF header (8 bytes minimum)
    let mut tiff_header = [0u8; 8];
    reader.read_exact(&mut tiff_header)?;

    // Verify TIFF signature (II for little-endian or MM for big-endian)
    if !((tiff_header[0] == b'I' && tiff_header[1] == b'I')
        || (tiff_header[0] == b'M' && tiff_header[1] == b'M'))
    {
        return Err(ExifError::Format("Not a valid TIFF file".to_string()));
    }

    // Verify TIFF magic number (0x002A for standard TIFF, 0x002B for BigTIFF, 0x4F52 for ORF, 0x5352 for SRW)
    let magic = if tiff_header[0] == b'I' {
        u16::from_le_bytes([tiff_header[2], tiff_header[3]])
    } else {
        u16::from_be_bytes([tiff_header[2], tiff_header[3]])
    };

    // Accept standard TIFF (0x2A), BigTIFF (0x2B), and various RAW format markers
    if magic != 0x002A && magic != 0x002B && magic != 0x4F52 && magic != 0x5352 && magic != 0x0055 {
        return Err(ExifError::Format(format!(
            "Unsupported TIFF magic number: 0x{:04X}",
            magic
        )));
    }

    // For standard TIFF files, read the entire file into memory
    // (RAW files can be large, but EXIF data is typically in the first few MB)
    reader.seek(SeekFrom::Start(0))?;
    let mut tiff_data = Vec::new();
    let bytes_read = reader.read_to_end(&mut tiff_data)?;

    if bytes_read < 8 {
        return Err(ExifError::Format("TIFF file too short".to_string()));
    }

    // Create APP1 segment format: "Exif\0\0" + TIFF data
    // This allows the existing parser to work with TIFF-based RAW files
    let mut app1_data = Vec::with_capacity(6 + tiff_data.len());
    app1_data.extend_from_slice(b"Exif\0\0");
    app1_data.extend_from_slice(&tiff_data);

    Ok(app1_data)
}

/// Detect specific TIFF-based RAW format from header
#[allow(dead_code)]
pub fn detect_tiff_format(header: &[u8]) -> Option<&'static str> {
    if header.len() < 12 {
        return None;
    }

    // Check TIFF signature
    if !((header[0] == b'I' && header[1] == b'I') || (header[0] == b'M' && header[1] == b'M')) {
        return None;
    }

    let is_little_endian = header[0] == b'I';

    // Check magic number at offset 2-3
    let magic = if is_little_endian {
        u16::from_le_bytes([header[2], header[3]])
    } else {
        u16::from_be_bytes([header[2], header[3]])
    };

    // Check for format-specific markers
    match magic {
        0x002A => {
            // Standard TIFF - check for specific RAW format markers
            // CR2 has "CR" at offset 8-9 in some implementations
            if header.len() >= 10 {
                // DNG version check (DNG has version at offset 8)
                let byte8 = if is_little_endian {
                    u32::from_le_bytes([header[8], header[9], header[10], header[11]])
                } else {
                    u32::from_be_bytes([header[8], header[9], header[10], header[11]])
                };

                // DNG versions are typically 1.x.x.x format
                if byte8 & 0xFF000000 == 0x01000000 || byte8 & 0x000000FF == 0x01 {
                    return Some("DNG");
                }
            }

            // Check for CR2 marker (Canon Raw 2)
            if header.len() >= 10 && header[8] == b'C' && header[9] == b'R' {
                return Some("CR2");
            }

            Some("TIFF")
        }
        0x002B => Some("BigTIFF"),
        0x4F52 => Some("ORF"), // Olympus Raw Format
        0x5352 => Some("SRW"), // Samsung Raw Format
        0x0055 => Some("RW2"), // Panasonic Raw Format
        _ => None,
    }
}
