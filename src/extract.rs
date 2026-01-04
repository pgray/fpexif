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
                _ => {}
            }
        }

        // Check for JPEG data
        let is_jpeg = compression == Some(6) || compression == Some(7);

        // Method 1: JPEGInterchangeFormat (typical for thumbnails)
        if let (Some(offset), Some(length)) = (jpeg_offset, jpeg_length) {
            if length > 0 && offset as usize + length as usize <= data.len() {
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
        }

        // Method 2: StripOffsets with JPEG compression
        if is_jpeg {
            if let (Some(offset), Some(length)) = (strip_offsets, strip_byte_counts) {
                if length > 0 && offset as usize + length as usize <= data.len() {
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
