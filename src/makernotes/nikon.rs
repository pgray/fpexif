// makernotes/nikon.rs - Nikon maker notes parsing

use crate::data_types::{Endianness, ExifValue};
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Cursor;

// Common Nikon MakerNote tag IDs
pub const NIKON_VERSION: u16 = 0x0001;
pub const NIKON_ISO_SETTING: u16 = 0x0002;
pub const NIKON_COLOR_MODE: u16 = 0x0003;
pub const NIKON_QUALITY: u16 = 0x0004;
pub const NIKON_WHITE_BALANCE: u16 = 0x0005;
pub const NIKON_SHARPNESS: u16 = 0x0006;
pub const NIKON_FOCUS_MODE: u16 = 0x0007;
pub const NIKON_FLASH_SETTING: u16 = 0x0008;
pub const NIKON_FLASH_TYPE: u16 = 0x0009;
pub const NIKON_WHITE_BALANCE_FINE: u16 = 0x000B;
pub const NIKON_WB_RB_LEVELS: u16 = 0x000C;
pub const NIKON_PROGRAM_SHIFT: u16 = 0x000D;
pub const NIKON_EXPOSURE_DIFFERENCE: u16 = 0x000E;
pub const NIKON_ISO_SELECTION: u16 = 0x000F;
pub const NIKON_DATA_DUMP: u16 = 0x0010;
pub const NIKON_PREVIEW_IFD: u16 = 0x0011;
pub const NIKON_FLASH_EXPOSURE_COMP: u16 = 0x0012;
pub const NIKON_ISO_SETTING_2: u16 = 0x0013;
pub const NIKON_COLOR_BALANCE_A: u16 = 0x0014;
pub const NIKON_IMAGE_BOUNDARY: u16 = 0x0016;
pub const NIKON_FLASH_EXPOSURE_BRACKET_VALUE: u16 = 0x0017;
pub const NIKON_EXPOSURE_BRACKET_VALUE: u16 = 0x0018;
pub const NIKON_IMAGE_PROCESSING: u16 = 0x0019;
pub const NIKON_CROP_HI_SPEED: u16 = 0x001A;
pub const NIKON_EXPOSURE_TUNING: u16 = 0x001B;
pub const NIKON_SERIAL_NUMBER: u16 = 0x001D;
pub const NIKON_COLOR_SPACE: u16 = 0x001E;
pub const NIKON_VR_INFO: u16 = 0x001F;
pub const NIKON_IMAGE_AUTHENTICATION: u16 = 0x0020;
pub const NIKON_FACE_DETECT: u16 = 0x0021;
pub const NIKON_ACTIVE_D_LIGHTING: u16 = 0x0022;
pub const NIKON_PICTURE_CONTROL_DATA: u16 = 0x0023;
pub const NIKON_WORLD_TIME: u16 = 0x0024;
pub const NIKON_ISO_INFO: u16 = 0x0025;
pub const NIKON_VIGNETTE_CONTROL: u16 = 0x002A;
pub const NIKON_DISTORTION_CONTROL: u16 = 0x002B;
pub const NIKON_LENS_DATA: u16 = 0x0083;
pub const NIKON_SHOT_INFO: u16 = 0x0091;
pub const NIKON_COLOR_BALANCE: u16 = 0x0097;
pub const NIKON_LENS_TYPE: u16 = 0x0098;
pub const NIKON_LENS: u16 = 0x0099;
pub const NIKON_FLASH_INFO: u16 = 0x00A8;

/// Get the name of a Nikon MakerNote tag
pub fn get_nikon_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        NIKON_VERSION => Some("NikonMakerNoteVersion"),
        NIKON_ISO_SETTING => Some("ISO"),
        NIKON_COLOR_MODE => Some("ColorMode"),
        NIKON_QUALITY => Some("Quality"),
        NIKON_WHITE_BALANCE => Some("WhiteBalance"),
        NIKON_SHARPNESS => Some("Sharpness"),
        NIKON_FOCUS_MODE => Some("FocusMode"),
        NIKON_FLASH_SETTING => Some("FlashSetting"),
        NIKON_FLASH_TYPE => Some("FlashType"),
        NIKON_SERIAL_NUMBER => Some("SerialNumber"),
        NIKON_COLOR_SPACE => Some("ColorSpace"),
        NIKON_VR_INFO => Some("VRInfo"),
        NIKON_ACTIVE_D_LIGHTING => Some("ActiveDLighting"),
        NIKON_PICTURE_CONTROL_DATA => Some("PictureControlData"),
        NIKON_VIGNETTE_CONTROL => Some("VignetteControl"),
        NIKON_DISTORTION_CONTROL => Some("DistortionControl"),
        NIKON_LENS_DATA => Some("LensData"),
        NIKON_SHOT_INFO => Some("ShotInfo"),
        NIKON_LENS_TYPE => Some("LensType"),
        NIKON_LENS => Some("Lens"),
        NIKON_FLASH_INFO => Some("FlashInfo"),
        _ => None,
    }
}

/// Decode Nikon ASCII tag values to human-readable strings
fn decode_nikon_ascii_value(tag_id: u16, value: &str) -> String {
    match tag_id {
        NIKON_QUALITY => match value.trim() {
            "RAW" => "RAW",
            "FINE" => "Fine",
            "NORMAL" => "Normal",
            "BASIC" => "Basic",
            "RAW+FINE" | "RAW + FINE" => "RAW + Fine",
            "RAW+NORMAL" | "RAW + NORMAL" => "RAW + Normal",
            "RAW+BASIC" | "RAW + BASIC" => "RAW + Basic",
            _ => value.trim(),
        }
        .to_string(),
        NIKON_WHITE_BALANCE => match value.trim() {
            "AUTO" | "AUTO1" | "AUTO2" => "Auto",
            "SUNNY" | "DIRECT SUNLIGHT" => "Daylight",
            "SHADE" => "Shade",
            "CLOUDY" => "Cloudy",
            "TUNGSTEN" | "INCANDESCENT" => "Tungsten",
            "FLUORESCENT" => "Fluorescent",
            "FLASH" => "Flash",
            "PRESET" => "Preset",
            _ => value.trim(),
        }
        .to_string(),
        NIKON_FOCUS_MODE => match value.trim() {
            "AF-S" => "AF-S",
            "AF-C" => "AF-C",
            "AF-A" => "AF-A",
            "MF" | "MANUAL" => "Manual",
            _ => value.trim(),
        }
        .to_string(),
        NIKON_SHARPNESS => match value.trim() {
            "AUTO" => "Auto",
            "NORMAL" => "Normal",
            "LOW" => "Low",
            "MED.L" | "MEDIUM LOW" => "Medium Low",
            "MED.H" | "MEDIUM HIGH" => "Medium High",
            "HIGH" => "High",
            "NONE" => "None",
            _ => value.trim(),
        }
        .to_string(),
        NIKON_COLOR_MODE => value.trim().to_string(),
        _ => value.trim().to_string(),
    }
}

/// Parse Nikon maker notes
pub fn parse_nikon_maker_notes(
    data: &[u8],
    endian: Endianness,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    // Nikon maker notes often start with "Nikon\0" header
    if data.len() < 10 {
        return Ok(tags);
    }

    let mut offset = 0;

    // Check for Nikon header
    if data.starts_with(b"Nikon\0") {
        offset = 10; // Skip "Nikon\0" + TIFF header
    }

    if offset >= data.len() {
        return Ok(tags);
    }

    let mut cursor = Cursor::new(&data[offset..]);

    // Read number of entries
    let num_entries = match endian {
        Endianness::Little => cursor
            .read_u16::<LittleEndian>()
            .map_err(|_| ExifError::Format("Failed to read Nikon maker note count".to_string()))?,
        Endianness::Big => cursor
            .read_u16::<BigEndian>()
            .map_err(|_| ExifError::Format("Failed to read Nikon maker note count".to_string()))?,
    };

    // Parse IFD entries
    for _ in 0..num_entries {
        if cursor.position() as usize + 12 > data[offset..].len() {
            break;
        }

        let tag_id = match endian {
            Endianness::Little => cursor.read_u16::<LittleEndian>(),
            Endianness::Big => cursor.read_u16::<BigEndian>(),
        }
        .ok();

        let tag_type = match endian {
            Endianness::Little => cursor.read_u16::<LittleEndian>(),
            Endianness::Big => cursor.read_u16::<BigEndian>(),
        }
        .ok();

        let count = match endian {
            Endianness::Little => cursor.read_u32::<LittleEndian>(),
            Endianness::Big => cursor.read_u32::<BigEndian>(),
        }
        .ok();

        let value_offset = match endian {
            Endianness::Little => cursor.read_u32::<LittleEndian>(),
            Endianness::Big => cursor.read_u32::<BigEndian>(),
        }
        .ok();

        if let (Some(tag_id), Some(tag_type), Some(count), Some(value_offset)) =
            (tag_id, tag_type, count, value_offset)
        {
            // Calculate value size in bytes
            let value_size = match tag_type {
                1 => count as usize,     // BYTE
                2 => count as usize,     // ASCII
                3 => count as usize * 2, // SHORT
                4 => count as usize * 4, // LONG
                5 => count as usize * 8, // RATIONAL
                7 => count as usize,     // UNDEFINED
                _ => 0,
            };

            // Determine if value is inline or at offset
            let value_bytes = if value_size <= 4 {
                // Inline value in the value_offset field
                match endian {
                    Endianness::Little => value_offset.to_le_bytes().to_vec(),
                    Endianness::Big => value_offset.to_be_bytes().to_vec(),
                }
            } else {
                // Value at offset
                let abs_offset = offset + value_offset as usize;
                if abs_offset + value_size <= data.len() {
                    data[abs_offset..abs_offset + value_size].to_vec()
                } else {
                    continue;
                }
            };

            // Parse the value based on type
            let value = match tag_type {
                1 => {
                    // BYTE
                    ExifValue::Byte(value_bytes[..count as usize].to_vec())
                }
                2 => {
                    // ASCII
                    let s = String::from_utf8_lossy(&value_bytes[..count as usize])
                        .trim_end_matches('\0')
                        .to_string();
                    // Apply value decoders for specific tags
                    let decoded = decode_nikon_ascii_value(tag_id, &s);
                    ExifValue::Ascii(decoded)
                }
                3 => {
                    // SHORT
                    let mut values = Vec::new();
                    let mut cursor = Cursor::new(&value_bytes);
                    for _ in 0..count {
                        if let Ok(v) = match endian {
                            Endianness::Little => cursor.read_u16::<LittleEndian>(),
                            Endianness::Big => cursor.read_u16::<BigEndian>(),
                        } {
                            values.push(v);
                        } else {
                            break;
                        }
                    }
                    // Special handling for ISO tag
                    if tag_id == NIKON_ISO_SETTING && values.len() >= 2 {
                        let iso = if values[1] > 0 { values[1] } else { values[0] };
                        ExifValue::Ascii(iso.to_string())
                    } else {
                        ExifValue::Short(values)
                    }
                }
                4 => {
                    // LONG
                    let mut values = Vec::new();
                    let mut cursor = Cursor::new(&value_bytes);
                    for _ in 0..count {
                        if let Ok(v) = match endian {
                            Endianness::Little => cursor.read_u32::<LittleEndian>(),
                            Endianness::Big => cursor.read_u32::<BigEndian>(),
                        } {
                            values.push(v);
                        } else {
                            break;
                        }
                    }
                    ExifValue::Long(values)
                }
                7 => {
                    // UNDEFINED - keep as binary
                    ExifValue::Undefined(value_bytes)
                }
                _ => {
                    // Unsupported type
                    continue;
                }
            };

            tags.insert(
                tag_id,
                MakerNoteTag {
                    tag_id,
                    tag_name: get_nikon_tag_name(tag_id),
                    value,
                },
            );
        }
    }

    Ok(tags)
}
