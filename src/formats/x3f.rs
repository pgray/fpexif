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
                // First try to find embedded TIFF data
                if let Some(exif_data) = find_tiff_in_properties(&prop_data) {
                    let mut app1_data = Vec::with_capacity(6 + exif_data.len());
                    app1_data.extend_from_slice(b"Exif\0\0");
                    app1_data.extend_from_slice(&exif_data);
                    return Ok(app1_data);
                }

                // For SD9/SD10: parse X3F properties and build synthetic TIFF
                if let Some(tiff_data) = parse_x3f_properties_to_tiff(&prop_data) {
                    let mut app1_data = Vec::with_capacity(6 + tiff_data.len());
                    app1_data.extend_from_slice(b"Exif\0\0");
                    app1_data.extend_from_slice(&tiff_data);
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

/// Parse X3F property section and build synthetic TIFF structure
/// This is used for SD9/SD10 which don't have embedded JPEG with EXIF
fn parse_x3f_properties_to_tiff(data: &[u8]) -> Option<Vec<u8>> {
    // SECp header structure:
    // 0-3: "SECp"
    // 4-7: version
    // 8-11: entry count
    // 12-15: format (0 = Unicode)
    // 16-19: reserved
    // 20-23: string data length
    // 24+: entry table (8 bytes each: name_offset, value_offset as u32)
    // Then: Unicode string data (UTF-16LE, null-terminated)

    if data.len() < 24 {
        return None;
    }

    let entry_count = u32::from_le_bytes([data[8], data[9], data[10], data[11]]) as usize;
    let format = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
    let string_len = u32::from_le_bytes([data[20], data[21], data[22], data[23]]) as usize;

    // We only support Unicode format (0)
    if format != 0 {
        return None;
    }

    // Sanity checks
    if entry_count > 1000 || entry_count == 0 {
        return None;
    }

    let entry_table_start = 24;
    let entry_table_size = entry_count * 8;
    let string_data_start = entry_table_start + entry_table_size;

    if string_data_start + string_len * 2 > data.len() {
        return None;
    }

    // Parse the Unicode string data into a lookup
    let string_data = &data[string_data_start..];

    // Helper to extract null-terminated UTF-16LE string at character offset
    let extract_string = |char_offset: usize| -> Option<String> {
        let byte_offset = char_offset * 2;
        if byte_offset >= string_data.len() {
            return None;
        }

        let mut chars = Vec::new();
        let mut pos = byte_offset;
        while pos + 1 < string_data.len() {
            let code_unit = u16::from_le_bytes([string_data[pos], string_data[pos + 1]]);
            if code_unit == 0 {
                break;
            }
            chars.push(code_unit);
            pos += 2;
        }

        String::from_utf16(&chars).ok()
    };

    // Parse entries into name-value pairs
    let mut properties: Vec<(String, String)> = Vec::new();
    for i in 0..entry_count {
        let entry_offset = entry_table_start + i * 8;
        if entry_offset + 8 > data.len() {
            break;
        }

        let name_offset = u32::from_le_bytes([
            data[entry_offset],
            data[entry_offset + 1],
            data[entry_offset + 2],
            data[entry_offset + 3],
        ]) as usize;
        let value_offset = u32::from_le_bytes([
            data[entry_offset + 4],
            data[entry_offset + 5],
            data[entry_offset + 6],
            data[entry_offset + 7],
        ]) as usize;

        if let (Some(name), Some(value)) =
            (extract_string(name_offset), extract_string(value_offset))
        {
            properties.push((name, value));
        }
    }

    if properties.is_empty() {
        return None;
    }

    // Build synthetic TIFF from properties
    build_tiff_from_properties(&properties)
}

/// Build a minimal TIFF structure from X3F properties
fn build_tiff_from_properties(properties: &[(String, String)]) -> Option<Vec<u8>> {
    // Map X3F property names to EXIF tags
    let mut make: Option<&str> = None;
    let mut model: Option<&str> = None;
    let mut datetime: Option<&str> = None;

    for (name, value) in properties {
        match name.as_str() {
            "CAMMANUF" => make = Some(value),
            "CAMMODEL" => model = Some(value),
            "TIME" => datetime = Some(value),
            // TODO: Add more tags as needed (ISO, SHUTTER, APERTURE, FLENGTH)
            _ => {}
        }
    }

    // We need at least Make or Model to create meaningful TIFF
    if make.is_none() && model.is_none() {
        return None;
    }

    // Build TIFF structure
    // TIFF header: 8 bytes (II + 0x002A + IFD0 offset)
    // IFD0: 2 bytes count + entries + 4 bytes next IFD
    // String data follows

    let mut tiff = Vec::new();

    // TIFF header (little-endian)
    tiff.extend_from_slice(b"II"); // Little-endian
    tiff.extend_from_slice(&0x002Au16.to_le_bytes()); // TIFF magic
    tiff.extend_from_slice(&8u32.to_le_bytes()); // IFD0 offset

    // Count IFD0 entries
    let mut ifd0_entries: Vec<(u16, u16, u32, Vec<u8>)> = Vec::new();

    // Add Make (tag 0x010F)
    if let Some(s) = make {
        let mut bytes = s.as_bytes().to_vec();
        bytes.push(0); // null terminator
        ifd0_entries.push((0x010F, 2, bytes.len() as u32, bytes));
    }

    // Add Model (tag 0x0110)
    if let Some(s) = model {
        let mut bytes = s.as_bytes().to_vec();
        bytes.push(0);
        ifd0_entries.push((0x0110, 2, bytes.len() as u32, bytes));
    }

    // Add DateTime (tag 0x0132) - need to convert from Unix timestamp
    if let Some(s) = datetime {
        if let Ok(ts) = s.parse::<i64>() {
            // Convert Unix timestamp to EXIF format "YYYY:MM:DD HH:MM:SS"
            let dt = format_unix_timestamp(ts);
            let mut bytes = dt.as_bytes().to_vec();
            bytes.push(0);
            ifd0_entries.push((0x0132, 2, bytes.len() as u32, bytes));
        }
    }

    if ifd0_entries.is_empty() {
        return None;
    }

    // Calculate offsets
    let ifd0_offset = 8usize;
    let entry_count = ifd0_entries.len();
    let ifd0_size = 2 + entry_count * 12 + 4; // count + entries + next IFD pointer
    let mut data_offset = ifd0_offset + ifd0_size;

    // Write IFD0 entry count
    tiff.extend_from_slice(&(entry_count as u16).to_le_bytes());

    // Prepare data area
    let mut data_area = Vec::new();

    // Write IFD0 entries
    for (tag, typ, count, data) in &ifd0_entries {
        tiff.extend_from_slice(&tag.to_le_bytes()); // Tag
        tiff.extend_from_slice(&typ.to_le_bytes()); // Type (2 = ASCII)
        tiff.extend_from_slice(&count.to_le_bytes()); // Count

        if data.len() <= 4 {
            // Inline value
            let mut inline = [0u8; 4];
            inline[..data.len()].copy_from_slice(data);
            tiff.extend_from_slice(&inline);
        } else {
            // Offset to data
            tiff.extend_from_slice(&(data_offset as u32).to_le_bytes());
            data_area.extend_from_slice(data);
            // Align to word boundary
            if data.len() % 2 != 0 {
                data_area.push(0);
            }
            data_offset += data.len() + (data.len() % 2);
        }
    }

    // Next IFD offset (0 = no more IFDs)
    tiff.extend_from_slice(&0u32.to_le_bytes());

    // Append data area
    tiff.extend_from_slice(&data_area);

    Some(tiff)
}

/// Format Unix timestamp as EXIF datetime string
fn format_unix_timestamp(ts: i64) -> String {
    // Simple conversion - for a proper implementation use chrono crate
    // This is a basic approximation
    let seconds_per_day = 86400i64;
    let seconds_per_hour = 3600i64;
    let seconds_per_minute = 60i64;

    // Days since 1970-01-01
    let mut days = ts / seconds_per_day;
    let mut remaining = ts % seconds_per_day;

    if remaining < 0 {
        days -= 1;
        remaining += seconds_per_day;
    }

    let hours = remaining / seconds_per_hour;
    remaining %= seconds_per_hour;
    let minutes = remaining / seconds_per_minute;
    let seconds = remaining % seconds_per_minute;

    // Calculate year/month/day (simplified - doesn't handle all edge cases)
    let mut year = 1970i32;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    let month_days = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for &mdays in &month_days {
        if days < mdays {
            break;
        }
        days -= mdays;
        month += 1;
    }
    let day = days + 1;

    format!(
        "{:04}:{:02}:{:02} {:02}:{:02}:{:02}",
        year, month, day, hours, minutes, seconds
    )
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
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
