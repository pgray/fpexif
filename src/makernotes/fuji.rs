// makernotes/fuji.rs - Fujifilm maker notes parsing

use crate::data_types::{Endianness, ExifValue};
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Cursor;

// Fujifilm MakerNote tag IDs
pub const FUJI_VERSION: u16 = 0x0000;
pub const FUJI_SERIAL_NUMBER: u16 = 0x0010;
pub const FUJI_QUALITY: u16 = 0x1000;
pub const FUJI_SHARPNESS: u16 = 0x1001;
pub const FUJI_WHITE_BALANCE: u16 = 0x1002;
pub const FUJI_SATURATION: u16 = 0x1003;
pub const FUJI_CONTRAST: u16 = 0x1004;
pub const FUJI_COLOR_TEMPERATURE: u16 = 0x1005;
pub const FUJI_CONTRAST_DETECTION_AF: u16 = 0x1006;
pub const FUJI_FLASH_MODE: u16 = 0x1010;
pub const FUJI_FLASH_EXPOSURE_COMP: u16 = 0x1011;
pub const FUJI_MACRO: u16 = 0x1020;
pub const FUJI_FOCUS_MODE: u16 = 0x1021;
pub const FUJI_AF_MODE: u16 = 0x1022;
pub const FUJI_FOCUS_PIXEL: u16 = 0x1023;
pub const FUJI_SLOW_SYNC: u16 = 0x1030;
pub const FUJI_PICTURE_MODE: u16 = 0x1031;
pub const FUJI_EXR_AUTO: u16 = 0x1033;
pub const FUJI_EXR_MODE: u16 = 0x1034;
pub const FUJI_AUTO_BRACKETING: u16 = 0x1100;
pub const FUJI_SEQUENCE_NUMBER: u16 = 0x1101;
pub const FUJI_BLUR_WARNING: u16 = 0x1300;
pub const FUJI_FOCUS_WARNING: u16 = 0x1301;
pub const FUJI_EXPOSURE_WARNING: u16 = 0x1302;
pub const FUJI_DYNAMIC_RANGE: u16 = 0x1400;
pub const FUJI_FILM_MODE: u16 = 0x1401;
pub const FUJI_DYNAMIC_RANGE_SETTING: u16 = 0x1402;
pub const FUJI_DEVELOPMENT_DYNAMIC_RANGE: u16 = 0x1403;
pub const FUJI_MIN_FOCAL_LENGTH: u16 = 0x1404;
pub const FUJI_MAX_FOCAL_LENGTH: u16 = 0x1405;
pub const FUJI_MAX_APERTURE_AT_MIN_FOCAL: u16 = 0x1406;
pub const FUJI_MAX_APERTURE_AT_MAX_FOCAL: u16 = 0x1407;
pub const FUJI_FILE_SOURCE: u16 = 0x8000;
pub const FUJI_ORDER_NUMBER: u16 = 0x8002;
pub const FUJI_FRAME_NUMBER: u16 = 0x8003;
pub const FUJI_FACES_DETECTED: u16 = 0x4100;
pub const FUJI_FACE_POSITIONS: u16 = 0x4103;
pub const FUJI_FACE_REC_INFO: u16 = 0x4282;
pub const FUJI_RAW_IMAGE_FULL_SIZE: u16 = 0x0100;
pub const FUJI_RAW_IMAGE_CROP_TOP_LEFT: u16 = 0x0110;
pub const FUJI_RAW_IMAGE_CROPPED_SIZE: u16 = 0x0111;
pub const FUJI_RAW_IMAGE_ASPECT_RATIO: u16 = 0x0115;
pub const FUJI_LENS_MOUNT_TYPE: u16 = 0x1600;
pub const FUJI_RATINGS_INFO: u16 = 0xB211;
pub const FUJI_GE_IMAGE_SIZE: u16 = 0xB212;

/// Get the name of a Fujifilm MakerNote tag
pub fn get_fuji_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        FUJI_VERSION => Some("Version"),
        FUJI_SERIAL_NUMBER => Some("InternalSerialNumber"),
        FUJI_QUALITY => Some("Quality"),
        FUJI_SHARPNESS => Some("Sharpness"),
        FUJI_WHITE_BALANCE => Some("WhiteBalance"),
        FUJI_SATURATION => Some("Saturation"),
        FUJI_CONTRAST => Some("Contrast"),
        FUJI_COLOR_TEMPERATURE => Some("ColorTemperature"),
        FUJI_CONTRAST_DETECTION_AF => Some("ContrastDetectionAF"),
        FUJI_FLASH_MODE => Some("FlashMode"),
        FUJI_FLASH_EXPOSURE_COMP => Some("FlashExposureComp"),
        FUJI_MACRO => Some("Macro"),
        FUJI_FOCUS_MODE => Some("FocusMode"),
        FUJI_AF_MODE => Some("AFMode"),
        FUJI_FOCUS_PIXEL => Some("FocusPixel"),
        FUJI_SLOW_SYNC => Some("SlowSync"),
        FUJI_PICTURE_MODE => Some("PictureMode"),
        FUJI_EXR_AUTO => Some("EXRAuto"),
        FUJI_EXR_MODE => Some("EXRMode"),
        FUJI_AUTO_BRACKETING => Some("AutoBracketing"),
        FUJI_SEQUENCE_NUMBER => Some("SequenceNumber"),
        FUJI_BLUR_WARNING => Some("BlurWarning"),
        FUJI_FOCUS_WARNING => Some("FocusWarning"),
        FUJI_EXPOSURE_WARNING => Some("ExposureWarning"),
        FUJI_DYNAMIC_RANGE => Some("DynamicRange"),
        FUJI_FILM_MODE => Some("FilmMode"),
        FUJI_DYNAMIC_RANGE_SETTING => Some("DynamicRangeSetting"),
        FUJI_DEVELOPMENT_DYNAMIC_RANGE => Some("DevelopmentDynamicRange"),
        FUJI_MIN_FOCAL_LENGTH => Some("MinFocalLength"),
        FUJI_MAX_FOCAL_LENGTH => Some("MaxFocalLength"),
        FUJI_MAX_APERTURE_AT_MIN_FOCAL => Some("MaxApertureAtMinFocal"),
        FUJI_MAX_APERTURE_AT_MAX_FOCAL => Some("MaxApertureAtMaxFocal"),
        FUJI_FILE_SOURCE => Some("FileSource"),
        FUJI_ORDER_NUMBER => Some("OrderNumber"),
        FUJI_FRAME_NUMBER => Some("FrameNumber"),
        FUJI_FACES_DETECTED => Some("FacesDetected"),
        FUJI_FACE_POSITIONS => Some("FacePositions"),
        FUJI_FACE_REC_INFO => Some("FaceRecInfo"),
        FUJI_RAW_IMAGE_FULL_SIZE => Some("RawImageFullSize"),
        FUJI_RAW_IMAGE_CROP_TOP_LEFT => Some("RawImageCropTopLeft"),
        FUJI_RAW_IMAGE_CROPPED_SIZE => Some("RawImageCroppedSize"),
        FUJI_RAW_IMAGE_ASPECT_RATIO => Some("RawImageAspectRatio"),
        FUJI_LENS_MOUNT_TYPE => Some("LensMountType"),
        FUJI_RATINGS_INFO => Some("RatingsInfo"),
        FUJI_GE_IMAGE_SIZE => Some("GEImageSize"),
        _ => None,
    }
}

/// Decode FilmMode value (tag 0x1401)
/// Values based on ExifTool FujiFilm.pm reference
fn decode_film_mode(value: u16) -> &'static str {
    match value {
        0x000 => "F0/Standard (Provia)",
        0x100 => "F1/Studio Portrait",
        0x110 => "F1a/Studio Portrait Enhanced Saturation",
        0x120 => "F1b/Studio Portrait Smooth Skin Tone (Astia)",
        0x130 => "F1c/Studio Portrait Increased Sharpness",
        0x200 => "F2/Fujichrome (Velvia)",
        0x300 => "F3/Studio Portrait Ex",
        0x400 => "F4/Velvia",
        0x500 => "Pro Neg. Std",
        0x501 => "Pro Neg. Hi",
        0x600 => "Classic Chrome",
        0x700 => "Eterna",
        0x800 => "Classic Negative",
        0x900 => "Bleach Bypass",
        0xa00 => "Nostalgic Neg",
        0xb00 => "Reala ACE",
        // Acros variants (in Saturation tag 0x1003, but sometimes reported here)
        0x801 => "Acros",
        0x802 => "Acros+Ye Filter",
        0x803 => "Acros+R Filter",
        0x804 => "Acros+G Filter",
        // Eterna variants
        0x701 => "Eterna Bleach Bypass",
        _ => "Unknown",
    }
}

/// Decode DynamicRange value (tag 0x1400)
fn decode_dynamic_range(value: u16) -> &'static str {
    match value {
        1 => "Standard",
        3 => "Wide",
        _ => "Unknown",
    }
}

/// Decode WhiteBalance value (tag 0x1002)
fn decode_white_balance(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        1 => "Auto (White Priority)",
        2 => "Auto (Ambience Priority)",
        256 => "Daylight",
        512 => "Cloudy",
        768 => "Daylight Fluorescent",
        769 => "Day White Fluorescent",
        770 => "White Fluorescent",
        771 => "Warm White Fluorescent",
        772 => "Living Room Warm White Fluorescent",
        1024 => "Incandescent",
        1280 => "Flash",
        1536 => "Underwater",
        3840 => "Custom",
        3841 => "Custom2",
        3842 => "Custom3",
        3843 => "Custom4",
        3844 => "Custom5",
        6144 => "Kelvin",
        _ => "Unknown",
    }
}

/// Decode Sharpness value (tag 0x1001)
fn decode_sharpness(value: u16) -> &'static str {
    match value {
        0x00 => "-4 (softest)",
        0x01 => "-3 (very soft)",
        0x02 => "-2 (soft)",
        0x03 => "-1",
        0x04 => "0 (normal)",
        0x05 => "+1",
        0x06 => "+2",
        0x07 => "+3",
        0x08 => "+4 (hardest)",
        0x82 => "Medium Soft",
        0x84 => "Medium Hard",
        0x100 => "Film Simulation",
        0x8000 => "n/a (Film Simulation)",
        0xffff => "n/a",
        _ => "Unknown",
    }
}

/// Decode Saturation value (tag 0x1003)
fn decode_saturation(value: u16) -> &'static str {
    match value {
        0x0 => "0 (normal)",
        0x80 => "+1 (medium high)",
        0x100 => "+2 (high)",
        0x180 => "+3 (very high)",
        0x200 => "+4 (highest)",
        0x300 => "+4.5",
        0x301 => "Acros",
        0x302 => "Acros+Ye Filter",
        0x303 => "Acros+R Filter",
        0x304 => "Acros+G Filter",
        0x8000 => "Film Simulation",
        0xffff => "n/a",
        _ => "Unknown",
    }
}

/// Decode Macro value (tag 0x1020)
fn decode_macro(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        _ => "Unknown",
    }
}

/// Decode FocusMode value (tag 0x1021)
fn decode_focus_mode(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        1 => "Manual",
        65535 => "Movie",
        _ => "Unknown",
    }
}

/// Decode AFMode value (tag 0x1022)
fn decode_af_mode(value: u16) -> &'static str {
    match value {
        0 => "No",
        1 => "Single Point",
        256 => "Zone",
        512 => "Wide/Tracking",
        768 => "Wide/Tracking (PriorityFace)",
        _ => "Unknown",
    }
}

/// Decode SlowSync value (tag 0x1030)
fn decode_slow_sync(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        _ => "Unknown",
    }
}

/// Decode AutoBracketing value (tag 0x1100)
fn decode_auto_bracketing(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        2 => "Pre-shot",
        _ => "Unknown",
    }
}

/// Decode BlurWarning value (tag 0x1300)
fn decode_blur_warning(value: u16) -> &'static str {
    match value {
        0 => "None",
        1 => "Blur Warning",
        _ => "Unknown",
    }
}

/// Decode FocusWarning value (tag 0x1301)
fn decode_focus_warning(value: u16) -> &'static str {
    match value {
        0 => "Good",
        1 => "Out of focus",
        _ => "Unknown",
    }
}

/// Decode ExposureWarning value (tag 0x1302)
fn decode_exposure_warning(value: u16) -> &'static str {
    match value {
        0 => "Good",
        1 => "Bad exposure",
        _ => "Unknown",
    }
}

/// Decode PictureMode value (tag 0x1031)
fn decode_picture_mode(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        1 => "Portrait",
        2 => "Landscape",
        3 => "Macro",
        4 => "Sports",
        5 => "Night Scene",
        6 => "Program AE",
        7 => "Natural Light",
        8 => "Anti-blur",
        9 => "Beach & Snow",
        10 => "Sunset",
        11 => "Museum",
        12 => "Party",
        13 => "Flower",
        14 => "Text",
        256 => "Aperture Priority",
        512 => "Shutter Priority",
        768 => "Manual",
        _ => "Unknown",
    }
}

/// Decode DynamicRangeSetting value (tag 0x1402)
fn decode_dynamic_range_setting(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        1 => "Manual",
        _ => "Unknown",
    }
}

/// Parse Fujifilm maker notes
pub fn parse_fuji_maker_notes(
    data: &[u8],
    _endian: Endianness,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    if data.len() < 12 {
        return Ok(tags);
    }

    // Fuji maker notes start with "FUJIFILM" header (8 bytes)
    // followed by offset to IFD (4 bytes, little-endian)
    if data.len() >= 12 && &data[0..8] == b"FUJIFILM" {
        // Read IFD offset (little-endian)
        let mut cursor = Cursor::new(&data[8..12]);
        let ifd_offset = cursor
            .read_u32::<LittleEndian>()
            .map_err(|_| ExifError::Format("Failed to read Fuji IFD offset".to_string()))?
            as usize;

        // IFD offset is relative to start of maker note data
        if ifd_offset >= data.len() {
            return Ok(tags);
        }

        // Parse IFD
        let ifd_data = &data[ifd_offset..];
        if ifd_data.len() < 2 {
            return Ok(tags);
        }

        let mut cursor = Cursor::new(ifd_data);

        // Read number of entries (always little-endian for Fuji)
        let num_entries = cursor
            .read_u16::<LittleEndian>()
            .map_err(|_| ExifError::Format("Failed to read Fuji maker note count".to_string()))?;

        // Parse each entry
        for _ in 0..num_entries {
            if cursor.position() as usize + 12 > ifd_data.len() {
                break;
            }

            let tag_id = match cursor.read_u16::<LittleEndian>() {
                Ok(id) => id,
                Err(_) => break,
            };

            let data_type = match cursor.read_u16::<LittleEndian>() {
                Ok(dt) => dt,
                Err(_) => break,
            };

            let count = match cursor.read_u32::<LittleEndian>() {
                Ok(c) => c,
                Err(_) => break,
            };

            let value_offset = match cursor.read_u32::<LittleEndian>() {
                Ok(o) => o,
                Err(_) => break,
            };

            // Parse the value based on type
            // For simplicity, we'll only handle a few common types
            let value = match data_type {
                1 => {
                    // BYTE
                    if count <= 4 {
                        // Value is in the offset field
                        let bytes = value_offset.to_be_bytes();
                        ExifValue::Byte(bytes[..count as usize].to_vec())
                    } else {
                        // Value is at offset
                        let offset = value_offset as usize;
                        if offset + count as usize <= data.len() {
                            ExifValue::Byte(data[offset..offset + count as usize].to_vec())
                        } else {
                            continue;
                        }
                    }
                }
                2 => {
                    // ASCII
                    let offset = value_offset as usize;
                    if offset + count as usize <= data.len() {
                        let string_bytes = &data[offset..offset + count as usize];
                        let s = String::from_utf8_lossy(string_bytes).to_string();
                        ExifValue::Ascii(s)
                    } else {
                        continue;
                    }
                }
                3 => {
                    // SHORT
                    let mut values = Vec::new();
                    if count == 1 {
                        // Value is in the offset field (first 2 bytes)
                        values.push((value_offset >> 16) as u16);
                    } else if count == 2 {
                        // Both values are in the offset field
                        values.push((value_offset >> 16) as u16);
                        values.push((value_offset & 0xFFFF) as u16);
                    } else {
                        // Value is at offset
                        let offset = value_offset as usize;
                        if offset + (count as usize * 2) <= data.len() {
                            let mut cursor = Cursor::new(&data[offset..]);
                            for _ in 0..count {
                                if let Ok(v) = cursor.read_u16::<LittleEndian>() {
                                    values.push(v);
                                } else {
                                    break;
                                }
                            }
                        } else {
                            continue;
                        }
                    }

                    // Apply value decoders for single-value SHORT tags
                    if values.len() == 1 {
                        let v = values[0];
                        let decoded = match tag_id {
                            FUJI_FILM_MODE => Some(decode_film_mode(v).to_string()),
                            FUJI_DYNAMIC_RANGE => Some(decode_dynamic_range(v).to_string()),
                            FUJI_WHITE_BALANCE => Some(decode_white_balance(v).to_string()),
                            FUJI_SHARPNESS => Some(decode_sharpness(v).to_string()),
                            FUJI_SATURATION => Some(decode_saturation(v).to_string()),
                            FUJI_MACRO => Some(decode_macro(v).to_string()),
                            FUJI_FOCUS_MODE => Some(decode_focus_mode(v).to_string()),
                            FUJI_AF_MODE => Some(decode_af_mode(v).to_string()),
                            FUJI_SLOW_SYNC => Some(decode_slow_sync(v).to_string()),
                            FUJI_PICTURE_MODE => Some(decode_picture_mode(v).to_string()),
                            FUJI_AUTO_BRACKETING => Some(decode_auto_bracketing(v).to_string()),
                            FUJI_BLUR_WARNING => Some(decode_blur_warning(v).to_string()),
                            FUJI_FOCUS_WARNING => Some(decode_focus_warning(v).to_string()),
                            FUJI_EXPOSURE_WARNING => Some(decode_exposure_warning(v).to_string()),
                            FUJI_DYNAMIC_RANGE_SETTING => {
                                Some(decode_dynamic_range_setting(v).to_string())
                            }
                            _ => None,
                        };

                        if let Some(s) = decoded {
                            ExifValue::Ascii(s)
                        } else {
                            ExifValue::Short(values)
                        }
                    } else if values.len() == 2 && tag_id == FUJI_FOCUS_PIXEL {
                        // Special formatting for FocusPixel: "X Y" space-separated
                        let formatted = format!("{} {}", values[0], values[1]);
                        ExifValue::Ascii(formatted)
                    } else {
                        ExifValue::Short(values)
                    }
                }
                4 => {
                    // LONG
                    if count == 1 {
                        ExifValue::Long(vec![value_offset])
                    } else {
                        let offset = value_offset as usize;
                        if offset + (count as usize * 4) <= data.len() {
                            let mut values = Vec::new();
                            let mut cursor = Cursor::new(&data[offset..]);
                            for _ in 0..count {
                                if let Ok(v) = cursor.read_u32::<LittleEndian>() {
                                    values.push(v);
                                } else {
                                    break;
                                }
                            }
                            ExifValue::Long(values)
                        } else {
                            continue;
                        }
                    }
                }
                _ => {
                    // Unsupported type for now
                    continue;
                }
            };

            tags.insert(
                tag_id,
                MakerNoteTag {
                    tag_id,
                    tag_name: get_fuji_tag_name(tag_id),
                    value,
                },
            );
        }
    }

    Ok(tags)
}
