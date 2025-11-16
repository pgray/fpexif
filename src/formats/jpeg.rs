// formats/jpeg.rs - JPEG format EXIF extraction
use crate::errors::{ExifError, ExifResult};
use std::io::{Read, Seek, SeekFrom};

/// Extract EXIF APP1 segment from a JPEG file
/// Returns the raw APP1 data starting with "Exif\0\0" marker
pub fn extract_exif_segment<R: Read + Seek>(mut reader: R) -> ExifResult<Vec<u8>> {
    // Check for JPEG markers
    let mut marker_buffer = [0u8; 2];
    reader.read_exact(&mut marker_buffer)?;

    // JPEG marker should start with 0xFF 0xD8 (SOI - Start of Image)
    if marker_buffer != [0xFF, 0xD8] {
        return Err(ExifError::Format("Not a valid JPEG file".to_string()));
    }

    // Find the APP1 marker (0xFF 0xE1) which contains EXIF data
    loop {
        reader.read_exact(&mut marker_buffer)?;

        // Check for end of image (0xFF 0xD9)
        if marker_buffer == [0xFF, 0xD9] {
            return Err(ExifError::Format("No EXIF data found".to_string()));
        }

        // Check if we found the APP1 marker
        if marker_buffer[0] == 0xFF && marker_buffer[1] == 0xE1 {
            break;
        }

        // Not the APP1 marker - skip this segment
        // First, read the length (2 bytes)
        let mut length_buffer = [0u8; 2];
        reader.read_exact(&mut length_buffer)?;

        // Calculate the length (big-endian, includes the length bytes themselves)
        let length = u16::from_be_bytes(length_buffer) as u64 - 2;

        // Skip to the next marker
        reader.seek(SeekFrom::Current(length as i64))?;
    }

    // Read APP1 length (2 bytes)
    let mut length_buffer = [0u8; 2];
    reader.read_exact(&mut length_buffer)?;
    let app1_length = u16::from_be_bytes(length_buffer) as usize - 2;

    // Read APP1 data
    let mut app1_data = vec![0u8; app1_length];
    reader.read_exact(&mut app1_data)?;

    // Check for "Exif\0\0" marker
    if app1_data.len() < 6 || &app1_data[0..6] != b"Exif\0\0" {
        return Err(ExifError::Format(
            "Not a valid EXIF APP1 segment".to_string(),
        ));
    }

    Ok(app1_data)
}
