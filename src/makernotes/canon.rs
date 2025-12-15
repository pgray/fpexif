// makernotes/canon.rs - Canon maker notes parsing

use crate::data_types::{Endianness, ExifValue};
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use std::collections::HashMap;

// Canon MakerNote tag IDs
pub const CANON_CAMERA_SETTINGS: u16 = 0x0001;
pub const CANON_FOCAL_LENGTH: u16 = 0x0002;
pub const CANON_FLASH_INFO: u16 = 0x0003;
pub const CANON_SHOT_INFO: u16 = 0x0004;
pub const CANON_PANORAMA: u16 = 0x0005;
pub const CANON_IMAGE_TYPE: u16 = 0x0006;
pub const CANON_FIRMWARE_VERSION: u16 = 0x0007;
pub const CANON_FILE_NUMBER: u16 = 0x0008;
pub const CANON_OWNER_NAME: u16 = 0x0009;
pub const CANON_SERIAL_NUMBER: u16 = 0x000C;
pub const CANON_CAMERA_INFO: u16 = 0x000D;
pub const CANON_FILE_LENGTH: u16 = 0x000E;
pub const CANON_CUSTOM_FUNCTIONS: u16 = 0x000F;
pub const CANON_MODEL_ID: u16 = 0x0010;
pub const CANON_MOVIE_INFO: u16 = 0x0011;
pub const CANON_AF_INFO: u16 = 0x0012;
pub const CANON_THUMBNAIL_IMAGE_VALID_AREA: u16 = 0x0013;
pub const CANON_SERIAL_NUMBER_FORMAT: u16 = 0x0015;
pub const CANON_SUPER_MACRO: u16 = 0x001A;
pub const CANON_DATE_STAMP_MODE: u16 = 0x001C;
pub const CANON_MY_COLORS: u16 = 0x001D;
pub const CANON_FIRMWARE_REVISION: u16 = 0x001E;
pub const CANON_CATEGORIES: u16 = 0x0023;
pub const CANON_FACE_DETECT: u16 = 0x0024;
pub const CANON_FACE_DETECT_2: u16 = 0x0025;
pub const CANON_AF_INFO_2: u16 = 0x0026;
pub const CANON_CONTRAST_INFO: u16 = 0x0027;
pub const CANON_IMAGE_UNIQUE_ID: u16 = 0x0028;
pub const CANON_WB_INFO: u16 = 0x0029;
pub const CANON_FACE_DETECT_3: u16 = 0x002F;
pub const CANON_TIME_INFO: u16 = 0x0035;
pub const CANON_BATTERY_TYPE: u16 = 0x0038;
pub const CANON_AF_INFO_3: u16 = 0x003C;
pub const CANON_RAW_DATA_OFFSET: u16 = 0x0081;
pub const CANON_ORIGINAL_DECISION_DATA_OFFSET: u16 = 0x0083;
pub const CANON_PERSONAL_FUNCTIONS: u16 = 0x0090;
pub const CANON_PERSONAL_FUNCTION_VALUES: u16 = 0x0091;
pub const CANON_FILE_INFO: u16 = 0x0093;
pub const CANON_AF_POINTS_IN_FOCUS_1D: u16 = 0x0094;
pub const CANON_LENS_MODEL: u16 = 0x0095;
pub const CANON_SERIAL_INFO: u16 = 0x0096;
pub const CANON_DUST_REMOVAL_DATA: u16 = 0x0097;
pub const CANON_CROP_INFO: u16 = 0x0098;
pub const CANON_CUSTOM_FUNCTIONS_2: u16 = 0x0099;
pub const CANON_ASPECT_INFO: u16 = 0x009A;
pub const CANON_PROCESSING_INFO: u16 = 0x00A0;
pub const CANON_TONE_CURVE_TABLE: u16 = 0x00A1;
pub const CANON_SHARPNESS_TABLE: u16 = 0x00A2;
pub const CANON_SHARPNESS_FREQ_TABLE: u16 = 0x00A3;
pub const CANON_WHITE_BALANCE_TABLE: u16 = 0x00A4;
pub const CANON_COLOR_BALANCE: u16 = 0x00A9;
pub const CANON_MEASURED_COLOR: u16 = 0x00AA;
pub const CANON_COLOR_TEMPERATURE: u16 = 0x00AE;
pub const CANON_CANON_FLAGS: u16 = 0x00B0;
pub const CANON_MODIFIED_INFO: u16 = 0x00B1;
pub const CANON_TONE_CURVE_MATCHING: u16 = 0x00B2;
pub const CANON_WHITE_BALANCE_MATCHING: u16 = 0x00B3;
pub const CANON_COLOR_SPACE: u16 = 0x00B4;
pub const CANON_PREVIEW_IMAGE_INFO: u16 = 0x00B6;
pub const CANON_VRD_OFFSET: u16 = 0x00D0;
pub const CANON_SENSOR_INFO: u16 = 0x00E0;
pub const CANON_COLOR_DATA: u16 = 0x4001;
pub const CANON_CRWPARAM: u16 = 0x4002;
pub const CANON_COLOR_INFO: u16 = 0x4003;
pub const CANON_FLAVOR: u16 = 0x4005;
pub const CANON_PICTURE_STYLE_USER_DEF: u16 = 0x4008;
pub const CANON_PICTURE_STYLE_PC: u16 = 0x4009;
pub const CANON_CUSTOM_PICTURE_STYLE_FILE_NAME: u16 = 0x4010;
pub const CANON_AF_MICRO_ADJ: u16 = 0x4013;
pub const CANON_VIGNETTING_CORR: u16 = 0x4015;
pub const CANON_VIGNETTING_CORR_2: u16 = 0x4016;
pub const CANON_LIGHTING_OPT: u16 = 0x4018;
pub const CANON_LENS_INFO: u16 = 0x4019;
pub const CANON_AMBIANCE_INFO: u16 = 0x4020;
pub const CANON_MULTI_EXP: u16 = 0x4021;
pub const CANON_FILTER_INFO: u16 = 0x4024;
pub const CANON_HDR_INFO: u16 = 0x4025;
pub const CANON_AF_CONFIG: u16 = 0x4028;

/// Get the name of a Canon MakerNote tag
pub fn get_canon_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        CANON_CAMERA_SETTINGS => Some("CanonCameraSettings"),
        CANON_FOCAL_LENGTH => Some("CanonFocalLength"),
        CANON_FLASH_INFO => Some("CanonFlashInfo"),
        CANON_SHOT_INFO => Some("CanonShotInfo"),
        CANON_PANORAMA => Some("CanonPanorama"),
        CANON_IMAGE_TYPE => Some("CanonImageType"),
        CANON_FIRMWARE_VERSION => Some("CanonFirmwareVersion"),
        CANON_FILE_NUMBER => Some("FileNumber"),
        CANON_OWNER_NAME => Some("OwnerName"),
        CANON_SERIAL_NUMBER => Some("SerialNumber"),
        CANON_CAMERA_INFO => Some("CanonCameraInfo"),
        CANON_FILE_LENGTH => Some("FileLength"),
        CANON_CUSTOM_FUNCTIONS => Some("CanonCustomFunctions"),
        CANON_MODEL_ID => Some("CanonModelID"),
        CANON_MOVIE_INFO => Some("CanonMovieInfo"),
        CANON_AF_INFO => Some("CanonAFInfo"),
        CANON_THUMBNAIL_IMAGE_VALID_AREA => Some("ThumbnailImageValidArea"),
        CANON_SERIAL_NUMBER_FORMAT => Some("SerialNumberFormat"),
        CANON_SUPER_MACRO => Some("SuperMacro"),
        CANON_DATE_STAMP_MODE => Some("DateStampMode"),
        CANON_MY_COLORS => Some("MyColors"),
        CANON_FIRMWARE_REVISION => Some("FirmwareRevision"),
        CANON_CATEGORIES => Some("Categories"),
        CANON_FACE_DETECT => Some("FaceDetect1"),
        CANON_FACE_DETECT_2 => Some("FaceDetect2"),
        CANON_AF_INFO_2 => Some("AFInfo2"),
        CANON_CONTRAST_INFO => Some("ContrastInfo"),
        CANON_IMAGE_UNIQUE_ID => Some("ImageUniqueID"),
        CANON_WB_INFO => Some("WBInfo"),
        CANON_FACE_DETECT_3 => Some("FaceDetect3"),
        CANON_TIME_INFO => Some("TimeInfo"),
        CANON_BATTERY_TYPE => Some("BatteryType"),
        CANON_AF_INFO_3 => Some("AFInfo3"),
        CANON_RAW_DATA_OFFSET => Some("RawDataOffset"),
        CANON_ORIGINAL_DECISION_DATA_OFFSET => Some("OriginalDecisionDataOffset"),
        CANON_PERSONAL_FUNCTIONS => Some("PersonalFunctions"),
        CANON_PERSONAL_FUNCTION_VALUES => Some("PersonalFunctionValues"),
        CANON_FILE_INFO => Some("FileInfo"),
        CANON_AF_POINTS_IN_FOCUS_1D => Some("AFPointsInFocus1D"),
        CANON_LENS_MODEL => Some("LensModel"),
        CANON_SERIAL_INFO => Some("InternalSerialNumber"),
        CANON_DUST_REMOVAL_DATA => Some("DustRemovalData"),
        CANON_CROP_INFO => Some("CropInfo"),
        CANON_CUSTOM_FUNCTIONS_2 => Some("CustomFunctions2"),
        CANON_ASPECT_INFO => Some("AspectInfo"),
        CANON_PROCESSING_INFO => Some("ProcessingInfo"),
        CANON_TONE_CURVE_TABLE => Some("ToneCurveTable"),
        CANON_SHARPNESS_TABLE => Some("SharpnessTable"),
        CANON_SHARPNESS_FREQ_TABLE => Some("SharpnessFreqTable"),
        CANON_WHITE_BALANCE_TABLE => Some("WhiteBalanceTable"),
        CANON_COLOR_BALANCE => Some("ColorBalance"),
        CANON_MEASURED_COLOR => Some("MeasuredColor"),
        CANON_COLOR_TEMPERATURE => Some("ColorTemperature"),
        CANON_CANON_FLAGS => Some("CanonFlags"),
        CANON_MODIFIED_INFO => Some("ModifiedInfo"),
        CANON_TONE_CURVE_MATCHING => Some("ToneCurveMatching"),
        CANON_WHITE_BALANCE_MATCHING => Some("WhiteBalanceMatching"),
        CANON_COLOR_SPACE => Some("ColorSpace"),
        CANON_PREVIEW_IMAGE_INFO => Some("PreviewImageInfo"),
        CANON_VRD_OFFSET => Some("VRDOffset"),
        CANON_SENSOR_INFO => Some("SensorInfo"),
        CANON_COLOR_DATA => Some("ColorData"),
        CANON_CRWPARAM => Some("CRWParam"),
        CANON_COLOR_INFO => Some("ColorInfo"),
        CANON_FLAVOR => Some("Flavor"),
        CANON_PICTURE_STYLE_USER_DEF => Some("PictureStyleUserDef"),
        CANON_PICTURE_STYLE_PC => Some("PictureStylePC"),
        CANON_CUSTOM_PICTURE_STYLE_FILE_NAME => Some("CustomPictureStyleFileName"),
        CANON_AF_MICRO_ADJ => Some("AFMicroAdj"),
        CANON_VIGNETTING_CORR => Some("VignettingCorr"),
        CANON_VIGNETTING_CORR_2 => Some("VignettingCorr2"),
        CANON_LIGHTING_OPT => Some("LightingOpt"),
        CANON_LENS_INFO => Some("LensInfo"),
        CANON_AMBIANCE_INFO => Some("AmbianceInfo"),
        CANON_MULTI_EXP => Some("MultiExp"),
        CANON_FILTER_INFO => Some("FilterInfo"),
        CANON_HDR_INFO => Some("HDRInfo"),
        CANON_AF_CONFIG => Some("AFConfig"),
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

/// Read i16 with given endianness
fn read_i16(data: &[u8], endian: Endianness) -> i16 {
    match endian {
        Endianness::Little => i16::from_le_bytes([data[0], data[1]]),
        Endianness::Big => i16::from_be_bytes([data[0], data[1]]),
    }
}

/// Read i32 with given endianness
fn read_i32(data: &[u8], endian: Endianness) -> i32 {
    match endian {
        Endianness::Little => i32::from_le_bytes([data[0], data[1], data[2], data[3]]),
        Endianness::Big => i32::from_be_bytes([data[0], data[1], data[2], data[3]]),
    }
}

/// Parse a single IFD entry and return the tag value
///
/// Canon maker notes use offsets relative to the TIFF header, not the MakerNote data.
/// If tiff_data is provided, we use it to resolve offsets; otherwise we try the MakerNote data.
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
    // Canon maker notes use offsets relative to the TIFF header
    let value_data: &[u8] = if total_size <= 4 {
        value_offset_bytes
    } else {
        let offset = read_u32(value_offset_bytes, endian) as usize;

        // Try to use TIFF data first (Canon uses TIFF-relative offsets)
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
            let s = value_data[..count.min(value_data.len())]
                .iter()
                .take_while(|&&b| b != 0)
                .map(|&b| b as char)
                .collect::<String>();
            ExifValue::Ascii(s)
        }
        3 => {
            // SHORT
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                if i * 2 + 2 <= value_data.len() {
                    values.push(read_u16(&value_data[i * 2..], endian));
                }
            }
            ExifValue::Short(values)
        }
        4 => {
            // LONG
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                if i * 4 + 4 <= value_data.len() {
                    values.push(read_u32(&value_data[i * 4..], endian));
                }
            }
            ExifValue::Long(values)
        }
        5 => {
            // RATIONAL
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
        6 => {
            // SBYTE
            ExifValue::SByte(
                value_data[..count.min(value_data.len())]
                    .iter()
                    .map(|&b| b as i8)
                    .collect(),
            )
        }
        7 => {
            // UNDEFINED
            ExifValue::Undefined(value_data[..count.min(value_data.len())].to_vec())
        }
        8 => {
            // SSHORT
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                if i * 2 + 2 <= value_data.len() {
                    values.push(read_i16(&value_data[i * 2..], endian));
                }
            }
            ExifValue::SShort(values)
        }
        9 => {
            // SLONG
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                if i * 4 + 4 <= value_data.len() {
                    values.push(read_i32(&value_data[i * 4..], endian));
                }
            }
            ExifValue::SLong(values)
        }
        10 => {
            // SRATIONAL
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                if i * 8 + 8 <= value_data.len() {
                    let num = read_i32(&value_data[i * 8..], endian);
                    let den = read_i32(&value_data[i * 8 + 4..], endian);
                    values.push((num, den));
                }
            }
            ExifValue::SRational(values)
        }
        _ => return None,
    };

    Some((tag_id, value))
}

/// Decode Canon CameraSettings array sub-fields
///
/// This function decodes the CanonCameraSettings array into individual fields
/// that can be easily interpreted.
pub fn decode_camera_settings(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // Macro mode (index 1)
    if data.len() > 1 {
        let macro_mode = match data[1] {
            1 => "Macro",
            2 => "Normal",
            _ => "Unknown",
        };
        decoded.insert(
            "MacroMode".to_string(),
            ExifValue::Ascii(macro_mode.to_string()),
        );
    }

    // Self timer (index 2)
    if data.len() > 2 {
        decoded.insert("SelfTimer".to_string(), ExifValue::Short(vec![data[2]]));
    }

    // Quality (index 3)
    if data.len() > 3 {
        let quality = match data[3] {
            1 => "Economy",
            2 => "Normal",
            3 => "Fine",
            4 => "RAW",
            5 => "Superfine",
            130 => "Normal Movie",
            _ => "Unknown",
        };
        decoded.insert("Quality".to_string(), ExifValue::Ascii(quality.to_string()));
    }

    // Flash mode (index 4)
    if data.len() > 4 {
        let flash_mode = match data[4] {
            0 => "Flash Not Fired",
            1 => "Auto",
            2 => "On",
            3 => "Red-eye reduction",
            4 => "Slow-sync",
            5 => "Auto + Red-eye reduction",
            6 => "On + Red-eye reduction",
            16 => "External flash",
            _ => "Unknown",
        };
        decoded.insert(
            "FlashMode".to_string(),
            ExifValue::Ascii(flash_mode.to_string()),
        );
    }

    // Drive mode (index 5)
    if data.len() > 5 {
        let drive_mode = match data[5] {
            0 => "Single",
            1 => "Continuous",
            2 => "Movie",
            3 => "Continuous, Speed Priority",
            4 => "Continuous, Low",
            5 => "Continuous, High",
            6 => "Silent Single",
            9 => "Single, Silent",
            10 => "Continuous, Silent",
            _ => "Unknown",
        };
        decoded.insert(
            "DriveMode".to_string(),
            ExifValue::Ascii(drive_mode.to_string()),
        );
    }

    // Focus mode (index 7)
    if data.len() > 7 {
        let focus_mode = match data[7] {
            0 => "One-shot AF",
            1 => "AI Servo AF",
            2 => "AI Focus AF",
            3 => "Manual Focus",
            4 => "Single",
            5 => "Continuous",
            6 => "Manual Focus",
            16 => "Pan Focus",
            256 => "Manual",
            _ => "Unknown",
        };
        decoded.insert(
            "FocusMode".to_string(),
            ExifValue::Ascii(focus_mode.to_string()),
        );
    }

    // Metering mode (index 17)
    if data.len() > 17 {
        let metering_mode = match data[17] {
            3 => "Evaluative",
            4 => "Partial",
            5 => "Center-weighted average",
            _ => "Unknown",
        };
        decoded.insert(
            "MeteringMode".to_string(),
            ExifValue::Ascii(metering_mode.to_string()),
        );
    }

    // Focus range (index 18)
    if data.len() > 18 {
        let focus_range = match data[18] {
            0 => "Manual",
            1 => "Auto",
            2 => "Not Known",
            3 => "Macro",
            7 => "Very Close",
            8 => "Close",
            9 => "Middle Range",
            10 => "Far Range",
            11 => "Pan Focus",
            14 => "Super Macro",
            15 => "Infinity",
            _ => "Unknown",
        };
        decoded.insert(
            "FocusRange".to_string(),
            ExifValue::Ascii(focus_range.to_string()),
        );
    }

    // Exposure mode (index 20)
    if data.len() > 20 {
        let exposure_mode = match data[20] {
            0 => "Easy",
            1 => "Program AE",
            2 => "Shutter speed priority AE",
            3 => "Aperture-priority AE",
            4 => "Manual",
            5 => "Depth-of-field AE",
            6 => "M-Dep",
            7 => "Bulb",
            _ => "Unknown",
        };
        decoded.insert(
            "ExposureMode".to_string(),
            ExifValue::Ascii(exposure_mode.to_string()),
        );
    }

    // Max focal length (index 23)
    if data.len() > 23 {
        decoded.insert(
            "MaxFocalLength".to_string(),
            ExifValue::Short(vec![data[23]]),
        );
    }

    // Min focal length (index 24)
    if data.len() > 24 {
        decoded.insert(
            "MinFocalLength".to_string(),
            ExifValue::Short(vec![data[24]]),
        );
    }

    // Focal units per mm (index 25)
    if data.len() > 25 {
        decoded.insert(
            "FocalUnitsPerMM".to_string(),
            ExifValue::Short(vec![data[25]]),
        );
    }

    // Max aperture (index 26)
    if data.len() > 26 {
        decoded.insert("MaxAperture".to_string(), ExifValue::Short(vec![data[26]]));
    }

    // Min aperture (index 27)
    if data.len() > 27 {
        decoded.insert("MinAperture".to_string(), ExifValue::Short(vec![data[27]]));
    }

    decoded
}

/// Decode Canon ShotInfo array sub-fields
///
/// This function decodes the CanonShotInfo array into individual fields
/// that can be easily interpreted.
pub fn decode_shot_info(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // Auto ISO (index 1)
    if data.len() > 1 {
        decoded.insert("AutoISO".to_string(), ExifValue::Short(vec![data[1]]));
    }

    // Base ISO (index 2)
    if data.len() > 2 {
        decoded.insert("BaseISO".to_string(), ExifValue::Short(vec![data[2]]));
    }

    // Measured EV (index 3)
    if data.len() > 3 {
        decoded.insert("MeasuredEV".to_string(), ExifValue::Short(vec![data[3]]));
    }

    // Target aperture (index 4)
    if data.len() > 4 {
        decoded.insert(
            "TargetAperture".to_string(),
            ExifValue::Short(vec![data[4]]),
        );
    }

    // Target exposure time (index 5)
    if data.len() > 5 {
        decoded.insert(
            "TargetExposureTime".to_string(),
            ExifValue::Short(vec![data[5]]),
        );
    }

    // White balance (index 7)
    if data.len() > 7 {
        let wb = match data[7] {
            0 => "Auto",
            1 => "Daylight",
            2 => "Cloudy",
            3 => "Tungsten",
            4 => "Fluorescent",
            5 => "Flash",
            6 => "Custom",
            7 => "Black & White",
            8 => "Shade",
            9 => "Manual Temperature",
            10 => "PC Set1",
            11 => "PC Set2",
            12 => "PC Set3",
            14 => "Daylight Fluorescent",
            15 => "Custom 1",
            16 => "Custom 2",
            17 => "Underwater",
            _ => "Unknown",
        };
        decoded.insert("WhiteBalance".to_string(), ExifValue::Ascii(wb.to_string()));
    }

    // Sequence number (index 9)
    if data.len() > 9 {
        decoded.insert(
            "SequenceNumber".to_string(),
            ExifValue::Short(vec![data[9]]),
        );
    }

    // Flash exposure compensation (index 15)
    if data.len() > 15 {
        decoded.insert(
            "FlashExposureComp".to_string(),
            ExifValue::Short(vec![data[15]]),
        );
    }

    // Auto exposure bracketing (index 16)
    if data.len() > 16 {
        let aeb = match data[16] {
            0xFFFF | 0 => "Off",
            1 => "On",
            _ => "Unknown",
        };
        decoded.insert(
            "AEBBracketValue".to_string(),
            ExifValue::Ascii(aeb.to_string()),
        );
    }

    // Subject distance (index 19)
    if data.len() > 19 {
        decoded.insert(
            "SubjectDistance".to_string(),
            ExifValue::Short(vec![data[19]]),
        );
    }

    decoded
}

/// Decode Canon FocalLength array sub-fields
///
/// This function decodes the CanonFocalLength array into individual fields
/// that can be easily interpreted.
pub fn decode_focal_length(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // Focal type (index 0)
    if !data.is_empty() {
        let focal_type = match data[0] {
            1 => "Fixed",
            2 => "Zoom",
            _ => "Unknown",
        };
        decoded.insert(
            "FocalType".to_string(),
            ExifValue::Ascii(focal_type.to_string()),
        );
    }

    // Focal length (index 1)
    if data.len() > 1 {
        decoded.insert("FocalLength".to_string(), ExifValue::Short(vec![data[1]]));
    }

    // Focal plane X size (index 2)
    if data.len() > 2 {
        decoded.insert(
            "FocalPlaneXSize".to_string(),
            ExifValue::Short(vec![data[2]]),
        );
    }

    // Focal plane Y size (index 3)
    if data.len() > 3 {
        decoded.insert(
            "FocalPlaneYSize".to_string(),
            ExifValue::Short(vec![data[3]]),
        );
    }

    decoded
}

/// Decode Canon FileInfo array sub-fields
///
/// This function decodes the CanonFileInfo array into individual fields
/// that can be easily interpreted.
pub fn decode_file_info(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // File number (index 1)
    if data.len() > 1 {
        decoded.insert("FileNumber".to_string(), ExifValue::Short(vec![data[1]]));
    }

    // Bracket mode (index 3)
    if data.len() > 3 {
        let bracket_mode = match data[3] {
            0 => "Off",
            1 => "AEB",
            2 => "FEB",
            3 => "ISO",
            4 => "WB",
            _ => "Unknown",
        };
        decoded.insert(
            "BracketMode".to_string(),
            ExifValue::Ascii(bracket_mode.to_string()),
        );
    }

    // Bracket value (index 4)
    if data.len() > 4 {
        decoded.insert("BracketValue".to_string(), ExifValue::Short(vec![data[4]]));
    }

    // Bracket shot number (index 5)
    if data.len() > 5 {
        decoded.insert(
            "BracketShotNumber".to_string(),
            ExifValue::Short(vec![data[5]]),
        );
    }

    decoded
}

/// Parse Canon maker notes
///
/// Canon maker notes use TIFF-relative offsets, so we need access to the full
/// TIFF/EXIF data to properly resolve string and array values.
pub fn parse_canon_maker_notes(
    data: &[u8],
    endian: Endianness,
    tiff_data: Option<&[u8]>,
    tiff_offset: usize,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    if data.len() < 2 {
        return Ok(tags);
    }

    // Canon maker notes use standard TIFF IFD format starting immediately
    // Read number of entries
    let num_entries = read_u16(&data[0..2], endian) as usize;

    // Sanity check
    if num_entries > 500 || 2 + num_entries * 12 > data.len() {
        return Ok(tags);
    }

    // Parse each IFD entry (12 bytes each)
    for i in 0..num_entries {
        let entry_offset = 2 + i * 12;

        if let Some((tag_id, value)) =
            parse_ifd_entry(data, entry_offset, endian, tiff_data, tiff_offset)
        {
            // Skip large binary blobs to save memory (camera info, dust removal, color data)
            if matches!(
                tag_id,
                CANON_CAMERA_INFO | CANON_DUST_REMOVAL_DATA | CANON_COLOR_DATA
            ) {
                continue;
            }

            tags.insert(
                tag_id,
                MakerNoteTag {
                    tag_id,
                    tag_name: get_canon_tag_name(tag_id),
                    value,
                },
            );
        }
    }

    Ok(tags)
}
