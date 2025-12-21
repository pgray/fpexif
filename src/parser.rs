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
pub fn parse_exif_with_config<R>(reader: R, config: ParseConfig) -> ExifResult<crate::ExifData>
where
    R: Read + Seek,
{
    // Initialize an empty EXIF data container
    let mut exif_data = crate::ExifData::new();

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

    // Parse SubIFDs pointed to by SubIFDs tag (0x014A)
    // These contain RAW image data and metadata in some formats (e.g., Nikon TIFF)
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
                    // Add the SubIFD tags
                    for (tag_id, value) in subifd_tags {
                        exif_data.tags.insert(tag_id, value);
                    }
                }
            }
        }
    }

    // Parse thumbnail IFD (IFD1) if present
    // Use lenient parsing - some files have corrupt thumbnail IFD offsets
    if next_ifd_offset > 0 && tiff_offset + next_ifd_offset as usize + 2 <= app1_data.len() {
        if let Ok((thumbnail_tags, _)) = parse_ifd(
            &app1_data,
            tiff_offset + next_ifd_offset as usize,
            tiff_offset,
            endian,
            TagGroup::Thumbnail,
            config,
        ) {
            // Add the thumbnail tags
            for (tag_id, value) in thumbnail_tags {
                exif_data.tags.insert(tag_id, value);
            }
        }
    }

    // Parse maker notes if present (tag 0x927C)
    if let Some(ExifValue::Undefined(maker_note_data)) = exif_data.get_tag_by_id(0x927C) {
        // Get the camera make to determine which parser to use
        let make = exif_data
            .get_tag_by_id(0x010F) // Make tag
            .and_then(|v| match v {
                ExifValue::Ascii(s) => Some(s.as_str()),
                _ => None,
            });

        // Parse the maker notes
        // Pass the full app1_data and tiff_offset for manufacturers that use TIFF-relative offsets (e.g., Canon)
        if let Ok(parsed_maker_notes) = crate::makernotes::parse_maker_notes_with_tiff_data(
            maker_note_data,
            make,
            endian,
            Some(&app1_data),
            tiff_offset,
        ) {
            if !parsed_maker_notes.is_empty() {
                exif_data.maker_notes = Some(parsed_maker_notes);
            }
        }
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
