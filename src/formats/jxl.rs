// formats/jxl.rs - JPEG XL format EXIF extraction
use crate::errors::{ExifError, ExifResult};
use crate::formats::isobmff::{find_exif_data, IsobmffParser};
use std::io::{Read, Seek, SeekFrom};

// JPEG XL codestream signature
const JXL_CODESTREAM_SIGNATURE: [u8; 2] = [0xFF, 0x0A];

/// Extract EXIF data from JPEG XL file
/// JPEG XL has two container types:
/// 1. Naked codestream (starts with 0xFF 0x0A)
/// 2. ISO BMFF container (starts with box structure, "JXL " brand)
pub fn extract_exif_segment<R: Read + Seek>(mut reader: R) -> ExifResult<Vec<u8>> {
    // Read first bytes to detect container type
    let mut signature = [0u8; 12];
    reader.read_exact(&mut signature)?;
    reader.seek(SeekFrom::Start(0))?;

    // Check for naked codestream
    if signature[0..2] == JXL_CODESTREAM_SIGNATURE {
        return extract_from_codestream(reader);
    }

    // Check for ISO BMFF container
    if &signature[4..8] == b"ftyp" {
        return extract_from_isobmff(reader);
    }

    Err(ExifError::Format("Not a valid JPEG XL file".to_string()))
}

/// Extract EXIF from JPEG XL codestream format
fn extract_from_codestream<R: Read + Seek>(mut reader: R) -> ExifResult<Vec<u8>> {
    // Read signature
    let mut sig = [0u8; 2];
    reader.read_exact(&mut sig)?;

    if sig != JXL_CODESTREAM_SIGNATURE {
        return Err(ExifError::Format("Invalid JXL codestream".to_string()));
    }

    // JPEG XL codestream format is complex with various boxes
    // For simplicity, we'll search for EXIF data in the file
    reader.seek(SeekFrom::Start(0))?;

    // Read up to 5MB to search for EXIF
    let mut buffer = vec![0u8; 5 * 1024 * 1024];
    let bytes_read = reader.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    // Search for "Exif\0\0" marker
    if let Some(pos) = find_subsequence(&buffer, b"Exif\0\0") {
        if pos + 6 + 8 <= buffer.len() {
            let tiff_start = pos + 6;
            if (buffer[tiff_start] == b'I' && buffer[tiff_start + 1] == b'I')
                || (buffer[tiff_start] == b'M' && buffer[tiff_start + 1] == b'M')
            {
                let max_len = std::cmp::min(512 * 1024, buffer.len() - pos);
                return Ok(buffer[pos..pos + max_len].to_vec());
            }
        }
    }

    Err(ExifError::Format(
        "No EXIF data found in JPEG XL file".to_string(),
    ))
}

/// Extract EXIF from JPEG XL ISO BMFF container
fn extract_from_isobmff<R: Read + Seek>(mut reader: R) -> ExifResult<Vec<u8>> {
    let mut parser = IsobmffParser::new(&mut reader);

    // Find ftyp box
    let ftyp_box = parser.find_box(b"ftyp")?;
    match ftyp_box {
        Some(ftyp) => {
            // Read ftyp data to check brand
            let ftyp_data = parser.read_box_data(&ftyp)?;

            if ftyp_data.len() < 4 {
                return Err(ExifError::Format("Invalid ftyp box".to_string()));
            }

            // Check for JXL brands: "jxl ", "jxll"
            let major_brand = &ftyp_data[0..4];
            let mut is_jxl = major_brand == b"jxl " || major_brand == b"jxll";

            // Check compatible brands
            if !is_jxl {
                for i in (4..ftyp_data.len()).step_by(4) {
                    if i + 4 <= ftyp_data.len() {
                        let brand = &ftyp_data[i..i + 4];
                        if brand == b"jxl " || brand == b"jxll" {
                            is_jxl = true;
                            break;
                        }
                    }
                }
            }

            if !is_jxl {
                return Err(ExifError::Format(
                    "Not a valid JPEG XL file (wrong brand)".to_string(),
                ));
            }
        }
        None => {
            return Err(ExifError::Format("No ftyp box found".to_string()));
        }
    }

    // Search for EXIF in the ISO BMFF container
    find_exif_data(reader)
}

/// Helper function to find a subsequence
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_jxl_codestream_signature() {
        // Create enough data for signature check (12 bytes minimum)
        let data = vec![
            0xFF, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let cursor = Cursor::new(data);
        let result = extract_exif_segment(cursor);

        // Should fail because there's no EXIF, but not because of invalid signature
        match result {
            Err(ExifError::Format(msg)) => {
                assert!(msg.contains("No EXIF data"));
            }
            _ => panic!("Expected format error"),
        }
    }

    #[test]
    fn test_jxl_invalid_signature() {
        // Create 12 bytes with invalid signature
        let data = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let cursor = Cursor::new(data);
        let result = extract_exif_segment(cursor);

        assert!(matches!(result, Err(ExifError::Format(_))));
    }
}
