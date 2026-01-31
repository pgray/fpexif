// makernotes/samsung.rs - Samsung maker notes parsing

use crate::data_types::{Endianness, ExifValue};
use crate::define_tag_decoder;
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use std::collections::HashMap;

// exiv2 group names for Samsung sub-IFDs
pub const EXIV2_GROUP_SAMSUNG2: &str = "Samsung2";
pub const EXIV2_GROUP_SAMSUNG_PW: &str = "SamsungPw";

// Samsung Type2 (EXIF-format) MakerNote tag IDs
pub const SAMSUNG_MAKER_NOTE_VERSION: u16 = 0x0001;
pub const SAMSUNG_DEVICE_TYPE: u16 = 0x0002;
pub const SAMSUNG_SAMSUNG_MODEL_ID: u16 = 0x0003;
pub const SAMSUNG_ORIENTATION_INFO: u16 = 0x0011;
pub const SAMSUNG_SMART_ALBUM_COLOR: u16 = 0x0020;
pub const SAMSUNG_PICTURE_WIZARD: u16 = 0x0021;
pub const SAMSUNG_LOCAL_LOCATION_NAME: u16 = 0x0030;
pub const SAMSUNG_LOCATION_NAME: u16 = 0x0031;
pub const SAMSUNG_PREVIEW_IFD: u16 = 0x0035;
pub const SAMSUNG_RAW_DATA_BYTE_ORDER: u16 = 0x0040;
pub const SAMSUNG_WHITE_BALANCE_SETUP: u16 = 0x0041;
pub const SAMSUNG_CAMERA_TEMPERATURE: u16 = 0x0043;
pub const SAMSUNG_RAW_DATA_CFA_PATTERN: u16 = 0x0050;
pub const SAMSUNG_FACE_DETECT: u16 = 0x0100;
pub const SAMSUNG_FACE_RECOGNITION: u16 = 0x0120;
pub const SAMSUNG_FACE_NAME: u16 = 0x0123;
pub const SAMSUNG_FIRMWARE_NAME: u16 = 0xa001;
pub const SAMSUNG_SERIAL_NUMBER: u16 = 0xa002;
pub const SAMSUNG_LENS_TYPE: u16 = 0xa003;
pub const SAMSUNG_LENS_FIRMWARE: u16 = 0xa004;
pub const SAMSUNG_INTERNAL_LENS_SERIAL_NUMBER: u16 = 0xa005;
pub const SAMSUNG_SENSOR_AREAS: u16 = 0xa010;
pub const SAMSUNG_COLOR_SPACE: u16 = 0xa011;
pub const SAMSUNG_SMART_RANGE: u16 = 0xa012;
pub const SAMSUNG_EXPOSURE_COMPENSATION: u16 = 0xa013;
pub const SAMSUNG_ISO: u16 = 0xa014;
pub const SAMSUNG_EXPOSURE_TIME: u16 = 0xa018;
pub const SAMSUNG_FNUMBER: u16 = 0xa019;
pub const SAMSUNG_FOCAL_LENGTH_IN_35MM_FORMAT: u16 = 0xa01a;

// PictureWizard sub-IFD tags (stored as int16u array at tag 0x0021)
pub const PW_MODE: usize = 0;
pub const PW_COLOR: usize = 1;
pub const PW_SATURATION: usize = 2;
pub const PW_SHARPNESS: usize = 3;
pub const PW_CONTRAST: usize = 4;

/// Helper function to read a u16 value from bytes
fn read_u16(data: &[u8], endian: Endianness) -> u16 {
    match endian {
        Endianness::Little => u16::from_le_bytes([data[0], data[1]]),
        Endianness::Big => u16::from_be_bytes([data[0], data[1]]),
    }
}

/// Helper function to read a u32 value from bytes
fn read_u32(data: &[u8], endian: Endianness) -> u32 {
    match endian {
        Endianness::Little => u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
        Endianness::Big => u32::from_be_bytes([data[0], data[1], data[2], data[3]]),
    }
}

/// Returns the human-readable tag name for a Samsung tag ID
pub fn get_samsung_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        SAMSUNG_MAKER_NOTE_VERSION => Some("MakerNoteVersion"),
        SAMSUNG_DEVICE_TYPE => Some("DeviceType"),
        SAMSUNG_SAMSUNG_MODEL_ID => Some("SamsungModelID"),
        SAMSUNG_ORIENTATION_INFO => Some("OrientationInfo"),
        SAMSUNG_SMART_ALBUM_COLOR => Some("SmartAlbumColor"),
        SAMSUNG_PICTURE_WIZARD => Some("PictureWizard"),
        SAMSUNG_LOCAL_LOCATION_NAME => Some("LocalLocationName"),
        SAMSUNG_LOCATION_NAME => Some("LocationName"),
        SAMSUNG_PREVIEW_IFD => Some("PreviewIFD"),
        SAMSUNG_RAW_DATA_BYTE_ORDER => Some("RawDataByteOrder"),
        SAMSUNG_WHITE_BALANCE_SETUP => Some("WhiteBalanceSetup"),
        SAMSUNG_CAMERA_TEMPERATURE => Some("CameraTemperature"),
        SAMSUNG_RAW_DATA_CFA_PATTERN => Some("RawDataCFAPattern"),
        SAMSUNG_FACE_DETECT => Some("FaceDetect"),
        SAMSUNG_FACE_RECOGNITION => Some("FaceRecognition"),
        SAMSUNG_FACE_NAME => Some("FaceName"),
        SAMSUNG_FIRMWARE_NAME => Some("FirmwareName"),
        SAMSUNG_SERIAL_NUMBER => Some("SerialNumber"),
        SAMSUNG_LENS_TYPE => Some("LensType"),
        SAMSUNG_LENS_FIRMWARE => Some("LensFirmware"),
        SAMSUNG_INTERNAL_LENS_SERIAL_NUMBER => Some("InternalLensSerialNumber"),
        SAMSUNG_SENSOR_AREAS => Some("SensorAreas"),
        SAMSUNG_COLOR_SPACE => Some("ColorSpace"),
        SAMSUNG_SMART_RANGE => Some("SmartRange"),
        SAMSUNG_EXPOSURE_COMPENSATION => Some("ExposureCompensation"),
        SAMSUNG_ISO => Some("ISO"),
        SAMSUNG_EXPOSURE_TIME => Some("ExposureTime"),
        SAMSUNG_FNUMBER => Some("FNumber"),
        SAMSUNG_FOCAL_LENGTH_IN_35MM_FORMAT => Some("FocalLengthIn35mmFormat"),
        _ => None,
    }
}

/// Returns the lens name for a Samsung lens type ID
pub fn get_samsung_lens_name(lens_type: u16) -> &'static str {
    match lens_type {
        0 => "Built-in or Manual Lens",
        1 => "Samsung NX 30mm F2 Pancake",
        2 => "Samsung NX 18-55mm F3.5-5.6 OIS",
        3 => "Samsung NX 50-200mm F4-5.6 ED OIS",
        4 => "Samsung NX 20-50mm F3.5-5.6 ED",
        5 => "Samsung NX 20mm F2.8 Pancake",
        6 => "Samsung NX 18-200mm F3.5-6.3 ED OIS",
        7 => "Samsung NX 60mm F2.8 Macro ED OIS SSA",
        8 => "Samsung NX 16mm F2.4 Pancake",
        9 => "Samsung NX 85mm F1.4 ED SSA",
        10 => "Samsung NX 45mm F1.8",
        11 => "Samsung NX 45mm F1.8 2D/3D",
        12 => "Samsung NX 12-24mm F4-5.6 ED",
        13 => "Samsung NX 16-50mm F2-2.8 S ED OIS",
        14 => "Samsung NX 10mm F3.5 Fisheye",
        15 => "Samsung NX 16-50mm F3.5-5.6 Power Zoom ED OIS",
        20 => "Samsung NX 50-150mm F2.8 S ED OIS",
        21 => "Samsung NX 300mm F2.8 ED OIS",
        _ => "Unknown",
    }
}

// Tag decoders using the define_tag_decoder! macro

define_tag_decoder! {
    device_type,
    type: u32,
    both: {
        0x1000 => "Compact Digital Camera",
        0x2000 => "High-end NX Camera",
        0x3000 => "HXM Video Camera",
        0x12000 => "Cell Phone",
        0x300000 => "SMX Video Camera",
    }
}

define_tag_decoder! {
    color_space,
    both: {
        0 => "sRGB",
        1 => "Adobe RGB",
    }
}

define_tag_decoder! {
    smart_range,
    both: {
        0 => "Off",
        1 => "On",
    }
}

define_tag_decoder! {
    face_detect,
    both: {
        0 => "Off",
        1 => "On",
    }
}

define_tag_decoder! {
    face_recognition,
    both: {
        0 => "Off",
        1 => "On",
    }
}

define_tag_decoder! {
    raw_data_byte_order,
    both: {
        0 => "Little-endian",
        1 => "Big-endian",
    }
}

define_tag_decoder! {
    white_balance_setup,
    both: {
        0 => "Off",
        1 => "On",
    }
}

define_tag_decoder! {
    raw_data_cfa_pattern,
    both: {
        0 => "[Red,Green][Green,Blue]",
        1 => "[Green,Blue][Red,Green]",
        2 => "[Blue,Green][Green,Red]",
        3 => "[Green,Red][Blue,Green]",
    }
}

// PictureWizard Mode decoder
define_tag_decoder! {
    picture_wizard_mode,
    both: {
        0 => "Standard",
        1 => "Vivid",
        2 => "Portrait",
        3 => "Landscape",
        4 => "Forest",
        5 => "Retro",
        6 => "Cool",
        7 => "Calm",
        8 => "Classic",
        9 => "Custom1",
        10 => "Custom2",
        11 => "Custom3",
        255 => "n/a",
    }
}

/// Decode PictureWizard data (5 int16u values)
/// Returns a HashMap with decoded PictureWizard settings
fn decode_picture_wizard(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    if data.len() >= 5 {
        // Mode
        let mode_str = decode_picture_wizard_mode_exiftool(data[PW_MODE]);
        decoded.insert(
            "PictureWizardMode".to_string(),
            ExifValue::Ascii(mode_str.to_string()),
        );

        // Color (hue in degrees, or 65535 for neutral)
        let color_val = data[PW_COLOR];
        let color_str = if color_val == 65535 {
            "Neutral".to_string()
        } else {
            format!("{}", color_val)
        };
        decoded.insert(
            "PictureWizardColor".to_string(),
            ExifValue::Ascii(color_str),
        );

        // Saturation, Sharpness, Contrast (value - 4)
        decoded.insert(
            "PictureWizardSaturation".to_string(),
            ExifValue::SShort(vec![data[PW_SATURATION] as i16 - 4]),
        );
        decoded.insert(
            "PictureWizardSharpness".to_string(),
            ExifValue::SShort(vec![data[PW_SHARPNESS] as i16 - 4]),
        );
        decoded.insert(
            "PictureWizardContrast".to_string(),
            ExifValue::SShort(vec![data[PW_CONTRAST] as i16 - 4]),
        );
    }

    decoded
}

/// Parse a single IFD entry
fn parse_ifd_entry(
    data: &[u8],
    entry_offset: usize,
    endian: Endianness,
    tiff_data: Option<&[u8]>,
    tiff_offset: usize,
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
    let value_data: &[u8] = if total_size <= 4 {
        value_offset_bytes
    } else {
        let offset = read_u32(value_offset_bytes, endian) as usize;

        // Try to use TIFF data first (Samsung uses TIFF-relative offsets)
        if let Some(tiff) = tiff_data {
            let abs_offset = tiff_offset + offset;
            if abs_offset + total_size <= tiff.len() {
                &tiff[abs_offset..abs_offset + total_size]
            } else {
                // Fall back to MakerNote-relative offset
                if offset + total_size <= data.len() {
                    &data[offset..offset + total_size]
                } else {
                    return None;
                }
            }
        } else {
            // No TIFF data, try MakerNote-relative offset
            if offset + total_size <= data.len() {
                &data[offset..offset + total_size]
            } else {
                return None;
            }
        }
    };

    // Parse based on type
    let value = match tag_type {
        1 => {
            // BYTE
            ExifValue::Byte(value_data[..count.min(value_data.len())].to_vec())
        }
        2 => {
            // ASCII
            let bytes = value_data[..count.min(value_data.len())].to_vec();
            let s = String::from_utf8_lossy(&bytes)
                .trim_end_matches('\0')
                .to_string();
            ExifValue::Ascii(s)
        }
        3 => {
            // SHORT
            let mut vals = Vec::new();
            for i in 0..count {
                if i * 2 + 2 <= value_data.len() {
                    vals.push(read_u16(&value_data[i * 2..], endian));
                }
            }
            ExifValue::Short(vals)
        }
        4 => {
            // LONG
            let mut vals = Vec::new();
            for i in 0..count {
                if i * 4 + 4 <= value_data.len() {
                    vals.push(read_u32(&value_data[i * 4..], endian));
                }
            }
            ExifValue::Long(vals)
        }
        5 => {
            // RATIONAL
            let mut vals = Vec::new();
            for i in 0..count {
                if i * 8 + 8 <= value_data.len() {
                    let num = read_u32(&value_data[i * 8..], endian);
                    let den = read_u32(&value_data[i * 8 + 4..], endian);
                    vals.push((num, den));
                }
            }
            ExifValue::Rational(vals)
        }
        7 => {
            // UNDEFINED
            ExifValue::Undefined(value_data[..count.min(value_data.len())].to_vec())
        }
        8 => {
            // SSHORT
            let mut vals = Vec::new();
            for i in 0..count {
                if i * 2 + 2 <= value_data.len() {
                    vals.push(read_u16(&value_data[i * 2..], endian) as i16);
                }
            }
            ExifValue::SShort(vals)
        }
        9 => {
            // SLONG
            let mut vals = Vec::new();
            for i in 0..count {
                if i * 4 + 4 <= value_data.len() {
                    vals.push(read_u32(&value_data[i * 4..], endian) as i32);
                }
            }
            ExifValue::SLong(vals)
        }
        10 => {
            // SRATIONAL
            let mut vals = Vec::new();
            for i in 0..count {
                if i * 8 + 8 <= value_data.len() {
                    let num = read_u32(&value_data[i * 8..], endian) as i32;
                    let den = read_u32(&value_data[i * 8 + 4..], endian) as i32;
                    vals.push((num, den));
                }
            }
            ExifValue::SRational(vals)
        }
        _ => return None,
    };

    Some((tag_id, value))
}

/// Parse Samsung MakerNotes in EXIF-format (Type2)
///
/// Samsung cameras (except GX models which use Pentax format) store their maker notes
/// as a standard EXIF IFD. This function parses the IFD and decodes Samsung-specific tags.
///
/// # Arguments
///
/// * `data` - The maker note data
/// * `endian` - Byte order (little or big endian)
/// * `tiff_data` - Optional TIFF data for offset calculation
/// * `tiff_offset` - Offset to the TIFF header
/// * `model` - Optional camera model string (used to distinguish formats)
///
/// # Returns
///
/// A HashMap of tag IDs to MakerNoteTag entries
pub fn parse_samsung_maker_notes(
    data: &[u8],
    endian: Endianness,
    tiff_data: Option<&[u8]>,
    tiff_offset: usize,
    _model: Option<&str>,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut maker_notes = HashMap::new();

    // Parse the Samsung Type2 IFD (standard EXIF format)
    // Read number of entries (2 bytes)
    if data.len() < 2 {
        return Err(ExifError::Format("Samsung MakerNote too short".to_string()));
    }

    let num_entries = read_u16(data, endian) as usize;
    let mut entry_offset = 2;

    for _ in 0..num_entries {
        if let Some((tag_id, entry_value)) =
            parse_ifd_entry(data, entry_offset, endian, tiff_data, tiff_offset)
        {
            let tag_name = get_samsung_tag_name(tag_id);
            let mut value = entry_value.clone();
            let exiv2_group = Some(EXIV2_GROUP_SAMSUNG2);
            let exiv2_name = tag_name;

            // Apply tag-specific decoding
            match tag_id {
                SAMSUNG_DEVICE_TYPE => {
                    if let ExifValue::Long(ref vals) = entry_value {
                        if !vals.is_empty() {
                            let decoded = decode_device_type_exiftool(vals[0]);
                            value = ExifValue::Ascii(decoded.to_string());
                        }
                    }
                }
                SAMSUNG_LENS_TYPE => {
                    if let ExifValue::Short(ref vals) = entry_value {
                        if !vals.is_empty() {
                            let lens_name = get_samsung_lens_name(vals[0]);
                            value = ExifValue::Ascii(lens_name.to_string());
                        }
                    }
                }
                SAMSUNG_COLOR_SPACE => {
                    if let ExifValue::Short(ref vals) = entry_value {
                        if !vals.is_empty() {
                            let decoded = decode_color_space_exiftool(vals[0]);
                            value = ExifValue::Ascii(decoded.to_string());
                        }
                    }
                }
                SAMSUNG_SMART_RANGE => {
                    if let ExifValue::Short(ref vals) = entry_value {
                        if !vals.is_empty() {
                            let decoded = decode_smart_range_exiftool(vals[0]);
                            value = ExifValue::Ascii(decoded.to_string());
                        }
                    }
                }
                SAMSUNG_FACE_DETECT => {
                    if let ExifValue::Short(ref vals) = entry_value {
                        if !vals.is_empty() {
                            let decoded = decode_face_detect_exiftool(vals[0]);
                            value = ExifValue::Ascii(decoded.to_string());
                        }
                    }
                }
                SAMSUNG_FACE_RECOGNITION => {
                    if let ExifValue::Short(ref vals) = entry_value {
                        if !vals.is_empty() {
                            let decoded = decode_face_recognition_exiftool(vals[0]);
                            value = ExifValue::Ascii(decoded.to_string());
                        }
                    }
                }
                SAMSUNG_RAW_DATA_BYTE_ORDER => {
                    if let ExifValue::Short(ref vals) = entry_value {
                        if !vals.is_empty() {
                            let decoded = decode_raw_data_byte_order_exiftool(vals[0]);
                            value = ExifValue::Ascii(decoded.to_string());
                        }
                    }
                }
                SAMSUNG_WHITE_BALANCE_SETUP => {
                    if let ExifValue::Short(ref vals) = entry_value {
                        if !vals.is_empty() {
                            let decoded = decode_white_balance_setup_exiftool(vals[0]);
                            value = ExifValue::Ascii(decoded.to_string());
                        }
                    }
                }
                SAMSUNG_RAW_DATA_CFA_PATTERN => {
                    if let ExifValue::Short(ref vals) = entry_value {
                        if !vals.is_empty() {
                            let decoded = decode_raw_data_cfa_pattern_exiftool(vals[0]);
                            value = ExifValue::Ascii(decoded.to_string());
                        }
                    }
                }
                SAMSUNG_PICTURE_WIZARD => {
                    // PictureWizard is a sub-structure with 5 int16u values
                    if let ExifValue::Short(ref vals) = entry_value {
                        let pw_decoded = decode_picture_wizard(vals);

                        // Add each PictureWizard field as a separate tag
                        for (field_name, field_value) in pw_decoded {
                            let pw_tag = MakerNoteTag {
                                tag_id,
                                tag_name: Some("PictureWizard"),
                                value: field_value.clone(),
                                raw_value: None,
                                exiv2_group: Some(EXIV2_GROUP_SAMSUNG_PW),
                                exiv2_name: Some(Box::leak(field_name.clone().into_boxed_str())),
                            };

                            // Use a pseudo-tag ID for sub-fields (0x0021_00, 0x0021_01, etc.)
                            let sub_tag_id = if field_name == "PictureWizardMode" {
                                0x2100
                            } else if field_name == "PictureWizardColor" {
                                0x2101
                            } else if field_name == "PictureWizardSaturation" {
                                0x2102
                            } else if field_name == "PictureWizardSharpness" {
                                0x2103
                            } else if field_name == "PictureWizardContrast" {
                                0x2104
                            } else {
                                continue;
                            };

                            maker_notes.insert(sub_tag_id, pw_tag);
                        }
                        entry_offset += 12;
                        continue; // Don't add the raw PictureWizard tag
                    }
                }
                _ => {}
            }

            let tag = MakerNoteTag {
                tag_id,
                tag_name,
                value,
                raw_value: Some(entry_value.clone()),
                exiv2_group,
                exiv2_name,
            };

            maker_notes.insert(tag_id, tag);
        }

        entry_offset += 12;
    }

    Ok(maker_notes)
}
