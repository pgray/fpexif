// makernotes/pentax.rs - Pentax maker notes parsing
//
// Reference files:
// - exiftool/lib/Image/ExifTool/Pentax.pm
// - exiv2/src/pentaxmn_int.cpp

use crate::data_types::{Endianness, ExifValue};
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Cursor;

// Main Pentax MakerNote tag IDs
pub const PENTAX_VERSION: u16 = 0x0000;
pub const PENTAX_MODE: u16 = 0x0001;
pub const PENTAX_PREVIEW_RESOLUTION: u16 = 0x0002;
pub const PENTAX_PREVIEW_LENGTH: u16 = 0x0003;
pub const PENTAX_PREVIEW_OFFSET: u16 = 0x0004;
pub const PENTAX_MODEL_ID: u16 = 0x0005;
pub const PENTAX_DATE: u16 = 0x0006;
pub const PENTAX_TIME: u16 = 0x0007;
pub const PENTAX_QUALITY: u16 = 0x0008;
pub const PENTAX_IMAGE_SIZE: u16 = 0x0009;
pub const PENTAX_FLASH_MODE: u16 = 0x000B;
pub const PENTAX_FOCUS_MODE: u16 = 0x000D;
pub const PENTAX_AF_POINT_SELECTED: u16 = 0x000E;
pub const PENTAX_AF_POINTS_IN_FOCUS: u16 = 0x000F;
pub const PENTAX_FOCUS_POSITION: u16 = 0x0010;
pub const PENTAX_EXPOSURE_TIME: u16 = 0x0012;
pub const PENTAX_F_NUMBER: u16 = 0x0013;
pub const PENTAX_ISO: u16 = 0x0014;
pub const PENTAX_LIGHT_READING: u16 = 0x0015;
pub const PENTAX_EXPOSURE_COMPENSATION: u16 = 0x0016;
pub const PENTAX_METERING_MODE: u16 = 0x0017;
pub const PENTAX_AUTO_BRACKETING: u16 = 0x0018;
pub const PENTAX_WHITE_BALANCE: u16 = 0x0019;
pub const PENTAX_WHITE_BALANCE_MODE: u16 = 0x001A;
pub const PENTAX_BLUE_BALANCE: u16 = 0x001B;
pub const PENTAX_RED_BALANCE: u16 = 0x001C;
pub const PENTAX_FOCAL_LENGTH: u16 = 0x001D;
pub const PENTAX_DIGITAL_ZOOM: u16 = 0x001E;
pub const PENTAX_SATURATION: u16 = 0x001F;
pub const PENTAX_CONTRAST: u16 = 0x0020;
pub const PENTAX_SHARPNESS: u16 = 0x0021;
pub const PENTAX_WORLD_TIME_LOCATION: u16 = 0x0022;
pub const PENTAX_HOMETOWN_CITY: u16 = 0x0023;
pub const PENTAX_DESTINATION_CITY: u16 = 0x0024;
pub const PENTAX_HOMETOWN_DST: u16 = 0x0025;
pub const PENTAX_DESTINATION_DST: u16 = 0x0026;
pub const PENTAX_DSP_FIRMWARE_VERSION: u16 = 0x0027;
pub const PENTAX_CPU_FIRMWARE_VERSION: u16 = 0x0028;
pub const PENTAX_FRAME_NUMBER: u16 = 0x0029;
pub const PENTAX_EFFECTIVE_LV: u16 = 0x002D;
pub const PENTAX_IMAGE_EDITING: u16 = 0x0032;
pub const PENTAX_PICTURE_MODE: u16 = 0x0033;
pub const PENTAX_DRIVE_MODE: u16 = 0x0034;
pub const PENTAX_SENSOR_SIZE: u16 = 0x0035;
pub const PENTAX_COLOR_SPACE: u16 = 0x0037;
pub const PENTAX_IMAGE_AREA_OFFSET: u16 = 0x0038;
pub const PENTAX_RAW_IMAGE_SIZE: u16 = 0x0039;
pub const PENTAX_AF_POINTS_USED: u16 = 0x003C;
pub const PENTAX_LENS_TYPE: u16 = 0x003F;
pub const PENTAX_SENSITIVITY_ADJUST: u16 = 0x0040;
pub const PENTAX_IMAGE_PROCESSING: u16 = 0x0041;
pub const PENTAX_PREVIEW_IMAGE_BORDERS: u16 = 0x0042;
pub const PENTAX_LENS_DATA: u16 = 0x0043;
pub const PENTAX_SENSITIVITY_STEPS: u16 = 0x0044;
pub const PENTAX_CAMERAS_ORIENTATION: u16 = 0x0047;
pub const PENTAX_NR_LEVEL: u16 = 0x0048;
pub const PENTAX_SERIAL_NUMBER: u16 = 0x0049;
pub const PENTAX_DATA_DUMP: u16 = 0x0050;
pub const PENTAX_SHAKE_REDUCTION_INFO: u16 = 0x005C;
pub const PENTAX_SHUTTER_COUNT: u16 = 0x005D;
pub const PENTAX_BLACK_POINT: u16 = 0x0200;
pub const PENTAX_WHITE_POINT: u16 = 0x0201;
pub const PENTAX_COLOR_MATRIX_A: u16 = 0x0203;
pub const PENTAX_COLOR_MATRIX_B: u16 = 0x0204;
pub const PENTAX_AE_INFO: u16 = 0x0206;
pub const PENTAX_AF_INFO: u16 = 0x0207;
pub const PENTAX_FLASH_INFO: u16 = 0x0208;
pub const PENTAX_LENS_INFO: u16 = 0x0215;
pub const PENTAX_CAMERA_INFO: u16 = 0x0216;
pub const PENTAX_BATTERY_INFO: u16 = 0x0217;
pub const PENTAX_SHOT_INFO: u16 = 0x021F;
pub const PENTAX_HOMTOWN_CITY_CODE: u16 = 0x0227;
pub const PENTAX_DESTINATION_CITY_CODE: u16 = 0x0228;
pub const PENTAX_K_DC_AF_INFO: u16 = 0x0229;
pub const PENTAX_PIXEL_SHIFT_INFO: u16 = 0x022A;
pub const PENTAX_AF_POINT_INFO: u16 = 0x022B;
pub const PENTAX_HDR_INFO: u16 = 0x022E;
pub const PENTAX_TEMPERATURE_INFO: u16 = 0x0230;
pub const PENTAX_SERIAL_NUMBER_2: u16 = 0x0231;

/// Get the name of a Pentax maker note tag
pub fn get_pentax_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        PENTAX_VERSION => Some("PentaxVersion"),
        PENTAX_MODE => Some("PentaxMode"),
        PENTAX_PREVIEW_RESOLUTION => Some("PreviewResolution"),
        PENTAX_PREVIEW_LENGTH => Some("PreviewLength"),
        PENTAX_PREVIEW_OFFSET => Some("PreviewOffset"),
        PENTAX_MODEL_ID => Some("PentaxModelID"),
        PENTAX_DATE => Some("Date"),
        PENTAX_TIME => Some("Time"),
        PENTAX_QUALITY => Some("Quality"),
        PENTAX_IMAGE_SIZE => Some("ImageSize"),
        PENTAX_FLASH_MODE => Some("FlashMode"),
        PENTAX_FOCUS_MODE => Some("FocusMode"),
        PENTAX_AF_POINT_SELECTED => Some("AFPointSelected"),
        PENTAX_AF_POINTS_IN_FOCUS => Some("AFPointsInFocus"),
        PENTAX_FOCUS_POSITION => Some("FocusPosition"),
        PENTAX_EXPOSURE_TIME => Some("ExposureTime"),
        PENTAX_F_NUMBER => Some("FNumber"),
        PENTAX_ISO => Some("ISO"),
        PENTAX_LIGHT_READING => Some("LightReading"),
        PENTAX_EXPOSURE_COMPENSATION => Some("ExposureCompensation"),
        PENTAX_METERING_MODE => Some("MeteringMode"),
        PENTAX_AUTO_BRACKETING => Some("AutoBracketing"),
        PENTAX_WHITE_BALANCE => Some("WhiteBalance"),
        PENTAX_WHITE_BALANCE_MODE => Some("WhiteBalanceMode"),
        PENTAX_BLUE_BALANCE => Some("BlueBalance"),
        PENTAX_RED_BALANCE => Some("RedBalance"),
        PENTAX_FOCAL_LENGTH => Some("FocalLength"),
        PENTAX_DIGITAL_ZOOM => Some("DigitalZoom"),
        PENTAX_SATURATION => Some("Saturation"),
        PENTAX_CONTRAST => Some("Contrast"),
        PENTAX_SHARPNESS => Some("Sharpness"),
        PENTAX_WORLD_TIME_LOCATION => Some("WorldTimeLocation"),
        PENTAX_HOMETOWN_CITY => Some("HometownCity"),
        PENTAX_DESTINATION_CITY => Some("DestinationCity"),
        PENTAX_HOMETOWN_DST => Some("HometownDST"),
        PENTAX_DESTINATION_DST => Some("DestinationDST"),
        PENTAX_DSP_FIRMWARE_VERSION => Some("DSPFirmwareVersion"),
        PENTAX_CPU_FIRMWARE_VERSION => Some("CPUFirmwareVersion"),
        PENTAX_FRAME_NUMBER => Some("FrameNumber"),
        PENTAX_EFFECTIVE_LV => Some("EffectiveLV"),
        PENTAX_IMAGE_EDITING => Some("ImageEditing"),
        PENTAX_PICTURE_MODE => Some("PictureMode"),
        PENTAX_DRIVE_MODE => Some("DriveMode"),
        PENTAX_SENSOR_SIZE => Some("SensorSize"),
        PENTAX_COLOR_SPACE => Some("ColorSpace"),
        PENTAX_IMAGE_AREA_OFFSET => Some("ImageAreaOffset"),
        PENTAX_RAW_IMAGE_SIZE => Some("RawImageSize"),
        PENTAX_AF_POINTS_USED => Some("AFPointsUsed"),
        PENTAX_LENS_TYPE => Some("LensType"),
        PENTAX_SENSITIVITY_ADJUST => Some("SensitivityAdjust"),
        PENTAX_IMAGE_PROCESSING => Some("ImageProcessing"),
        PENTAX_PREVIEW_IMAGE_BORDERS => Some("PreviewImageBorders"),
        PENTAX_LENS_DATA => Some("LensData"),
        PENTAX_SENSITIVITY_STEPS => Some("SensitivitySteps"),
        PENTAX_CAMERAS_ORIENTATION => Some("CameraOrientation"),
        PENTAX_NR_LEVEL => Some("NoiseReductionLevel"),
        PENTAX_SERIAL_NUMBER => Some("SerialNumber"),
        PENTAX_DATA_DUMP => Some("DataDump"),
        PENTAX_SHAKE_REDUCTION_INFO => Some("ShakeReductionInfo"),
        PENTAX_SHUTTER_COUNT => Some("ShutterCount"),
        PENTAX_BLACK_POINT => Some("BlackPoint"),
        PENTAX_WHITE_POINT => Some("WhitePoint"),
        PENTAX_COLOR_MATRIX_A => Some("ColorMatrixA"),
        PENTAX_COLOR_MATRIX_B => Some("ColorMatrixB"),
        PENTAX_AE_INFO => Some("AEInfo"),
        PENTAX_AF_INFO => Some("AFInfo"),
        PENTAX_FLASH_INFO => Some("FlashInfo"),
        PENTAX_LENS_INFO => Some("LensInfo"),
        PENTAX_CAMERA_INFO => Some("CameraInfo"),
        PENTAX_BATTERY_INFO => Some("BatteryInfo"),
        PENTAX_SHOT_INFO => Some("ShotInfo"),
        PENTAX_HOMTOWN_CITY_CODE => Some("HometownCityCode"),
        PENTAX_DESTINATION_CITY_CODE => Some("DestinationCityCode"),
        PENTAX_K_DC_AF_INFO => Some("KDCAFInfo"),
        PENTAX_PIXEL_SHIFT_INFO => Some("PixelShiftInfo"),
        PENTAX_AF_POINT_INFO => Some("AFPointInfo"),
        PENTAX_HDR_INFO => Some("HDRInfo"),
        PENTAX_TEMPERATURE_INFO => Some("TemperatureInfo"),
        PENTAX_SERIAL_NUMBER_2 => Some("SerialNumber2"),
        _ => None,
    }
}

/// Decode Quality value - ExifTool format
pub fn decode_quality_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Good",
        1 => "Better",
        2 => "Best",
        3 => "TIFF",
        4 => "RAW",
        5 => "Premium",
        7 => "RAW (pixel shift enabled)",
        8 => "Dynamic Pixel Shift",
        9 => "Monochrome",
        65535 => "n/a",
        _ => "Unknown",
    }
}

/// Decode FocusMode value - ExifTool format
pub fn decode_focus_mode_exiftool(value: u16) -> &'static str {
    match value {
        0x00 => "Normal",
        0x01 => "Macro",
        0x02 => "Infinity",
        0x03 => "Manual",
        0x04 => "Super Macro",
        0x05 => "Pan Focus",
        0x06 => "Auto-area",
        0x07 => "Zone Select",
        0x10 => "AF-S (Focus-priority)",
        0x11 => "AF-S (Release-priority)",
        0x12 => "AF-C (Focus-priority)",
        0x13 => "AF-C (Release-priority)",
        0x14 => "Single Point",
        0x15 => "Tracking",
        0x16 => "Face Detection",
        0x20 => "Touch AF",
        _ => "Unknown",
    }
}

/// Decode MeteringMode value - ExifTool format
pub fn decode_metering_mode_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Multi-segment",
        1 => "Center-weighted average",
        2 => "Spot",
        6 => "Highlight",
        _ => "Unknown",
    }
}

/// Decode WhiteBalance value - ExifTool format
pub fn decode_white_balance_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        1 => "Daylight",
        2 => "Shade",
        3 => "Fluorescent",
        4 => "Tungsten",
        5 => "Manual",
        6 => "Daylight Fluorescent",
        7 => "Day White Fluorescent",
        8 => "White Fluorescent",
        9 => "Flash",
        10 => "Cloudy",
        11 => "Warm White Fluorescent",
        14 => "Multi Auto",
        15 => "Color Temperature Enhancement",
        17 => "Kelvin",
        65534 => "Unknown",
        65535 => "User-Selected",
        _ => "Unknown",
    }
}

/// Decode FlashMode value - ExifTool format
pub fn decode_flash_mode_exiftool(value: u16) -> &'static str {
    match value {
        0x0000 => "Auto, Did not fire",
        0x0001 => "Off, Did not fire",
        0x0002 => "On, Did not fire",
        0x0003 => "Auto, Did not fire, Red-eye reduction",
        0x0100 => "Auto, Fired",
        0x0102 => "On, Fired",
        0x0103 => "Auto, Fired, Red-eye reduction",
        0x0104 => "On, Red-eye reduction",
        0x0105 => "On, Wireless (Master)",
        0x0106 => "On, Wireless (Control)",
        0x0108 => "On, Soft",
        0x0109 => "On, Slow-sync",
        0x010a => "On, Slow-sync, Red-eye reduction",
        0x010b => "On, Trailing-curtain Sync",
        _ => "Unknown",
    }
}

/// Decode Saturation value - ExifTool format
pub fn decode_saturation_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Low",
        1 => "Normal",
        2 => "High",
        3 => "Medium Low",
        4 => "Medium High",
        5 => "Very Low",
        6 => "Very High",
        7 => "-4",
        8 => "+4",
        65535 => "None",
        _ => "Unknown",
    }
}

/// Decode Contrast value - ExifTool format
pub fn decode_contrast_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Low",
        1 => "Normal",
        2 => "High",
        3 => "Medium Low",
        4 => "Medium High",
        5 => "Very Low",
        6 => "Very High",
        7 => "-4",
        8 => "+4",
        _ => "Unknown",
    }
}

/// Decode Sharpness value - ExifTool format
pub fn decode_sharpness_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Soft",
        1 => "Normal",
        2 => "Hard",
        3 => "Medium Soft",
        4 => "Medium Hard",
        5 => "Very Soft",
        6 => "Very Hard",
        7 => "-4",
        8 => "+4",
        _ => "Unknown",
    }
}

/// Decode DriveMode value - ExifTool format
pub fn decode_drive_mode_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Single-frame",
        1 => "Continuous",
        2 => "Continuous (Hi)",
        3 => "Burst",
        4 => "Continuous (Lo)",
        255 => "Video",
        _ => "Unknown",
    }
}

/// Decode ColorSpace value - ExifTool format
pub fn decode_color_space_exiftool(value: u16) -> &'static str {
    match value {
        0 => "sRGB",
        1 => "Adobe RGB",
        _ => "Unknown",
    }
}

/// Parse Pentax maker notes
pub fn parse_pentax_maker_notes(
    data: &[u8],
    endian: Endianness,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    if data.len() < 10 {
        return Ok(tags);
    }

    // Check for Pentax header "AOC\0" (type 1) or "PENTAX \0" (type 2)
    let (base_offset, maker_endian, ifd_offset) = if data.starts_with(b"AOC\0") {
        // Type 1: "AOC\0" followed by endian marker at offset 4
        let type1_endian = if data.len() > 5 {
            match &data[4..6] {
                b"II" => Endianness::Little,
                b"MM" => Endianness::Big,
                _ => endian,
            }
        } else {
            endian
        };
        (0, type1_endian, 6) // IFD starts at offset 6
    } else if data.len() > 8 && &data[0..8] == b"PENTAX \0" {
        // Type 2: "PENTAX \0" with embedded TIFF header
        let type2_endian = if data.len() > 10 {
            match &data[8..10] {
                b"II" => Endianness::Little,
                b"MM" => Endianness::Big,
                _ => endian,
            }
        } else {
            endian
        };
        // IFD offset is at bytes 12-15
        if data.len() > 16 {
            let ifd_off = match type2_endian {
                Endianness::Little => u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
                Endianness::Big => u32::from_be_bytes([data[12], data[13], data[14], data[15]]),
            };
            (8, type2_endian, ifd_off as usize) // Base is at TIFF header (offset 8)
        } else {
            return Ok(tags);
        }
    } else {
        // Unknown format, try parsing as standard IFD
        (0, endian, 0)
    };

    if base_offset + ifd_offset >= data.len() {
        return Ok(tags);
    }

    let mut cursor = Cursor::new(&data[base_offset + ifd_offset..]);

    // Read number of entries
    let num_entries = match maker_endian {
        Endianness::Little => cursor
            .read_u16::<LittleEndian>()
            .map_err(|_| ExifError::Format("Failed to read Pentax maker note count".to_string()))?,
        Endianness::Big => cursor
            .read_u16::<BigEndian>()
            .map_err(|_| ExifError::Format("Failed to read Pentax maker note count".to_string()))?,
    };

    // Parse IFD entries
    for _ in 0..num_entries {
        if cursor.position() as usize + 12 > data[base_offset + ifd_offset..].len() {
            break;
        }

        let tag_id = match maker_endian {
            Endianness::Little => cursor.read_u16::<LittleEndian>(),
            Endianness::Big => cursor.read_u16::<BigEndian>(),
        }
        .ok();

        let tag_type = match maker_endian {
            Endianness::Little => cursor.read_u16::<LittleEndian>(),
            Endianness::Big => cursor.read_u16::<BigEndian>(),
        }
        .ok();

        let count = match maker_endian {
            Endianness::Little => cursor.read_u32::<LittleEndian>(),
            Endianness::Big => cursor.read_u32::<BigEndian>(),
        }
        .ok();

        let value_offset = match maker_endian {
            Endianness::Little => cursor.read_u32::<LittleEndian>(),
            Endianness::Big => cursor.read_u32::<BigEndian>(),
        }
        .ok();

        if let (Some(tag_id), Some(tag_type), Some(count), Some(value_offset)) =
            (tag_id, tag_type, count, value_offset)
        {
            // Calculate value size in bytes
            let value_size = match tag_type {
                1 => count as usize,     // BYTE
                2 => count as usize,     // ASCII
                3 => count as usize * 2, // SHORT
                4 => count as usize * 4, // LONG
                5 => count as usize * 8, // RATIONAL
                7 => count as usize,     // UNDEFINED
                _ => 0,
            };

            // Determine if value is inline or at offset
            let value_bytes = if value_size <= 4 {
                // Inline value in the value_offset field
                match maker_endian {
                    Endianness::Little => value_offset.to_le_bytes().to_vec(),
                    Endianness::Big => value_offset.to_be_bytes().to_vec(),
                }
            } else {
                // Value at offset (relative to base)
                let abs_offset = base_offset + value_offset as usize;
                if abs_offset + value_size <= data.len() {
                    data[abs_offset..abs_offset + value_size].to_vec()
                } else {
                    continue;
                }
            };

            // Parse the value based on type
            let value = match tag_type {
                1 => {
                    // BYTE
                    ExifValue::Byte(value_bytes[..(count as usize).min(value_bytes.len())].to_vec())
                }
                2 => {
                    // ASCII
                    let s = String::from_utf8_lossy(
                        &value_bytes[..(count as usize).min(value_bytes.len())],
                    )
                    .trim_end_matches('\0')
                    .to_string();
                    ExifValue::Ascii(s)
                }
                3 => {
                    // SHORT
                    let mut values = Vec::new();
                    let mut val_cursor = Cursor::new(&value_bytes);
                    for _ in 0..count {
                        if let Ok(v) = match maker_endian {
                            Endianness::Little => val_cursor.read_u16::<LittleEndian>(),
                            Endianness::Big => val_cursor.read_u16::<BigEndian>(),
                        } {
                            values.push(v);
                        } else {
                            break;
                        }
                    }

                    // Apply decoders for specific tags
                    if values.len() == 1 {
                        let v = values[0];
                        let decoded = match tag_id {
                            PENTAX_QUALITY => Some(decode_quality_exiftool(v).to_string()),
                            PENTAX_FOCUS_MODE => Some(decode_focus_mode_exiftool(v).to_string()),
                            PENTAX_METERING_MODE => {
                                Some(decode_metering_mode_exiftool(v).to_string())
                            }
                            PENTAX_WHITE_BALANCE => {
                                Some(decode_white_balance_exiftool(v).to_string())
                            }
                            PENTAX_FLASH_MODE => Some(decode_flash_mode_exiftool(v).to_string()),
                            PENTAX_SATURATION => Some(decode_saturation_exiftool(v).to_string()),
                            PENTAX_CONTRAST => Some(decode_contrast_exiftool(v).to_string()),
                            PENTAX_SHARPNESS => Some(decode_sharpness_exiftool(v).to_string()),
                            PENTAX_DRIVE_MODE => Some(decode_drive_mode_exiftool(v).to_string()),
                            PENTAX_COLOR_SPACE => Some(decode_color_space_exiftool(v).to_string()),
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
                    let mut values = Vec::new();
                    let mut val_cursor = Cursor::new(&value_bytes);
                    for _ in 0..count {
                        if let Ok(v) = match maker_endian {
                            Endianness::Little => val_cursor.read_u32::<LittleEndian>(),
                            Endianness::Big => val_cursor.read_u32::<BigEndian>(),
                        } {
                            values.push(v);
                        } else {
                            break;
                        }
                    }
                    ExifValue::Long(values)
                }
                5 => {
                    // RATIONAL
                    let mut values = Vec::new();
                    let mut val_cursor = Cursor::new(&value_bytes);
                    for _ in 0..count {
                        let numerator = match maker_endian {
                            Endianness::Little => val_cursor.read_u32::<LittleEndian>(),
                            Endianness::Big => val_cursor.read_u32::<BigEndian>(),
                        };
                        let denominator = match maker_endian {
                            Endianness::Little => val_cursor.read_u32::<LittleEndian>(),
                            Endianness::Big => val_cursor.read_u32::<BigEndian>(),
                        };

                        if let (Ok(num), Ok(den)) = (numerator, denominator) {
                            values.push((num, den));
                        } else {
                            break;
                        }
                    }
                    ExifValue::Rational(values)
                }
                7 => {
                    // UNDEFINED
                    ExifValue::Undefined(value_bytes)
                }
                _ => {
                    // Unsupported type
                    continue;
                }
            };

            tags.insert(
                tag_id,
                MakerNoteTag {
                    tag_id,
                    tag_name: get_pentax_tag_name(tag_id),
                    value,
                },
            );
        }
    }

    Ok(tags)
}
