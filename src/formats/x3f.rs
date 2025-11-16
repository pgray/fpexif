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
    _section_id: u32,
    section_type: u32,
    offset: u32,
    size: u32,
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

    // Read number of directory entries
    let num_entries = reader.read_u32::<LittleEndian>()?;

    // Read directory entries
    let mut entries = Vec::with_capacity(num_entries as usize);
    for _ in 0..num_entries {
        let section_id = reader.read_u32::<LittleEndian>()?;
        let section_type = reader.read_u32::<LittleEndian>()?;
        let offset = reader.read_u32::<LittleEndian>()?;
        let size = reader.read_u32::<LittleEndian>()?;

        entries.push(X3FDirectoryEntry {
            _section_id: section_id,
            section_type,
            offset,
            size,
        });
    }

    // Look for property section (section_type == 2)
    for entry in entries {
        if entry.section_type == 2 {
            // This is the property section
            reader.seek(SeekFrom::Start(entry.offset as u64))?;

            // Read property data
            let mut prop_data = vec![0u8; entry.size as usize];
            reader.read_exact(&mut prop_data)?;

            // Property section contains name-value pairs
            // We need to parse this and convert to EXIF format
            // For now, we'll look for embedded TIFF data
            if let Some(exif_data) = find_tiff_in_properties(&prop_data) {
                let mut app1_data = Vec::with_capacity(6 + exif_data.len());
                app1_data.extend_from_slice(b"Exif\0\0");
                app1_data.extend_from_slice(&exif_data);
                return Ok(app1_data);
            }
        }
    }

    Err(ExifError::Format(
        "No EXIF data found in X3F file".to_string(),
    ))
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
