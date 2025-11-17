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

        if let (Some(tag_id), Some(tag_type), Some(count), Some(_value_offset)) =
            (tag_id, tag_type, count, value_offset)
        {
            let value = match tag_type {
                2 => ExifValue::Ascii(format!("Nikon tag 0x{:04X}", tag_id)),
                3 => ExifValue::Short(vec![0]),
                4 => ExifValue::Long(vec![count]),
                _ => ExifValue::Undefined(vec![]),
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
