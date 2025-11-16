// formats/cr3.rs - Canon CR3 format EXIF extraction
// CR3 uses ISO Base Media File Format (similar to MP4)
use crate::errors::{ExifError, ExifResult};
use std::io::{Read, Seek, SeekFrom};

/// Extract EXIF APP1 segment from a Canon CR3 file
/// CR3 files use ISO Base Media File Format and store EXIF in a CMT metadata track
pub fn extract_exif_segment<R: Read + Seek>(mut reader: R) -> ExifResult<Vec<u8>> {
    // Read file type box (first 12 bytes minimum)
    let mut header = [0u8; 20];
    reader.read_exact(&mut header)?;

    // Verify ftyp box
    if &header[4..8] != b"ftyp" {
        return Err(ExifError::Format("Not a valid CR3 file".to_string()));
    }

    // Verify CR3 brand (crx )
    if &header[8..12] != b"crx " {
        return Err(ExifError::Format(
            "Not a Canon CR3 file (wrong brand)".to_string(),
        ));
    }

    // Reset to beginning to search for metadata
    reader.seek(SeekFrom::Start(0))?;

    // Read up to 10MB to find EXIF data (CR3 files can be very large)
    // EXIF metadata is typically in the first few MB
    let mut buffer = vec![0u8; 10 * 1024 * 1024];
    let bytes_read = reader.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    // Search for EXIF data in uuid boxes or other metadata containers
    // CR3 format stores EXIF in various locations including:
    // - CMT1/CMT2/CMT3/CMT4 metadata tracks
    // - uuid boxes with EXIF data

    // Search for "Exif\0\0" marker which might be embedded
    if let Some(pos) = find_subsequence(&buffer, b"Exif\0\0") {
        // Found potential EXIF data
        // Try to extract a reasonable amount of data after the marker
        let exif_start = pos;

        // Look for TIFF header after "Exif\0\0"
        if pos + 6 + 8 <= buffer.len() {
            let tiff_start = pos + 6;
            if (buffer[tiff_start] == b'I' && buffer[tiff_start + 1] == b'I')
                || (buffer[tiff_start] == b'M' && buffer[tiff_start + 1] == b'M')
            {
                // Found valid TIFF header, extract a reasonable chunk
                // We'll extract up to 1MB or until end of buffer
                let max_len = std::cmp::min(1024 * 1024, buffer.len() - exif_start);
                return Ok(buffer[exif_start..exif_start + max_len].to_vec());
            }
        }
    }

    // Fallback: Search for standalone TIFF data that might contain EXIF
    // Look for TIFF header (II or MM) followed by valid magic number
    let mut pos = 0;
    while pos + 8 <= buffer.len() {
        if (buffer[pos] == b'I' && buffer[pos + 1] == b'I')
            || (buffer[pos] == b'M' && buffer[pos + 1] == b'M')
        {
            // Check magic number
            let magic = if buffer[pos] == b'I' {
                u16::from_le_bytes([buffer[pos + 2], buffer[pos + 3]])
            } else {
                u16::from_be_bytes([buffer[pos + 2], buffer[pos + 3]])
            };

            if magic == 0x002A {
                // Found potential TIFF/EXIF data
                // Wrap it in APP1 format
                let max_len = std::cmp::min(512 * 1024, buffer.len() - pos);
                let tiff_data = &buffer[pos..pos + max_len];

                let mut app1_data = Vec::with_capacity(6 + tiff_data.len());
                app1_data.extend_from_slice(b"Exif\0\0");
                app1_data.extend_from_slice(tiff_data);
                return Ok(app1_data);
            }
        }
        pos += 1;
    }

    Err(ExifError::Format(
        "No EXIF data found in CR3 file".to_string(),
    ))
}

/// Helper function to find a subsequence in a byte slice
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
