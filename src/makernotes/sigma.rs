// makernotes/sigma.rs - Sigma/Foveon maker notes parsing

use crate::data_types::{Endianness, ExifValue};
use crate::define_tag_decoder;
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use std::collections::HashMap;

// Sigma MakerNote tag IDs
pub const SIGMA_SERIAL_NUMBER: u16 = 0x0002;
pub const SIGMA_DRIVE_MODE: u16 = 0x0003;
pub const SIGMA_RESOLUTION_MODE: u16 = 0x0004;
pub const SIGMA_AF_MODE: u16 = 0x0005;
pub const SIGMA_FOCUS_SETTING: u16 = 0x0006;
pub const SIGMA_WHITE_BALANCE: u16 = 0x0007;
pub const SIGMA_EXPOSURE_MODE: u16 = 0x0008;
pub const SIGMA_METERING_MODE: u16 = 0x0009;
pub const SIGMA_LENS_FOCAL_RANGE: u16 = 0x000a;
pub const SIGMA_COLOR_SPACE: u16 = 0x000b;
pub const SIGMA_EXPOSURE_COMPENSATION: u16 = 0x000c;
pub const SIGMA_CONTRAST: u16 = 0x000d;
pub const SIGMA_SHADOW: u16 = 0x000e;
pub const SIGMA_HIGHLIGHT: u16 = 0x000f;
pub const SIGMA_SATURATION: u16 = 0x0010;
pub const SIGMA_SHARPNESS: u16 = 0x0011;
pub const SIGMA_X3_FILL_LIGHT: u16 = 0x0012;
pub const SIGMA_COLOR_ADJUSTMENT: u16 = 0x0014;
pub const SIGMA_ADJUSTMENT_MODE: u16 = 0x0015;
pub const SIGMA_QUALITY: u16 = 0x0016;
pub const SIGMA_FIRMWARE: u16 = 0x0017;
pub const SIGMA_SOFTWARE: u16 = 0x0018;
pub const SIGMA_AUTO_BRACKET: u16 = 0x0019;
pub const SIGMA_LENS_TYPE: u16 = 0x0027;
pub const SIGMA_LENS_FOCAL_RANGE_2: u16 = 0x002a;
pub const SIGMA_LENS_MAX_APERTURE_RANGE: u16 = 0x002b;
pub const SIGMA_COLOR_MODE: u16 = 0x002c;
pub const SIGMA_FLASH_EXPOSURE_COMP: u16 = 0x003a;

/// Get human-readable tag name for a Sigma tag ID
pub fn get_sigma_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        SIGMA_SERIAL_NUMBER => Some("SerialNumber"),
        SIGMA_DRIVE_MODE => Some("DriveMode"),
        SIGMA_RESOLUTION_MODE => Some("ResolutionMode"),
        SIGMA_AF_MODE => Some("AFMode"),
        SIGMA_FOCUS_SETTING => Some("FocusSetting"),
        SIGMA_WHITE_BALANCE => Some("WhiteBalance"),
        SIGMA_EXPOSURE_MODE => Some("ExposureMode"),
        SIGMA_METERING_MODE => Some("MeteringMode"),
        SIGMA_LENS_FOCAL_RANGE => Some("LensFocalRange"),
        SIGMA_COLOR_SPACE => Some("ColorSpace"),
        SIGMA_EXPOSURE_COMPENSATION => Some("ExposureCompensation"),
        SIGMA_CONTRAST => Some("Contrast"),
        SIGMA_SHADOW => Some("Shadow"),
        SIGMA_HIGHLIGHT => Some("Highlight"),
        SIGMA_SATURATION => Some("Saturation"),
        SIGMA_SHARPNESS => Some("Sharpness"),
        SIGMA_X3_FILL_LIGHT => Some("X3FillLight"),
        SIGMA_COLOR_ADJUSTMENT => Some("ColorAdjustment"),
        SIGMA_ADJUSTMENT_MODE => Some("AdjustmentMode"),
        SIGMA_QUALITY => Some("Quality"),
        SIGMA_FIRMWARE => Some("Firmware"),
        SIGMA_SOFTWARE => Some("Software"),
        SIGMA_AUTO_BRACKET => Some("AutoBracket"),
        SIGMA_LENS_TYPE => Some("LensType"),
        SIGMA_LENS_FOCAL_RANGE_2 => Some("LensFocalRange"),
        SIGMA_LENS_MAX_APERTURE_RANGE => Some("LensMaxApertureRange"),
        SIGMA_COLOR_MODE => Some("ColorMode"),
        SIGMA_FLASH_EXPOSURE_COMP => Some("FlashExposureComp"),
        _ => None,
    }
}

/// Read u16 from data with endianness
fn read_u16(data: &[u8], endian: Endianness) -> u16 {
    match endian {
        Endianness::Little => u16::from_le_bytes([data[0], data[1]]),
        Endianness::Big => u16::from_be_bytes([data[0], data[1]]),
    }
}

/// Read u32 from data with endianness
fn read_u32(data: &[u8], endian: Endianness) -> u32 {
    match endian {
        Endianness::Little => u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
        Endianness::Big => u32::from_be_bytes([data[0], data[1], data[2], data[3]]),
    }
}

// Decode ExposureMode tag (0x0008)
// ExifTool: PrintConv => { P => 'Program AE', A => 'Aperture-priority AE', S => 'Shutter speed priority AE', M => 'Manual' }
// exiv2: print0x0008 function in sigmamn_int.cpp
define_tag_decoder! {
    exposure_mode_sigma,
    type: u8,
    both: {
        b'P' => "Program AE",
        b'A' => "Aperture-priority AE",
        b'S' => "Shutter speed priority AE",
        b'M' => "Manual",
    }
}

// Decode MeteringMode tag (0x0009)
// ExifTool: PrintConv => { A => 'Average', C => 'Center-weighted average', 8 => 'Multi-segment' }
// exiv2: print0x0009 function in sigmamn_int.cpp
define_tag_decoder! {
    metering_mode_sigma,
    type: u8,
    exiftool: {
        b'A' => "Average",
        b'C' => "Center-weighted average",
        b'8' => "Multi-segment",
    },
    exiv2: {
        b'A' => "Average",
        b'C' => "Center",
        b'8' => "8-Segment",
    }
}

// Decode ColorMode tag (0x002c)
// ExifTool: Sigma.pm tag 0x002c PrintConv
// exiv2: Not found in exiv2 (not decoded there)
define_tag_decoder! {
    color_mode,
    type: u32,
    both: {
        0 => "n/a",
        1 => "Sepia",
        2 => "B&W",
        3 => "Standard",
        4 => "Vivid",
        5 => "Neutral",
        6 => "Portrait",
        7 => "Landscape",
        8 => "FOV Classic Blue",
    }
}

/// Strip label prefix from Sigma tag values
/// Many Sigma tags have format like "Expo:+1.0" or "Cont:0" - we want just the value
fn strip_label(value: &str) -> String {
    if let Some(colon_pos) = value.find(':') {
        let after_colon = &value[colon_pos + 1..];
        // Skip leading space if present
        if let Some(stripped) = after_colon.strip_prefix(' ') {
            stripped.to_string()
        } else {
            after_colon.to_string()
        }
    } else {
        value.to_string()
    }
}

/// Decode exposure mode from ASCII character
fn decode_exposure_mode(value: &str) -> String {
    if value.is_empty() {
        return value.to_string();
    }
    let first_char = value.as_bytes()[0];
    decode_exposure_mode_sigma_exiftool(first_char).to_string()
}

/// Decode metering mode from ASCII character
fn decode_metering_mode(value: &str) -> String {
    if value.is_empty() {
        return value.to_string();
    }
    let first_char = value.as_bytes()[0];
    decode_metering_mode_sigma_exiftool(first_char).to_string()
}

/// Parse a single IFD entry and return tag ID and value
fn parse_ifd_entry(
    data: &[u8],
    entry_offset: usize,
    endian: Endianness,
) -> Option<(u16, ExifValue)> {
    if entry_offset + 12 > data.len() {
        return None;
    }

    let tag_id = read_u16(&data[entry_offset..], endian);
    let tag_type = read_u16(&data[entry_offset + 2..], endian);
    let count = read_u32(&data[entry_offset + 4..], endian) as usize;
    let value_offset_bytes = &data[entry_offset + 8..entry_offset + 12];

    // Calculate the size of the data
    let type_size = match tag_type {
        1 | 2 | 6 | 7 => 1, // BYTE, ASCII, SBYTE, UNDEFINED
        3 | 8 => 2,         // SHORT, SSHORT
        4 | 9 | 11 => 4,    // LONG, SLONG, FLOAT
        5 | 10 | 12 => 8,   // RATIONAL, SRATIONAL, DOUBLE
        _ => return None,
    };

    let total_size = count * type_size;

    // Get the actual data location
    // Sigma maker notes use offsets relative to the MakerNote start
    let value_data: &[u8] = if total_size <= 4 {
        &value_offset_bytes[..total_size.min(4)]
    } else {
        let offset = read_u32(value_offset_bytes, endian) as usize;
        if offset + total_size <= data.len() {
            &data[offset..offset + total_size]
        } else {
            return None;
        }
    };

    // Parse based on type
    let value = match tag_type {
        1 => {
            // BYTE
            ExifValue::Byte(value_data[..count.min(value_data.len())].to_vec())
        }
        2 => {
            // ASCII - null-terminated string
            let mut s = value_data[..count.min(value_data.len())].to_vec();
            // Remove trailing nulls
            while s.last() == Some(&0) {
                s.pop();
            }
            ExifValue::Ascii(String::from_utf8_lossy(&s).to_string())
        }
        3 => {
            // SHORT
            let mut values = Vec::new();
            for i in 0..count.min(value_data.len() / 2) {
                values.push(read_u16(&value_data[i * 2..], endian));
            }
            ExifValue::Short(values)
        }
        4 => {
            // LONG
            let mut values = Vec::new();
            for i in 0..count.min(value_data.len() / 4) {
                values.push(read_u32(&value_data[i * 4..], endian));
            }
            ExifValue::Long(values)
        }
        5 => {
            // RATIONAL
            let mut values = Vec::new();
            for i in 0..count.min(value_data.len() / 8) {
                let num = read_u32(&value_data[i * 8..], endian);
                let den = read_u32(&value_data[i * 8 + 4..], endian);
                values.push((num, den));
            }
            ExifValue::Rational(values)
        }
        7 => {
            // UNDEFINED
            ExifValue::Undefined(value_data[..count.min(value_data.len())].to_vec())
        }
        8 => {
            // SSHORT
            let mut values = Vec::new();
            for i in 0..count.min(value_data.len() / 2) {
                let v = read_u16(&value_data[i * 2..], endian);
                values.push(v as i16);
            }
            ExifValue::SShort(values)
        }
        9 => {
            // SLONG
            let mut values = Vec::new();
            for i in 0..count.min(value_data.len() / 4) {
                let v = read_u32(&value_data[i * 4..], endian);
                values.push(v as i32);
            }
            ExifValue::SLong(values)
        }
        10 => {
            // SRATIONAL
            let mut values = Vec::new();
            for i in 0..count.min(value_data.len() / 8) {
                let num = read_u32(&value_data[i * 8..], endian) as i32;
                let den = read_u32(&value_data[i * 8 + 4..], endian) as i32;
                values.push((num, den));
            }
            ExifValue::SRational(values)
        }
        _ => return None,
    };

    Some((tag_id, value))
}

/// Parse Sigma maker notes
///
/// Sigma maker notes are stored as a simple IFD with mostly ASCII string values.
/// Some tags store numeric values as strings (e.g., "Expo:+1.0"), while others
/// use rational or int32u format.
pub fn parse_sigma_maker_notes(
    data: &[u8],
    endian: Endianness,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    if data.len() < 2 {
        return Ok(tags);
    }

    // Read number of IFD entries
    let entry_count = read_u16(data, endian) as usize;

    // Calculate IFD end position
    let ifd_end = 2 + entry_count * 12;
    if data.len() < ifd_end {
        return Err(ExifError::Format(
            "Sigma MakerNote IFD truncated".to_string(),
        ));
    }

    // Parse each IFD entry
    for i in 0..entry_count {
        let entry_offset = 2 + i * 12;

        // Parse IFD entry
        if let Some((tag_id, value)) = parse_ifd_entry(data, entry_offset, endian) {
            let tag_name = get_sigma_tag_name(tag_id);

            // Decode specific tags based on their ID
            match tag_id {
                SIGMA_EXPOSURE_MODE => {
                    // ExposureMode is stored as ASCII character (P, A, S, M)
                    if let ExifValue::Ascii(ref s) = value {
                        let decoded = decode_exposure_mode(s);
                        tags.insert(
                            tag_id,
                            MakerNoteTag::new(tag_id, tag_name, ExifValue::Ascii(decoded)),
                        );
                    } else {
                        tags.insert(tag_id, MakerNoteTag::new(tag_id, tag_name, value));
                    }
                }
                SIGMA_METERING_MODE => {
                    // MeteringMode is stored as ASCII character (A, C, 8)
                    if let ExifValue::Ascii(ref s) = value {
                        let decoded = decode_metering_mode(s);
                        tags.insert(
                            tag_id,
                            MakerNoteTag::new(tag_id, tag_name, ExifValue::Ascii(decoded)),
                        );
                    } else {
                        tags.insert(tag_id, MakerNoteTag::new(tag_id, tag_name, value));
                    }
                }
                SIGMA_EXPOSURE_COMPENSATION
                | SIGMA_CONTRAST
                | SIGMA_SHADOW
                | SIGMA_HIGHLIGHT
                | SIGMA_SATURATION
                | SIGMA_SHARPNESS
                | SIGMA_X3_FILL_LIGHT
                | SIGMA_COLOR_ADJUSTMENT
                | SIGMA_QUALITY => {
                    // These tags may have label prefixes like "Expo:", "Cont:", etc.
                    // Strip the prefix for cleaner output
                    if let ExifValue::Ascii(ref s) = value {
                        let stripped = strip_label(s);
                        tags.insert(
                            tag_id,
                            MakerNoteTag::new(tag_id, tag_name, ExifValue::Ascii(stripped)),
                        );
                    } else {
                        tags.insert(tag_id, MakerNoteTag::new(tag_id, tag_name, value));
                    }
                }
                SIGMA_COLOR_MODE => {
                    // ColorMode is stored as int32u
                    if let ExifValue::Long(ref values) = value {
                        if !values.is_empty() {
                            let decoded = decode_color_mode_exiftool(values[0]);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::new(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                ),
                            );
                        }
                    } else {
                        tags.insert(tag_id, MakerNoteTag::new(tag_id, tag_name, value));
                    }
                }
                SIGMA_LENS_TYPE => {
                    // LensType can be stored as either string (hex) or int16u
                    // For now, just store the raw value
                    tags.insert(tag_id, MakerNoteTag::new(tag_id, tag_name, value));
                }
                _ => {
                    // For all other tags, store the value as-is
                    tags.insert(tag_id, MakerNoteTag::new(tag_id, tag_name, value));
                }
            }
        }
    }

    Ok(tags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_label() {
        assert_eq!(strip_label("Expo:+1.0"), "+1.0");
        assert_eq!(strip_label("Expo: +1.0"), "+1.0");
        assert_eq!(strip_label("Cont:0"), "0");
        assert_eq!(strip_label("Satu:-1"), "-1");
        assert_eq!(strip_label("NoColon"), "NoColon");
    }

    #[test]
    fn test_decode_exposure_mode() {
        assert_eq!(decode_exposure_mode("P"), "Program AE");
        assert_eq!(decode_exposure_mode("A"), "Aperture-priority AE");
        assert_eq!(decode_exposure_mode("S"), "Shutter speed priority AE");
        assert_eq!(decode_exposure_mode("M"), "Manual");
        assert_eq!(decode_exposure_mode("X"), "Unknown");
    }

    #[test]
    fn test_decode_metering_mode() {
        assert_eq!(decode_metering_mode("A"), "Average");
        assert_eq!(decode_metering_mode("C"), "Center-weighted average");
        assert_eq!(decode_metering_mode("8"), "Multi-segment");
        assert_eq!(decode_metering_mode("X"), "Unknown");
    }

    #[test]
    fn test_decode_color_mode() {
        assert_eq!(decode_color_mode_exiftool(0), "n/a");
        assert_eq!(decode_color_mode_exiftool(1), "Sepia");
        assert_eq!(decode_color_mode_exiftool(2), "B&W");
        assert_eq!(decode_color_mode_exiftool(3), "Standard");
        assert_eq!(decode_color_mode_exiftool(4), "Vivid");
        assert_eq!(decode_color_mode_exiftool(5), "Neutral");
        assert_eq!(decode_color_mode_exiftool(6), "Portrait");
        assert_eq!(decode_color_mode_exiftool(7), "Landscape");
        assert_eq!(decode_color_mode_exiftool(8), "FOV Classic Blue");
        assert_eq!(decode_color_mode_exiftool(99), "Unknown");
    }
}
