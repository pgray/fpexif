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

/// Extract Compression tag (0x000B) from the first IFD of a Panasonic RW2 file
/// RW2 files have Compression in IFD0 but use embedded JPEG for EXIF, so this
/// value needs to be extracted separately
pub fn extract_rw2_compression<R: Read + Seek>(mut reader: R) -> ExifResult<Option<u16>> {
    // Read TIFF header
    let mut header = [0u8; 8];
    reader.read_exact(&mut header)?;

    // Verify TIFF signature and check if it's RW2 (magic 0x0055)
    if !((header[0] == b'I' && header[1] == b'I') || (header[0] == b'M' && header[1] == b'M')) {
        return Ok(None);
    }

    let is_little_endian = header[0] == b'I';
    let magic = if is_little_endian {
        u16::from_le_bytes([header[2], header[3]])
    } else {
        u16::from_be_bytes([header[2], header[3]])
    };

    // Only process RW2 files
    if magic != 0x0055 {
        return Ok(None);
    }

    // Read IFD0 offset
    let ifd_offset = if is_little_endian {
        u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize
    } else {
        u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as usize
    };

    // Read IFD0 content
    reader.seek(SeekFrom::Start(ifd_offset as u64))?;
    let mut ifd_header = [0u8; 2];
    reader.read_exact(&mut ifd_header)?;

    let num_entries = if is_little_endian {
        u16::from_le_bytes(ifd_header)
    } else {
        u16::from_be_bytes(ifd_header)
    } as usize;

    // Search for Panasonic Compression tag (0x000B in RW2, not standard 0x0103)
    for _ in 0..num_entries {
        let mut entry = [0u8; 12];
        reader.read_exact(&mut entry)?;

        let tag_id = if is_little_endian {
            u16::from_le_bytes([entry[0], entry[1]])
        } else {
            u16::from_be_bytes([entry[0], entry[1]])
        };

        if tag_id == 0x000B {
            // Panasonic Compression tag found - read value from bytes 8-9 (for SHORT type)
            let compression = if is_little_endian {
                u16::from_le_bytes([entry[8], entry[9]])
            } else {
                u16::from_be_bytes([entry[8], entry[9]])
            };
            return Ok(Some(compression));
        }
    }

    Ok(None)
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

/// RW2-specific metadata extracted from Panasonic RAW IFD0
/// This includes sensor info, white balance data, distortion info, etc.
#[derive(Debug, Clone, Default)]
pub struct Rw2Metadata {
    /// Tag name -> string value
    pub tags: std::collections::HashMap<String, String>,
}

impl Rw2Metadata {
    pub fn new() -> Self {
        Self {
            tags: std::collections::HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: &str, value: String) {
        self.tags.insert(key.to_string(), value);
    }
}

/// Check if a reader contains an RW2 file and extract RW2-specific metadata if so
pub fn extract_rw2_metadata_if_rw2<R: Read + Seek>(
    mut reader: R,
) -> ExifResult<Option<Rw2Metadata>> {
    // Read TIFF header
    let mut header = [0u8; 8];
    reader.read_exact(&mut header)?;

    // Reset to beginning
    reader.seek(SeekFrom::Start(0))?;

    // Verify TIFF signature
    if !((header[0] == b'I' && header[1] == b'I') || (header[0] == b'M' && header[1] == b'M')) {
        return Ok(None);
    }

    let is_little_endian = header[0] == b'I';
    let magic = if is_little_endian {
        u16::from_le_bytes([header[2], header[3]])
    } else {
        u16::from_be_bytes([header[2], header[3]])
    };

    // Only process RW2 files (magic 0x0055)
    if magic != 0x0055 {
        return Ok(None);
    }

    // Read the entire file into memory
    let mut tiff_data = Vec::new();
    reader.read_to_end(&mut tiff_data)?;

    Ok(Some(extract_rw2_ifd0_metadata(
        &tiff_data,
        is_little_endian,
    )))
}

/// Extract metadata from RW2 IFD0
fn extract_rw2_ifd0_metadata(tiff_data: &[u8], is_little_endian: bool) -> Rw2Metadata {
    let mut metadata = Rw2Metadata::new();

    if tiff_data.len() < 8 {
        return metadata;
    }

    // Read IFD0 offset from TIFF header (bytes 4-7)
    let ifd_offset = if is_little_endian {
        u32::from_le_bytes([tiff_data[4], tiff_data[5], tiff_data[6], tiff_data[7]]) as usize
    } else {
        u32::from_be_bytes([tiff_data[4], tiff_data[5], tiff_data[6], tiff_data[7]]) as usize
    };

    if ifd_offset + 2 > tiff_data.len() {
        return metadata;
    }

    // Read number of IFD entries
    let num_entries = if is_little_endian {
        u16::from_le_bytes([tiff_data[ifd_offset], tiff_data[ifd_offset + 1]]) as usize
    } else {
        u16::from_be_bytes([tiff_data[ifd_offset], tiff_data[ifd_offset + 1]]) as usize
    };

    // Process each IFD entry
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

        let tag_type = if is_little_endian {
            u16::from_le_bytes([tiff_data[entry_offset + 2], tiff_data[entry_offset + 3]])
        } else {
            u16::from_be_bytes([tiff_data[entry_offset + 2], tiff_data[entry_offset + 3]])
        };

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

        let value_offset_bytes = &tiff_data[entry_offset + 8..entry_offset + 12];

        // Process specific RW2 IFD0 tags
        match tag_id {
            // PanasonicRawVersion (0x0001)
            0x0001 => {
                if let Some(val) = read_rw2_string(
                    tiff_data,
                    value_offset_bytes,
                    tag_type,
                    count,
                    is_little_endian,
                ) {
                    metadata.insert("PanasonicRawVersion", val);
                }
            }
            // SensorWidth (0x0002)
            0x0002 => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("SensorWidth", val.to_string());
                }
            }
            // SensorHeight (0x0003)
            0x0003 => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("SensorHeight", val.to_string());
                }
            }
            // SensorTopBorder (0x0004)
            0x0004 => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("SensorTopBorder", val.to_string());
                }
            }
            // SensorLeftBorder (0x0005)
            0x0005 => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("SensorLeftBorder", val.to_string());
                }
            }
            // SensorBottomBorder (0x0006)
            0x0006 => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("SensorBottomBorder", val.to_string());
                }
            }
            // SensorRightBorder (0x0007)
            0x0007 => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("SensorRightBorder", val.to_string());
                }
            }
            // SamplesPerPixel (0x0008)
            0x0008 => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("SamplesPerPixel", val.to_string());
                }
            }
            // CFAPattern (0x0009)
            0x0009 => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    let pattern = decode_rw2_cfa_pattern(val);
                    metadata.insert("CFAPattern", pattern.to_string());
                }
            }
            // BitsPerSample (0x000A)
            0x000A => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("BitsPerSample", val.to_string());
                }
            }
            // LinearityLimitRed (0x000E)
            0x000E => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("LinearityLimitRed", val.to_string());
                }
            }
            // LinearityLimitGreen (0x000F)
            0x000F => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("LinearityLimitGreen", val.to_string());
                }
            }
            // LinearityLimitBlue (0x0010)
            0x0010 => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("LinearityLimitBlue", val.to_string());
                }
            }
            // RedBalance (0x0011)
            0x0011 => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    let balance = val as f64 / 256.0;
                    metadata.insert("RedBalance", format!("{:.6}", balance));
                }
            }
            // BlueBalance (0x0012)
            0x0012 => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    let balance = val as f64 / 256.0;
                    metadata.insert("BlueBalance", format!("{:.6}", balance));
                }
            }
            // WBInfo (0x0013) - subdirectory with WBType/WB_RBLevels (older RW2)
            0x0013 => {
                if let Some(offset) = read_rw2_long(value_offset_bytes, is_little_endian) {
                    parse_rw2_wb_info(
                        tiff_data,
                        offset as usize,
                        count,
                        is_little_endian,
                        false,
                        &mut metadata,
                    );
                }
            }
            // ISO (0x0017)
            0x0017 => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("ISO", val.to_string());
                }
            }
            // HighISOMultiplierRed (0x0018)
            0x0018 => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    let mult = val as f64 / 256.0;
                    metadata.insert("HighISOMultiplierRed", format!("{:.6}", mult));
                }
            }
            // HighISOMultiplierGreen (0x0019)
            0x0019 => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    let mult = val as f64 / 256.0;
                    metadata.insert("HighISOMultiplierGreen", format!("{:.6}", mult));
                }
            }
            // HighISOMultiplierBlue (0x001A)
            0x001A => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    let mult = val as f64 / 256.0;
                    metadata.insert("HighISOMultiplierBlue", format!("{:.6}", mult));
                }
            }
            // BlackLevelRed (0x001C)
            0x001C => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("BlackLevelRed", val.to_string());
                }
            }
            // BlackLevelGreen (0x001D)
            0x001D => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("BlackLevelGreen", val.to_string());
                }
            }
            // BlackLevelBlue (0x001E)
            0x001E => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("BlackLevelBlue", val.to_string());
                }
            }
            // WBRedLevel (0x0024)
            0x0024 => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("WBRedLevel", val.to_string());
                }
            }
            // WBGreenLevel (0x0025)
            0x0025 => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("WBGreenLevel", val.to_string());
                }
            }
            // WBBlueLevel (0x0026)
            0x0026 => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("WBBlueLevel", val.to_string());
                }
            }
            // WBInfo2 (0x0027) - subdirectory with WBType/WB_RGBLevels (newer RW2)
            0x0027 => {
                if let Some(offset) = read_rw2_long(value_offset_bytes, is_little_endian) {
                    parse_rw2_wb_info(
                        tiff_data,
                        offset as usize,
                        count,
                        is_little_endian,
                        true,
                        &mut metadata,
                    );
                }
            }
            // RawFormat (0x002D)
            0x002D => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("RawFormat", val.to_string());
                }
            }
            // CropTop (0x002F)
            0x002F => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("CropTop", val.to_string());
                }
            }
            // CropLeft (0x0030)
            0x0030 => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("CropLeft", val.to_string());
                }
            }
            // CropBottom (0x0031)
            0x0031 => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("CropBottom", val.to_string());
                }
            }
            // CropRight (0x0032)
            0x0032 => {
                if let Some(val) = read_rw2_short(value_offset_bytes, is_little_endian) {
                    metadata.insert("CropRight", val.to_string());
                }
            }
            // RowsPerStrip (0x0116)
            0x0116 => {
                if let Some(val) = read_rw2_long(value_offset_bytes, is_little_endian) {
                    metadata.insert("RowsPerStrip", val.to_string());
                }
            }
            // DistortionInfo (0x0119) - distortion correction subdirectory
            0x0119 => {
                if let Some(offset) = read_rw2_long(value_offset_bytes, is_little_endian) {
                    parse_rw2_distortion_info(
                        tiff_data,
                        offset as usize,
                        count,
                        is_little_endian,
                        &mut metadata,
                    );
                }
            }
            _ => {}
        }

        entry_offset += 12;
    }

    metadata
}

/// Read a SHORT (u16) value from IFD value/offset bytes
fn read_rw2_short(bytes: &[u8], is_little_endian: bool) -> Option<u16> {
    if bytes.len() < 2 {
        return None;
    }
    Some(if is_little_endian {
        u16::from_le_bytes([bytes[0], bytes[1]])
    } else {
        u16::from_be_bytes([bytes[0], bytes[1]])
    })
}

/// Read a LONG (u32) value from IFD value/offset bytes
fn read_rw2_long(bytes: &[u8], is_little_endian: bool) -> Option<u32> {
    if bytes.len() < 4 {
        return None;
    }
    Some(if is_little_endian {
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    } else {
        u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    })
}

/// Read a string value from an IFD entry
fn read_rw2_string(
    tiff_data: &[u8],
    value_bytes: &[u8],
    tag_type: u16,
    count: usize,
    is_little_endian: bool,
) -> Option<String> {
    // Type 2 = ASCII, Type 7 = UNDEFINED
    if tag_type != 2 && tag_type != 7 {
        return None;
    }

    let data_bytes = if count <= 4 {
        // Value fits inline
        &value_bytes[..count.min(4)]
    } else {
        // Value is at an offset
        let offset = if is_little_endian {
            u32::from_le_bytes([
                value_bytes[0],
                value_bytes[1],
                value_bytes[2],
                value_bytes[3],
            ]) as usize
        } else {
            u32::from_be_bytes([
                value_bytes[0],
                value_bytes[1],
                value_bytes[2],
                value_bytes[3],
            ]) as usize
        };

        if offset + count > tiff_data.len() {
            return None;
        }
        &tiff_data[offset..offset + count]
    };

    // Convert to string, stripping null terminators
    let s: String = data_bytes
        .iter()
        .take_while(|&&b| b != 0)
        .map(|&b| b as char)
        .collect();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

/// Decode CFAPattern value to string
fn decode_rw2_cfa_pattern(value: u16) -> &'static str {
    match value {
        0 => "n/a",
        1 => "[Red,Green][Green,Blue]",
        2 => "[Green,Red][Blue,Green]",
        3 => "[Green,Blue][Red,Green]",
        4 => "[Blue,Green][Green,Red]",
        _ => "Unknown",
    }
}

/// Decode WBType (LightSource) value to string
fn decode_rw2_wb_type(value: u16) -> &'static str {
    // EXIF LightSource values
    match value {
        0 => "Unknown",
        1 => "Daylight",
        2 => "Fluorescent",
        3 => "Tungsten (Incandescent)",
        4 => "Flash",
        9 => "Fine Weather",
        10 => "Cloudy",
        11 => "Shade",
        12 => "Daylight Fluorescent",
        13 => "Day White Fluorescent",
        14 => "Cool White Fluorescent",
        15 => "White Fluorescent",
        16 => "Warm White Fluorescent",
        17 => "Standard Light A",
        18 => "Standard Light B",
        19 => "Standard Light C",
        20 => "D55",
        21 => "D65",
        22 => "D75",
        23 => "D50",
        24 => "ISO Studio Tungsten",
        255 => "Other",
        _ => "Unknown",
    }
}

/// Parse WBInfo or WBInfo2 subdirectory
/// WBInfo uses 2-element RB levels, WBInfo2 uses 3-element RGB levels
fn parse_rw2_wb_info(
    tiff_data: &[u8],
    offset: usize,
    count: usize,
    is_little_endian: bool,
    is_rgb: bool,
    metadata: &mut Rw2Metadata,
) {
    // The data is an array of int16u values
    let byte_count = count * 2;
    if offset + byte_count > tiff_data.len() {
        return;
    }

    let data = &tiff_data[offset..offset + byte_count];

    // Helper to read u16 at word index
    let read_u16 = |idx: usize| -> u16 {
        let byte_idx = idx * 2;
        if byte_idx + 2 > data.len() {
            return 0;
        }
        if is_little_endian {
            u16::from_le_bytes([data[byte_idx], data[byte_idx + 1]])
        } else {
            u16::from_be_bytes([data[byte_idx], data[byte_idx + 1]])
        }
    };

    // First value is NumWBEntries
    let num_entries = read_u16(0);
    metadata.insert("NumWBEntries", num_entries.to_string());

    if is_rgb {
        // WBInfo2: 4 words per entry (1 type + 3 RGB levels)
        // Entry 1: indices 1,2,3,4 (WBType1, R, G, B)
        // Entry 2: indices 5,6,7,8 (WBType2, R, G, B)
        // etc.
        for i in 0..7u16 {
            let type_idx = 1 + (i as usize * 4);
            let levels_idx = type_idx + 1;

            if type_idx >= count || levels_idx + 2 >= count {
                break;
            }

            let wb_type = read_u16(type_idx);
            let r = read_u16(levels_idx);
            let g = read_u16(levels_idx + 1);
            let b = read_u16(levels_idx + 2);

            let type_name = decode_rw2_wb_type(wb_type);
            metadata.insert(&format!("WBType{}", i + 1), type_name.to_string());
            metadata.insert(
                &format!("WB_RGBLevels{}", i + 1),
                format!("{} {} {}", r, g, b),
            );
        }
    } else {
        // WBInfo: 3 words per entry (1 type + 2 RB levels)
        // Entry 1: indices 1,2,3 (WBType1, R, B)
        // Entry 2: indices 4,5,6 (WBType2, R, B)
        // etc.
        for i in 0..7u16 {
            let type_idx = 1 + (i as usize * 3);
            let levels_idx = type_idx + 1;

            if type_idx >= count || levels_idx + 1 >= count {
                break;
            }

            let wb_type = read_u16(type_idx);
            let r = read_u16(levels_idx);
            let b = read_u16(levels_idx + 1);

            let type_name = decode_rw2_wb_type(wb_type);
            metadata.insert(&format!("WBType{}", i + 1), type_name.to_string());
            metadata.insert(&format!("WB_RBLevels{}", i + 1), format!("{} {}", r, b));
        }
    }
}

/// Parse DistortionInfo subdirectory
fn parse_rw2_distortion_info(
    tiff_data: &[u8],
    offset: usize,
    count: usize,
    is_little_endian: bool,
    metadata: &mut Rw2Metadata,
) {
    // The data is an array of int16s values
    let byte_count = count * 2;
    if offset + byte_count > tiff_data.len() {
        return;
    }

    let data = &tiff_data[offset..offset + byte_count];

    // Helper to read i16 at word index
    let read_i16 = |idx: usize| -> i16 {
        let byte_idx = idx * 2;
        if byte_idx + 2 > data.len() {
            return 0;
        }
        if is_little_endian {
            i16::from_le_bytes([data[byte_idx], data[byte_idx + 1]])
        } else {
            i16::from_be_bytes([data[byte_idx], data[byte_idx + 1]])
        }
    };

    // DistortionParam02 at index 2
    if count > 2 {
        let val = read_i16(2) as f64 / 32768.0;
        metadata.insert("DistortionParam02", format!("{}", val));
    }

    // DistortionParam04 at index 4
    if count > 4 {
        let val = read_i16(4) as f64 / 32768.0;
        metadata.insert("DistortionParam04", format!("{}", val));
    }

    // DistortionScale at index 5
    if count > 5 {
        let val = read_i16(5);
        let scale = 1.0 / (1.0 + val as f64 / 32768.0);
        metadata.insert("DistortionScale", format!("{}", scale));
    }

    // DistortionCorrection at index 7 (lower 4 bits)
    if count > 7 {
        let val = read_i16(7) & 0x0F;
        let correction = if val == 0 { "Off" } else { "On" };
        metadata.insert("DistortionCorrection", correction.to_string());
    }

    // DistortionParam08 at index 8
    if count > 8 {
        let val = read_i16(8) as f64 / 32768.0;
        metadata.insert("DistortionParam08", format!("{}", val));
    }

    // DistortionParam09 at index 9
    if count > 9 {
        let val = read_i16(9) as f64 / 32768.0;
        metadata.insert("DistortionParam09", format!("{}", val));
    }

    // DistortionParam11 at index 11
    if count > 11 {
        let val = read_i16(11) as f64 / 32768.0;
        metadata.insert("DistortionParam11", format!("{}", val));
    }

    // DistortionN at index 12
    if count > 12 {
        let val = read_i16(12);
        metadata.insert("DistortionN", val.to_string());
    }
}
