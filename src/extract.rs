// extract.rs - JPEG extraction from RAW files

use crate::errors::{ExifError, ExifResult};
use std::io::{Read, Seek, SeekFrom};

/// Type of embedded JPEG to extract
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JpegType {
    /// Largest available preview (usually full-size JPEG)
    Preview,
    /// Small thumbnail (usually from IFD1)
    Thumbnail,
    /// All embedded JPEGs
    All,
}

/// Information about an embedded JPEG
#[derive(Debug, Clone)]
pub struct EmbeddedJpeg {
    /// Offset from start of file
    pub offset: u64,
    /// Length in bytes
    pub length: u64,
    /// Description (e.g., "IFD0 Preview", "IFD1 Thumbnail")
    pub description: String,
    /// Image dimensions if known
    pub dimensions: Option<(u32, u32)>,
}

/// Result of JPEG validation
#[derive(Debug, Clone)]
pub struct JpegValidation {
    /// Whether the JPEG structure is valid
    pub valid: bool,
    /// Image width (from SOF marker)
    pub width: Option<u32>,
    /// Image height (from SOF marker)
    pub height: Option<u32>,
    /// Whether EOI marker was found
    pub has_eoi: bool,
    /// Error message if invalid
    pub error: Option<String>,
}

/// Validate JPEG data and extract dimensions
///
/// Parses the JPEG segment structure to verify it's a valid JPEG
/// and extracts width/height from the SOF (Start of Frame) marker.
pub fn validate_jpeg(data: &[u8]) -> JpegValidation {
    if data.len() < 4 {
        return JpegValidation {
            valid: false,
            width: None,
            height: None,
            has_eoi: false,
            error: Some("Data too short".to_string()),
        };
    }

    // Check SOI (Start of Image)
    if data[0] != 0xFF || data[1] != 0xD8 {
        return JpegValidation {
            valid: false,
            width: None,
            height: None,
            has_eoi: false,
            error: Some("Missing SOI marker".to_string()),
        };
    }

    let mut pos = 2;
    let mut width = None;
    let mut height = None;
    let mut has_eoi = false;

    // Parse JPEG segments
    while pos + 2 <= data.len() {
        // Each segment starts with 0xFF
        if data[pos] != 0xFF {
            // Skip padding bytes (0xFF can be followed by more 0xFF)
            pos += 1;
            continue;
        }

        // Skip any padding 0xFF bytes
        while pos + 1 < data.len() && data[pos + 1] == 0xFF {
            pos += 1;
        }

        if pos + 2 > data.len() {
            break;
        }

        let marker = data[pos + 1];
        pos += 2;

        match marker {
            0xD8 => {
                // SOI - shouldn't appear again
            }
            0xD9 => {
                // EOI (End of Image)
                has_eoi = true;
                break;
            }
            0x00 => {
                // Stuffed byte, skip
            }
            0xD0..=0xD7 => {
                // RST markers (no length)
            }
            0xC0 | 0xC1 | 0xC2 | 0xC3 | 0xC5 | 0xC6 | 0xC7 | 0xC9 | 0xCA | 0xCB | 0xCD | 0xCE
            | 0xCF => {
                // SOF markers - contains dimensions
                if pos + 7 <= data.len() {
                    let _length = u16::from_be_bytes([data[pos], data[pos + 1]]);
                    let _precision = data[pos + 2];
                    height = Some(u16::from_be_bytes([data[pos + 3], data[pos + 4]]) as u32);
                    width = Some(u16::from_be_bytes([data[pos + 5], data[pos + 6]]) as u32);
                }
                // Skip segment
                if pos + 2 <= data.len() {
                    let length = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
                    pos += length;
                }
            }
            0xDA => {
                // SOS (Start of Scan) - entropy-coded data follows
                // Skip to find EOI at the end
                if find_eoi(&data[pos..]).is_some() {
                    has_eoi = true;
                }
                break;
            }
            _ => {
                // Other segments have length field
                if pos + 2 <= data.len() {
                    let length = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
                    if length < 2 {
                        return JpegValidation {
                            valid: false,
                            width,
                            height,
                            has_eoi: false,
                            error: Some(format!("Invalid segment length at offset {}", pos)),
                        };
                    }
                    pos += length;
                } else {
                    break;
                }
            }
        }
    }

    JpegValidation {
        valid: width.is_some() && height.is_some(),
        width,
        height,
        has_eoi,
        error: if width.is_none() || height.is_none() {
            Some("No SOF marker found".to_string())
        } else {
            None
        },
    }
}

/// Find EOI marker in data (searching from end is faster)
fn find_eoi(data: &[u8]) -> Option<usize> {
    // Search from the end since EOI is typically at the end
    if data.len() < 2 {
        return None;
    }
    (0..data.len() - 1)
        .rev()
        .find(|&i| data[i] == 0xFF && data[i + 1] == 0xD9)
}

/// Extract embedded JPEG(s) from a RAW file
pub fn extract_jpegs<R: Read + Seek>(
    mut reader: R,
    jpeg_type: JpegType,
) -> ExifResult<Vec<(EmbeddedJpeg, Vec<u8>)>> {
    // Read file header to determine format
    let mut header = [0u8; 16];
    reader.read_exact(&mut header)?;
    reader.seek(SeekFrom::Start(0))?;

    // Determine format and extract
    if header[0..2] == [0x49, 0x49] || header[0..2] == [0x4D, 0x4D] {
        // TIFF-based (NEF, DNG, ORF, ARW, CR2, etc.)
        extract_from_tiff(reader, jpeg_type)
    } else if &header[0..4] == b"FUJI" {
        // Fujifilm RAF
        extract_from_raf(reader, jpeg_type)
    } else if header[0..4] == [0x00, 0x4D, 0x52, 0x4D] {
        // Minolta MRW
        extract_from_mrw(reader, jpeg_type)
    } else if &header[4..8] == b"ftyp" {
        // ISO Base Media (CR3, HEIF)
        extract_from_isobmff(reader, jpeg_type)
    } else {
        Err(ExifError::Format(
            "Unknown or unsupported RAW format".to_string(),
        ))
    }
}

/// Extract JPEGs from TIFF-based RAW files
fn extract_from_tiff<R: Read + Seek>(
    mut reader: R,
    jpeg_type: JpegType,
) -> ExifResult<Vec<(EmbeddedJpeg, Vec<u8>)>> {
    // Read entire file
    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;

    if data.len() < 8 {
        return Err(ExifError::Format("File too short".to_string()));
    }

    let is_little_endian = data[0] == b'I';
    let read_u16 = |offset: usize| -> u16 {
        if is_little_endian {
            u16::from_le_bytes([data[offset], data[offset + 1]])
        } else {
            u16::from_be_bytes([data[offset], data[offset + 1]])
        }
    };
    let read_u32 = |offset: usize| -> u32 {
        if is_little_endian {
            u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ])
        } else {
            u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ])
        }
    };

    let mut jpegs = Vec::new();
    let mut ifd_offsets_to_process = vec![(read_u32(4) as usize, "IFD0".to_string())];
    let mut processed_offsets = std::collections::HashSet::new();

    while let Some((ifd_offset, ifd_name)) = ifd_offsets_to_process.pop() {
        if ifd_offset == 0 || ifd_offset >= data.len() || processed_offsets.contains(&ifd_offset) {
            continue;
        }
        processed_offsets.insert(ifd_offset);

        if ifd_offset + 2 > data.len() {
            continue;
        }

        let entry_count = read_u16(ifd_offset) as usize;
        let entries_end = ifd_offset + 2 + entry_count * 12;

        if entries_end + 4 > data.len() {
            continue;
        }

        let mut compression: Option<u16> = None;
        let mut jpeg_offset: Option<u32> = None;
        let mut jpeg_length: Option<u32> = None;
        let mut strip_offsets: Option<u32> = None;
        let mut strip_byte_counts: Option<u32> = None;
        let mut image_width: Option<u32> = None;
        let mut image_height: Option<u32> = None;
        let mut subfile_type: Option<u32> = None;

        for i in 0..entry_count {
            let entry_offset = ifd_offset + 2 + i * 12;
            if entry_offset + 12 > data.len() {
                break;
            }

            let tag_id = read_u16(entry_offset);
            let tag_type = read_u16(entry_offset + 2);
            let count = read_u32(entry_offset + 4);
            let value_offset = entry_offset + 8;

            // Get value (inline or from offset)
            let get_value = || -> u32 {
                if tag_type == 3 && count == 1 {
                    // SHORT - value is stored inline
                    read_u16(value_offset) as u32
                } else {
                    // LONG or pointer to data
                    read_u32(value_offset)
                }
            };

            match tag_id {
                0x00FE => subfile_type = Some(get_value()),
                0x0100 => image_width = Some(get_value()),
                0x0101 => image_height = Some(get_value()),
                0x0103 => compression = Some(get_value() as u16),
                0x0111 => strip_offsets = Some(get_value()),
                0x0117 => strip_byte_counts = Some(get_value()),
                0x0201 => jpeg_offset = Some(get_value()),
                0x0202 => jpeg_length = Some(get_value()),
                0x002E => {
                    // JpgFromRaw (Panasonic RW2) - embedded JPEG with EXIF
                    // tag_type 7 = UNDEFINED, value is offset to data
                    let offset = read_u32(value_offset) as usize;
                    let length = count as usize;
                    if length > 0 && offset + length <= data.len() {
                        let jpeg_data = &data[offset..offset + length];
                        if jpeg_data.len() >= 2 && jpeg_data[0] == 0xFF && jpeg_data[1] == 0xD8 {
                            jpegs.push((
                                EmbeddedJpeg {
                                    offset: offset as u64,
                                    length: length as u64,
                                    description: format!("{} JpgFromRaw", ifd_name),
                                    dimensions: None,
                                },
                                jpeg_data.to_vec(),
                            ));
                        }
                    }
                }
                0x014A => {
                    // SubIFDs - queue them for processing
                    let subifd_offset = read_u32(value_offset) as usize;
                    for idx in 0..count as usize {
                        let off = if count == 1 {
                            subifd_offset
                        } else {
                            let ptr = subifd_offset + idx * 4;
                            if ptr + 4 <= data.len() {
                                read_u32(ptr) as usize
                            } else {
                                continue;
                            }
                        };
                        ifd_offsets_to_process.push((off, format!("{} SubIFD{}", ifd_name, idx)));
                    }
                }
                0x8769 => {
                    // EXIF IFD - queue for processing
                    let exif_offset = get_value() as usize;
                    ifd_offsets_to_process.push((exif_offset, format!("{} EXIF", ifd_name)));
                }
                0x927C => {
                    // MakerNote - check for Olympus format
                    let mn_offset = read_u32(value_offset) as usize;
                    let mn_length = count as usize;
                    if mn_offset + 12 <= data.len() && mn_length > 12 {
                        // Check for Olympus header "OLYMPUS\0II"
                        if &data[mn_offset..mn_offset + 8] == b"OLYMPUS\0"
                            && &data[mn_offset + 8..mn_offset + 10] == b"II"
                        {
                            // Parse Olympus MakerNotes for embedded JPEGs
                            if let Some(extracted) =
                                extract_olympus_makernotes(&data, mn_offset, is_little_endian)
                            {
                                jpegs.extend(extracted);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Check for JPEG data
        let is_jpeg = compression == Some(6) || compression == Some(7);

        // Method 1: JPEGInterchangeFormat (typical for thumbnails)
        if let (Some(offset), Some(length)) = (jpeg_offset, jpeg_length)
            && length > 0
            && offset as usize + length as usize <= data.len()
        {
            let jpeg_data = &data[offset as usize..(offset + length) as usize];
            // Verify JPEG magic
            if jpeg_data.len() >= 2 && jpeg_data[0] == 0xFF && jpeg_data[1] == 0xD8 {
                let desc = if subfile_type == Some(1) {
                    format!("{} Thumbnail", ifd_name)
                } else {
                    format!("{} Preview", ifd_name)
                };
                jpegs.push((
                    EmbeddedJpeg {
                        offset: offset as u64,
                        length: length as u64,
                        description: desc,
                        dimensions: match (image_width, image_height) {
                            (Some(w), Some(h)) => Some((w, h)),
                            _ => None,
                        },
                    },
                    jpeg_data.to_vec(),
                ));
            }
        }

        // Method 2: StripOffsets with JPEG compression
        if is_jpeg
            && let (Some(offset), Some(length)) = (strip_offsets, strip_byte_counts)
            && length > 0
            && offset as usize + length as usize <= data.len()
        {
            let jpeg_data = &data[offset as usize..(offset + length) as usize];
            // Verify JPEG magic
            if jpeg_data.len() >= 2 && jpeg_data[0] == 0xFF && jpeg_data[1] == 0xD8 {
                let desc = if subfile_type == Some(0) {
                    format!("{} Full Preview", ifd_name)
                } else {
                    format!("{} Preview", ifd_name)
                };
                jpegs.push((
                    EmbeddedJpeg {
                        offset: offset as u64,
                        length: length as u64,
                        description: desc,
                        dimensions: match (image_width, image_height) {
                            (Some(w), Some(h)) => Some((w, h)),
                            _ => None,
                        },
                    },
                    jpeg_data.to_vec(),
                ));
            }
        }

        // Queue next IFD in chain
        let next_ifd = read_u32(entries_end) as usize;
        if next_ifd != 0 {
            let next_name = if ifd_name == "IFD0" {
                "IFD1".to_string()
            } else {
                format!("{}+1", ifd_name)
            };
            ifd_offsets_to_process.push((next_ifd, next_name));
        }
    }

    // Filter based on requested type
    let filtered = match jpeg_type {
        JpegType::All => jpegs,
        JpegType::Preview => {
            // Return largest JPEG
            jpegs
                .into_iter()
                .max_by_key(|(info, _)| info.length)
                .into_iter()
                .collect()
        }
        JpegType::Thumbnail => {
            // Return smallest JPEG or one marked as thumbnail
            jpegs
                .into_iter()
                .filter(|(info, _)| info.description.contains("Thumbnail") || info.length < 100_000)
                .min_by_key(|(info, _)| info.length)
                .into_iter()
                .collect()
        }
    };

    Ok(filtered)
}

/// Extract JPEGs from Olympus MakerNotes
/// Olympus stores thumbnail at tag 0x0100 and preview info at 0x2020 CameraSettings IFD
fn extract_olympus_makernotes(
    data: &[u8],
    mn_offset: usize,
    _file_endian: bool,
) -> Option<Vec<(EmbeddedJpeg, Vec<u8>)>> {
    // Olympus MakerNotes structure:
    // 0-7: "OLYMPUS\0"
    // 8-9: Byte order "II" or "MM"
    // 10: Version (usually 0x03)
    // 11: Unknown
    // 12-15: IFD offset from start of MakerNotes

    if mn_offset + 16 > data.len() {
        return None;
    }

    let is_little_endian = &data[mn_offset + 8..mn_offset + 10] == b"II";
    let base = mn_offset; // Olympus offsets are relative to MakerNotes start

    let read_u16 = |offset: usize| -> u16 {
        if offset + 2 > data.len() {
            return 0;
        }
        if is_little_endian {
            u16::from_le_bytes([data[offset], data[offset + 1]])
        } else {
            u16::from_be_bytes([data[offset], data[offset + 1]])
        }
    };
    let read_u32 = |offset: usize| -> u32 {
        if offset + 4 > data.len() {
            return 0;
        }
        if is_little_endian {
            u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ])
        } else {
            u32::from_be_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ])
        }
    };

    let mut jpegs = Vec::new();

    // IFD starts at offset 12 from MakerNotes start
    let ifd_offset = base + 12;
    if ifd_offset + 2 > data.len() {
        return None;
    }

    let entry_count = read_u16(ifd_offset) as usize;
    if entry_count == 0 || entry_count > 500 {
        return None;
    }

    let mut camera_settings_offset: Option<usize> = None;

    for i in 0..entry_count {
        let entry_offset = ifd_offset + 2 + i * 12;
        if entry_offset + 12 > data.len() {
            break;
        }

        let tag_id = read_u16(entry_offset);
        let tag_type = read_u16(entry_offset + 2);
        let count = read_u32(entry_offset + 4) as usize;
        let value_offset = entry_offset + 8;

        match tag_id {
            0x0100 => {
                // ThumbnailImage - raw JPEG data
                // Type 7 = UNDEFINED, value is offset to data
                if tag_type == 7 && count > 100 {
                    let offset = read_u32(value_offset) as usize;
                    let abs_offset = base + offset;
                    if abs_offset + count <= data.len() {
                        let jpeg_data = &data[abs_offset..abs_offset + count];
                        if jpeg_data.len() >= 2 && jpeg_data[0] == 0xFF && jpeg_data[1] == 0xD8 {
                            jpegs.push((
                                EmbeddedJpeg {
                                    offset: abs_offset as u64,
                                    length: count as u64,
                                    description: "Olympus Thumbnail".to_string(),
                                    dimensions: None,
                                },
                                jpeg_data.to_vec(),
                            ));
                        }
                    }
                }
            }
            0x2020 => {
                // CameraSettings IFD pointer
                // Type can be: 13 (IFD), 4 (LONG), or 7 (UNDEFINED for older models)
                let offset = read_u32(value_offset) as usize;
                if offset > 0 && offset < 0x1000000 {
                    camera_settings_offset = Some(base + offset);
                }
            }
            _ => {}
        }
    }

    // Parse CameraSettings IFD for PreviewImage
    if let Some(cs_offset) = camera_settings_offset
        && cs_offset + 2 <= data.len()
    {
        let cs_count = read_u16(cs_offset) as usize;
        if cs_count > 0 && cs_count < 500 {
            let mut preview_valid = false;
            let mut preview_start: Option<u32> = None;
            let mut preview_length: Option<u32> = None;

            for i in 0..cs_count {
                let entry_offset = cs_offset + 2 + i * 12;
                if entry_offset + 12 > data.len() {
                    break;
                }

                let tag_id = read_u16(entry_offset);
                let value_offset = entry_offset + 8;

                match tag_id {
                    0x0100 => {
                        // PreviewImageValid
                        preview_valid = read_u32(value_offset) == 1;
                    }
                    0x0101 => {
                        // PreviewImageStart (absolute offset from file start)
                        preview_start = Some(read_u32(value_offset));
                    }
                    0x0102 => {
                        // PreviewImageLength
                        preview_length = Some(read_u32(value_offset));
                    }
                    _ => {}
                }
            }

            if preview_valid && let (Some(start), Some(length)) = (preview_start, preview_length) {
                // Preview offset is relative to MakerNote base
                let abs_start = base + start as usize;
                let length = length as usize;
                if abs_start + length <= data.len() && length > 0 {
                    let jpeg_data = &data[abs_start..abs_start + length];
                    if jpeg_data.len() >= 2 && jpeg_data[0] == 0xFF && jpeg_data[1] == 0xD8 {
                        jpegs.push((
                            EmbeddedJpeg {
                                offset: abs_start as u64,
                                length: length as u64,
                                description: "Olympus Preview".to_string(),
                                dimensions: None,
                            },
                            jpeg_data.to_vec(),
                        ));
                    }
                }
            }
        }
    }

    if jpegs.is_empty() { None } else { Some(jpegs) }
}

/// Extract JPEGs from Fujifilm RAF files
fn extract_from_raf<R: Read + Seek>(
    mut reader: R,
    jpeg_type: JpegType,
) -> ExifResult<Vec<(EmbeddedJpeg, Vec<u8>)>> {
    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;

    if data.len() < 84 {
        return Err(ExifError::Format("RAF file too short".to_string()));
    }

    // RAF header structure:
    // 0-15: "FUJIFILMCCD-RAW "
    // 16-19: Format version
    // 20-27: Camera ID
    // 28-59: Camera name
    // 60-63: Directory version
    // 64-67: Unknown
    // 68-71: JPEG offset
    // 72-75: JPEG length
    // 76-79: CFA header offset
    // 80-83: CFA header length

    let jpeg_offset = u32::from_be_bytes([data[84], data[85], data[86], data[87]]) as usize;
    let jpeg_length = u32::from_be_bytes([data[88], data[89], data[90], data[91]]) as usize;

    if jpeg_offset == 0 || jpeg_length == 0 {
        return Ok(Vec::new());
    }

    if jpeg_offset + jpeg_length > data.len() {
        return Err(ExifError::Format(
            "RAF JPEG offset/length out of bounds".to_string(),
        ));
    }

    let jpeg_data = data[jpeg_offset..jpeg_offset + jpeg_length].to_vec();

    // Verify JPEG magic
    if jpeg_data.len() < 2 || jpeg_data[0] != 0xFF || jpeg_data[1] != 0xD8 {
        return Ok(Vec::new());
    }

    let result = vec![(
        EmbeddedJpeg {
            offset: jpeg_offset as u64,
            length: jpeg_length as u64,
            description: "RAF Embedded JPEG".to_string(),
            dimensions: None,
        },
        jpeg_data,
    )];

    match jpeg_type {
        JpegType::All | JpegType::Preview => Ok(result),
        JpegType::Thumbnail => Ok(Vec::new()), // RAF doesn't have separate thumbnail
    }
}

/// Extract JPEGs from Minolta MRW files
fn extract_from_mrw<R: Read + Seek>(
    mut reader: R,
    _jpeg_type: JpegType,
) -> ExifResult<Vec<(EmbeddedJpeg, Vec<u8>)>> {
    // MRW files have a TIFF structure after the MRW header
    // Skip to find embedded JPEG in PRD block
    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;

    // MRW structure: blocks until we find TIFF data
    // Each block: 4-byte tag, 4-byte length, data
    let mut offset = 8; // Skip MRM header

    while offset + 8 <= data.len() {
        let block_tag = &data[offset..offset + 4];
        let block_len = u32::from_be_bytes([
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ]) as usize;

        if block_tag == b"\x00TTW" {
            // TIFF block - contains preview
            let tiff_start = offset + 8;
            if tiff_start + block_len <= data.len() {
                let tiff_data = &data[tiff_start..tiff_start + block_len];
                // Parse TIFF to find JPEG
                let cursor = std::io::Cursor::new(tiff_data);
                return extract_from_tiff(cursor, JpegType::All);
            }
        }

        offset += 8 + block_len;
    }

    Ok(Vec::new())
}

/// Extract JPEGs from ISO Base Media Format (CR3, HEIF)
fn extract_from_isobmff<R: Read + Seek>(
    mut reader: R,
    jpeg_type: JpegType,
) -> ExifResult<Vec<(EmbeddedJpeg, Vec<u8>)>> {
    let mut data = Vec::new();
    reader.read_to_end(&mut data)?;

    let mut jpegs = Vec::new();
    let mut offset = 0;

    // Parse boxes looking for preview image
    while offset + 8 <= data.len() {
        let box_size = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        let box_type = &data[offset + 4..offset + 8];

        if box_size == 0 || box_size > data.len() - offset {
            break;
        }

        // Look for prvw (preview) or thmb (thumbnail) boxes
        if box_type == b"prvw" || box_type == b"thmb" {
            // CR3 preview box structure varies, but JPEG data is usually after a header
            // Try to find JPEG magic within the box
            for i in (offset + 8)..(offset + box_size).min(data.len() - 1) {
                if data[i] == 0xFF && data[i + 1] == 0xD8 {
                    // Found JPEG start, find end
                    let jpeg_start = i;
                    let mut jpeg_end = jpeg_start;
                    for j in (jpeg_start + 2)..(offset + box_size).min(data.len() - 1) {
                        if data[j] == 0xFF && data[j + 1] == 0xD9 {
                            jpeg_end = j + 2;
                            break;
                        }
                    }
                    if jpeg_end > jpeg_start {
                        let jpeg_data = data[jpeg_start..jpeg_end].to_vec();
                        let desc = if box_type == b"thmb" {
                            "CR3 Thumbnail"
                        } else {
                            "CR3 Preview"
                        };
                        jpegs.push((
                            EmbeddedJpeg {
                                offset: jpeg_start as u64,
                                length: (jpeg_end - jpeg_start) as u64,
                                description: desc.to_string(),
                                dimensions: None,
                            },
                            jpeg_data,
                        ));
                    }
                    break;
                }
            }
        }

        offset += box_size;
    }

    // Filter based on type
    match jpeg_type {
        JpegType::All => Ok(jpegs),
        JpegType::Preview => Ok(jpegs
            .into_iter()
            .max_by_key(|(info, _)| info.length)
            .into_iter()
            .collect()),
        JpegType::Thumbnail => Ok(jpegs
            .into_iter()
            .min_by_key(|(info, _)| info.length)
            .into_iter()
            .collect()),
    }
}

/// List embedded JPEGs without extracting data
pub fn list_jpegs<R: Read + Seek>(reader: R) -> ExifResult<Vec<EmbeddedJpeg>> {
    let jpegs = extract_jpegs(reader, JpegType::All)?;
    Ok(jpegs.into_iter().map(|(info, _)| info).collect())
}
