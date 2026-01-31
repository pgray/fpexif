// makernotes/casio.rs - Casio maker notes parsing
//
// Reference files:
// - exiftool/lib/Image/ExifTool/Casio.pm
// - exiv2/src/casiomn_int.cpp
//
// Casio cameras use two main makernote formats:
// - Type 1: Older cameras (QV-series), direct IFD format
// - Type 2: Newer cameras (EX-series, Exilim), starts with "QVC\0\0\0" header

use crate::data_types::{Endianness, ExifValue};
use crate::define_tag_decoder;
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Cursor;

// exiv2 group names for Casio MakerNotes
// Casio has two main formats: Type1 (older) and Type2 (newer)
pub const EXIV2_GROUP_CASIO: &str = "Casio";
pub const EXIV2_GROUP_CASIO2: &str = "Casio2";

// Casio Type 1 MakerNote Tag IDs (older cameras like QV-series)
pub const CASIO_RECORDING_MODE: u16 = 0x0001;
pub const CASIO_QUALITY: u16 = 0x0002;
pub const CASIO_FOCUS_MODE: u16 = 0x0003;
pub const CASIO_FLASH_MODE: u16 = 0x0004;
pub const CASIO_FLASH_INTENSITY: u16 = 0x0005;
pub const CASIO_OBJECT_DISTANCE: u16 = 0x0006;
pub const CASIO_WHITE_BALANCE: u16 = 0x0007;
pub const CASIO_DIGITAL_ZOOM: u16 = 0x000A;
pub const CASIO_SHARPNESS: u16 = 0x000B;
pub const CASIO_CONTRAST: u16 = 0x000C;
pub const CASIO_SATURATION: u16 = 0x000D;
pub const CASIO_ISO: u16 = 0x0014;
pub const CASIO_FIRMWARE_DATE: u16 = 0x0015;
pub const CASIO_ENHANCEMENT: u16 = 0x0016;
pub const CASIO_COLOR_FILTER: u16 = 0x0017;
pub const CASIO_AF_POINT: u16 = 0x0018;
pub const CASIO_FLASH_INTENSITY2: u16 = 0x0019;

// Casio Type 2 MakerNote Tag IDs (newer cameras like EX-series)
pub const CASIO2_PREVIEW_IMAGE_SIZE: u16 = 0x0002;
pub const CASIO2_PREVIEW_IMAGE_LENGTH: u16 = 0x0003;
pub const CASIO2_PREVIEW_IMAGE_START: u16 = 0x0004;
pub const CASIO2_QUALITY_MODE: u16 = 0x0008;
pub const CASIO2_IMAGE_SIZE: u16 = 0x0009;
pub const CASIO2_FOCUS_MODE: u16 = 0x000D;
pub const CASIO2_ISO_SPEED: u16 = 0x0014;
pub const CASIO2_WHITE_BALANCE: u16 = 0x0019;
pub const CASIO2_FOCAL_LENGTH: u16 = 0x001D;
pub const CASIO2_SATURATION: u16 = 0x001F;
pub const CASIO2_CONTRAST: u16 = 0x0020;
pub const CASIO2_SHARPNESS: u16 = 0x0021;
pub const CASIO2_FIRMWARE_DATE: u16 = 0x2001;
pub const CASIO2_WHITE_BALANCE_BIAS: u16 = 0x2011;
pub const CASIO2_WHITE_BALANCE2: u16 = 0x2012;
pub const CASIO2_AF_POINT_POSITION: u16 = 0x2021;
pub const CASIO2_OBJECT_DISTANCE: u16 = 0x2022;
pub const CASIO2_FLASH_DISTANCE: u16 = 0x2034;
pub const CASIO2_RECORD_MODE: u16 = 0x3000;
pub const CASIO2_RELEASE_MODE: u16 = 0x3001;
pub const CASIO2_QUALITY: u16 = 0x3002;
pub const CASIO2_FOCUS_MODE2: u16 = 0x3003;
pub const CASIO2_BEST_SHOT_MODE: u16 = 0x3007;
pub const CASIO2_AUTO_ISO: u16 = 0x3008;
pub const CASIO2_AF_MODE: u16 = 0x3009;
pub const CASIO2_SHARPNESS2: u16 = 0x3011;
pub const CASIO2_CONTRAST2: u16 = 0x3012;
pub const CASIO2_SATURATION2: u16 = 0x3013;
pub const CASIO2_ISO: u16 = 0x3014;
pub const CASIO2_COLOR_MODE: u16 = 0x3015;
pub const CASIO2_ENHANCEMENT: u16 = 0x3016;
pub const CASIO2_COLOR_FILTER: u16 = 0x3017;
pub const CASIO2_DRIVE_MODE: u16 = 0x3103;

/// Get the name of a Casio Type 1 MakerNote tag
pub fn get_casio_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        CASIO_RECORDING_MODE => Some("RecordingMode"),
        CASIO_QUALITY => Some("Quality"),
        CASIO_FOCUS_MODE => Some("FocusMode"),
        CASIO_FLASH_MODE => Some("FlashMode"),
        CASIO_FLASH_INTENSITY => Some("FlashIntensity"),
        CASIO_OBJECT_DISTANCE => Some("ObjectDistance"),
        CASIO_WHITE_BALANCE => Some("WhiteBalance"),
        CASIO_DIGITAL_ZOOM => Some("DigitalZoom"),
        CASIO_SHARPNESS => Some("Sharpness"),
        CASIO_CONTRAST => Some("Contrast"),
        CASIO_SATURATION => Some("Saturation"),
        CASIO_ISO => Some("ISO"),
        CASIO_FIRMWARE_DATE => Some("FirmwareDate"),
        CASIO_ENHANCEMENT => Some("Enhancement"),
        CASIO_COLOR_FILTER => Some("ColorFilter"),
        CASIO_AF_POINT => Some("AFPoint"),
        CASIO_FLASH_INTENSITY2 => Some("FlashIntensity"),
        _ => None,
    }
}

/// Get the name of a Casio Type 2 MakerNote tag
pub fn get_casio2_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        CASIO2_PREVIEW_IMAGE_SIZE => Some("PreviewImageSize"),
        CASIO2_PREVIEW_IMAGE_LENGTH => Some("PreviewImageLength"),
        CASIO2_PREVIEW_IMAGE_START => Some("PreviewImageStart"),
        CASIO2_QUALITY_MODE => Some("QualityMode"),
        CASIO2_IMAGE_SIZE => Some("ImageSize"),
        CASIO2_FOCUS_MODE => Some("FocusMode"),
        CASIO2_ISO_SPEED => Some("ISOSpeed"),
        CASIO2_WHITE_BALANCE => Some("WhiteBalance"),
        CASIO2_FOCAL_LENGTH => Some("FocalLength"),
        CASIO2_SATURATION => Some("Saturation"),
        CASIO2_CONTRAST => Some("Contrast"),
        CASIO2_SHARPNESS => Some("Sharpness"),
        CASIO2_FIRMWARE_DATE => Some("FirmwareDate"),
        CASIO2_WHITE_BALANCE_BIAS => Some("WhiteBalanceBias"),
        CASIO2_WHITE_BALANCE2 => Some("WhiteBalance"),
        CASIO2_AF_POINT_POSITION => Some("AFPointPosition"),
        CASIO2_OBJECT_DISTANCE => Some("ObjectDistance"),
        CASIO2_FLASH_DISTANCE => Some("FlashDistance"),
        CASIO2_RECORD_MODE => Some("RecordMode"),
        CASIO2_RELEASE_MODE => Some("ReleaseMode"),
        CASIO2_QUALITY => Some("Quality"),
        CASIO2_FOCUS_MODE2 => Some("FocusMode"),
        CASIO2_BEST_SHOT_MODE => Some("BestShotMode"),
        CASIO2_AUTO_ISO => Some("AutoISO"),
        CASIO2_AF_MODE => Some("AFMode"),
        CASIO2_SHARPNESS2 => Some("Sharpness"),
        CASIO2_CONTRAST2 => Some("Contrast"),
        CASIO2_SATURATION2 => Some("Saturation"),
        CASIO2_ISO => Some("ISO"),
        CASIO2_COLOR_MODE => Some("ColorMode"),
        CASIO2_ENHANCEMENT => Some("Enhancement"),
        CASIO2_COLOR_FILTER => Some("ColorFilter"),
        CASIO2_DRIVE_MODE => Some("DriveMode"),
        _ => None,
    }
}

// Casio Type 1 decoders (exiftool: Casio.pm, exiv2: casiomn_int.cpp)

define_tag_decoder! {
    quality,
    both: {
        1 => "Economy",
        2 => "Normal",
        3 => "Fine",
    }
}

define_tag_decoder! {
    focus_mode,
    exiftool: {
        2 => "Macro",
        3 => "Auto",
        4 => "Manual",
        5 => "Infinity",
        7 => "Spot AF",
    },
    exiv2: {
        2 => "Macro",
        3 => "Auto",
        4 => "Manual",
        5 => "Infinity",
        7 => "Sport AF",
    }
}

define_tag_decoder! {
    flash_mode,
    both: {
        1 => "Auto",
        2 => "On",
        3 => "Off",
        4 => "Off",
        5 => "Red-eye Reduction",
    }
}

define_tag_decoder! {
    flash_intensity,
    both: {
        11 => "Weak",
        12 => "Low",
        13 => "Normal",
        14 => "High",
        15 => "Strong",
    }
}

define_tag_decoder! {
    white_balance,
    both: {
        1 => "Auto",
        2 => "Tungsten",
        3 => "Daylight",
        4 => "Fluorescent",
        5 => "Shade",
        129 => "Manual",
    }
}

define_tag_decoder! {
    sharpness,
    both: {
        0 => "Normal",
        1 => "Soft",
        2 => "Hard",
        16 => "Normal",
        17 => "+1",
        18 => "-1",
    }
}

define_tag_decoder! {
    contrast,
    both: {
        0 => "Normal",
        1 => "Low",
        2 => "High",
        16 => "Normal",
        17 => "+1",
        18 => "-1",
    }
}

define_tag_decoder! {
    saturation,
    both: {
        0 => "Normal",
        1 => "Low",
        2 => "High",
        16 => "Normal",
        17 => "+1",
        18 => "-1",
    }
}

define_tag_decoder! {
    enhancement,
    both: {
        1 => "Off",
        2 => "Red",
        3 => "Green",
        4 => "Blue",
        5 => "Flesh Tones",
    }
}

define_tag_decoder! {
    color_filter,
    both: {
        1 => "Off",
        2 => "Black & White",
        3 => "Sepia",
        4 => "Red",
        5 => "Green",
        6 => "Blue",
        7 => "Yellow",
        8 => "Pink",
        9 => "Purple",
    }
}

define_tag_decoder! {
    flash_intensity2,
    both: {
        1 => "Normal",
        2 => "Weak",
        3 => "Strong",
    }
}

// Casio Type 2 decoders

define_tag_decoder! {
    focus_mode2,
    both: {
        0 => "Normal",
        1 => "Macro",
    }
}

define_tag_decoder! {
    white_balance2,
    both: {
        0 => "Auto",
        1 => "Daylight",
        2 => "Shade",
        3 => "Tungsten",
        4 => "Fluorescent",
        5 => "Manual",
    }
}

define_tag_decoder! {
    saturation2,
    both: {
        0 => "Low",
        1 => "Normal",
        2 => "High",
    }
}

define_tag_decoder! {
    contrast2,
    both: {
        0 => "Low",
        1 => "Normal",
        2 => "High",
    }
}

define_tag_decoder! {
    sharpness2,
    both: {
        0 => "Soft",
        1 => "Normal",
        2 => "Hard",
    }
}

define_tag_decoder! {
    white_balance2_alt,
    exiftool: {
        0 => "Manual",
        1 => "Daylight",
        2 => "Cloudy",
        3 => "Shade",
        4 => "Flash?",
        6 => "Fluorescent",
        9 => "Tungsten?",
        10 => "Tungsten",
        12 => "Flash",
    },
    exiv2: {
        0 => "Manual",
        1 => "Daylight",
        2 => "Cloudy",
        3 => "Shade",
        4 => "Flash",
        6 => "Fluorescent",
        9 => "Tungsten",
        10 => "Tungsten",
        12 => "Flash",
    }
}

define_tag_decoder! {
    release_mode,
    both: {
        1 => "Normal",
        3 => "AE Bracketing",
        11 => "WB Bracketing",
        13 => "Contrast Bracketing",
        19 => "High Speed Burst",
    }
}

define_tag_decoder! {
    focus_mode2_alt,
    exiftool: {
        0 => "Manual",
        1 => "Focus Lock",
        2 => "Macro",
        3 => "Single-Area Auto Focus",
        5 => "Infinity",
        6 => "Multi-Area Auto Focus",
        8 => "Super Macro",
    },
    exiv2: {
        0 => "Manual",
        1 => "Focus Lock",
        2 => "Macro",
        3 => "Single-Area Auto Focus",
        5 => "Infinity",
        6 => "Multi-Area Auto Focus",
        8 => "Super Macro",
    }
}

define_tag_decoder! {
    auto_iso,
    exiftool: {
        1 => "On",
        2 => "Off",
        7 => "On (high sensitivity)",
        8 => "On (anti-shake)",
        10 => "High Speed",
    },
    exiv2: {
        1 => "On",
        2 => "Off",
        7 => "On (high sensitivity)",
        8 => "On (anti-shake)",
        10 => "High Speed",
    }
}

define_tag_decoder! {
    af_mode,
    exiftool: {
        0 => "Off",
        1 => "Spot",
        2 => "Multi",
        3 => "Face Detection",
        4 => "Tracking",
        5 => "Intelligent",
    },
    exiv2: {
        0 => "Off",
        1 => "Spot",
        2 => "Multi",
        3 => "Face Detection",
        4 => "Tracking",
        5 => "Intelligent",
    }
}

define_tag_decoder! {
    color_mode,
    both: {
        0 => "Off",
        2 => "Black & White",
        3 => "Sepia",
    }
}

define_tag_decoder! {
    enhancement2,
    exiftool: {
        0 => "Off",
        1 => "Scenery",
        3 => "Green",
        5 => "Underwater",
        9 => "Flesh Tones",
    },
    exiv2: {
        0 => "Off",
        1 => "Scenery",
        3 => "Green",
        5 => "Underwater",
        9 => "Flesh Tones",
    }
}

define_tag_decoder! {
    color_filter2,
    both: {
        0 => "Off",
        1 => "Blue",
        3 => "Green",
        4 => "Yellow",
        5 => "Red",
        6 => "Purple",
        7 => "Pink",
    }
}

define_tag_decoder! {
    drive_mode,
    both: {
        0 => "Single Shot",
        1 => "Continuous Shooting",
        2 => "Continuous (2 fps)",
        3 => "Continuous (3 fps)",
        4 => "Continuous (4 fps)",
        5 => "Continuous (5 fps)",
        6 => "Continuous (6 fps)",
        7 => "Continuous (7 fps)",
        10 => "Continuous (10 fps)",
        12 => "Continuous (12 fps)",
        15 => "Continuous (15 fps)",
        20 => "Continuous (20 fps)",
        30 => "Continuous (30 fps)",
        40 => "Continuous (40 fps)",
        60 => "Continuous (60 fps)",
        240 => "Auto-N",
    }
}

define_tag_decoder! {
    record_mode,
    both: {
        2 => "Program AE",
        3 => "Shutter Priority",
        4 => "Aperture Priority",
        5 => "Manual",
        6 => "Best Shot",
        17 => "Movie",
        19 => "Movie (19)",
        20 => "YouTube Movie",
    }
}

/// Helper function to decode a u16 value from IFD entry
fn decode_u16_from_entry(
    data: &[u8],
    tag_type: u16,
    count: u32,
    value_bytes: &[u8],
    endian: Endianness,
) -> Option<u16> {
    // Type 3 = unsigned short (u16)
    if tag_type != 3 || count == 0 {
        return None;
    }

    // For inline values (<=4 bytes), read from value_bytes
    if count * 2 <= 4 {
        if value_bytes.len() >= 2 {
            let mut cursor = Cursor::new(value_bytes);
            match endian {
                Endianness::Little => cursor.read_u16::<LittleEndian>().ok(),
                Endianness::Big => cursor.read_u16::<BigEndian>().ok(),
            }
        } else {
            None
        }
    } else {
        // For values >4 bytes, value_bytes contains the offset
        if value_bytes.len() >= 4 {
            let mut cursor = Cursor::new(value_bytes);
            let offset = match endian {
                Endianness::Little => cursor.read_u32::<LittleEndian>().ok()?,
                Endianness::Big => cursor.read_u32::<BigEndian>().ok()?,
            } as usize;

            if offset + 2 <= data.len() {
                let mut val_cursor = Cursor::new(&data[offset..offset + 2]);
                match endian {
                    Endianness::Little => val_cursor.read_u16::<LittleEndian>().ok(),
                    Endianness::Big => val_cursor.read_u16::<BigEndian>().ok(),
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Parse Casio maker notes (handles both Type 1 and Type 2 formats)
///
/// Casio has two main formats:
/// - Type 1: Older cameras (QV-series), direct IFD format
/// - Type 2: Newer cameras (EX-series, Exilim), starts with "QVC\0\0\0" header
pub fn parse_casio_maker_notes(
    data: &[u8],
    endian: Endianness,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    if data.len() < 6 {
        return Ok(HashMap::new());
    }

    // Check for Type 2 header "QVC\0\0\0"
    if data.len() >= 6 && &data[0..3] == b"QVC" && data[3] == 0 && data[4] == 0 && data[5] == 0 {
        // Type 2 format - skip 6 byte header
        parse_casio_ifd(&data[6..], endian, true)
    } else {
        // Type 1 format - no header
        parse_casio_ifd(data, endian, false)
    }
}

/// Parse Casio IFD format (used by both Type 1 and Type 2)
fn parse_casio_ifd(
    data: &[u8],
    endian: Endianness,
    is_type2: bool,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    if data.len() < 2 {
        return Ok(tags);
    }

    let mut cursor = Cursor::new(data);

    // Read number of entries
    let entry_count = match endian {
        Endianness::Little => cursor.read_u16::<LittleEndian>(),
        Endianness::Big => cursor.read_u16::<BigEndian>(),
    }
    .ok();

    let entry_count = match entry_count {
        Some(count) if count > 0 && count < 500 => count,
        _ => return Ok(tags),
    };

    // Parse IFD entries
    for _ in 0..entry_count {
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

        // Read the 4-byte value/offset field
        let mut value_bytes = [0u8; 4];
        if std::io::Read::read_exact(&mut cursor, &mut value_bytes).is_err() {
            break;
        }

        if let (Some(tag_id), Some(tag_type), Some(count)) = (tag_id, tag_type, count) {
            let tag_name = if is_type2 {
                get_casio2_tag_name(tag_id)
            } else {
                get_casio_tag_name(tag_id)
            };

            let exiv2_group = if is_type2 {
                EXIV2_GROUP_CASIO2
            } else {
                EXIV2_GROUP_CASIO
            };

            // Handle specific tags based on type
            if is_type2 {
                match tag_id {
                    CASIO2_FOCUS_MODE => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_focus_mode2_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "FocusMode",
                                ),
                            );
                        }
                    }
                    CASIO2_FOCUS_MODE2 => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_focus_mode2_alt_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "FocusMode2",
                                ),
                            );
                        }
                    }
                    CASIO2_WHITE_BALANCE => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_white_balance2_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "WhiteBalance",
                                ),
                            );
                        }
                    }
                    CASIO2_WHITE_BALANCE2 => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_white_balance2_alt_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "WhiteBalance2",
                                ),
                            );
                        }
                    }
                    CASIO2_SATURATION => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_saturation2_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "Saturation",
                                ),
                            );
                        }
                    }
                    CASIO2_CONTRAST => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_contrast2_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "Contrast",
                                ),
                            );
                        }
                    }
                    CASIO2_SHARPNESS => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_sharpness2_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "Sharpness",
                                ),
                            );
                        }
                    }
                    CASIO2_ENHANCEMENT => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_enhancement2_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "Enhancement",
                                ),
                            );
                        }
                    }
                    CASIO2_COLOR_FILTER => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_color_filter2_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "ColorFilter",
                                ),
                            );
                        }
                    }
                    CASIO2_DRIVE_MODE => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_drive_mode_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "DriveMode",
                                ),
                            );
                        }
                    }
                    CASIO2_RECORD_MODE => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_record_mode_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "RecordMode",
                                ),
                            );
                        }
                    }
                    CASIO2_RELEASE_MODE => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_release_mode_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "ReleaseMode",
                                ),
                            );
                        }
                    }
                    CASIO2_QUALITY => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_quality_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "Quality",
                                ),
                            );
                        }
                    }
                    CASIO2_AF_MODE => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_af_mode_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "AFMode",
                                ),
                            );
                        }
                    }
                    _ => {
                        // Generic handling
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            if let Some(name) = tag_name {
                                tags.insert(
                                    tag_id,
                                    MakerNoteTag::new(
                                        tag_id,
                                        Some(name),
                                        ExifValue::Short(vec![val]),
                                    ),
                                );
                            }
                        }
                    }
                }
            } else {
                // Type 1 tags
                match tag_id {
                    CASIO_FOCUS_MODE => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_focus_mode_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "FocusMode",
                                ),
                            );
                        }
                    }
                    CASIO_FLASH_MODE => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_flash_mode_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "FlashMode",
                                ),
                            );
                        }
                    }
                    CASIO_WHITE_BALANCE => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_white_balance_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "WhiteBalance",
                                ),
                            );
                        }
                    }
                    CASIO_SHARPNESS => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_sharpness_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "Sharpness",
                                ),
                            );
                        }
                    }
                    CASIO_CONTRAST => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_contrast_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "Contrast",
                                ),
                            );
                        }
                    }
                    CASIO_SATURATION => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_saturation_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "Saturation",
                                ),
                            );
                        }
                    }
                    CASIO_ENHANCEMENT => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_enhancement_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "Enhancement",
                                ),
                            );
                        }
                    }
                    CASIO_COLOR_FILTER => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_color_filter_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "ColorFilter",
                                ),
                            );
                        }
                    }
                    CASIO_QUALITY => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            let decoded = decode_quality_exiftool(val);
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Ascii(decoded.to_string()),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "Quality",
                                ),
                            );
                        }
                    }
                    CASIO_AF_POINT => {
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            tags.insert(
                                tag_id,
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    tag_name,
                                    ExifValue::Short(vec![val]),
                                    ExifValue::Short(vec![val]),
                                    exiv2_group,
                                    "AFPoint",
                                ),
                            );
                        }
                    }
                    _ => {
                        // Generic handling
                        if let Some(val) =
                            decode_u16_from_entry(data, tag_type, count, &value_bytes, endian)
                        {
                            if let Some(name) = tag_name {
                                tags.insert(
                                    tag_id,
                                    MakerNoteTag::new(
                                        tag_id,
                                        Some(name),
                                        ExifValue::Short(vec![val]),
                                    ),
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(tags)
}
