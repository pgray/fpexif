// makernotes/canon.rs - Canon maker notes parsing

use crate::data_types::{Endianness, ExifValue};
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Cursor;

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
        CANON_FILE_NUMBER => Some("CanonFileNumber"),
        CANON_OWNER_NAME => Some("OwnerName"),
        CANON_SERIAL_NUMBER => Some("InternalSerialNumber"),
        CANON_CAMERA_INFO => Some("CanonCameraInfo"),
        CANON_FILE_LENGTH => Some("CanonFileLength"),
        CANON_CUSTOM_FUNCTIONS => Some("CanonCustomFunctions"),
        CANON_MODEL_ID => Some("CanonModelID"),
        CANON_MOVIE_INFO => Some("CanonMovieInfo"),
        CANON_AF_INFO => Some("CanonAFInfo"),
        CANON_LENS_MODEL => Some("LensModel"),
        CANON_SERIAL_INFO => Some("SerialNumberFormat"),
        CANON_PICTURE_STYLE_USER_DEF => Some("PictureStyleUserDef"),
        CANON_PICTURE_STYLE_PC => Some("PictureStylePC"),
        CANON_LENS_INFO => Some("LensInfo"),
        CANON_COLOR_SPACE => Some("CanonColorSpace"),
        _ => None,
    }
}

/// Parse Canon maker notes
pub fn parse_canon_maker_notes(
    data: &[u8],
    endian: Endianness,
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
            .map_err(|_| ExifError::Format("Failed to read Canon maker note count".to_string()))?,
        Endianness::Big => cursor
            .read_u16::<BigEndian>()
            .map_err(|_| ExifError::Format("Failed to read Canon maker note count".to_string()))?,
    };

    // Canon maker notes use standard TIFF IFD format
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

        if let (Some(tag_id), Some(tag_type), Some(count), Some(_value_offset)) =
            (tag_id, tag_type, count, value_offset)
        {
            // For now, we'll store basic info
            // Full parsing of nested structures would go here
            let value = match tag_type {
                2 => {
                    // ASCII string
                    ExifValue::Ascii(format!("Canon tag 0x{:04X}", tag_id))
                }
                3 => {
                    // SHORT
                    ExifValue::Short(vec![0])
                }
                4 => {
                    // LONG
                    ExifValue::Long(vec![count])
                }
                _ => ExifValue::Undefined(vec![]),
            };

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
