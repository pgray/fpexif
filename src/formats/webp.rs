// formats/webp.rs - WebP format EXIF extraction
use crate::errors::{ExifError, ExifResult};
use crate::formats::riff::RiffParser;
use std::io::{Read, Seek};

/// Extract EXIF data from WebP file
/// WebP files use RIFF container format with EXIF data in "EXIF" chunk
pub fn extract_exif_segment<R: Read + Seek>(reader: R) -> ExifResult<Vec<u8>> {
    // Parse RIFF container
    let mut parser = RiffParser::new(reader)?;

    // Verify this is a WebP file
    if parser.form_type() != b"WEBP" {
        return Err(ExifError::Format(
            "Not a valid WebP file (RIFF form type is not WEBP)".to_string(),
        ));
    }

    // Look for EXIF chunk
    // Note: WebP can have VP8, VP8L, or VP8X chunks for image data
    // EXIF data is stored in a separate "EXIF" chunk
    let exif_chunk = parser.find_chunk(b"EXIF")?;

    match exif_chunk {
        Some(chunk) => {
            // Read EXIF chunk data
            let exif_data = parser.read_chunk_data(&chunk)?;

            // WebP EXIF chunk contains raw TIFF data (starting with II or MM)
            // We need to wrap it in APP1 format: "Exif\0\0" + TIFF data
            if exif_data.len() < 8 {
                return Err(ExifError::Format("EXIF chunk too short".to_string()));
            }

            // Verify TIFF header
            if !((exif_data[0] == b'I' && exif_data[1] == b'I')
                || (exif_data[0] == b'M' && exif_data[1] == b'M'))
            {
                return Err(ExifError::Format(
                    "Invalid TIFF header in WebP EXIF chunk".to_string(),
                ));
            }

            // Create APP1 segment format
            let mut app1_data = Vec::with_capacity(6 + exif_data.len());
            app1_data.extend_from_slice(b"Exif\0\0");
            app1_data.extend_from_slice(&exif_data);

            Ok(app1_data)
        }
        None => Err(ExifError::Format(
            "No EXIF data found in WebP file".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_webp_invalid_form_type() {
        // Create a RIFF file that's not WebP
        let mut riff_data = Vec::new();
        riff_data.extend_from_slice(b"RIFF");
        riff_data.extend_from_slice(&20u32.to_le_bytes());
        riff_data.extend_from_slice(b"TEST"); // Not WEBP

        let cursor = Cursor::new(riff_data);
        let result = extract_exif_segment(cursor);

        assert!(matches!(result, Err(ExifError::Format(_))));
    }

    #[test]
    fn test_webp_no_exif_chunk() {
        // Create a minimal WebP without EXIF
        let mut webp_data = Vec::new();
        webp_data.extend_from_slice(b"RIFF");
        webp_data.extend_from_slice(&20u32.to_le_bytes());
        webp_data.extend_from_slice(b"WEBP");
        // Add a VP8 chunk (no EXIF)
        webp_data.extend_from_slice(b"VP8 ");
        webp_data.extend_from_slice(&8u32.to_le_bytes());
        webp_data.extend_from_slice(&[0u8; 8]);

        let cursor = Cursor::new(webp_data);
        let result = extract_exif_segment(cursor);

        assert!(matches!(result, Err(ExifError::Format(_))));
    }
}
