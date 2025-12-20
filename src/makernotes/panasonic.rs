// makernotes/panasonic.rs - Panasonic/Lumix maker notes parsing

use crate::data_types::{Endianness, ExifValue};
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
pub const PANA_ACCELEROMETER_X: u16 = 0x008A;
pub const PANA_ACCELEROMETER_Y: u16 = 0x008B;
pub const PANA_ACCELEROMETER_Z: u16 = 0x008C;
pub const PANA_CAMERA_ORIENTATION: u16 = 0x008D;
pub const PANA_ROLL_ANGLE: u16 = 0x008E;
pub const PANA_PITCH_ANGLE: u16 = 0x008F;
pub const PANA_BATTERY_LEVEL: u16 = 0x0096;
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
        PANA_ACCELEROMETER_X => Some("AccelerometerX"),
        PANA_ACCELEROMETER_Y => Some("AccelerometerY"),
        PANA_ACCELEROMETER_Z => Some("AccelerometerZ"),
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

/// Decode ImageQuality value
fn decode_image_quality(value: u16) -> &'static str {
    match value {
        2 => "High",
        3 => "Standard",
        6 => "Very High",
        7 => "RAW",
        9 => "Motion Picture",
        _ => "Unknown",
    }
}

/// Decode WhiteBalance value
fn decode_white_balance(value: u16) -> &'static str {
    match value {
        1 => "Auto",
        2 => "Daylight",
        3 => "Cloudy",
        4 => "Incandescent",
        5 => "Manual",
        8 => "Flash",
        10 => "Black & White",
        11 => "Shade",
        12 => "Kelvin",
        _ => "Unknown",
    }
}

/// Decode FocusMode value
fn decode_focus_mode(value: u16) -> &'static str {
    match value {
        1 => "Auto",
        2 => "Manual",
        4 => "Auto, Focus button",
        5 => "Auto, Continuous",
        6 => "AF-S",
        7 => "AF-C",
        8 => "AF-F",
        _ => "Unknown",
    }
}

/// Decode AFAreaMode value
fn decode_af_area_mode(value: u16) -> &'static str {
    match value {
        0 => "Face Detect",
        1 => "Spot Mode",
        2 => "Multi-area",
        3 => "1-area",
        4 => "Tracking",
        16 => "23-area",
        17 => "49-area",
        18 => "Custom Multi",
        _ => "Unknown",
    }
}

/// Decode ImageStabilization value
fn decode_image_stabilization(value: u16) -> &'static str {
    match value {
        2 => "On, Mode 1",
        3 => "Off",
        4 => "On, Mode 2",
        5 => "Panning",
        _ => "Unknown",
    }
}

/// Decode MacroMode value
fn decode_macro_mode(value: u16) -> &'static str {
    match value {
        1 => "On",
        2 => "Off",
        257 => "Tele-macro",
        258 => "Macro-zoom",
        _ => "Unknown",
    }
}

/// Decode ShootingMode value
fn decode_shooting_mode(value: u16) -> &'static str {
    match value {
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
        _ => "Unknown",
    }
}

/// Decode PhotoStyle value
fn decode_photo_style(value: u16) -> &'static str {
    match value {
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
        _ => "Unknown",
    }
}

/// Decode ShutterType value
fn decode_shutter_type(value: u16) -> &'static str {
    match value {
        0 => "Mechanical",
        1 => "Electronic",
        2 => "Hybrid",
        _ => "Unknown",
    }
}

/// Decode ContrastMode value (enhanced)
fn decode_contrast_mode(value: u16) -> &'static str {
    match value {
        0 => "Normal",
        1 => "Low",
        2 => "High",
        6 => "Medium Low",
        7 => "Medium High",
        13 => "High Dynamic",
        256 => "Low",
        272 => "Standard",
        288 => "High",
        _ => "Unknown",
    }
}

/// Decode BurstMode value
fn decode_burst_mode(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        2 => "Auto Exposure Bracketing",
        3 => "Focus Bracketing",
        4 => "Unlimited",
        8 => "White Balance Bracketing",
        17 => "On (with flash)",
        18 => "Aperture Bracketing",
        _ => "Unknown",
    }
}

/// Decode IntelligentResolution value
fn decode_intelligent_resolution(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "Low",
        2 => "Standard",
        3 => "High",
        4 => "Extended",
        _ => "Unknown",
    }
}

/// Decode ClearRetouch value
fn decode_clear_retouch(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        _ => "Unknown",
    }
}

/// Decode TouchAE value
fn decode_touch_ae(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        _ => "Unknown",
    }
}

/// Decode FlashCurtain value
fn decode_flash_curtain(value: u16) -> &'static str {
    match value {
        0 => "n/a",
        1 => "1st",
        2 => "2nd",
        _ => "Unknown",
    }
}

/// Decode HDRShot value
fn decode_hdr_shot(value: u16) -> &'static str {
    match value {
        0 => "No",
        1 => "Yes",
        _ => "Unknown",
    }
}

/// Parse Panasonic maker notes
pub fn parse_panasonic_maker_notes(
    data: &[u8],
    endian: Endianness,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    // Panasonic maker notes use standard IFD format
    if data.len() < 2 {
        return Ok(tags);
    }

    let mut cursor = Cursor::new(data);

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
                1 => count as usize,     // BYTE
                2 => count as usize,     // ASCII
                3 => count as usize * 2, // SHORT
                4 => count as usize * 4, // LONG
                5 => count as usize * 8, // RATIONAL
                7 => count as usize,     // UNDEFINED
                _ => 0,
            };

            // Parse value based on type
            let value = match tag_type {
                1 => {
                    // BYTE
                    if count <= 4 {
                        let bytes = match endian {
                            Endianness::Little => value_offset.to_le_bytes(),
                            Endianness::Big => value_offset.to_be_bytes(),
                        };
                        ExifValue::Byte(bytes[..count as usize].to_vec())
                    } else {
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
                        let s = String::from_utf8_lossy(&data[offset..offset + count as usize])
                            .trim_end_matches('\0')
                            .to_string();
                        ExifValue::Ascii(s)
                    } else {
                        continue;
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
                            PANA_IMAGE_QUALITY => Some(decode_image_quality(v).to_string()),
                            PANA_WHITE_BALANCE => Some(decode_white_balance(v).to_string()),
                            PANA_FOCUS_MODE => Some(decode_focus_mode(v).to_string()),
                            PANA_AF_AREA_MODE => Some(decode_af_area_mode(v).to_string()),
                            PANA_IMAGE_STABILIZATION => {
                                Some(decode_image_stabilization(v).to_string())
                            }
                            PANA_MACRO_MODE => Some(decode_macro_mode(v).to_string()),
                            PANA_SHOOTING_MODE => Some(decode_shooting_mode(v).to_string()),
                            PANA_PHOTO_STYLE => Some(decode_photo_style(v).to_string()),
                            PANA_SHUTTER_TYPE => Some(decode_shutter_type(v).to_string()),
                            PANA_CONTRAST_MODE => Some(decode_contrast_mode(v).to_string()),
                            PANA_BURST_MODE => Some(decode_burst_mode(v).to_string()),
                            PANA_INTELLIGENT_RESOLUTION => {
                                Some(decode_intelligent_resolution(v).to_string())
                            }
                            PANA_CLEAR_RETOUCH => Some(decode_clear_retouch(v).to_string()),
                            PANA_TOUCH_AE => Some(decode_touch_ae(v).to_string()),
                            PANA_FLASH_CURTAIN => Some(decode_flash_curtain(v).to_string()),
                            PANA_HDR_SHOT => Some(decode_hdr_shot(v).to_string()),
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
                7 => {
                    // UNDEFINED
                    let offset = value_offset as usize;
                    if offset + count as usize <= data.len() {
                        ExifValue::Undefined(data[offset..offset + count as usize].to_vec())
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
