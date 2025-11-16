// formats/png.rs - PNG format EXIF extraction
use crate::errors::{ExifError, ExifResult};
use std::io::{Read, Seek, SeekFrom};

// PNG signature: 0x89 0x50 0x4E 0x47 0x0D 0x0A 0x1A 0x0A
const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

/// Extract EXIF data from PNG file
/// PNG files can contain EXIF data in the eXIf chunk (PNG 1.5+)
pub fn extract_exif_segment<R: Read + Seek>(mut reader: R) -> ExifResult<Vec<u8>> {
    // Verify PNG signature
    let mut signature = [0u8; 8];
    reader.read_exact(&mut signature)?;

    if signature != PNG_SIGNATURE {
        return Err(ExifError::Format("Not a valid PNG file".to_string()));
    }

    // Parse chunks to find eXIf chunk
    loop {
        // Read chunk length (4 bytes, big-endian)
        let mut length_bytes = [0u8; 4];
        if reader.read_exact(&mut length_bytes).is_err() {
            // End of file reached without finding eXIf chunk
            return Err(ExifError::Format(
                "No EXIF data found in PNG file".to_string(),
            ));
        }
        let chunk_length = u32::from_be_bytes(length_bytes) as usize;

        // Read chunk type (4 bytes)
        let mut chunk_type = [0u8; 4];
        reader.read_exact(&mut chunk_type)?;

        // Check if this is the eXIf chunk
        if &chunk_type == b"eXIf" {
            // Read the EXIF data
            let mut exif_data = vec![0u8; chunk_length];
            reader.read_exact(&mut exif_data)?;

            // PNG eXIf chunk contains raw TIFF data
            // We need to wrap it in APP1 format: "Exif\0\0" + TIFF data
            let mut app1_data = Vec::with_capacity(6 + exif_data.len());
            app1_data.extend_from_slice(b"Exif\0\0");
            app1_data.extend_from_slice(&exif_data);

            return Ok(app1_data);
        }

        // Skip chunk data + CRC (4 bytes)
        let skip_bytes = chunk_length + 4;
        reader.seek(SeekFrom::Current(skip_bytes as i64))?;

        // Check for critical chunks that come before eXIf
        // IEND is the last chunk, so we can stop if we reach it
        if &chunk_type == b"IEND" {
            return Err(ExifError::Format(
                "No EXIF data found in PNG file".to_string(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_png_signature() {
        // Create a minimal PNG with just signature
        let mut png_data = Vec::new();
        png_data.extend_from_slice(&PNG_SIGNATURE);

        let cursor = Cursor::new(png_data);
        let result = extract_exif_segment(cursor);

        // Should fail because there's no eXIf chunk, but not because of invalid signature
        assert!(matches!(result, Err(ExifError::Format(_))));
    }

    #[test]
    fn test_invalid_signature() {
        let invalid_data = vec![0x00, 0x00, 0x00, 0x00];
        let cursor = Cursor::new(invalid_data);
        let result = extract_exif_segment(cursor);

        assert!(matches!(result, Err(ExifError::Format(_))));
    }
}
