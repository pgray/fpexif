// makernotes/leica.rs - Leica maker notes parsing
//
// Leica cameras use different MakerNote formats:
// - M-series (M8, M9, M Monochrom, M240, M10, etc.) - Leica2 format (0x300+ tags)
// - X-series (X1, X2, X Vario), T, CL, SL - Leica5 format (0x300+ tags)
// - D-Lux, Digilux, V-Lux series - Panasonic format (handled by panasonic.rs)
//
// References:
// - ExifTool: Panasonic.pm (contains Leica2 and Leica5 tables)
// - exiv2: Not implemented (Leica uses Panasonic tags in exiv2)

use crate::data_types::{Endianness, ExifValue};
use crate::define_tag_decoder;
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Cursor;

// exiv2 group name (Leica tags are shown as "Panasonic" in exiv2, but we use "Leica" for clarity)
pub const EXIV2_GROUP_LEICA: &str = "Leica";

// Leica MakerNote tag IDs (Leica2 format - M8, M9, M240)
pub const LEICA_QUALITY: u16 = 0x0300;
pub const LEICA_USER_PROFILE: u16 = 0x0302;
pub const LEICA_SERIAL_NUMBER: u16 = 0x0303;
pub const LEICA_WHITE_BALANCE: u16 = 0x0304;
pub const LEICA_LENS_TYPE: u16 = 0x0310;
pub const LEICA_EXTERNAL_SENSOR_BRIGHTNESS: u16 = 0x0311;
pub const LEICA_MEASURED_LV: u16 = 0x0312;
pub const LEICA_APPROXIMATE_F_NUMBER: u16 = 0x0313;
pub const LEICA_CAMERA_TEMPERATURE: u16 = 0x0320;
pub const LEICA_COLOR_TEMPERATURE: u16 = 0x0321;
pub const LEICA_WB_RED_LEVEL: u16 = 0x0322;
pub const LEICA_WB_GREEN_LEVEL: u16 = 0x0323;
pub const LEICA_WB_BLUE_LEVEL: u16 = 0x0324;
pub const LEICA_UV_IR_FILTER_CORRECTION: u16 = 0x0325;
pub const LEICA_CCD_VERSION: u16 = 0x0330;
pub const LEICA_CCD_BOARD_VERSION: u16 = 0x0331;
pub const LEICA_CONTROLLER_BOARD_VERSION: u16 = 0x0332;
pub const LEICA_M16C_VERSION: u16 = 0x0333;
pub const LEICA_IMAGE_ID_NUMBER: u16 = 0x0340;

// Leica5 format tag IDs (X1, X2, X Vario, T, CL, SL)
pub const LEICA5_LENS_TYPE: u16 = 0x0303; // String format for Leica T
pub const LEICA5_SERIAL_NUMBER: u16 = 0x0305;
pub const LEICA5_ORIGINAL_FILENAME: u16 = 0x0407;
pub const LEICA5_ORIGINAL_DIRECTORY: u16 = 0x0408;
pub const LEICA5_FOCUS_INFO: u16 = 0x040A;
pub const LEICA5_EXPOSURE_MODE: u16 = 0x040D;
pub const LEICA5_SHOT_INFO: u16 = 0x0410;
pub const LEICA5_FILM_MODE: u16 = 0x0412;
pub const LEICA5_WB_RGB_LEVELS: u16 = 0x0413;
pub const LEICA5_INTERNAL_SERIAL_NUMBER: u16 = 0x0500;

/// Get the name of a Leica MakerNote tag
pub fn get_leica_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        // Leica2 format (M8, M9, M240, etc.)
        LEICA_QUALITY => Some("Quality"),
        LEICA_USER_PROFILE => Some("UserProfile"),
        LEICA_SERIAL_NUMBER => Some("SerialNumber"),
        LEICA_WHITE_BALANCE => Some("WhiteBalance"),
        LEICA_LENS_TYPE => Some("LensType"),
        LEICA_EXTERNAL_SENSOR_BRIGHTNESS => Some("ExternalSensorBrightnessValue"),
        LEICA_MEASURED_LV => Some("MeasuredLV"),
        LEICA_APPROXIMATE_F_NUMBER => Some("ApproximateFNumber"),
        LEICA_CAMERA_TEMPERATURE => Some("CameraTemperature"),
        LEICA_COLOR_TEMPERATURE => Some("ColorTemperature"),
        LEICA_WB_RED_LEVEL => Some("WBRedLevel"),
        LEICA_WB_GREEN_LEVEL => Some("WBGreenLevel"),
        LEICA_WB_BLUE_LEVEL => Some("WBBlueLevel"),
        LEICA_UV_IR_FILTER_CORRECTION => Some("UV-IRFilterCorrection"),
        LEICA_CCD_VERSION => Some("CCDVersion"),
        LEICA_CCD_BOARD_VERSION => Some("CCDBoardVersion"),
        LEICA_CONTROLLER_BOARD_VERSION => Some("ControllerBoardVersion"),
        LEICA_M16C_VERSION => Some("M16CVersion"),
        LEICA_IMAGE_ID_NUMBER => Some("ImageIDNumber"),

        // Leica5 format (X1, X2, T, CL, SL)
        LEICA5_SERIAL_NUMBER => Some("SerialNumber"),
        LEICA5_ORIGINAL_FILENAME => Some("OriginalFileName"),
        LEICA5_ORIGINAL_DIRECTORY => Some("OriginalDirectory"),
        LEICA5_FOCUS_INFO => Some("FocusInfo"),
        LEICA5_EXPOSURE_MODE => Some("ExposureMode"),
        LEICA5_SHOT_INFO => Some("ShotInfo"),
        LEICA5_FILM_MODE => Some("FilmMode"),
        LEICA5_WB_RGB_LEVELS => Some("WB_RGBLevels"),
        LEICA5_INTERNAL_SERIAL_NUMBER => Some("InternalSerialNumber"),

        _ => None,
    }
}

// Quality (tag 0x0300): Panasonic.pm Leica2
define_tag_decoder! {
    quality,
    both: {
        1 => "Fine",
        2 => "Basic",
    }
}

// UserProfile (tag 0x0302): Panasonic.pm Leica2
define_tag_decoder! {
    user_profile,
    both: {
        1 => "User Profile 1",
        2 => "User Profile 2",
        3 => "User Profile 3",
        4 => "User Profile 0 (Dynamic)",
    }
}

// WhiteBalance (tag 0x0304): Panasonic.pm Leica2
// Note: Values above 0x8000 are Kelvin color temperatures
define_tag_decoder! {
    white_balance,
    both: {
        0 => "Auto or Manual",
        1 => "Daylight",
        2 => "Fluorescent",
        3 => "Tungsten",
        4 => "Flash",
        10 => "Cloudy",
        11 => "Shade",
    }
}

// UV-IRFilterCorrection (tag 0x0325): Panasonic.pm Leica2
define_tag_decoder! {
    uv_ir_filter_correction,
    both: {
        0 => "Not Active",
        1 => "Active",
    }
}

// ExposureMode (tag 0x040D): Panasonic.pm Leica5
// Format: int8u[4], matches on first bytes
pub fn decode_exposure_mode_exiftool(data: &[u8]) -> &'static str {
    if data.len() < 4 {
        return "Unknown";
    }

    match (data[0], data[1], data[2], data[3]) {
        (0, 0, 0, 0) => "Program AE",
        (0, 1, 0, 0) => "Program AE (1)",
        (1, 0, 0, 0) => "Aperture-priority AE",
        (1, 1, 0, 0) => "Aperture-priority AE (1)",
        (2, 0, 0, 0) => "Shutter speed priority AE",
        (3, 0, 0, 0) => "Manual",
        _ => "Unknown",
    }
}

pub fn decode_exposure_mode_exiv2(data: &[u8]) -> &'static str {
    // exiv2 doesn't implement Leica-specific tags, use ExifTool format
    decode_exposure_mode_exiftool(data)
}

/// Decode Leica lens type
/// The value is encoded as: (LensID << 2) | FrameSelectorBits
/// We extract LensID and frame selector, then format as "LensID FrameSelector"
pub fn decode_lens_type(value: u32) -> String {
    let lens_id = value >> 2;
    let frame_selector = value & 0x3;

    // Common Leica M lenses (from ExifTool Panasonic.pm %leicaLensTypes)
    let lens_name = match (lens_id, frame_selector) {
        (0, 0) => "Uncoded lens",
        (1, _) => "Elmarit-M 21mm f/2.8",
        (3, _) => "Elmarit-M 28mm f/2.8 (III)",
        (4, _) => "Tele-Elmarit-M 90mm f/2.8 (II)",
        (5, _) => "Summilux-M 50mm f/1.4 (II)",
        (6, 0) => "Summilux-M 35mm f/1.4",
        (6, _) => "Summicron-M 35mm f/2 (IV)",
        (7, _) => "Summicron-M 90mm f/2 (II)",
        (9, 0) => "Apo-Telyt-M 135mm f/3.4",
        (9, _) => "Elmarit-M 135mm f/2.8 (I/II)",
        (11, _) => "Summaron-M 28mm f/5.6",
        (12, _) => "Thambar-M 90mm f/2.2",
        (16, 0) => "Tri-Elmar-M 16-18-21mm f/4 ASPH.",
        (16, 1) => "Tri-Elmar-M 16-18-21mm f/4 ASPH. (at 16mm)",
        (16, 2) => "Tri-Elmar-M 16-18-21mm f/4 ASPH. (at 18mm)",
        (16, 3) => "Tri-Elmar-M 16-18-21mm f/4 ASPH. (at 21mm)",
        (23, _) => "Summicron-M 50mm f/2 (III)",
        (24, _) => "Elmarit-M 21mm f/2.8 ASPH.",
        (25, _) => "Elmarit-M 24mm f/2.8 ASPH.",
        (26, _) => "Summicron-M 28mm f/2 ASPH.",
        (27, _) => "Elmarit-M 28mm f/2.8 (IV)",
        (28, _) => "Elmarit-M 28mm f/2.8 ASPH.",
        (29, 0) => "Summilux-M 35mm f/1.4 ASPHERICAL",
        (29, _) => "Summilux-M 35mm f/1.4 ASPH.",
        (30, _) => "Summicron-M 35mm f/2 ASPH.",
        (31, _) => "Noctilux-M 50mm f/1",
        (32, _) => "Noctilux-M 50mm f/1.2",
        (33, _) => "Summilux-M 50mm f/1.4 ASPH.",
        (34, _) => "Summicron-M 50mm f/2 (IV)",
        (35, _) => "Summicron-M 50mm f/2 (V)",
        (36, _) => "Elmar-M 50mm f/2.8",
        (37, _) => "Summilux-M 75mm f/1.4",
        (38, _) => "Apo-Summicron-M 75mm f/2 ASPH.",
        (39, _) => "Apo-Summicron-M 90mm f/2 ASPH.",
        (40, _) => "Elmarit-M 90mm f/2.8",
        (41, _) => "Macro-Elmar-M 90mm f/4",
        (42, _) => "Tri-Elmar-M 28-35-50mm f/4 ASPH.",
        (42, 1) => "Tri-Elmar-M 28-35-50mm f/4 ASPH. (at 28mm)",
        (42, 2) => "Tri-Elmar-M 28-35-50mm f/4 ASPH. (at 35mm)",
        (42, 3) => "Tri-Elmar-M 28-35-50mm f/4 ASPH. (at 50mm)",
        (43, _) => "Summarit-M 35mm f/2.4 ASPH.",
        (44, _) => "Summarit-M 50mm f/2.4",
        (45, _) => "Summarit-M 75mm f/2.4",
        (46, _) => "Summarit-M 90mm f/2.4",
        (47, _) => "Summaron-M 28mm f/5.6 (2016)",
        (48, _) => "Elmar-M 24mm f/3.8 ASPH.",
        (49, _) => "Macro-Adapter M",
        (50, _) => "Apo-Summicron-M 50mm f/2 ASPH.",
        (51, _) => "Noctilux-M 75mm f/1.25 ASPH.",
        (52, _) => "Apo-Summicron-M 35mm f/2 ASPH.",
        (53, _) => "Summilux-M 28mm f/1.4 ASPH.",
        (54, _) => "Summilux-M 90mm f/1.5 ASPH.",
        _ => return format!("{} {}", lens_id, frame_selector),
    };

    lens_name.to_string()
}

/// Parse Leica maker notes
/// Parse Leica maker notes
///
/// # Arguments
/// * `data` - The maker note data (contents of MakerNote tag)
/// * `endian` - Byte order
/// * `model` - Camera model string (used to determine format)
/// * `tiff_data` - Optional full TIFF/EXIF data (not used for Leica)
/// * `tiff_offset` - Offset of TIFF header (not used for Leica)
pub fn parse_leica_maker_notes(
    data: &[u8],
    endian: Endianness,
    _model: Option<&str>,
    _tiff_data: Option<&[u8]>,
    _tiff_offset: usize,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    if data.len() < 2 {
        return Ok(tags);
    }

    let mut cursor = Cursor::new(data);

    // Read number of entries
    let num_entries = match endian {
        Endianness::Little => cursor
            .read_u16::<LittleEndian>()
            .map_err(|_| ExifError::Format("Failed to read Leica IFD entry count".to_string()))?,
        Endianness::Big => cursor
            .read_u16::<BigEndian>()
            .map_err(|_| ExifError::Format("Failed to read Leica IFD entry count".to_string()))?,
    };

    // Sanity check: IFD should have reasonable number of entries
    if num_entries == 0 || num_entries > 500 {
        return Ok(tags);
    }

    // Parse IFD entries
    for _ in 0..num_entries {
        if cursor.position() as usize + 12 > data.len() {
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
            let tag_name = get_leica_tag_name(tag_id);

            // Parse value based on tag type
            let value = parse_tag_value(tag_id, tag_type, count, value_offset, data, endian);

            if let Some(val) = value {
                tags.insert(tag_id, MakerNoteTag::new(tag_id, tag_name, val));
            }
        }
    }

    Ok(tags)
}

/// Parse a single tag value
fn parse_tag_value(
    tag_id: u16,
    tag_type: u16,
    count: u32,
    value_offset: u32,
    data: &[u8],
    endian: Endianness,
) -> Option<ExifValue> {
    // Calculate value size in bytes
    let value_size = match tag_type {
        1 => count as usize,      // BYTE
        2 => count as usize,      // ASCII
        3 => count as usize * 2,  // SHORT
        4 => count as usize * 4,  // LONG
        5 => count as usize * 8,  // RATIONAL
        9 => count as usize * 4,  // SLONG
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
        // Value at offset (relative to beginning of maker note data)
        let abs_offset = value_offset as usize;
        if abs_offset + value_size <= data.len() {
            data[abs_offset..abs_offset + value_size].to_vec()
        } else {
            return None;
        }
    };

    match tag_type {
        2 => {
            // ASCII - convert to string and decode specific tags
            let s = String::from_utf8_lossy(&value_bytes[..(count as usize).min(value_bytes.len())])
                .trim_end_matches('\0')
                .to_string();

            match tag_id {
                LEICA5_INTERNAL_SERIAL_NUMBER if s.len() >= 11 => {
                    // Format: "AAAYYMMDDN NNN" -> "(AAA) YYYY:MM:DD no. NNNN"
                    let prefix = &s[0..3];
                    if let (Ok(yy), Ok(mm), Ok(dd), Ok(num)) = (
                        s[3..5].parse::<u32>(),
                        s[5..7].parse::<u32>(),
                        s[7..9].parse::<u32>(),
                        s[9..].parse::<u32>(),
                    ) {
                        let year = if yy < 70 { 2000 + yy } else { 1900 + yy };
                        Some(ExifValue::Ascii(format!(
                            "({}) {}:{:02}:{:02} no. {}",
                            prefix, year, mm, dd, num
                        )))
                    } else {
                        Some(ExifValue::Ascii(s))
                    }
                }
                _ => Some(ExifValue::Ascii(s)),
            }
        }
        3 => {
            // SHORT
            if count == 1 {
                let val = match endian {
                    Endianness::Little => u16::from_le_bytes([value_bytes[0], value_bytes[1]]),
                    Endianness::Big => u16::from_be_bytes([value_bytes[0], value_bytes[1]]),
                };

                match tag_id {
                    LEICA_QUALITY => Some(ExifValue::Ascii(decode_quality_exiftool(val).to_string())),
                    LEICA_USER_PROFILE => Some(ExifValue::Ascii(decode_user_profile_exiftool(val).to_string())),
                    LEICA_WHITE_BALANCE => {
                        if val >= 0x8000 {
                            let kelvin = (val as u32) * 10000 / 65536 + 2500;
                            Some(ExifValue::Ascii(format!("{} K", kelvin)))
                        } else {
                            Some(ExifValue::Ascii(decode_white_balance_exiftool(val).to_string()))
                        }
                    }
                    LEICA_UV_IR_FILTER_CORRECTION => {
                        Some(ExifValue::Ascii(decode_uv_ir_filter_correction_exiftool(val).to_string()))
                    }
                    _ => Some(ExifValue::Short(vec![val])),
                }
            } else {
                Some(ExifValue::Short(vec![])) // Multiple values - not implemented for MVP
            }
        }
        4 => {
            // LONG
            if count == 1 {
                let val = match endian {
                    Endianness::Little => u32::from_le_bytes([
                        value_bytes[0],
                        value_bytes[1],
                        value_bytes[2],
                        value_bytes[3],
                    ]),
                    Endianness::Big => u32::from_be_bytes([
                        value_bytes[0],
                        value_bytes[1],
                        value_bytes[2],
                        value_bytes[3],
                    ]),
                };

                match tag_id {
                    LEICA_SERIAL_NUMBER | LEICA5_SERIAL_NUMBER => {
                        Some(ExifValue::Ascii(format!("{:07}", val)))
                    }
                    LEICA_LENS_TYPE => Some(ExifValue::Ascii(decode_lens_type(val))),
                    _ => Some(ExifValue::Long(vec![val])),
                }
            } else {
                Some(ExifValue::Long(vec![])) // Multiple values - not implemented for MVP
            }
        }
        9 => {
            // SLONG
            if count == 1 {
                let val = match endian {
                    Endianness::Little => i32::from_le_bytes([
                        value_bytes[0],
                        value_bytes[1],
                        value_bytes[2],
                        value_bytes[3],
                    ]),
                    Endianness::Big => i32::from_be_bytes([
                        value_bytes[0],
                        value_bytes[1],
                        value_bytes[2],
                        value_bytes[3],
                    ]),
                };

                match tag_id {
                    LEICA_CAMERA_TEMPERATURE => Some(ExifValue::Ascii(format!("{} C", val))),
                    _ => Some(ExifValue::SLong(vec![val])),
                }
            } else {
                Some(ExifValue::SLong(vec![])) // Multiple values - not implemented for MVP
            }
        }
        5 => {
            // RATIONAL
            if count == 1 && value_bytes.len() >= 8 {
                let num = match endian {
                    Endianness::Little => u32::from_le_bytes([
                        value_bytes[0],
                        value_bytes[1],
                        value_bytes[2],
                        value_bytes[3],
                    ]),
                    Endianness::Big => u32::from_be_bytes([
                        value_bytes[0],
                        value_bytes[1],
                        value_bytes[2],
                        value_bytes[3],
                    ]),
                };
                let den = match endian {
                    Endianness::Little => u32::from_le_bytes([
                        value_bytes[4],
                        value_bytes[5],
                        value_bytes[6],
                        value_bytes[7],
                    ]),
                    Endianness::Big => u32::from_be_bytes([
                        value_bytes[4],
                        value_bytes[5],
                        value_bytes[6],
                        value_bytes[7],
                    ]),
                };
                Some(ExifValue::Rational(vec![(num, den)]))
            } else {
                Some(ExifValue::Rational(vec![])) // Multiple values - not implemented for MVP
            }
        }
        1 => {
            // BYTE
            if tag_id == LEICA5_EXPOSURE_MODE && value_bytes.len() >= 4 {
                Some(ExifValue::Ascii(decode_exposure_mode_exiftool(&value_bytes[0..4]).to_string()))
            } else {
                Some(ExifValue::Byte(value_bytes[..(count as usize).min(value_bytes.len())].to_vec()))
            }
        }
        _ => {
            // Unknown type
            None
        }
    }
}
