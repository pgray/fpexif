// parser.rs - EXIF raw binary parsing implementation
use crate::data_types::{Endianness, ExifValue};
use crate::errors::{ExifError, ExifResult};
use crate::formats;
use crate::tags::ExifTagId;
use crate::tags::TagGroup;
use std::collections::HashMap;
use std::io::{Read, Seek};

/// Configuration for EXIF parsing
#[derive(Debug, Clone, Copy)]
pub struct ParseConfig {
    /// Whether to fail on parse errors or continue
    pub strict: bool,
    /// Whether to print verbose debug output
    pub verbose: bool,
}

impl ParseConfig {
    /// Create a new parse configuration with default settings
    pub fn new() -> Self {
        Self {
            strict: true,
            verbose: false,
        }
    }

    /// Create a strict parsing configuration
    pub fn strict() -> Self {
        Self {
            strict: true,
            verbose: false,
        }
    }

    /// Create a lenient parsing configuration
    pub fn lenient() -> Self {
        Self {
            strict: false,
            verbose: false,
        }
    }
}

impl Default for ParseConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse EXIF data from a reader with boolean parameters (deprecated, use parse_exif_with_config)
pub fn parse_exif<R>(reader: R, strict: bool, verbose: bool) -> ExifResult<crate::ExifData>
where
    R: Read + Seek,
{
    let config = ParseConfig { strict, verbose };
    parse_exif_with_config(reader, config)
}

/// Parse EXIF data from a reader with configuration
pub fn parse_exif_with_config<R>(mut reader: R, config: ParseConfig) -> ExifResult<crate::ExifData>
where
    R: Read + Seek,
{
    // Initialize an empty EXIF data container
    let mut exif_data = crate::ExifData::new();

    // First, check for RAF-specific metadata (for Fujifilm RAF files)
    if let Ok(Some(raf_metadata)) = formats::extract_raf_metadata_if_raf(&mut reader) {
        exif_data.set_raf_metadata(raf_metadata);
    }

    // Reset reader position for MRW metadata extraction
    reader.seek(std::io::SeekFrom::Start(0))?;

    // Check for MRW-specific metadata (RIF block for Minolta RAW files)
    if let Ok(Some(mrw_metadata)) = formats::extract_mrw_metadata_if_mrw(&mut reader) {
        exif_data.set_mrw_metadata(mrw_metadata);
    }

    // Reset reader position for RW2 Compression extraction
    reader.seek(std::io::SeekFrom::Start(0))?;

    // For Panasonic RW2 files, extract Compression from main IFD0 before EXIF extraction
    // (RW2 files use embedded JPEG for EXIF which loses the main Compression value)
    let rw2_compression = formats::tiff::extract_rw2_compression(&mut reader)?;

    // Reset reader position for EXIF extraction
    reader.seek(std::io::SeekFrom::Start(0))?;

    // Extract EXIF APP1 segment using format-specific handlers
    let app1_data = formats::extract_exif_segment(reader)?;

    // Parse the TIFF header, which starts after the Exif marker
    let tiff_offset = 6;
    if app1_data.len() < tiff_offset + 8 {
        return Err(ExifError::Format("EXIF data too short".to_string()));
    }

    // Determine endianness from TIFF header (II = little endian, MM = big endian)
    let endian = match (app1_data[tiff_offset], app1_data[tiff_offset + 1]) {
        (b'I', b'I') => Endianness::Little,
        (b'M', b'M') => Endianness::Big,
        _ => return Err(ExifError::Format("Invalid TIFF header".to_string())),
    };

    // Store the endianness in the result
    exif_data.endian = endian;

    // Read TIFF header version
    // Accept standard TIFF (0x002A), BigTIFF (0x002B), ORF (0x4F52), SRW (0x5352), RW2 (0x0055)
    let tiff_version = read_u16(&app1_data[tiff_offset + 2..tiff_offset + 4], endian);
    if tiff_version != 0x002A
        && tiff_version != 0x002B
        && tiff_version != 0x4F52
        && tiff_version != 0x5352
        && tiff_version != 0x0055
    {
        return Err(ExifError::Format(format!(
            "Invalid TIFF version: 0x{:04X}",
            tiff_version
        )));
    }

    // Read offset to first IFD (Image File Directory)
    let ifd_offset = read_u32(&app1_data[tiff_offset + 4..tiff_offset + 8], endian) as usize;
    if tiff_offset + ifd_offset + 2 > app1_data.len() {
        return Err(ExifError::Format("Invalid IFD offset".to_string()));
    }

    // Parse the main IFD (IFD0)
    let (tags, next_ifd_offset) = parse_ifd(
        &app1_data,
        tiff_offset + ifd_offset,
        tiff_offset,
        endian,
        TagGroup::Main,
        config,
    )?;

    // Add the parsed tags to our result
    for (tag_id, value) in tags {
        exif_data.tags.insert(tag_id, value);
    }

    // Helper closure to parse and add SubIFD tags (lenient - ignores errors)
    let mut parse_subifd = |pointer_tag_id: u16, tag_group: TagGroup| {
        if let Some(ExifValue::Long(offsets)) = exif_data.get_tag_by_id(pointer_tag_id) {
            if !offsets.is_empty() {
                let subifd_offset = offsets[0] as usize;
                if let Ok((subifd_tags, _)) = parse_ifd(
                    &app1_data,
                    tiff_offset + subifd_offset,
                    tiff_offset,
                    endian,
                    tag_group,
                    config,
                ) {
                    // Add the SubIFD tags
                    for (tag_id, value) in subifd_tags {
                        exif_data.tags.insert(tag_id, value);
                    }
                }
            }
        }
    };

    // Parse SubIFDs: EXIF, GPS, and Interoperability
    // These use lenient parsing to handle corrupt offsets gracefully
    parse_subifd(0x8769, TagGroup::Exif); // EXIF SubIFD
    parse_subifd(0x8825, TagGroup::Gps); // GPS SubIFD
    parse_subifd(0xA005, TagGroup::Interop); // Interoperability SubIFD

    // Follow the entire IFD chain (IFD1 → IFD2 → IFD3 → ...)
    // This is important for Canon CR2 files where IFD3 contains raw data metadata
    // Only override specific tags that should come from raw data IFD:
    // StripOffsets, StripByteCounts, RowsPerStrip, SamplesPerPixel, etc.
    // Use lenient parsing - some files have corrupt IFD offsets
    let mut current_ifd_offset = next_ifd_offset;
    let mut ifd_count = 0;
    const MAX_IFD_CHAIN: u32 = 10; // Prevent infinite loops

    // Tags that should be taken from the last (raw data) IFD
    const RAW_DATA_TAGS: [u16; 7] = [
        0x0111, // StripOffsets
        0x0117, // StripByteCounts
        0x0116, // RowsPerStrip
        0x0115, // SamplesPerPixel
        0x011C, // PlanarConfiguration
        0x0106, // PhotometricInterpretation
        0x0103, // Compression (for raw data IFD)
    ];

    // Helper to check if a compression value is RAW-specific
    let is_raw_compression = |v: u16| {
        matches!(
            v,
            34713   // Nikon NEF Compressed
                | 32767 // Sony ARW Compressed
                | 34316 // Panasonic RAW 1
                | 34826 // Panasonic RAW 2
                | 34828 // Panasonic RAW 3
                | 34830 // Panasonic RAW 4
                | 65535 // Pentax PEF Compressed
                | 65000 // Kodak DCR Compressed
        )
    };

    while current_ifd_offset > 0
        && ifd_count < MAX_IFD_CHAIN
        && tiff_offset + current_ifd_offset as usize + 2 <= app1_data.len()
    {
        if let Ok((ifd_tags, next_offset)) = parse_ifd(
            &app1_data,
            tiff_offset + current_ifd_offset as usize,
            tiff_offset,
            endian,
            TagGroup::Main,
            config,
        ) {
            // Only override specific raw-data tags, skip others
            for (tag_id, value) in ifd_tags {
                if RAW_DATA_TAGS.contains(&tag_id.id) {
                    // Special handling for Compression (0x0103):
                    // IFD0 contains the main image compression. Later IFDs (IFD1, IFD2, etc.)
                    // typically contain thumbnails with different compression.
                    // Only allow RAW-specific compression values from later IFDs to override.
                    if tag_id.id == 0x0103 {
                        // Check if we already have a compression value from IFD0
                        let has_existing = exif_data.get_tag_by_id(0x0103).is_some();

                        // Check if new value is RAW-specific (should always override)
                        let new_is_raw = if let ExifValue::Short(vals) = &value {
                            !vals.is_empty() && is_raw_compression(vals[0])
                        } else {
                            false
                        };

                        // Only allow override if:
                        // - we don't have an existing value, OR
                        // - new value is RAW-specific
                        if has_existing && !new_is_raw {
                            continue;
                        }
                    }
                    exif_data.tags.insert(tag_id, value);
                }
            }
            current_ifd_offset = next_offset;
        } else {
            break;
        }
        ifd_count += 1;
    }

    // Parse SubIFDs pointed to by SubIFDs tag (0x014A)
    // These contain RAW image data and metadata in some formats (e.g., Canon CR2, Nikon NEF)
    // Parse AFTER IFD1 so raw data values override thumbnail values
    let subifd_offsets = exif_data.get_tag_by_id(0x014A).and_then(|v| match v {
        ExifValue::Long(offsets) => Some(offsets.clone()),
        _ => None,
    });
    if let Some(offsets) = subifd_offsets {
        for offset in offsets {
            if offset > 0 {
                let subifd_offset = offset as usize;
                if let Ok((subifd_tags, _)) = parse_ifd(
                    &app1_data,
                    tiff_offset + subifd_offset,
                    tiff_offset,
                    endian,
                    TagGroup::Main,
                    config,
                ) {
                    // Check if this SubIFD is a main image (SubfileType = 0)
                    // SubfileType 0 = Full-resolution image, 1 = Reduced-resolution (thumbnail)
                    let is_main_image = subifd_tags.iter().any(|(tid, val)| {
                        tid.id == 0x00FE
                            && matches!(val, ExifValue::Long(v) if !v.is_empty() && v[0] == 0)
                    });

                    // Add the SubIFD tags
                    for (tag_id, value) in subifd_tags {
                        // Special handling for Compression tag (0x0103):
                        // Prefer main image (SubfileType=0) Compression over thumbnail
                        if tag_id.id == 0x0103 {
                            // Helper to check if a compression value is RAW-specific
                            let is_raw_compression = |v: u16| {
                                matches!(
                                    v,
                                    34713   // Nikon NEF Compressed
                                        | 32767 // Sony ARW Compressed
                                        | 34316 // Panasonic RAW 1
                                        | 34826 // Panasonic RAW 2
                                        | 34828 // Panasonic RAW 3
                                        | 34830 // Panasonic RAW 4
                                        | 65535 // Pentax PEF Compressed
                                        | 65000 // Kodak DCR Compressed
                                )
                            };

                            // Check if new value is RAW-specific
                            let new_is_raw = if let ExifValue::Short(vals) = &value {
                                !vals.is_empty() && is_raw_compression(vals[0])
                            } else {
                                false
                            };

                            // Check if IFD0 already has JPEG compression (7)
                            // JPEG in IFD0 is typically the preview that ExifTool reports
                            let existing_is_jpeg = if let Some(ExifValue::Short(vals)) =
                                exif_data.get_tag_by_id(0x0103)
                            {
                                !vals.is_empty() && vals[0] == 7
                            } else {
                                false
                            };

                            // Allow override if: RAW-specific, or this is a main image SubIFD
                            // BUT: don't overwrite JPEG with Uncompressed (ExifTool prefers JPEG)
                            if !new_is_raw
                                && !is_main_image
                                && exif_data.get_tag_by_id(0x0103).is_some()
                            {
                                continue;
                            }

                            // Don't overwrite JPEG (7) with Uncompressed (1) - ExifTool prefers JPEG
                            if existing_is_jpeg && !new_is_raw {
                                continue;
                            }
                        }

                        // For non-main SubIFDs (SubfileType != 0), skip dimension-related tags
                        // to prevent thumbnail/preview dimensions from overwriting main image
                        if !is_main_image {
                            let is_dimension_tag = matches!(
                                tag_id.id,
                                0x0100  // ImageWidth
                                    | 0x0101 // ImageHeight
                                    | 0x0102 // BitsPerSample
                                    | 0x0106 // PhotometricInterpretation
                                    | 0x0111 // StripOffsets
                                    | 0x0115 // SamplesPerPixel
                                    | 0x0116 // RowsPerStrip
                                    | 0x0117 // StripByteCounts
                            );
                            if is_dimension_tag && exif_data.get_tag_by_id(tag_id.id).is_some() {
                                continue;
                            }
                        }

                        exif_data.tags.insert(tag_id, value);
                    }
                }
            }
        }
    }

    // Get the camera make to determine which parser to use
    // Clone to avoid borrowing exif_data while we modify it
    let make: Option<String> = exif_data
        .get_tag_by_id(0x010F) // Make tag
        .and_then(|v| match v {
            ExifValue::Ascii(s) => Some(s.clone()),
            _ => None,
        });

    // Get the camera model for model-specific formatting (e.g., Canon SerialNumber)
    let model: Option<String> = exif_data
        .get_tag_by_id(0x0110) // Model tag
        .and_then(|v| match v {
            ExifValue::Ascii(s) => Some(s.clone()),
            _ => None,
        });

    // Parse maker notes if present (tag 0x927C - standard EXIF MakerNote)
    if let Some(ExifValue::Undefined(maker_note_data)) = exif_data.get_tag_by_id(0x927C) {
        // Find the MakerNote's file offset for manufacturers that need it (e.g., Olympus PreviewImageStart)
        let makernote_file_offset =
            if let Some(ExifValue::Long(exif_offsets)) = exif_data.get_tag_by_id(0x8769) {
                // Get the EXIF SubIFD offset
                if !exif_offsets.is_empty() {
                    let exif_ifd_offset = tiff_offset + exif_offsets[0] as usize;
                    // Find MakerNote (0x927C) data offset within EXIF SubIFD
                    find_tag_data_offset(&app1_data, exif_ifd_offset, tiff_offset, 0x927C, endian)
                } else {
                    None
                }
            } else {
                None
            };

        // Parse the maker notes
        // Pass the full app1_data and tiff_offset for manufacturers that use TIFF-relative offsets (e.g., Canon)
        if let Ok(parsed_maker_notes) = crate::makernotes::parse_maker_notes_with_tiff_data(
            maker_note_data,
            make.as_deref(),
            model.as_deref(),
            endian,
            Some(&app1_data),
            tiff_offset,
            makernote_file_offset,
        ) {
            if !parsed_maker_notes.is_empty() {
                exif_data.maker_notes = Some(parsed_maker_notes);
            }
        }
    }

    // Also check for DNG's DNGPrivateData tag (0xC634) which can contain MakerNotes
    // This is used by Pentax/Samsung/Ricoh DNG files to store original MakerNotes
    if exif_data.maker_notes.is_none() {
        if let Some(private_data) = exif_data.get_tag_by_id(0xC634) {
            let private_bytes = match private_data {
                ExifValue::Undefined(b) => Some(b.as_slice()),
                ExifValue::Byte(b) => Some(b.as_slice()),
                _ => None,
            };

            if let Some(data) = private_bytes {
                // Check for Pentax/Samsung format: "PENTAX \0" or "SAMSUNG\0" at start
                if data.len() > 10
                    && (data.starts_with(b"PENTAX \0") || data.starts_with(b"SAMSUNG\0"))
                {
                    // Byte order at offset 8-9: "MM" or "II"
                    let mn_endian = if &data[8..10] == b"MM" {
                        Endianness::Big
                    } else {
                        Endianness::Little
                    };

                    // MakerNote IFD starts at offset 10
                    let mn_data = &data[10..];
                    if let Ok(parsed) = crate::makernotes::parse_maker_notes_with_tiff_data(
                        mn_data,
                        make.as_deref(),
                        model.as_deref(),
                        mn_endian,
                        Some(data), // Use the private data as the base for relative offsets
                        10,         // Offset to IFD within the private data
                        None,       // DNG doesn't need makernote_file_offset
                    ) {
                        if !parsed.is_empty() {
                            exif_data.maker_notes = Some(parsed);
                        }
                    }
                }
                // Check for Ricoh format: "RICOH\0II" or "RICOH\0MM"
                else if data.len() > 8 && data.starts_with(b"RICOH\0") {
                    let mn_endian = if &data[6..8] == b"MM" {
                        Endianness::Big
                    } else {
                        Endianness::Little
                    };

                    // MakerNote IFD starts at offset 8
                    let mn_data = &data[8..];
                    if let Ok(parsed) = crate::makernotes::parse_maker_notes_with_tiff_data(
                        mn_data,
                        make.as_deref(),
                        model.as_deref(),
                        mn_endian,
                        Some(data),
                        8,
                        None, // DNG doesn't need makernote_file_offset
                    ) {
                        if !parsed.is_empty() {
                            exif_data.maker_notes = Some(parsed);
                        }
                    }
                }
            }
        }
    }

    // Override Compression for RW2 files with the value from main IFD0
    if let Some(compression) = rw2_compression {
        let tag_id = crate::tags::ExifTagId::new(0x0103, crate::tags::TagGroup::Main);
        exif_data
            .tags
            .insert(tag_id, ExifValue::Short(vec![compression]));
    }

    Ok(exif_data)
}

/// Parse an Image File Directory (IFD)
fn parse_ifd(
    data: &[u8],
    offset: usize,
    base_offset: usize,
    endian: Endianness,
    ifd_type: TagGroup,
    config: ParseConfig,
) -> ExifResult<(HashMap<ExifTagId, ExifValue>, u32)> {
    // Get number of entries in this IFD
    if offset + 2 > data.len() {
        return Err(ExifError::Format("Invalid IFD offset".to_string()));
    }

    let entry_count = read_u16(&data[offset..offset + 2], endian) as usize;
    if config.verbose {
        println!("IFD has {} entries", entry_count);
    }

    // Sanity check: a typical IFD has at most a few hundred entries
    // Values above 1000 usually indicate corrupt data or wrong offset
    if entry_count > 1000 {
        return Err(ExifError::Format(format!(
            "IFD entry count too large ({}), likely corrupt data",
            entry_count
        )));
    }

    // Calculate the size of the entire IFD
    let ifd_size = 2 + entry_count * 12 + 4;
    if offset + ifd_size > data.len() {
        return Err(ExifError::Format(format!(
            "IFD extends beyond data length (offset: {}, size: {}, data length: {})",
            offset,
            ifd_size,
            data.len()
        )));
    }

    let mut tags = HashMap::new();

    // Parse each IFD entry (12 bytes each)
    for i in 0..entry_count {
        let entry_offset = offset + 2 + i * 12;

        // Parse the IFD entry
        let tag_id = read_u16(&data[entry_offset..entry_offset + 2], endian);
        let tag_type = read_u16(&data[entry_offset + 2..entry_offset + 4], endian);
        let count = read_u32(&data[entry_offset + 4..entry_offset + 8], endian);
        let value_offset = &data[entry_offset + 8..entry_offset + 12];

        // Create an ExifTagId with the appropriate IFD type
        let exif_tag_id = ExifTagId::new(tag_id, ifd_type);

        // Parse the value based on the type and count
        match parse_tag_value(data, tag_type, count, value_offset, base_offset, endian) {
            Ok(value) => {
                tags.insert(exif_tag_id, value);
            }
            Err(err) => {
                if config.verbose {
                    println!("Error parsing tag 0x{:04X}: {}", tag_id, err);
                }
                // If in strict mode, propagate the error
                if config.strict {
                    return Err(err);
                }
                // Otherwise, continue with the next tag
                continue;
            }
        }
    }

    // Read next IFD offset (4 bytes after the entries)
    let next_offset = read_u32(
        &data[offset + 2 + entry_count * 12..offset + 2 + entry_count * 12 + 4],
        endian,
    );

    Ok((tags, next_offset))
}

/// Parse a tag value based on its type and count
fn parse_tag_value(
    data: &[u8],
    tag_type: u16,
    count: u32,
    value_offset: &[u8],
    base_offset: usize,
    endian: Endianness,
) -> ExifResult<ExifValue> {
    match tag_type {
        1 => {
            // BYTE (8-bit unsigned integer)
            parse_byte_array(data, count, value_offset, base_offset, endian)
        }
        2 => {
            // ASCII (null-terminated string)
            parse_ascii(data, count, value_offset, base_offset, endian)
        }
        3 => {
            // SHORT (16-bit unsigned integer)
            parse_short_array(data, count, value_offset, base_offset, endian)
        }
        4 => {
            // LONG (32-bit unsigned integer)
            parse_long_array(data, count, value_offset, base_offset, endian)
        }
        5 => {
            // RATIONAL (two 32-bit unsigned integers: numerator/denominator)
            parse_rational_array(data, count, value_offset, base_offset, endian)
        }
        6 => {
            // SBYTE (8-bit signed integer)
            parse_sbyte_array(data, count, value_offset, base_offset, endian)
        }
        7 => {
            // UNDEFINED (8-bit byte with no interpretation)
            parse_undefined_array(data, count, value_offset, base_offset, endian)
        }
        8 => {
            // SSHORT (16-bit signed integer)
            parse_sshort_array(data, count, value_offset, base_offset, endian)
        }
        9 => {
            // SLONG (32-bit signed integer)
            parse_slong_array(data, count, value_offset, base_offset, endian)
        }
        10 => {
            // SRATIONAL (two 32-bit signed integers: numerator/denominator)
            parse_srational_array(data, count, value_offset, base_offset, endian)
        }
        11 => {
            // FLOAT (32-bit IEEE floating point)
            parse_float_array(data, count, value_offset, base_offset, endian)
        }
        12 => {
            // DOUBLE (64-bit IEEE floating point)
            parse_double_array(data, count, value_offset, base_offset, endian)
        }
        13 => {
            // IFD (32-bit unsigned integer, same as LONG) - used for SubIFD pointers
            parse_long_array(data, count, value_offset, base_offset, endian)
        }
        _ => {
            // Unknown tag type (including BigTIFF types 16-18 and vendor-specific types)
            // For unknown types, we don't know the element size, so we can't safely
            // calculate the actual byte count. Store only the inline value (4 bytes max)
            // or return empty if the value is at an offset.
            parse_unknown_type(value_offset)
        }
    }
}

// Helper functions for reading values with the correct endianness
fn read_u16(data: &[u8], endian: Endianness) -> u16 {
    let bytes = [data[0], data[1]];
    match endian {
        Endianness::Little => u16::from_le_bytes(bytes),
        Endianness::Big => u16::from_be_bytes(bytes),
    }
}

fn read_u32(data: &[u8], endian: Endianness) -> u32 {
    let bytes = [data[0], data[1], data[2], data[3]];
    match endian {
        Endianness::Little => u32::from_le_bytes(bytes),
        Endianness::Big => u32::from_be_bytes(bytes),
    }
}

fn read_i16(data: &[u8], endian: Endianness) -> i16 {
    let bytes = [data[0], data[1]];
    match endian {
        Endianness::Little => i16::from_le_bytes(bytes),
        Endianness::Big => i16::from_be_bytes(bytes),
    }
}

fn read_i32(data: &[u8], endian: Endianness) -> i32 {
    let bytes = [data[0], data[1], data[2], data[3]];
    match endian {
        Endianness::Little => i32::from_le_bytes(bytes),
        Endianness::Big => i32::from_be_bytes(bytes),
    }
}

fn read_f32(data: &[u8], endian: Endianness) -> f32 {
    let bytes = [data[0], data[1], data[2], data[3]];
    match endian {
        Endianness::Little => f32::from_le_bytes(bytes),
        Endianness::Big => f32::from_be_bytes(bytes),
    }
}

fn read_f64(data: &[u8], endian: Endianness) -> f64 {
    let bytes = [
        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
    ];
    match endian {
        Endianness::Little => f64::from_le_bytes(bytes),
        Endianness::Big => f64::from_be_bytes(bytes),
    }
}

// Parse functions for each data type
fn parse_byte_array(
    data: &[u8],
    count: u32,
    value_offset: &[u8],
    base_offset: usize,
    endian: Endianness,
) -> ExifResult<ExifValue> {
    let count = count as usize;
    let mut values = Vec::with_capacity(count);

    // If count <= 4, the values are stored directly in the value_offset field
    if count <= 4 {
        values.extend_from_slice(&value_offset[0..count]);
    } else {
        // Otherwise, value_offset contains an offset to the values
        let offset = read_u32(value_offset, endian) as usize + base_offset;
        if offset + count > data.len() {
            return Err(ExifError::Format(
                "Byte array extends beyond data".to_string(),
            ));
        }
        values.extend_from_slice(&data[offset..offset + count]);
    }

    Ok(ExifValue::Byte(values))
}

fn parse_ascii(
    data: &[u8],
    count: u32,
    value_offset: &[u8],
    base_offset: usize,
    endian: Endianness,
) -> ExifResult<ExifValue> {
    let count = count as usize;
    let mut ascii_data: Vec<u8>;

    // If count <= 4, the ASCII string is stored directly in the value_offset field
    if count <= 4 {
        ascii_data = value_offset[0..count].to_vec();
    } else {
        // Otherwise, value_offset contains an offset to the ASCII string
        let offset = read_u32(value_offset, endian) as usize + base_offset;
        if offset + count > data.len() {
            return Err(ExifError::Format(
                "ASCII string extends beyond data".to_string(),
            ));
        }
        ascii_data = data[offset..offset + count].to_vec();
    }

    // ASCII strings are null-terminated, so remove the trailing null byte if present
    if !ascii_data.is_empty() && ascii_data[ascii_data.len() - 1] == 0 {
        ascii_data.pop();
    }

    // Convert the ASCII data to a UTF-8 string, replacing invalid characters
    let string = String::from_utf8_lossy(&ascii_data).into_owned();
    Ok(ExifValue::Ascii(string))
}

fn parse_short_array(
    data: &[u8],
    count: u32,
    value_offset: &[u8],
    base_offset: usize,
    endian: Endianness,
) -> ExifResult<ExifValue> {
    let count = count as usize;
    let mut values = Vec::with_capacity(count);

    // If count <= 2, the values are stored directly in the value_offset field
    if count <= 2 {
        for i in 0..count {
            let value = read_u16(&value_offset[i * 2..(i + 1) * 2], endian);
            values.push(value);
        }
    } else {
        // Otherwise, value_offset contains an offset to the values
        let offset = read_u32(value_offset, endian) as usize + base_offset;
        if offset + count * 2 > data.len() {
            return Err(ExifError::Format(
                "Short array extends beyond data".to_string(),
            ));
        }
        for i in 0..count {
            let value = read_u16(&data[offset + i * 2..offset + (i + 1) * 2], endian);
            values.push(value);
        }
    }

    Ok(ExifValue::Short(values))
}

fn parse_long_array(
    data: &[u8],
    count: u32,
    value_offset: &[u8],
    base_offset: usize,
    endian: Endianness,
) -> ExifResult<ExifValue> {
    let count = count as usize;
    let mut values = Vec::with_capacity(count);

    // If count == 1, the value is stored directly in the value_offset field
    if count == 1 {
        let value = read_u32(value_offset, endian);
        values.push(value);
    } else {
        // Otherwise, value_offset contains an offset to the values
        let offset = read_u32(value_offset, endian) as usize + base_offset;
        if offset + count * 4 > data.len() {
            return Err(ExifError::Format(
                "Long array extends beyond data".to_string(),
            ));
        }
        for i in 0..count {
            let value = read_u32(&data[offset + i * 4..offset + (i + 1) * 4], endian);
            values.push(value);
        }
    }

    Ok(ExifValue::Long(values))
}

fn parse_rational_array(
    data: &[u8],
    count: u32,
    value_offset: &[u8],
    base_offset: usize,
    endian: Endianness,
) -> ExifResult<ExifValue> {
    let count = count as usize;
    let mut values = Vec::with_capacity(count);

    // Rational values are always stored at the offset, never inline
    let offset = read_u32(value_offset, endian) as usize + base_offset;
    if offset + count * 8 > data.len() {
        return Err(ExifError::Format(
            "Rational array extends beyond data".to_string(),
        ));
    }

    for i in 0..count {
        let numerator = read_u32(&data[offset + i * 8..offset + i * 8 + 4], endian);
        let denominator = read_u32(&data[offset + i * 8 + 4..offset + i * 8 + 8], endian);
        values.push((numerator, denominator));
    }

    Ok(ExifValue::Rational(values))
}

fn parse_sbyte_array(
    data: &[u8],
    count: u32,
    value_offset: &[u8],
    base_offset: usize,
    endian: Endianness,
) -> ExifResult<ExifValue> {
    let count = count as usize;
    let mut values = Vec::with_capacity(count);

    // If count <= 4, the values are stored directly in the value_offset field
    if count <= 4 {
        values.extend(value_offset[0..count].iter().map(|&b| b as i8));
    } else {
        // Otherwise, value_offset contains an offset to the values
        let offset = read_u32(value_offset, endian) as usize + base_offset;
        if offset + count > data.len() {
            return Err(ExifError::Format(
                "SByte array extends beyond data".to_string(),
            ));
        }
        for i in 0..count {
            values.push(data[offset + i] as i8);
        }
    }

    Ok(ExifValue::SByte(values))
}

fn parse_undefined_array(
    data: &[u8],
    count: u32,
    value_offset: &[u8],
    base_offset: usize,
    endian: Endianness,
) -> ExifResult<ExifValue> {
    // Undefined type is treated like BYTE array
    let result = parse_byte_array(data, count, value_offset, base_offset, endian)?;
    match result {
        ExifValue::Byte(bytes) => Ok(ExifValue::Undefined(bytes)),
        _ => Err(ExifError::Format(
            "Failed to parse UNDEFINED type".to_string(),
        )),
    }
}

/// Parse an unknown tag type by storing only the inline 4-byte value
/// We don't attempt to follow offsets since we don't know the element size
fn parse_unknown_type(value_offset: &[u8]) -> ExifResult<ExifValue> {
    // Store the raw 4 bytes from the value field
    // For unknown types, this is the safest approach since we can't
    // determine the actual data size or location
    Ok(ExifValue::Undefined(value_offset.to_vec()))
}

fn parse_sshort_array(
    data: &[u8],
    count: u32,
    value_offset: &[u8],
    base_offset: usize,
    endian: Endianness,
) -> ExifResult<ExifValue> {
    let count = count as usize;
    let mut values = Vec::with_capacity(count);

    // If count <= 2, the values are stored directly in the value_offset field
    if count <= 2 {
        for i in 0..count {
            let value = read_i16(&value_offset[i * 2..(i + 1) * 2], endian);
            values.push(value);
        }
    } else {
        // Otherwise, value_offset contains an offset to the values
        let offset = read_u32(value_offset, endian) as usize + base_offset;
        if offset + count * 2 > data.len() {
            return Err(ExifError::Format(
                "SShort array extends beyond data".to_string(),
            ));
        }
        for i in 0..count {
            let value = read_i16(&data[offset + i * 2..offset + (i + 1) * 2], endian);
            values.push(value);
        }
    }

    Ok(ExifValue::SShort(values))
}

fn parse_slong_array(
    data: &[u8],
    count: u32,
    value_offset: &[u8],
    base_offset: usize,
    endian: Endianness,
) -> ExifResult<ExifValue> {
    let count = count as usize;
    let mut values = Vec::with_capacity(count);

    // If count == 1, the value is stored directly in the value_offset field
    if count == 1 {
        let value = read_i32(value_offset, endian);
        values.push(value);
    } else {
        // Otherwise, value_offset contains an offset to the values
        let offset = read_u32(value_offset, endian) as usize + base_offset;
        if offset + count * 4 > data.len() {
            return Err(ExifError::Format(
                "SLong array extends beyond data".to_string(),
            ));
        }
        for i in 0..count {
            let value = read_i32(&data[offset + i * 4..offset + (i + 1) * 4], endian);
            values.push(value);
        }
    }

    Ok(ExifValue::SLong(values))
}

fn parse_srational_array(
    data: &[u8],
    count: u32,
    value_offset: &[u8],
    base_offset: usize,
    endian: Endianness,
) -> ExifResult<ExifValue> {
    let count = count as usize;
    let mut values = Vec::with_capacity(count);

    // SRational values are always stored at the offset, never inline
    let offset = read_u32(value_offset, endian) as usize + base_offset;
    if offset + count * 8 > data.len() {
        return Err(ExifError::Format(
            "SRational array extends beyond data".to_string(),
        ));
    }

    for i in 0..count {
        let numerator = read_i32(&data[offset + i * 8..offset + i * 8 + 4], endian);
        let denominator = read_i32(&data[offset + i * 8 + 4..offset + i * 8 + 8], endian);
        values.push((numerator, denominator));
    }

    Ok(ExifValue::SRational(values))
}

fn parse_float_array(
    data: &[u8],
    count: u32,
    value_offset: &[u8],
    base_offset: usize,
    endian: Endianness,
) -> ExifResult<ExifValue> {
    let count = count as usize;
    let mut values = Vec::with_capacity(count);

    // If count == 1, the value is stored directly in the value_offset field
    if count == 1 {
        let value = read_f32(value_offset, endian);
        values.push(value);
    } else {
        // Otherwise, value_offset contains an offset to the values
        let offset = read_u32(value_offset, endian) as usize + base_offset;
        if offset + count * 4 > data.len() {
            return Err(ExifError::Format(
                "Float array extends beyond data".to_string(),
            ));
        }
        for i in 0..count {
            let value = read_f32(&data[offset + i * 4..offset + (i + 1) * 4], endian);
            values.push(value);
        }
    }

    Ok(ExifValue::Float(values))
}

fn parse_double_array(
    data: &[u8],
    count: u32,
    value_offset: &[u8],
    base_offset: usize,
    endian: Endianness,
) -> ExifResult<ExifValue> {
    let count = count as usize;
    let mut values = Vec::with_capacity(count);

    // Double values are always stored at the offset, never inline
    let offset = read_u32(value_offset, endian) as usize + base_offset;
    if offset + count * 8 > data.len() {
        return Err(ExifError::Format(
            "Double array extends beyond data".to_string(),
        ));
    }

    for i in 0..count {
        let value = read_f64(&data[offset + i * 8..offset + (i + 1) * 8], endian);
        values.push(value);
    }

    Ok(ExifValue::Double(values))
}

/// Find a tag's data offset within an IFD
/// Returns the absolute offset (from data start) where the tag's data is located
fn find_tag_data_offset(
    data: &[u8],
    ifd_offset: usize,
    base_offset: usize,
    tag_id: u16,
    endian: Endianness,
) -> Option<usize> {
    if ifd_offset + 2 > data.len() {
        return None;
    }

    let entry_count = read_u16(&data[ifd_offset..ifd_offset + 2], endian) as usize;
    if entry_count > 1000 {
        return None;
    }

    for i in 0..entry_count {
        let entry_offset = ifd_offset + 2 + i * 12;
        if entry_offset + 12 > data.len() {
            return None;
        }

        let entry_tag = read_u16(&data[entry_offset..entry_offset + 2], endian);
        if entry_tag == tag_id {
            let tag_type = read_u16(&data[entry_offset + 2..entry_offset + 4], endian);
            let count = read_u32(&data[entry_offset + 4..entry_offset + 8], endian) as usize;
            let value_bytes = &data[entry_offset + 8..entry_offset + 12];

            // Calculate the size of the value based on type
            let type_size = match tag_type {
                1 | 2 | 6 | 7 => 1, // BYTE, ASCII, SBYTE, UNDEFINED
                3 | 8 => 2,         // SHORT, SSHORT
                4 | 9 | 13 => 4,    // LONG, SLONG, IFD
                5 | 10 => 8,        // RATIONAL, SRATIONAL
                11 => 4,            // FLOAT
                12 => 8,            // DOUBLE
                _ => 1,             // Unknown, assume 1
            };

            let total_size = count * type_size;
            if total_size <= 4 {
                // Value is inline - return the entry offset + 8 (where value starts)
                return Some(entry_offset + 8);
            } else {
                // Value is at offset
                let data_offset = read_u32(value_bytes, endian) as usize + base_offset;
                return Some(data_offset);
            }
        }
    }

    None
}
