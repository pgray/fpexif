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

    let is_little_endian = tiff_header[0] == b'I';

    // Verify TIFF magic number (0x002A for standard TIFF, 0x002B for BigTIFF, 0x4F52 for ORF, 0x5352 for SRW)
    let magic = if is_little_endian {
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

    // Special handling for Panasonic RW2 files (magic 0x0055)
    // RW2 files have a Panasonic-specific IFD that reuses standard TIFF tag IDs with wrong types.
    // The correct EXIF data is in an embedded JPEG pointed to by tag 0x002E (JpgFromRaw).
    if magic == 0x0055 {
        if let Some(embedded_exif) = extract_rw2_embedded_exif(&tiff_data, is_little_endian) {
            return Ok(embedded_exif);
        }
        // Fall through to default behavior if we can't find embedded JPEG
    }

    // Create APP1 segment format: "Exif\0\0" + TIFF data
    // This allows the existing parser to work with TIFF-based RAW files
    let mut app1_data = Vec::with_capacity(6 + tiff_data.len());
    app1_data.extend_from_slice(b"Exif\0\0");
    app1_data.extend_from_slice(&tiff_data);

    Ok(app1_data)
}

/// Extract EXIF data from embedded JPEG in Panasonic RW2 files
/// RW2 files store the real EXIF data in an embedded JPEG pointed to by tag 0x002E
fn extract_rw2_embedded_exif(tiff_data: &[u8], is_little_endian: bool) -> Option<Vec<u8>> {
    if tiff_data.len() < 8 {
        return None;
    }

    // Read IFD0 offset from TIFF header (bytes 4-7)
    let ifd_offset = if is_little_endian {
        u32::from_le_bytes([tiff_data[4], tiff_data[5], tiff_data[6], tiff_data[7]]) as usize
    } else {
        u32::from_be_bytes([tiff_data[4], tiff_data[5], tiff_data[6], tiff_data[7]]) as usize
    };

    if ifd_offset + 2 > tiff_data.len() {
        return None;
    }

    // Read number of IFD entries
    let num_entries = if is_little_endian {
        u16::from_le_bytes([tiff_data[ifd_offset], tiff_data[ifd_offset + 1]]) as usize
    } else {
        u16::from_be_bytes([tiff_data[ifd_offset], tiff_data[ifd_offset + 1]]) as usize
    };

    // Search for tag 0x002E (JpgFromRaw) in the IFD
    let mut entry_offset = ifd_offset + 2;
    for _ in 0..num_entries {
        if entry_offset + 12 > tiff_data.len() {
            break;
        }

        let tag_id = if is_little_endian {
            u16::from_le_bytes([tiff_data[entry_offset], tiff_data[entry_offset + 1]])
        } else {
            u16::from_be_bytes([tiff_data[entry_offset], tiff_data[entry_offset + 1]])
        };

        if tag_id == 0x002E {
            // Found JpgFromRaw tag - get the data count and offset
            let count = if is_little_endian {
                u32::from_le_bytes([
                    tiff_data[entry_offset + 4],
                    tiff_data[entry_offset + 5],
                    tiff_data[entry_offset + 6],
                    tiff_data[entry_offset + 7],
                ]) as usize
            } else {
                u32::from_be_bytes([
                    tiff_data[entry_offset + 4],
                    tiff_data[entry_offset + 5],
                    tiff_data[entry_offset + 6],
                    tiff_data[entry_offset + 7],
                ]) as usize
            };

            let data_offset = if is_little_endian {
                u32::from_le_bytes([
                    tiff_data[entry_offset + 8],
                    tiff_data[entry_offset + 9],
                    tiff_data[entry_offset + 10],
                    tiff_data[entry_offset + 11],
                ]) as usize
            } else {
                u32::from_be_bytes([
                    tiff_data[entry_offset + 8],
                    tiff_data[entry_offset + 9],
                    tiff_data[entry_offset + 10],
                    tiff_data[entry_offset + 11],
                ]) as usize
            };

            // The embedded JPEG starts at data_offset
            // Look for JPEG SOI marker (0xFFD8) and APP1 marker (0xFFE1) with "Exif\0\0"
            if data_offset + count <= tiff_data.len() {
                let jpeg_data = &tiff_data[data_offset..data_offset + count];

                // Look for APP1 EXIF segment within the JPEG
                if let Some(exif_data) = extract_exif_from_jpeg(jpeg_data) {
                    return Some(exif_data);
                }
            }
            break;
        }

        entry_offset += 12;
    }

    None
}

/// Extract EXIF APP1 segment from embedded JPEG data
fn extract_exif_from_jpeg(jpeg_data: &[u8]) -> Option<Vec<u8>> {
    // Check for JPEG SOI marker
    if jpeg_data.len() < 4 || jpeg_data[0] != 0xFF || jpeg_data[1] != 0xD8 {
        return None;
    }

    let mut offset = 2;
    while offset + 4 < jpeg_data.len() {
        // Check for marker
        if jpeg_data[offset] != 0xFF {
            offset += 1;
            continue;
        }

        let marker = jpeg_data[offset + 1];

        // Skip padding bytes (0xFF)
        if marker == 0xFF {
            offset += 1;
            continue;
        }

        // Check for APP1 marker (0xE1)
        if marker == 0xE1 {
            // Read segment length (big-endian)
            let length =
                u16::from_be_bytes([jpeg_data[offset + 2], jpeg_data[offset + 3]]) as usize;

            if offset + 2 + length > jpeg_data.len() {
                return None;
            }

            // Check for "Exif\0\0" identifier
            let segment_data = &jpeg_data[offset + 4..offset + 2 + length];
            if segment_data.len() >= 6 && &segment_data[0..6] == b"Exif\0\0" {
                // Return the full APP1 data including "Exif\0\0" prefix
                return Some(segment_data.to_vec());
            }
        }

        // Skip to next marker
        if marker == 0xD8 || marker == 0xD9 || (0xD0..=0xD7).contains(&marker) {
            // Standalone markers without length
            offset += 2;
        } else {
            // Markers with length
            if offset + 4 > jpeg_data.len() {
                return None;
            }
            let length =
                u16::from_be_bytes([jpeg_data[offset + 2], jpeg_data[offset + 3]]) as usize;
            offset += 2 + length;
        }
    }

    None
}

/// Detect specific TIFF-based RAW format from TIFF data
/// This function examines the TIFF header and first IFD to identify specific camera formats
#[allow(dead_code)]
pub fn detect_tiff_format(tiff_data: &[u8]) -> Option<&'static str> {
    if tiff_data.len() < 12 {
        return None;
    }

    // Check TIFF signature
    if !((tiff_data[0] == b'I' && tiff_data[1] == b'I')
        || (tiff_data[0] == b'M' && tiff_data[1] == b'M'))
    {
        return None;
    }

    let is_little_endian = tiff_data[0] == b'I';

    // Check magic number at offset 2-3
    let magic = if is_little_endian {
        u16::from_le_bytes([tiff_data[2], tiff_data[3]])
    } else {
        u16::from_be_bytes([tiff_data[2], tiff_data[3]])
    };

    // Check for format-specific markers
    match magic {
        0x002A => {
            // Standard TIFF - check for specific RAW format markers
            // Try to identify format by reading Make tag from first IFD
            if let Some(make) = extract_make_from_ifd(tiff_data, is_little_endian) {
                let make_lower = make.to_lowercase();

                // Sony ARW - Sony Alpha RAW
                if make_lower.contains("sony") {
                    return Some("ARW");
                }

                // Pentax PEF - Pentax Electronic File
                if make_lower.contains("pentax") || make_lower.contains("asahi") {
                    return Some("PEF");
                }

                // Nikon NRW - Nikon RAW (Coolpix series)
                // NEF is for DSLRs, NRW is for Coolpix compacts
                // Both use "NIKON" as make, differentiation would require model tag
                if make_lower.contains("nikon") {
                    // For now, we'll identify both as NEF since they're structurally identical
                    // A more detailed check would examine the Model tag
                    return Some("NEF");
                }

                // Leica RWL - typically saved as DNG
                if make_lower.contains("leica") {
                    // Check if it's a DNG variant
                    if has_dng_tags(tiff_data, is_little_endian) {
                        return Some("DNG");
                    }
                    return Some("RWL");
                }
            }

            // Check for DNG version tag
            if has_dng_tags(tiff_data, is_little_endian) {
                return Some("DNG");
            }

            // Check for CR2 marker (Canon Raw 2)
            if tiff_data.len() >= 10 && tiff_data[8] == b'C' && tiff_data[9] == b'R' {
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

/// Extract the Make tag (0x010F) from the first IFD
fn extract_make_from_ifd(tiff_data: &[u8], is_little_endian: bool) -> Option<String> {
    if tiff_data.len() < 8 {
        return None;
    }

    // Read IFD0 offset from bytes 4-7
    let ifd_offset = if is_little_endian {
        u32::from_le_bytes([tiff_data[4], tiff_data[5], tiff_data[6], tiff_data[7]]) as usize
    } else {
        u32::from_be_bytes([tiff_data[4], tiff_data[5], tiff_data[6], tiff_data[7]]) as usize
    };

    if ifd_offset + 2 > tiff_data.len() {
        return None;
    }

    // Read number of directory entries
    let num_entries = if is_little_endian {
        u16::from_le_bytes([tiff_data[ifd_offset], tiff_data[ifd_offset + 1]])
    } else {
        u16::from_be_bytes([tiff_data[ifd_offset], tiff_data[ifd_offset + 1]])
    };

    // Each IFD entry is 12 bytes
    let mut offset = ifd_offset + 2;
    for _ in 0..num_entries {
        if offset + 12 > tiff_data.len() {
            break;
        }

        // Read tag ID
        let tag_id = if is_little_endian {
            u16::from_le_bytes([tiff_data[offset], tiff_data[offset + 1]])
        } else {
            u16::from_be_bytes([tiff_data[offset], tiff_data[offset + 1]])
        };

        // Check if this is the Make tag (0x010F)
        if tag_id == 0x010F {
            // Read value offset (bytes 8-11 of the entry)
            let value_offset = if is_little_endian {
                u32::from_le_bytes([
                    tiff_data[offset + 8],
                    tiff_data[offset + 9],
                    tiff_data[offset + 10],
                    tiff_data[offset + 11],
                ]) as usize
            } else {
                u32::from_be_bytes([
                    tiff_data[offset + 8],
                    tiff_data[offset + 9],
                    tiff_data[offset + 10],
                    tiff_data[offset + 11],
                ]) as usize
            };

            // Read the Make string
            if value_offset < tiff_data.len() {
                let make_bytes = &tiff_data[value_offset..];
                if let Some(null_pos) = make_bytes.iter().position(|&b| b == 0) {
                    if let Ok(make_str) = std::str::from_utf8(&make_bytes[..null_pos]) {
                        return Some(make_str.to_string());
                    }
                }
            }
            break;
        }

        offset += 12;
    }

    None
}

/// Check if the TIFF data contains DNG-specific tags
fn has_dng_tags(tiff_data: &[u8], is_little_endian: bool) -> bool {
    if tiff_data.len() < 8 {
        return false;
    }

    // Read IFD0 offset
    let ifd_offset = if is_little_endian {
        u32::from_le_bytes([tiff_data[4], tiff_data[5], tiff_data[6], tiff_data[7]]) as usize
    } else {
        u32::from_be_bytes([tiff_data[4], tiff_data[5], tiff_data[6], tiff_data[7]]) as usize
    };

    if ifd_offset + 2 > tiff_data.len() {
        return false;
    }

    // Read number of directory entries
    let num_entries = if is_little_endian {
        u16::from_le_bytes([tiff_data[ifd_offset], tiff_data[ifd_offset + 1]])
    } else {
        u16::from_be_bytes([tiff_data[ifd_offset], tiff_data[ifd_offset + 1]])
    };

    // Look for DNGVersion tag (0xC612)
    let mut offset = ifd_offset + 2;
    for _ in 0..num_entries {
        if offset + 12 > tiff_data.len() {
            break;
        }

        let tag_id = if is_little_endian {
            u16::from_le_bytes([tiff_data[offset], tiff_data[offset + 1]])
        } else {
            u16::from_be_bytes([tiff_data[offset], tiff_data[offset + 1]])
        };

        // DNGVersion tag
        if tag_id == 0xC612 {
            return true;
        }

        offset += 12;
    }

    false
}
