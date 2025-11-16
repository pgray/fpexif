// formats/raf.rs - Fujifilm RAF format EXIF extraction
use crate::errors::{ExifError, ExifResult};
use std::io::{Read, Seek, SeekFrom};

const RAF_SIGNATURE: &[u8] = b"FUJIFILMCCD-RAW";

/// Extract EXIF APP1 segment from a Fujifilm RAF file
/// RAF files contain embedded JPEG data with EXIF information
pub fn extract_exif_segment<R: Read + Seek>(mut reader: R) -> ExifResult<Vec<u8>> {
    // Verify RAF signature
    let mut signature = [0u8; 15];
    reader.read_exact(&mut signature)?;

    if signature != RAF_SIGNATURE {
        return Err(ExifError::Format("Not a valid RAF file".to_string()));
    }

    // Reset to beginning to search for embedded JPEG
    reader.seek(SeekFrom::Start(0))?;

    // Read the file into memory (RAF files typically have EXIF in the first few KB)
    // We'll read up to 1MB to be safe
    let mut buffer = vec![0u8; 1024 * 1024];
    let bytes_read = reader.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    // Search for JPEG SOI marker followed by APP1 marker (FF D8 FF E1)
    let mut pos = 0;
    while pos + 4 <= buffer.len() {
        if buffer[pos] == 0xFF
            && buffer[pos + 1] == 0xD8
            && buffer[pos + 2] == 0xFF
            && buffer[pos + 3] == 0xE1
        {
            // Found JPEG with APP1 marker, now read the APP1 segment
            pos += 4; // Skip the markers

            if pos + 2 > buffer.len() {
                return Err(ExifError::Format("Truncated APP1 segment".to_string()));
            }

            // Read APP1 length (2 bytes, big-endian)
            let app1_length = u16::from_be_bytes([buffer[pos], buffer[pos + 1]]) as usize - 2;
            pos += 2;

            if pos + app1_length > buffer.len() {
                return Err(ExifError::Format(
                    "APP1 segment extends beyond buffer".to_string(),
                ));
            }

            // Extract APP1 data
            let app1_data = buffer[pos..pos + app1_length].to_vec();

            // Verify "Exif\0\0" marker
            if app1_data.len() < 6 || &app1_data[0..6] != b"Exif\0\0" {
                return Err(ExifError::Format(
                    "Not a valid EXIF APP1 segment in RAF".to_string(),
                ));
            }

            return Ok(app1_data);
        }
        pos += 1;
    }

    Err(ExifError::Format(
        "No embedded EXIF data found in RAF file".to_string(),
    ))
}
