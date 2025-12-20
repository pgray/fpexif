// makernotes/sony.rs - Sony maker notes parsing

use crate::data_types::{Endianness, ExifValue};
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use std::collections::HashMap;

// Common Sony MakerNote tag IDs
pub const SONY_CAMERA_INFO: u16 = 0x0010;
pub const SONY_FOCUS_INFO: u16 = 0x0020;
pub const SONY_IMAGE_QUALITY: u16 = 0x0102;
pub const SONY_FLASH_EXPOSURE_COMP: u16 = 0x0104;
pub const SONY_TELECONVERTER: u16 = 0x0105;
pub const SONY_WHITE_BALANCE_FINE_TUNE: u16 = 0x0112;
pub const SONY_CAMERA_SETTINGS: u16 = 0x0114;
pub const SONY_WHITE_BALANCE: u16 = 0x0115;
pub const SONY_EXTRA_INFO: u16 = 0x0116;
pub const SONY_PRINT_IM: u16 = 0x0E00;
pub const SONY_MULTI_BURST_MODE: u16 = 0x1000;
pub const SONY_MULTI_BURST_IMAGE_WIDTH: u16 = 0x1001;
pub const SONY_MULTI_BURST_IMAGE_HEIGHT: u16 = 0x1002;
pub const SONY_PANORAMA: u16 = 0x1003;
pub const SONY_PREVIEW_IMAGE: u16 = 0x2001;
pub const SONY_RATING: u16 = 0x2002;
pub const SONY_CONTRAST: u16 = 0x2004;
pub const SONY_SATURATION: u16 = 0x2005;
pub const SONY_SHARPNESS: u16 = 0x2006;
pub const SONY_BRIGHTNESS: u16 = 0x2007;
pub const SONY_LONG_EXPOSURE_NOISE_REDUCTION: u16 = 0x2008;
pub const SONY_HIGH_ISO_NOISE_REDUCTION: u16 = 0x2009;
pub const SONY_HDR: u16 = 0x200A;
pub const SONY_MULTI_FRAME_NOISE_REDUCTION: u16 = 0x200B;
pub const SONY_PICTURE_EFFECT: u16 = 0x200E;
pub const SONY_SOFT_SKIN_EFFECT: u16 = 0x200F;
pub const SONY_VIGNETTING_CORRECTION: u16 = 0x2011;
pub const SONY_LATERAL_CHROMATIC_ABERRATION: u16 = 0x2012;
pub const SONY_DISTORTION_CORRECTION: u16 = 0x2013;
pub const SONY_WB_SHIFT_AB: u16 = 0x2014;
pub const SONY_WB_SHIFT_GM: u16 = 0x2015;
pub const SONY_AUTO_PORTRAIT_FRAMED: u16 = 0x2016;
pub const SONY_FOCUS_MODE: u16 = 0x201B;
pub const SONY_AF_POINT_SELECTED: u16 = 0x201E;
pub const SONY_SHOT_INFO: u16 = 0x3000;
pub const SONY_FILE_FORMAT: u16 = 0xB000;
pub const SONY_SONY_MODEL_ID: u16 = 0xB001;
pub const SONY_CREATIVE_STYLE: u16 = 0xB020;
pub const SONY_COLOR_TEMPERATURE: u16 = 0xB021;
pub const SONY_COLOR_COMPENSATION_FILTER: u16 = 0xB022;
pub const SONY_SCENE_MODE: u16 = 0xB023;
pub const SONY_ZONE_MATCHING: u16 = 0xB024;
pub const SONY_DYNAMIC_RANGE_OPTIMIZER: u16 = 0xB025;
pub const SONY_IMAGE_STABILIZATION: u16 = 0xB026;
pub const SONY_LENS_ID: u16 = 0xB027;
pub const SONY_MINOLTA_MAKER_NOTE: u16 = 0xB028;
pub const SONY_COLOR_MODE: u16 = 0xB029;
pub const SONY_LENS_SPEC: u16 = 0xB02A;
pub const SONY_FULL_IMAGE_SIZE: u16 = 0xB02B;
pub const SONY_PREVIEW_IMAGE_SIZE: u16 = 0xB02C;
pub const SONY_MACRO: u16 = 0xB040;
pub const SONY_EXPOSURE_MODE: u16 = 0xB041;
pub const SONY_FOCUS_MODE_2: u16 = 0xB042;
pub const SONY_AF_MODE: u16 = 0xB043;
pub const SONY_AF_ILLUMINATOR: u16 = 0xB044;
pub const SONY_QUALITY_2: u16 = 0xB047;
pub const SONY_FLASH_LEVEL: u16 = 0xB048;
pub const SONY_RELEASE_MODE: u16 = 0xB049;
pub const SONY_SEQUENCE_NUMBER: u16 = 0xB04A;
pub const SONY_ANTI_BLUR: u16 = 0xB04B;
pub const SONY_LONG_EXPOSURE_NOISE_REDUCTION_2: u16 = 0xB04E;
pub const SONY_DYNAMIC_RANGE_OPTIMIZER_2: u16 = 0xB04F;
pub const SONY_HIGH_ISO_NOISE_REDUCTION_2: u16 = 0xB050;
pub const SONY_INTELLIGENT_AUTO: u16 = 0xB052;
pub const SONY_WHITE_BALANCE_2: u16 = 0xB054;

/// Get the name of a Sony MakerNote tag
pub fn get_sony_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        SONY_CAMERA_INFO => Some("CameraInfo"),
        SONY_FOCUS_INFO => Some("FocusInfo"),
        SONY_IMAGE_QUALITY => Some("ImageQuality"),
        SONY_FLASH_EXPOSURE_COMP => Some("FlashExposureComp"),
        SONY_TELECONVERTER => Some("Teleconverter"),
        SONY_WHITE_BALANCE => Some("WhiteBalance"),
        SONY_CAMERA_SETTINGS => Some("CameraSettings"),
        SONY_RATING => Some("Rating"),
        SONY_CONTRAST => Some("Contrast"),
        SONY_SATURATION => Some("Saturation"),
        SONY_SHARPNESS => Some("Sharpness"),
        SONY_BRIGHTNESS => Some("Brightness"),
        SONY_LONG_EXPOSURE_NOISE_REDUCTION => Some("LongExposureNoiseReduction"),
        SONY_HIGH_ISO_NOISE_REDUCTION => Some("HighISONoiseReduction"),
        SONY_HDR => Some("HDR"),
        SONY_MULTI_FRAME_NOISE_REDUCTION => Some("MultiFrameNoiseReduction"),
        SONY_PICTURE_EFFECT => Some("PictureEffect"),
        SONY_VIGNETTING_CORRECTION => Some("VignettingCorrection"),
        SONY_DISTORTION_CORRECTION => Some("DistortionCorrection"),
        SONY_FOCUS_MODE => Some("FocusMode"),
        SONY_AF_POINT_SELECTED => Some("AFPointSelected"),
        SONY_SONY_MODEL_ID => Some("SonyModelID"),
        SONY_CREATIVE_STYLE => Some("CreativeStyle"),
        SONY_COLOR_TEMPERATURE => Some("ColorTemperature"),
        SONY_SCENE_MODE => Some("SceneMode"),
        SONY_DYNAMIC_RANGE_OPTIMIZER => Some("DynamicRangeOptimizer"),
        SONY_IMAGE_STABILIZATION => Some("ImageStabilization"),
        SONY_LENS_ID => Some("LensID"),
        SONY_LENS_SPEC => Some("LensSpec"),
        SONY_MACRO => Some("Macro"),
        SONY_EXPOSURE_MODE => Some("ExposureMode"),
        SONY_AF_MODE => Some("AFMode"),
        SONY_QUALITY_2 => Some("Quality"),
        SONY_FLASH_LEVEL => Some("FlashLevel"),
        SONY_RELEASE_MODE => Some("ReleaseMode"),
        SONY_ANTI_BLUR => Some("AntiBlur"),
        SONY_INTELLIGENT_AUTO => Some("IntelligentAuto"),
        _ => None,
    }
}

/// Read u16 with given endianness
fn read_u16(data: &[u8], endian: Endianness) -> u16 {
    match endian {
        Endianness::Little => u16::from_le_bytes([data[0], data[1]]),
        Endianness::Big => u16::from_be_bytes([data[0], data[1]]),
    }
}

/// Read u32 with given endianness
fn read_u32(data: &[u8], endian: Endianness) -> u32 {
    match endian {
        Endianness::Little => u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
        Endianness::Big => u32::from_be_bytes([data[0], data[1], data[2], data[3]]),
    }
}

/// Decode CreativeStyle value (tag 0xB020)
pub fn decode_creative_style(value: u16) -> &'static str {
    match value {
        0 => "None",
        1 => "Standard",
        2 => "Vivid",
        3 => "Portrait",
        4 => "Landscape",
        5 => "Sunset",
        6 => "Night",
        8 => "B&W",
        9 => "Clear",
        10 => "Deep",
        11 => "Light",
        12 => "Autumn",
        13 => "Neutral",
        14 => "Sepia",
        15 => "Night View/Portrait",
        16 => "Soft",
        17 => "Spring",
        18 => "Summer",
        19 => "Winter",
        20 => "Portrait (Adobe RGB)",
        21 => "Landscape (Adobe RGB)",
        22 => "Night Scene",
        23 => "Flowers",
        24 => "Saloon",
        25 => "Illustration",
        26 => "High Contrast Mono",
        27 => "Retro Photo",
        _ => "Unknown",
    }
}

/// Decode ExposureMode value (tag 0xB041)
pub fn decode_exposure_mode(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        1 => "Portrait",
        2 => "Landscape",
        3 => "Macro",
        4 => "Sports",
        5 => "Sunset",
        6 => "Night Scene",
        7 => "Beach/Snow",
        8 => "Fireworks",
        9 => "Anti Motion Blur",
        10 => "Pet",
        11 => "Backlight Correction HDR",
        12 => "Superior Auto",
        13 => "High Sensitivity",
        14 => "Night Portrait",
        15 => "Hand-held Twilight",
        16 => "Sweep Panorama",
        17 => "3D Sweep Panorama",
        18 => "Background Defocus",
        19 => "Soft Skin",
        20 => "Portrait (with Smile Shutter)",
        _ => "Unknown",
    }
}

/// Decode AFMode value (tag 0xB043)
pub fn decode_af_mode(value: u16) -> &'static str {
    match value {
        0 => "Default",
        1 => "Multi",
        2 => "Center",
        3 => "Spot",
        4 => "Flexible Spot",
        5 => "Touch",
        6 => "Continuous",
        7 => "Face Detected",
        8 => "Tracking",
        9 => "Zone",
        10 => "Expand Flexible Spot",
        11 => "Flexible Spot (S)",
        12 => "Flexible Spot (M)",
        13 => "Flexible Spot (L)",
        14 => "Eye AF",
        _ => "Unknown",
    }
}

/// Decode DynamicRangeOptimizer value (tag 0xB025)
/// Note: This tag can be either SHORT or LONG depending on camera model
pub fn decode_dynamic_range_optimizer(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "Standard",
        2 => "Advanced Auto",
        3 => "Auto",
        4 => "Advanced Lv1",
        5 => "Advanced Lv2",
        6 => "Advanced Lv3",
        7 => "Advanced Lv4",
        8 => "Advanced Lv5",
        9 => "Lv1",
        10 => "Lv2",
        11 => "Lv3",
        12 => "Lv4",
        13 => "Lv5",
        _ => "Unknown",
    }
}

/// Decode FocusMode value (tag 0x201B)
/// Note: This tag can be BYTE or SHORT depending on camera model
pub fn decode_focus_mode(value: u16) -> &'static str {
    match value {
        0 => "Manual",
        1 => "AF-S",
        2 => "AF-C",
        3 => "AF-A",
        4 => "DMF",
        5 => "AF-D",
        6 => "Single-shot AF",
        7 => "Automatic AF",
        _ => "Unknown",
    }
}

/// Decode ImageStabilization value (tag 0xB026)
/// Note: This tag can be either SHORT or LONG depending on camera model
pub fn decode_image_stabilization(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        2 => "On (2)",
        3 => "On (Shooting)",
        _ => "Unknown",
    }
}

/// Parse a single IFD entry from Sony maker notes
fn parse_ifd_entry(
    data: &[u8],
    entry_offset: usize,
    endian: Endianness,
    base_offset: usize,
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
    // Sony maker notes use offsets relative to the maker note start
    let value_data: &[u8] = if total_size <= 4 {
        value_offset_bytes
    } else {
        let offset = read_u32(value_offset_bytes, endian) as usize;
        // Try maker note-relative offset
        let abs_offset = if offset >= base_offset {
            offset - base_offset
        } else {
            offset
        };
        if abs_offset + total_size <= data.len() {
            &data[abs_offset..abs_offset + total_size]
        } else if offset + total_size <= data.len() {
            &data[offset..offset + total_size]
        } else {
            return None;
        }
    };

    // Parse based on type
    let value = match tag_type {
        1 | 6 => {
            // BYTE or SBYTE
            let bytes = value_data[..count.min(value_data.len())].to_vec();

            // Apply value decoders for single-byte tags
            if bytes.len() == 1 {
                let v = bytes[0] as u16;
                let decoded = match tag_id {
                    SONY_FOCUS_MODE => Some(decode_focus_mode(v).to_string()),
                    _ => None,
                };

                if let Some(s) = decoded {
                    ExifValue::Ascii(s)
                } else {
                    ExifValue::Byte(bytes)
                }
            } else {
                ExifValue::Byte(bytes)
            }
        }
        2 => {
            // ASCII
            let s = value_data[..count.min(value_data.len())]
                .iter()
                .take_while(|&&b| b != 0)
                .map(|&b| b as char)
                .collect::<String>();
            ExifValue::Ascii(s)
        }
        3 | 8 => {
            // SHORT or SSHORT
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                if i * 2 + 2 <= value_data.len() {
                    values.push(read_u16(&value_data[i * 2..], endian));
                }
            }

            // Apply value decoders for single-value tags
            if values.len() == 1 {
                let v = values[0];
                let decoded = match tag_id {
                    SONY_CREATIVE_STYLE => Some(decode_creative_style(v).to_string()),
                    SONY_EXPOSURE_MODE => Some(decode_exposure_mode(v).to_string()),
                    SONY_AF_MODE => Some(decode_af_mode(v).to_string()),
                    SONY_DYNAMIC_RANGE_OPTIMIZER => {
                        Some(decode_dynamic_range_optimizer(v).to_string())
                    }
                    SONY_FOCUS_MODE => Some(decode_focus_mode(v).to_string()),
                    SONY_IMAGE_STABILIZATION => Some(decode_image_stabilization(v).to_string()),
                    _ => None,
                };

                if let Some(s) = decoded {
                    ExifValue::Ascii(s)
                } else {
                    ExifValue::Short(values)
                }
            } else {
                ExifValue::Short(values)
            }
        }
        4 | 9 => {
            // LONG or SLONG
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                if i * 4 + 4 <= value_data.len() {
                    values.push(read_u32(&value_data[i * 4..], endian));
                }
            }

            // Apply value decoders for single-value LONG tags
            if values.len() == 1 && values[0] <= u16::MAX as u32 {
                let v = values[0] as u16;
                let decoded = match tag_id {
                    SONY_DYNAMIC_RANGE_OPTIMIZER => {
                        Some(decode_dynamic_range_optimizer(v).to_string())
                    }
                    SONY_IMAGE_STABILIZATION => Some(decode_image_stabilization(v).to_string()),
                    _ => None,
                };

                if let Some(s) = decoded {
                    ExifValue::Ascii(s)
                } else {
                    ExifValue::Long(values)
                }
            } else {
                ExifValue::Long(values)
            }
        }
        5 | 10 => {
            // RATIONAL or SRATIONAL
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                if i * 8 + 8 <= value_data.len() {
                    let num = read_u32(&value_data[i * 8..], endian);
                    let den = read_u32(&value_data[i * 8 + 4..], endian);
                    values.push((num, den));
                }
            }
            ExifValue::Rational(values)
        }
        7 => {
            // UNDEFINED
            ExifValue::Undefined(value_data[..total_size.min(value_data.len())].to_vec())
        }
        _ => ExifValue::Undefined(vec![]),
    };

    Some((tag_id, value))
}

/// Parse Sony maker notes
pub fn parse_sony_maker_notes(
    data: &[u8],
    endian: Endianness,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    // Sony maker notes can start with "SONY DSC " or be in standard TIFF IFD format
    let mut offset = 0;
    let base_offset = 0usize;

    if data.len() >= 12 && data.starts_with(b"SONY DSC ") {
        offset = 12;
    }

    if offset >= data.len() || data.len() < offset + 2 {
        return Ok(tags);
    }

    // Read number of entries
    let num_entries = read_u16(&data[offset..], endian);
    offset += 2;

    // Parse IFD entries
    for i in 0..num_entries as usize {
        let entry_offset = offset + i * 12;
        if entry_offset + 12 > data.len() {
            break;
        }

        if let Some((tag_id, value)) = parse_ifd_entry(data, entry_offset, endian, base_offset) {
            tags.insert(
                tag_id,
                MakerNoteTag {
                    tag_id,
                    tag_name: get_sony_tag_name(tag_id),
                    value,
                },
            );
        }
    }

    Ok(tags)
}
