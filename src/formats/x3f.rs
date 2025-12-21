// X3F (Sigma/Foveon) format support
//
// X3F is Sigma's proprietary RAW format for cameras with Foveon sensors.
// File structure:
// - File identifier: "FOVb" (4 bytes)
// - Version (4 bytes)
// - Unique identifier (16 bytes)
// - Mark pattern (4 bytes)
// - Columns (4 bytes, little-endian)
// - Rows (4 bytes, little-endian)
// - Rotation (4 bytes, little-endian)
// - Directory sections at the end of the file
// - Property section contains EXIF-like metadata

use crate::errors::{ExifError, ExifResult};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Seek, SeekFrom};

const X3F_SIGNATURE: &[u8; 4] = b"FOVb";

#[derive(Debug)]
struct X3FHeader {
    _version: [u8; 4],
    _columns: u32,
    _rows: u32,
    _rotation: u32,
}

#[derive(Debug)]
struct X3FDirectoryEntry {
    offset: u32,
    size: u32,
    tag: [u8; 4], // 4-char ASCII tag like "PROP", "IMAG", "IMA2"
}

pub fn extract_exif_segment<R: Read + Seek>(mut reader: R) -> ExifResult<Vec<u8>> {
    // Verify X3F signature
    let mut signature = [0u8; 4];
    reader.read_exact(&mut signature)?;

    if &signature != X3F_SIGNATURE {
        return Err(ExifError::Format("Invalid X3F signature".to_string()));
    }

    // Get file size and validate minimum size
    // Minimum: 4 (sig) + 4 (ver) + 16 (uid) + 4 (mark) + 12 (dims) + 4 (dir_offset) = 44 bytes
    let file_size = reader.seek(SeekFrom::End(0))?;
    if file_size < 44 {
        return Err(ExifError::Format(format!(
            "X3F file too small: {} bytes (minimum 44 bytes required)",
            file_size
        )));
    }

    // Reset to read header
    reader.seek(SeekFrom::Start(4))?; // Skip signature we already read

    // Read version
    let mut version = [0u8; 4];
    reader.read_exact(&mut version)?;

    // Skip unique identifier (16 bytes)
    reader.seek(SeekFrom::Current(16))?;

    // Skip mark pattern (4 bytes)
    reader.seek(SeekFrom::Current(4))?;

    // Read image dimensions (not critical but part of header)
    let columns = reader.read_u32::<LittleEndian>()?;
    let rows = reader.read_u32::<LittleEndian>()?;
    let rotation = reader.read_u32::<LittleEndian>()?;

    let _header = X3FHeader {
        _version: version,
        _columns: columns,
        _rows: rows,
        _rotation: rotation,
    };

    // Directory is at the end of the file
    // Seek to end - 4 to read directory offset
    reader.seek(SeekFrom::End(-4))?;
    let directory_offset = reader.read_u32::<LittleEndian>()?;

    // Validate directory offset is within file bounds
    if directory_offset as u64 >= file_size {
        return Err(ExifError::Format(format!(
            "Invalid directory offset: {} (file size: {})",
            directory_offset, file_size
        )));
    }

    // Seek to directory
    reader.seek(SeekFrom::Start(directory_offset as u64))?;

    // Read directory header "SECd" (4 bytes) + version (4 bytes) + entry count (4 bytes)
    let mut dir_header = [0u8; 4];
    reader.read_exact(&mut dir_header)?;

    if &dir_header != b"SECd" {
        return Err(ExifError::Format(format!(
            "Invalid X3F directory header: expected 'SECd', got {:?}",
            dir_header
        )));
    }

    // Skip version (4 bytes)
    let _dir_version = reader.read_u32::<LittleEndian>()?;

    // Read number of directory entries
    let num_entries = reader.read_u32::<LittleEndian>()?;

    // Sanity check: ExifTool limits to 2-20 entries
    if !(1..=100).contains(&num_entries) {
        return Err(ExifError::Format(format!(
            "Invalid X3F directory: unexpected entry count ({})",
            num_entries
        )));
    }

    // Read directory entries (12 bytes each: offset, size, 4-char tag)
    let mut entries = Vec::with_capacity(num_entries as usize);
    for _ in 0..num_entries {
        let offset = reader.read_u32::<LittleEndian>()?;
        let size = reader.read_u32::<LittleEndian>()?;
        let mut tag = [0u8; 4];
        reader.read_exact(&mut tag)?;

        entries.push(X3FDirectoryEntry { offset, size, tag });
    }

    // First try to find embedded JPEG with EXIF in IMA2 section (available in most cameras except SD9/SD10)
    for entry in &entries {
        if &entry.tag == b"IMA2" || &entry.tag == b"IMAG" {
            reader.seek(SeekFrom::Start(entry.offset as u64))?;

            // Read section header (28 bytes for image sections)
            // Format: "SECi" + version(4) + type(4) + format(4) + width(4) + height(4) + rowSize(4)
            let mut sec_header = [0u8; 28];
            if reader.read_exact(&mut sec_header).is_ok() {
                // Check if this is JPEG data (version 2.0, type 2, format 18 = 0x12)
                // SECi\0\0\x02\0\x02\0\0\0\x12\0\0\0
                if &sec_header[0..4] == b"SECi"
                    && sec_header[6] == 0x02
                    && sec_header[8] == 0x02
                    && sec_header[12] == 0x12
                {
                    let jpeg_size = entry.size.saturating_sub(28) as usize;
                    if jpeg_size > 0 {
                        let mut jpeg_data = vec![0u8; jpeg_size];
                        reader.read_exact(&mut jpeg_data)?;

                        // Check for JPEG with EXIF (FFD8 FFE1)
                        if jpeg_data.len() > 4
                            && jpeg_data[0] == 0xFF
                            && jpeg_data[1] == 0xD8
                            && jpeg_data[2] == 0xFF
                            && jpeg_data[3] == 0xE1
                        {
                            // Extract EXIF segment from JPEG
                            if let Some(exif_data) = extract_exif_from_jpeg(&jpeg_data) {
                                return Ok(exif_data);
                            }
                        }
                    }
                }
            }
        }
    }

    // Fall back to looking for TIFF data in property section
    for entry in &entries {
        if &entry.tag == b"PROP" {
            reader.seek(SeekFrom::Start(entry.offset as u64))?;

            let mut prop_data = vec![0u8; entry.size as usize];
            reader.read_exact(&mut prop_data)?;

            // Property section should start with "SECp"
            if prop_data.len() >= 4 && &prop_data[0..4] == b"SECp" {
                if let Some(exif_data) = find_tiff_in_properties(&prop_data) {
                    let mut app1_data = Vec::with_capacity(6 + exif_data.len());
                    app1_data.extend_from_slice(b"Exif\0\0");
                    app1_data.extend_from_slice(&exif_data);
                    return Ok(app1_data);
                }
            }
        }
    }

    Err(ExifError::Format(
        "No EXIF data found in X3F file".to_string(),
    ))
}

// Extract EXIF APP1 segment from JPEG data
fn extract_exif_from_jpeg(jpeg_data: &[u8]) -> Option<Vec<u8>> {
    // Skip JPEG SOI marker (FFD8)
    if jpeg_data.len() < 4 {
        return None;
    }

    let mut pos = 2;
    while pos + 4 < jpeg_data.len() {
        if jpeg_data[pos] != 0xFF {
            return None;
        }

        let marker = jpeg_data[pos + 1];
        let segment_len = u16::from_be_bytes([jpeg_data[pos + 2], jpeg_data[pos + 3]]) as usize;

        if marker == 0xE1 {
            // APP1 segment - check if it's EXIF
            if pos + 4 + segment_len <= jpeg_data.len() {
                let segment_data = &jpeg_data[pos + 4..pos + 2 + segment_len];
                if segment_data.starts_with(b"Exif\0\0") {
                    return Some(segment_data.to_vec());
                }
            }
        }

        // Move to next marker
        pos += 2 + segment_len;

        // Stop at SOS marker
        if marker == 0xDA {
            break;
        }
    }

    None
}

// Helper function to find TIFF data within property section
fn find_tiff_in_properties(data: &[u8]) -> Option<Vec<u8>> {
    // Look for TIFF header (II or MM followed by 0x002A)
    for i in 0..data.len().saturating_sub(4) {
        if (data[i] == b'I' && data[i + 1] == b'I' && data[i + 2] == 0x2A && data[i + 3] == 0x00)
            || (data[i] == b'M'
                && data[i + 1] == b'M'
                && data[i + 2] == 0x00
                && data[i + 3] == 0x2A)
        {
            // Found TIFF header, extract from here to end
            // In a real implementation, we'd determine the actual size
            // For now, take a reasonable chunk
            let remaining = &data[i..];
            if remaining.len() >= 8 {
                return Some(remaining.to_vec());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_invalid_x3f_signature() {
        let data = vec![0x00, 0x00, 0x00, 0x00]; // Invalid signature

        let cursor = Cursor::new(data);
        let result = extract_exif_segment(cursor);

        assert!(result.is_err(), "Should reject invalid X3F signature");
    }

    #[test]
    fn test_valid_x3f_signature() {
        let mut data = Vec::new();
        data.extend_from_slice(X3F_SIGNATURE); // Valid signature
        data.extend_from_slice(&[0x02, 0x00, 0x00, 0x00]); // Version
        data.extend_from_slice(&[0u8; 16]); // Unique ID
        data.extend_from_slice(&[0u8; 4]); // Mark pattern
        data.extend_from_slice(&100u32.to_le_bytes()); // Columns
        data.extend_from_slice(&100u32.to_le_bytes()); // Rows
        data.extend_from_slice(&0u32.to_le_bytes()); // Rotation

        let cursor = Cursor::new(data);
        let result = extract_exif_segment(cursor);

        // Will fail because we don't have proper directory structure
        assert!(result.is_err());
    }

    #[test]
    fn test_find_tiff_in_properties() {
        let mut data = Vec::new();
        data.extend_from_slice(b"PROP:");
        data.extend_from_slice(b"II"); // Little-endian TIFF
        data.extend_from_slice(&[0x2A, 0x00]); // TIFF magic
        data.extend_from_slice(&[0x08, 0x00, 0x00, 0x00]); // IFD offset
        data.extend_from_slice(b"more data");

        let result = find_tiff_in_properties(&data);
        assert!(result.is_some());

        let tiff_data = result.unwrap();
        assert_eq!(tiff_data[0], b'I');
        assert_eq!(tiff_data[1], b'I');
    }
}
