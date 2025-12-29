// makernotes/panasonic.rs - Panasonic/Lumix maker notes parsing

use crate::data_types::{Endianness, ExifValue};
use crate::define_tag_decoder;
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Cursor;

// Panasonic MakerNote tag IDs
pub const PANA_IMAGE_QUALITY: u16 = 0x0001;
pub const PANA_FIRMWARE_VERSION: u16 = 0x0002;
pub const PANA_WHITE_BALANCE: u16 = 0x0003;
pub const PANA_FOCUS_MODE: u16 = 0x0007;
pub const PANA_AF_AREA_MODE: u16 = 0x000F;
pub const PANA_IMAGE_STABILIZATION: u16 = 0x001A;
pub const PANA_MACRO_MODE: u16 = 0x001C;
pub const PANA_SHOOTING_MODE: u16 = 0x001F;
pub const PANA_AUDIO: u16 = 0x0020;
pub const PANA_FLASH_BIAS: u16 = 0x0024;
pub const PANA_INTERNAL_SERIAL_NUMBER: u16 = 0x0025;
pub const PANA_EXIF_VERSION: u16 = 0x0026;
pub const PANA_COLOR_EFFECT: u16 = 0x0028;
pub const PANA_TIME_SINCE_POWER_ON: u16 = 0x0029;
pub const PANA_BURST_MODE: u16 = 0x002A;
pub const PANA_SEQUENCE_NUMBER: u16 = 0x002B;
pub const PANA_CONTRAST_MODE: u16 = 0x002C;
pub const PANA_NOISE_REDUCTION: u16 = 0x002D;
pub const PANA_SELF_TIMER: u16 = 0x002E;
pub const PANA_ROTATION: u16 = 0x0030;
pub const PANA_AF_ASSIST_LAMP: u16 = 0x0031;
pub const PANA_COLOR_MODE: u16 = 0x0032;
pub const PANA_BABY_AGE: u16 = 0x0033;
pub const PANA_OPTICAL_ZOOM_MODE: u16 = 0x0034;
pub const PANA_CONVERSION_LENS: u16 = 0x0035;
pub const PANA_TRAVEL_DAY: u16 = 0x0036;
pub const PANA_CONTRAST: u16 = 0x0039;
pub const PANA_WORLD_TIME_LOCATION: u16 = 0x003A;
pub const PANA_TEXT_STAMP: u16 = 0x003B;
pub const PANA_PROGRAM_ISO: u16 = 0x003C;
pub const PANA_ADVANCED_SCENE_MODE: u16 = 0x003D;
pub const PANA_TEXT_STAMP_2: u16 = 0x003E;
pub const PANA_FACE_DETECTED: u16 = 0x003F;
pub const PANA_LENS_TYPE: u16 = 0x0051;
pub const PANA_LENS_SERIAL_NUMBER: u16 = 0x0052;
pub const PANA_ACCESSORY_TYPE: u16 = 0x0053;
pub const PANA_ACCESSORY_SERIAL_NUMBER: u16 = 0x0054;
pub const PANA_SHADING_COMPENSATION: u16 = 0x008A;
pub const PANA_WB_SHIFT_INTELLIGENT_AUTO: u16 = 0x008B;
pub const PANA_ACCELEROMETER_Z: u16 = 0x008C;
pub const PANA_ACCELEROMETER_X: u16 = 0x008D;
pub const PANA_ACCELEROMETER_Y: u16 = 0x008E;
pub const PANA_CAMERA_ORIENTATION: u16 = 0x008F;
pub const PANA_ROLL_ANGLE: u16 = 0x0090;
pub const PANA_PITCH_ANGLE: u16 = 0x0091;
pub const PANA_BATTERY_LEVEL: u16 = 0x0038;
pub const PANA_CITY: u16 = 0x006D;
pub const PANA_LANDMARK: u16 = 0x006E;
pub const PANA_INTELLIGENT_RESOLUTION: u16 = 0x0070;
pub const PANA_CLEAR_RETOUCH: u16 = 0x0077;
pub const PANA_PHOTO_STYLE: u16 = 0x0089;
pub const PANA_HDR_SHOT: u16 = 0x0093;
pub const PANA_SHUTTER_TYPE: u16 = 0x009A;
pub const PANA_FLASH_CURTAIN: u16 = 0x00AB;
pub const PANA_TOUCH_AE: u16 = 0x00AE;

/// Get the name of a Panasonic MakerNote tag
pub fn get_panasonic_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        PANA_IMAGE_QUALITY => Some("ImageQuality"),
        PANA_FIRMWARE_VERSION => Some("FirmwareVersion"),
        PANA_WHITE_BALANCE => Some("WhiteBalance"),
        PANA_FOCUS_MODE => Some("FocusMode"),
        PANA_AF_AREA_MODE => Some("AFAreaMode"),
        PANA_IMAGE_STABILIZATION => Some("ImageStabilization"),
        PANA_MACRO_MODE => Some("MacroMode"),
        PANA_SHOOTING_MODE => Some("ShootingMode"),
        PANA_AUDIO => Some("Audio"),
        PANA_FLASH_BIAS => Some("FlashBias"),
        PANA_INTERNAL_SERIAL_NUMBER => Some("InternalSerialNumber"),
        PANA_EXIF_VERSION => Some("ExifVersion"),
        PANA_COLOR_EFFECT => Some("ColorEffect"),
        PANA_TIME_SINCE_POWER_ON => Some("TimeSincePowerOn"),
        PANA_BURST_MODE => Some("BurstMode"),
        PANA_SEQUENCE_NUMBER => Some("SequenceNumber"),
        PANA_CONTRAST_MODE => Some("ContrastMode"),
        PANA_NOISE_REDUCTION => Some("NoiseReduction"),
        PANA_SELF_TIMER => Some("SelfTimer"),
        PANA_ROTATION => Some("Rotation"),
        PANA_AF_ASSIST_LAMP => Some("AFAssistLamp"),
        PANA_COLOR_MODE => Some("ColorMode"),
        PANA_BABY_AGE => Some("BabyAge"),
        PANA_OPTICAL_ZOOM_MODE => Some("OpticalZoomMode"),
        PANA_CONVERSION_LENS => Some("ConversionLens"),
        PANA_TRAVEL_DAY => Some("TravelDay"),
        PANA_CONTRAST => Some("Contrast"),
        PANA_WORLD_TIME_LOCATION => Some("WorldTimeLocation"),
        PANA_TEXT_STAMP => Some("TextStamp"),
        PANA_PROGRAM_ISO => Some("ProgramISO"),
        PANA_ADVANCED_SCENE_MODE => Some("AdvancedSceneMode"),
        PANA_TEXT_STAMP_2 => Some("TextStamp2"),
        PANA_FACE_DETECTED => Some("FaceDetected"),
        PANA_LENS_TYPE => Some("LensType"),
        PANA_LENS_SERIAL_NUMBER => Some("LensSerialNumber"),
        PANA_ACCESSORY_TYPE => Some("AccessoryType"),
        PANA_ACCESSORY_SERIAL_NUMBER => Some("AccessorySerialNumber"),
        PANA_SHADING_COMPENSATION => Some("ShadingCompensation"),
        PANA_WB_SHIFT_INTELLIGENT_AUTO => Some("WBShiftIntelligentAuto"),
        PANA_ACCELEROMETER_Z => Some("AccelerometerZ"),
        PANA_ACCELEROMETER_X => Some("AccelerometerX"),
        PANA_ACCELEROMETER_Y => Some("AccelerometerY"),
        PANA_CAMERA_ORIENTATION => Some("CameraOrientation"),
        PANA_ROLL_ANGLE => Some("RollAngle"),
        PANA_PITCH_ANGLE => Some("PitchAngle"),
        PANA_BATTERY_LEVEL => Some("BatteryLevel"),
        PANA_CITY => Some("City"),
        PANA_LANDMARK => Some("Landmark"),
        PANA_INTELLIGENT_RESOLUTION => Some("IntelligentResolution"),
        PANA_CLEAR_RETOUCH => Some("ClearRetouch"),
        PANA_PHOTO_STYLE => Some("PhotoStyle"),
        PANA_HDR_SHOT => Some("HDRShot"),
        PANA_SHUTTER_TYPE => Some("ShutterType"),
        PANA_FLASH_CURTAIN => Some("FlashCurtain"),
        PANA_TOUCH_AE => Some("TouchAE"),
        _ => None,
    }
}

// ImageQuality (tag 0x0001): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    image_quality,
    exiftool: {
        1 => "TIFF",
        2 => "High",
        3 => "Standard",
        6 => "Very High",
        7 => "RAW",
        9 => "Motion Picture",
        11 => "Full HD Movie",
        12 => "4K Movie",
    },
    exiv2: {
        1 => "TIFF",
        2 => "High",
        3 => "Normal",
        6 => "Very High",
        7 => "Raw",
        9 => "Motion Picture",
        11 => "Full HD Movie",
        12 => "4k Movie",
    }
}

// WhiteBalance (tag 0x0003): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    white_balance,
    exiftool: {
        1 => "Auto",
        2 => "Daylight",
        3 => "Cloudy",
        4 => "Incandescent",
        5 => "Manual",
        8 => "Flash",
        10 => "Black & White",
        11 => "Shade",
        12 => "Kelvin",
    },
    exiv2: {
        1 => "Auto",
        2 => "Daylight",
        3 => "Cloudy",
        4 => "Halogen",
        5 => "Manual",
        8 => "Flash",
        10 => "Black and white",
        11 => "Manual",
        12 => "Shade",
        13 => "Kelvin",
    }
}

// FocusMode (tag 0x0007)
define_tag_decoder! {
    panasonic_focus_mode,
    both: {
        1 => "Auto",
        2 => "Manual",
        4 => "Auto, Focus button",
        5 => "Auto, Continuous",
        6 => "AF-S",
        7 => "AF-C",
        8 => "AF-F",
    }
}

// AFAreaMode (tag 0x000F)
define_tag_decoder! {
    panasonic_af_area_mode,
    both: {
        0 => "Face Detect",
        1 => "Spot Mode",
        2 => "Multi-area",
        3 => "1-area",
        4 => "Tracking",
        16 => "23-area",
        17 => "49-area",
        18 => "Custom Multi",
    }
}

// Legacy aliases
pub fn decode_focus_mode_exiftool(value: u16) -> &'static str {
    decode_panasonic_focus_mode_exiftool(value)
}

pub fn decode_af_area_mode_exiftool(value: u16) -> &'static str {
    decode_panasonic_af_area_mode_exiftool(value)
}

// ImageStabilization (tag 0x001A): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    image_stabilization,
    exiftool: {
        2 => "On, Optical",
        3 => "Off",
        4 => "On, Mode 2",
        5 => "On, Optical Panning",
        6 => "On, Body-only",
        7 => "On, Body-only Panning",
        9 => "Dual IS",
        10 => "Dual IS Panning",
        11 => "Dual2 IS",
        12 => "Dual2 IS Panning",
    },
    exiv2: {
        2 => "On, Mode 1",
        3 => "Off",
        4 => "On, Mode 2",
        5 => "Panning",
        6 => "On, Mode 3",
    }
}

// MacroMode (tag 0x001C): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    macro_mode,
    exiftool: {
        1 => "On",
        2 => "Off",
        257 => "Tele-macro",
        513 => "Macro Zoom",
    },
    exiv2: {
        1 => "On",
        2 => "Off",
        257 => "Tele-macro",
        513 => "Macro-zoom",
    }
}

// ShootingMode (tag 0x001F): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    shooting_mode,
    both: {
        1 => "Normal",
        2 => "Portrait",
        3 => "Scenery",
        4 => "Sports",
        5 => "Night Portrait",
        6 => "Program",
        7 => "Aperture Priority",
        8 => "Shutter Priority",
        9 => "Macro",
        10 => "Spot",
        11 => "Manual",
        12 => "Movie Preview",
        13 => "Panning",
        14 => "Simple",
        15 => "Color Effects",
        18 => "Fireworks",
        19 => "Party",
        20 => "Snow",
        21 => "Night Scenery",
        22 => "Food",
        23 => "Baby",
        27 => "High Sensitivity",
        29 => "Underwater",
        33 => "Pet",
    }
}

// PhotoStyle (tag 0x0089): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    photo_style,
    exiftool: {
        0 => "Auto",
        1 => "Standard or Custom",
        2 => "Vivid",
        3 => "Natural",
        4 => "Monochrome",
        5 => "Scenery",
        6 => "Portrait",
        8 => "Cinelike D",
        9 => "Cinelike V",
        11 => "L.Monochrome",
        12 => "Like709",
        15 => "L.Monochrome D",
        17 => "V-Log",
        18 => "Cinelike D2",
    },
    exiv2: {
        0 => "NoAuto",
        1 => "Standard or Custom",
        2 => "Vivid",
        3 => "Natural",
        4 => "Monochrome",
        5 => "Scenery",
        6 => "Portrait",
    }
}

// ShutterType (tag 0x009A): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    shutter_type,
    both: {
        0 => "Mechanical",
        1 => "Electronic",
        2 => "Hybrid",
    }
}

// ContrastMode (tag 0x002C): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    contrast_mode,
    exiftool: {
        0 => "Normal",
        1 => "Low",
        2 => "High",
        6 => "Medium Low",
        7 => "Medium High",
        13 => "High Dynamic",
        256 => "Low",
        272 => "Standard",
        288 => "High",
    },
    exiv2: {
        0 => "Normal",
        1 => "Low",
        2 => "High",
        6 => "Medium low",
        7 => "Medium high",
        256 => "Low",
        272 => "Standard",
        288 => "High",
    }
}

// BurstMode (tag 0x002A): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    burst_mode,
    exiftool: {
        0 => "Off",
        1 => "On",
        2 => "Auto Exposure Bracketing",
        3 => "Focus Bracketing",
        4 => "Unlimited",
        8 => "White Balance Bracketing",
        17 => "On (with flash)",
        18 => "Aperture Bracketing",
    },
    exiv2: {
        0 => "Off",
        1 => "Low/High quality",
        2 => "Infinite",
    }
}

// IntelligentResolution (tag 0x0070): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    intelligent_resolution,
    both: {
        0 => "Off",
        1 => "Low",
        2 => "Standard",
        3 => "High",
        4 => "Extended",
    }
}

// ClearRetouch (tag 0x0077): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    clear_retouch,
    both: {
        0 => "Off",
        1 => "On",
    }
}

// TouchAE (tag 0x00AE): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    touch_ae,
    both: {
        0 => "Off",
        1 => "On",
    }
}

// FlashCurtain (tag 0x00AB): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    flash_curtain,
    both: {
        0 => "n/a",
        1 => "1st",
        2 => "2nd",
    }
}

// HDRShot (tag 0x0093): Panasonic.pm (exiv2 uses different tag)
define_tag_decoder! {
    hdr_shot,
    both: {
        0 => "No",
        1 => "Yes",
    }
}

// ShadingCompensation (tag 0x008A): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    shading_compensation,
    both: {
        0 => "Off",
        1 => "On",
    }
}

// Audio (tag 0x0020): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    audio,
    both: {
        1 => "Yes",
        2 => "No",
        3 => "Stereo",
    }
}

// ColorEffect (tag 0x0028): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    color_effect,
    both: {
        1 => "Off",
        2 => "Warm",
        3 => "Cool",
        4 => "Black & White",
        5 => "Sepia",
        6 => "Happy",
        8 => "Vivid",
    }
}

// NoiseReduction (tag 0x002D): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    noise_reduction,
    both: {
        0 => "Standard",
        1 => "Low (-1)",
        2 => "High (+1)",
        3 => "Lowest (-2)",
        4 => "Highest (+2)",
        5 => "+5",
        6 => "+6",
        65531 => "-5",
        65532 => "-4",
        65533 => "-3",
        65534 => "-2",
        65535 => "-1",
    }
}

// SelfTimer (tag 0x002E): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    self_timer,
    both: {
        0 => "Off (0)",
        1 => "Off",
        2 => "10 s",
        3 => "2 s",
        4 => "10 s / 3 pictures",
        258 => "2 s after shutter pressed",
        266 => "10 s after shutter pressed",
        778 => "3 photos after 10 s",
    }
}

// Rotation (tag 0x0030): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    rotation,
    both: {
        1 => "Horizontal (normal)",
        3 => "Rotate 180",
        6 => "Rotate 90 CW",
        8 => "Rotate 270 CW",
    }
}

// AFAssistLamp (tag 0x0031): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    af_assist_lamp,
    both: {
        1 => "Fired",
        2 => "Enabled but Not Used",
        3 => "Disabled but Required",
        4 => "Disabled and Not Required",
    }
}

// ColorMode (tag 0x0032): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    color_mode,
    both: {
        0 => "Normal",
        1 => "Natural",
        2 => "Vivid",
    }
}

// OpticalZoomMode (tag 0x0034): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    optical_zoom_mode,
    both: {
        1 => "Standard",
        2 => "Extended",
    }
}

// ConversionLens (tag 0x0035): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    conversion_lens,
    both: {
        1 => "Off",
        2 => "Wide",
        3 => "Telephoto",
        4 => "Macro",
    }
}

// BatteryLevel (tag 0x0038): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    battery_level,
    both: {
        1 => "Full",
        2 => "Medium",
        3 => "Low",
        4 => "Near Empty",
        7 => "Near Full",
        8 => "Medium Low",
        256 => "n/a",
    }
}

// WorldTimeLocation (tag 0x003A): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    world_time_location,
    both: {
        1 => "Home",
        2 => "Destination",
    }
}

// TextStamp (tag 0x003B): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    text_stamp,
    both: {
        1 => "Off",
        2 => "On",
    }
}

// CameraOrientation (tag 0x008F): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    camera_orientation,
    type: u8,
    both: {
        0 => "Normal",
        1 => "Rotate CW",
        2 => "Rotate 180",
        3 => "Rotate CCW",
        4 => "Tilt Upwards",
        5 => "Tilt Downwards",
    }
}

/// Parse Panasonic maker notes
pub fn parse_panasonic_maker_notes(
    data: &[u8],
    endian: Endianness,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    // Panasonic maker notes start with "Panasonic\0" header (12 bytes)
    // followed by standard IFD format
    if data.len() < 14 {
        return Ok(tags);
    }

    // Check for and skip Panasonic header
    let ifd_start = if data.starts_with(b"Panasonic\0") {
        12 // Skip "Panasonic\0" (10 bytes) + 2 bytes padding
    } else {
        0 // No header, start at beginning
    };

    let mut cursor = Cursor::new(&data[ifd_start..]);

    // Read number of entries
    let num_entries = match endian {
        Endianness::Little => cursor.read_u16::<LittleEndian>(),
        Endianness::Big => cursor.read_u16::<BigEndian>(),
    }
    .map_err(|_| ExifError::Format("Failed to read Panasonic maker note count".to_string()))?;

    // Parse each IFD entry
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
            // Calculate value size
            let value_size = match tag_type {
                1 => count as usize,      // BYTE
                2 => count as usize,      // ASCII
                3 => count as usize * 2,  // SHORT
                4 => count as usize * 4,  // LONG
                5 => count as usize * 8,  // RATIONAL
                6 => count as usize,      // SBYTE
                7 => count as usize,      // UNDEFINED
                8 => count as usize * 2,  // SSHORT
                9 => count as usize * 4,  // SLONG
                10 => count as usize * 8, // SRATIONAL
                _ => 0,
            };

            // Parse value based on type
            let value = match tag_type {
                1 => {
                    // BYTE
                    let bytes = if count <= 4 {
                        match endian {
                            Endianness::Little => {
                                value_offset.to_le_bytes()[..count as usize].to_vec()
                            }
                            Endianness::Big => {
                                value_offset.to_be_bytes()[..count as usize].to_vec()
                            }
                        }
                    } else {
                        let offset = value_offset as usize;
                        if offset + count as usize <= data.len() {
                            data[offset..offset + count as usize].to_vec()
                        } else {
                            continue;
                        }
                    };

                    // Try to decode specific tags
                    if count == 1 && !bytes.is_empty() {
                        let decoded = match tag_id {
                            PANA_CAMERA_ORIENTATION => {
                                Some(decode_camera_orientation_exiftool(bytes[0]).to_string())
                            }
                            PANA_INTELLIGENT_RESOLUTION => Some(
                                decode_intelligent_resolution_exiftool(bytes[0] as u16).to_string(),
                            ),
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
                    if count <= 4 {
                        // Inline value for short strings
                        let bytes = match endian {
                            Endianness::Little => value_offset.to_le_bytes(),
                            Endianness::Big => value_offset.to_be_bytes(),
                        };
                        let s = String::from_utf8_lossy(&bytes[..count as usize])
                            .trim_end_matches('\0')
                            .to_string();
                        ExifValue::Ascii(s)
                    } else {
                        let offset = value_offset as usize;
                        if offset + count as usize <= data.len() {
                            let s = String::from_utf8_lossy(&data[offset..offset + count as usize])
                                .trim_end_matches('\0')
                                .to_string();
                            ExifValue::Ascii(s)
                        } else {
                            continue;
                        }
                    }
                }
                3 => {
                    // SHORT
                    let mut values = Vec::new();
                    if count == 1 {
                        // Inline value
                        values.push(match endian {
                            Endianness::Little => (value_offset & 0xFFFF) as u16,
                            Endianness::Big => (value_offset >> 16) as u16,
                        });
                    } else if count == 2 {
                        // Two inline values
                        values.push(match endian {
                            Endianness::Little => (value_offset & 0xFFFF) as u16,
                            Endianness::Big => (value_offset >> 16) as u16,
                        });
                        values.push(match endian {
                            Endianness::Little => (value_offset >> 16) as u16,
                            Endianness::Big => (value_offset & 0xFFFF) as u16,
                        });
                    } else {
                        // Values at offset
                        let offset = value_offset as usize;
                        if offset + value_size <= data.len() {
                            let mut cursor = Cursor::new(&data[offset..]);
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
                        } else {
                            continue;
                        }
                    }

                    // Apply value decoders
                    if values.len() == 1 {
                        let v = values[0];
                        let decoded = match tag_id {
                            PANA_IMAGE_QUALITY => {
                                Some(decode_image_quality_exiftool(v).to_string())
                            }
                            PANA_WHITE_BALANCE => {
                                Some(decode_white_balance_exiftool(v).to_string())
                            }
                            PANA_FOCUS_MODE => Some(decode_focus_mode_exiftool(v).to_string()),
                            PANA_AF_AREA_MODE => Some(decode_af_area_mode_exiftool(v).to_string()),
                            PANA_IMAGE_STABILIZATION => {
                                Some(decode_image_stabilization_exiftool(v).to_string())
                            }
                            PANA_MACRO_MODE => Some(decode_macro_mode_exiftool(v).to_string()),
                            PANA_SHOOTING_MODE => {
                                Some(decode_shooting_mode_exiftool(v).to_string())
                            }
                            PANA_PHOTO_STYLE => Some(decode_photo_style_exiftool(v).to_string()),
                            PANA_SHUTTER_TYPE => Some(decode_shutter_type_exiftool(v).to_string()),
                            PANA_CONTRAST_MODE => {
                                Some(decode_contrast_mode_exiftool(v).to_string())
                            }
                            PANA_BURST_MODE => Some(decode_burst_mode_exiftool(v).to_string()),
                            PANA_INTELLIGENT_RESOLUTION => {
                                Some(decode_intelligent_resolution_exiftool(v).to_string())
                            }
                            PANA_CLEAR_RETOUCH => {
                                Some(decode_clear_retouch_exiftool(v).to_string())
                            }
                            PANA_TOUCH_AE => Some(decode_touch_ae_exiftool(v).to_string()),
                            PANA_FLASH_CURTAIN => {
                                Some(decode_flash_curtain_exiftool(v).to_string())
                            }
                            PANA_HDR_SHOT => Some(decode_hdr_shot_exiftool(v).to_string()),
                            PANA_AUDIO => Some(decode_audio_exiftool(v).to_string()),
                            PANA_COLOR_EFFECT => Some(decode_color_effect_exiftool(v).to_string()),
                            PANA_NOISE_REDUCTION => {
                                Some(decode_noise_reduction_exiftool(v).to_string())
                            }
                            PANA_SELF_TIMER => Some(decode_self_timer_exiftool(v).to_string()),
                            PANA_ROTATION => Some(decode_rotation_exiftool(v).to_string()),
                            PANA_AF_ASSIST_LAMP => {
                                Some(decode_af_assist_lamp_exiftool(v).to_string())
                            }
                            PANA_COLOR_MODE => Some(decode_color_mode_exiftool(v).to_string()),
                            PANA_OPTICAL_ZOOM_MODE => {
                                Some(decode_optical_zoom_mode_exiftool(v).to_string())
                            }
                            PANA_CONVERSION_LENS => {
                                Some(decode_conversion_lens_exiftool(v).to_string())
                            }
                            PANA_BATTERY_LEVEL => {
                                Some(decode_battery_level_exiftool(v).to_string())
                            }
                            PANA_WORLD_TIME_LOCATION => {
                                Some(decode_world_time_location_exiftool(v).to_string())
                            }
                            PANA_TEXT_STAMP | PANA_TEXT_STAMP_2 => {
                                Some(decode_text_stamp_exiftool(v).to_string())
                            }
                            PANA_SHADING_COMPENSATION => {
                                Some(decode_shading_compensation_exiftool(v).to_string())
                            }
                            PANA_PROGRAM_ISO | PANA_TRAVEL_DAY => {
                                // 65535 means "n/a" for these tags
                                if v == 65535 {
                                    Some("n/a".to_string())
                                } else {
                                    None
                                }
                            }
                            PANA_FLASH_BIAS => {
                                // FlashBias is signed, divided by 3
                                let signed_v = v as i16;
                                let bias = signed_v / 3;
                                Some(format!("{}", bias))
                            }
                            PANA_ACCELEROMETER_X | PANA_ACCELEROMETER_Y | PANA_ACCELEROMETER_Z => {
                                // Accelerometer values are signed
                                let signed_v = v as i16;
                                Some(format!("{}", signed_v))
                            }
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
                4 => {
                    // LONG
                    if count == 1 {
                        ExifValue::Long(vec![value_offset])
                    } else {
                        let offset = value_offset as usize;
                        if offset + value_size <= data.len() {
                            let mut values = Vec::new();
                            let mut cursor = Cursor::new(&data[offset..]);
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
                        } else {
                            continue;
                        }
                    }
                }
                5 => {
                    // RATIONAL (numerator/denominator pairs)
                    let offset = value_offset as usize;
                    if offset + value_size <= data.len() {
                        let mut values = Vec::new();
                        let mut cursor = Cursor::new(&data[offset..]);
                        for _ in 0..count {
                            if let (Ok(num), Ok(den)) = (
                                match endian {
                                    Endianness::Little => cursor.read_u32::<LittleEndian>(),
                                    Endianness::Big => cursor.read_u32::<BigEndian>(),
                                },
                                match endian {
                                    Endianness::Little => cursor.read_u32::<LittleEndian>(),
                                    Endianness::Big => cursor.read_u32::<BigEndian>(),
                                },
                            ) {
                                values.push((num, den));
                            } else {
                                break;
                            }
                        }
                        ExifValue::Rational(values)
                    } else {
                        continue;
                    }
                }
                6 => {
                    // SBYTE
                    if count <= 4 {
                        let bytes = match endian {
                            Endianness::Little => value_offset.to_le_bytes(),
                            Endianness::Big => value_offset.to_be_bytes(),
                        };
                        let sbytes: Vec<i8> =
                            bytes[..count as usize].iter().map(|&b| b as i8).collect();
                        ExifValue::SByte(sbytes)
                    } else {
                        let offset = value_offset as usize;
                        if offset + count as usize <= data.len() {
                            let sbytes: Vec<i8> = data[offset..offset + count as usize]
                                .iter()
                                .map(|&b| b as i8)
                                .collect();
                            ExifValue::SByte(sbytes)
                        } else {
                            continue;
                        }
                    }
                }
                7 => {
                    // UNDEFINED
                    let offset = value_offset as usize;
                    if offset + count as usize <= data.len() {
                        ExifValue::Undefined(data[offset..offset + count as usize].to_vec())
                    } else {
                        continue;
                    }
                }
                8 => {
                    // SSHORT
                    let mut values = Vec::new();
                    if count == 1 {
                        // Inline value
                        values.push(match endian {
                            Endianness::Little => (value_offset & 0xFFFF) as i16,
                            Endianness::Big => (value_offset >> 16) as i16,
                        });
                    } else if count == 2 {
                        // Two inline values
                        values.push(match endian {
                            Endianness::Little => (value_offset & 0xFFFF) as i16,
                            Endianness::Big => (value_offset >> 16) as i16,
                        });
                        values.push(match endian {
                            Endianness::Little => (value_offset >> 16) as i16,
                            Endianness::Big => (value_offset & 0xFFFF) as i16,
                        });
                    } else {
                        // Values at offset
                        let offset = value_offset as usize;
                        if offset + value_size <= data.len() {
                            let mut cursor = Cursor::new(&data[offset..]);
                            for _ in 0..count {
                                if let Ok(v) = match endian {
                                    Endianness::Little => cursor.read_i16::<LittleEndian>(),
                                    Endianness::Big => cursor.read_i16::<BigEndian>(),
                                } {
                                    values.push(v);
                                } else {
                                    break;
                                }
                            }
                        } else {
                            continue;
                        }
                    }
                    ExifValue::SShort(values)
                }
                9 => {
                    // SLONG
                    if count == 1 {
                        ExifValue::SLong(vec![value_offset as i32])
                    } else {
                        let offset = value_offset as usize;
                        if offset + value_size <= data.len() {
                            let mut values = Vec::new();
                            let mut cursor = Cursor::new(&data[offset..]);
                            for _ in 0..count {
                                if let Ok(v) = match endian {
                                    Endianness::Little => cursor.read_i32::<LittleEndian>(),
                                    Endianness::Big => cursor.read_i32::<BigEndian>(),
                                } {
                                    values.push(v);
                                } else {
                                    break;
                                }
                            }
                            ExifValue::SLong(values)
                        } else {
                            continue;
                        }
                    }
                }
                10 => {
                    // SRATIONAL (signed numerator/denominator pairs)
                    let offset = value_offset as usize;
                    if offset + value_size <= data.len() {
                        let mut values = Vec::new();
                        let mut cursor = Cursor::new(&data[offset..]);
                        for _ in 0..count {
                            if let (Ok(num), Ok(den)) = (
                                match endian {
                                    Endianness::Little => cursor.read_i32::<LittleEndian>(),
                                    Endianness::Big => cursor.read_i32::<BigEndian>(),
                                },
                                match endian {
                                    Endianness::Little => cursor.read_i32::<LittleEndian>(),
                                    Endianness::Big => cursor.read_i32::<BigEndian>(),
                                },
                            ) {
                                values.push((num, den));
                            } else {
                                break;
                            }
                        }
                        ExifValue::SRational(values)
                    } else {
                        continue;
                    }
                }
                _ => {
                    continue;
                }
            };

            tags.insert(
                tag_id,
                MakerNoteTag {
                    tag_id,
                    tag_name: get_panasonic_tag_name(tag_id),
                    value,
                },
            );
        }
    }

    Ok(tags)
}
