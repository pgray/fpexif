// makernotes/kodak.rs - Kodak maker notes parsing
//
// Reference files:
// - exiftool/lib/Image/ExifTool/Kodak.pm
//
// Kodak cameras use multiple makernote formats:
// - Type 1 (Main): Binary format used by most consumer cameras (C360, DX series, Z series, etc.)
// - IFD-based: Standard IFD format used by professional models (DC50, DC120, DCS Pro 14N, etc.)
// - Various Type2-11 formats for specific camera series
//
// This module focuses on the IFD-based format and common Type 1 tags.

use crate::data_types::{Endianness, ExifValue};
use crate::define_tag_decoder;
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Cursor;

// Kodak IFD MakerNote tag IDs
// These are used in professional Kodak cameras (DC50, DC120, DCS760C, DCS Pro 14N, etc.)
pub const KODAK_VERSION: u16 = 0x0000;
pub const KODAK_UNKNOWN_EV: u16 = 0x0001;
pub const KODAK_EXPOSURE_VALUE: u16 = 0x0003;
pub const KODAK_ORIGINAL_FILE_NAME: u16 = 0x03e9;
pub const KODAK_TAG: u16 = 0x03ea;
pub const KODAK_SENSOR_LEFT_BORDER: u16 = 0x03eb;
pub const KODAK_SENSOR_TOP_BORDER: u16 = 0x03ec;
pub const KODAK_SENSOR_IMAGE_WIDTH: u16 = 0x03ed;
pub const KODAK_SENSOR_IMAGE_HEIGHT: u16 = 0x03ee;
pub const KODAK_BLACK_LEVEL_TOP: u16 = 0x03ef;
pub const KODAK_BLACK_LEVEL_BOTTOM: u16 = 0x03f0;
pub const KODAK_TEXTUAL_INFO: u16 = 0x03f1;
pub const KODAK_FLASH_MODE: u16 = 0x03f2;
pub const KODAK_FLASH_COMPENSATION: u16 = 0x03f3;
pub const KODAK_WIND_MODE: u16 = 0x03f4;
pub const KODAK_FOCUS_MODE: u16 = 0x03f5;
pub const KODAK_MIN_APERTURE: u16 = 0x03f8;
pub const KODAK_MAX_APERTURE: u16 = 0x03f9;
pub const KODAK_WHITE_BALANCE_MODE: u16 = 0x03fa;
pub const KODAK_WHITE_BALANCE_DETECTED: u16 = 0x03fb;
pub const KODAK_WHITE_BALANCE: u16 = 0x03fc;
pub const KODAK_PROCESSING: u16 = 0x03fd;
pub const KODAK_IMAGE_ABSOLUTE_X: u16 = 0x03fe;
pub const KODAK_IMAGE_ABSOLUTE_Y: u16 = 0x03ff;
pub const KODAK_APPLICATION_KEY_STRING: u16 = 0x0400;
pub const KODAK_TIME: u16 = 0x0401;
pub const KODAK_GPS_STRING: u16 = 0x0402;
pub const KODAK_EVENT_LOG_CAPTURE: u16 = 0x0403;
pub const KODAK_COMPONENT_TABLE: u16 = 0x0404;
pub const KODAK_CUSTOM_ILLUMINANT: u16 = 0x0405;
pub const KODAK_CAMERA_TEMPERATURE: u16 = 0x0406;
pub const KODAK_ADAPTER_VOLTAGE: u16 = 0x0407;
pub const KODAK_BATTERY_VOLTAGE: u16 = 0x0408;
pub const KODAK_DAC_VOLTAGES: u16 = 0x0409;
pub const KODAK_ILLUMINANT_DETECTOR_DATA: u16 = 0x040a;
pub const KODAK_PIXEL_CLOCK_FREQUENCY: u16 = 0x040b;
pub const KODAK_CENTER_PIXEL: u16 = 0x040c;
pub const KODAK_BURST_COUNT: u16 = 0x040d;
pub const KODAK_BLACK_LEVEL_ROUGH: u16 = 0x040e;
pub const KODAK_OFFSET_MAP_HORIZONTAL: u16 = 0x040f;
pub const KODAK_OFFSET_MAP_VERTICAL: u16 = 0x0410;
pub const KODAK_HISTOGRAM: u16 = 0x0411;
pub const KODAK_VERTICAL_CLOCK_OVERLAPS: u16 = 0x0412;
pub const KODAK_SENSOR_TEMPERATURE: u16 = 0x0413;
pub const KODAK_XILINX_VERSION: u16 = 0x0414;
pub const KODAK_FIRMWARE_VERSION: u16 = 0x0415;
pub const KODAK_BLACK_LEVEL_ROUGH_AFTER: u16 = 0x0416;
pub const KODAK_BRIGHT_ROWS_TOP: u16 = 0x0417;
pub const KODAK_EVENT_LOG_PROCESS: u16 = 0x0418;
pub const KODAK_DAC_VOLTAGES_FLUSH: u16 = 0x0419;
pub const KODAK_FLASH_USED: u16 = 0x041a;
pub const KODAK_FLASH_TYPE: u16 = 0x041b;
pub const KODAK_SELF_TIMER: u16 = 0x041c;
pub const KODAK_AF_MODE: u16 = 0x041d;
pub const KODAK_LENS_TYPE: u16 = 0x041e;
pub const KODAK_IMAGE_CROP_X: u16 = 0x041f;
pub const KODAK_IMAGE_CROP_Y: u16 = 0x0420;
pub const KODAK_ADJUSTED_TBN_IMAGE_WIDTH: u16 = 0x0421;
pub const KODAK_ADJUSTED_TBN_IMAGE_HEIGHT: u16 = 0x0422;
pub const KODAK_INTEGRATION_TIME: u16 = 0x0423;
pub const KODAK_BRACKETING_MODE: u16 = 0x0424;
pub const KODAK_BRACKETING_STEP: u16 = 0x0425;
pub const KODAK_BRACKETING_COUNTER: u16 = 0x0426;
pub const KODAK_HUFFMAN_TABLE_LENGTH: u16 = 0x042e;
pub const KODAK_HUFFMAN_TABLE_VALUE: u16 = 0x042f;
pub const KODAK_MAIN_BOARD_VERSION: u16 = 0x0438;
pub const KODAK_IMAGER_BOARD_VERSION: u16 = 0x0439;
pub const KODAK_FOCUS_EDGE_MAP: u16 = 0x044c;
pub const KODAK_COLOR_TEMPERATURE: u16 = 0x0846;

// Kodak Type 1 (Main) binary format tag offsets
// Used by consumer cameras: C360, DX series, Z series, etc.
pub const KODAK_MODEL_OFFSET: usize = 0x00;
pub const KODAK_QUALITY_OFFSET: usize = 0x09;
pub const KODAK_BURST_MODE_OFFSET: usize = 0x0a;
pub const KODAK_IMAGE_WIDTH_OFFSET: usize = 0x0c;
pub const KODAK_IMAGE_HEIGHT_OFFSET: usize = 0x0e;
pub const KODAK_YEAR_CREATED_OFFSET: usize = 0x10;
pub const KODAK_MONTH_DAY_CREATED_OFFSET: usize = 0x12;
pub const KODAK_TIME_CREATED_OFFSET: usize = 0x14;
pub const KODAK_SHUTTER_MODE_OFFSET: usize = 0x1b;
pub const KODAK_METERING_MODE_OFFSET: usize = 0x1c;
pub const KODAK_SEQUENCE_NUMBER_OFFSET: usize = 0x1d;
pub const KODAK_F_NUMBER_OFFSET: usize = 0x1e;
pub const KODAK_EXPOSURE_TIME_OFFSET: usize = 0x20;
pub const KODAK_EXPOSURE_COMPENSATION_OFFSET: usize = 0x24;
pub const KODAK_FOCUS_MODE_BINARY_OFFSET: usize = 0x38;
pub const KODAK_WB_OFFSET: usize = 0x40;
pub const KODAK_FLASH_MODE_BINARY_OFFSET: usize = 0x5c;
pub const KODAK_FLASH_FIRED_OFFSET: usize = 0x5d;
pub const KODAK_ISO_SETTING_OFFSET: usize = 0x5e;
pub const KODAK_ISO_OFFSET: usize = 0x60;
pub const KODAK_TOTAL_ZOOM_OFFSET: usize = 0x62;
pub const KODAK_COLOR_MODE_OFFSET: usize = 0x66;
pub const KODAK_DIGITAL_ZOOM_OFFSET: usize = 0x68;
pub const KODAK_SHARPNESS_OFFSET: usize = 0x6b;

/// Get the name of a Kodak IFD maker note tag
pub fn get_kodak_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        KODAK_VERSION => Some("KodakVersion"),
        KODAK_UNKNOWN_EV => Some("UnknownEV"),
        KODAK_EXPOSURE_VALUE => Some("ExposureValue"),
        KODAK_ORIGINAL_FILE_NAME => Some("OriginalFileName"),
        KODAK_TAG => Some("KodakTag"),
        KODAK_SENSOR_LEFT_BORDER => Some("SensorLeftBorder"),
        KODAK_SENSOR_TOP_BORDER => Some("SensorTopBorder"),
        KODAK_SENSOR_IMAGE_WIDTH => Some("SensorImageWidth"),
        KODAK_SENSOR_IMAGE_HEIGHT => Some("SensorImageHeight"),
        KODAK_BLACK_LEVEL_TOP => Some("BlackLevelTop"),
        KODAK_BLACK_LEVEL_BOTTOM => Some("BlackLevelBottom"),
        KODAK_TEXTUAL_INFO => Some("TextualInfo"),
        KODAK_FLASH_MODE => Some("FlashMode"),
        KODAK_FLASH_COMPENSATION => Some("FlashCompensation"),
        KODAK_WIND_MODE => Some("WindMode"),
        KODAK_FOCUS_MODE => Some("FocusMode"),
        KODAK_MIN_APERTURE => Some("MinAperture"),
        KODAK_MAX_APERTURE => Some("MaxAperture"),
        KODAK_WHITE_BALANCE_MODE => Some("WhiteBalanceMode"),
        KODAK_WHITE_BALANCE_DETECTED => Some("WhiteBalanceDetected"),
        KODAK_WHITE_BALANCE => Some("WhiteBalance"),
        KODAK_PROCESSING => Some("Processing"),
        KODAK_IMAGE_ABSOLUTE_X => Some("ImageAbsoluteX"),
        KODAK_IMAGE_ABSOLUTE_Y => Some("ImageAbsoluteY"),
        KODAK_APPLICATION_KEY_STRING => Some("ApplicationKeyString"),
        KODAK_TIME => Some("Time"),
        KODAK_GPS_STRING => Some("GPSString"),
        KODAK_EVENT_LOG_CAPTURE => Some("EventLogCapture"),
        KODAK_COMPONENT_TABLE => Some("ComponentTable"),
        KODAK_CUSTOM_ILLUMINANT => Some("CustomIlluminant"),
        KODAK_CAMERA_TEMPERATURE => Some("CameraTemperature"),
        KODAK_ADAPTER_VOLTAGE => Some("AdapterVoltage"),
        KODAK_BATTERY_VOLTAGE => Some("BatteryVoltage"),
        KODAK_DAC_VOLTAGES => Some("DacVoltages"),
        KODAK_ILLUMINANT_DETECTOR_DATA => Some("IlluminantDetectorData"),
        KODAK_PIXEL_CLOCK_FREQUENCY => Some("PixelClockFrequency"),
        KODAK_CENTER_PIXEL => Some("CenterPixel"),
        KODAK_BURST_COUNT => Some("BurstCount"),
        KODAK_BLACK_LEVEL_ROUGH => Some("BlackLevelRough"),
        KODAK_OFFSET_MAP_HORIZONTAL => Some("OffsetMapHorizontal"),
        KODAK_OFFSET_MAP_VERTICAL => Some("OffsetMapVertical"),
        KODAK_HISTOGRAM => Some("Histogram"),
        KODAK_VERTICAL_CLOCK_OVERLAPS => Some("VerticalClockOverlaps"),
        KODAK_SENSOR_TEMPERATURE => Some("SensorTemperature"),
        KODAK_XILINX_VERSION => Some("XilinxVersion"),
        KODAK_FIRMWARE_VERSION => Some("FirmwareVersion"),
        KODAK_BLACK_LEVEL_ROUGH_AFTER => Some("BlackLevelRoughAfter"),
        KODAK_BRIGHT_ROWS_TOP => Some("BrightRowsTop"),
        KODAK_EVENT_LOG_PROCESS => Some("EventLogProcess"),
        KODAK_DAC_VOLTAGES_FLUSH => Some("DacVoltagesFlush"),
        KODAK_FLASH_USED => Some("FlashUsed"),
        KODAK_FLASH_TYPE => Some("FlashType"),
        KODAK_SELF_TIMER => Some("SelfTimer"),
        KODAK_AF_MODE => Some("AFMode"),
        KODAK_LENS_TYPE => Some("LensType"),
        KODAK_IMAGE_CROP_X => Some("ImageCropX"),
        KODAK_IMAGE_CROP_Y => Some("ImageCropY"),
        KODAK_ADJUSTED_TBN_IMAGE_WIDTH => Some("AdjustedTbnImageWidth"),
        KODAK_ADJUSTED_TBN_IMAGE_HEIGHT => Some("AdjustedTbnImageHeight"),
        KODAK_INTEGRATION_TIME => Some("IntegrationTime"),
        KODAK_BRACKETING_MODE => Some("BracketingMode"),
        KODAK_BRACKETING_STEP => Some("BracketingStep"),
        KODAK_BRACKETING_COUNTER => Some("BracketingCounter"),
        KODAK_HUFFMAN_TABLE_LENGTH => Some("HuffmanTableLength"),
        KODAK_HUFFMAN_TABLE_VALUE => Some("HuffmanTableValue"),
        KODAK_MAIN_BOARD_VERSION => Some("MainBoardVersion"),
        KODAK_IMAGER_BOARD_VERSION => Some("ImagerBoardVersion"),
        KODAK_FOCUS_EDGE_MAP => Some("FocusEdgeMap"),
        KODAK_COLOR_TEMPERATURE => Some("ColorTemperature"),
        _ => None,
    }
}

// ===== Tag decoders using the define_tag_decoder! macro =====
// Note: Kodak has no exiv2 support, so we use 'both:' for identical exiftool/exiv2 output

define_tag_decoder! {
    kodak_quality,
    type: u8,
    both: {
        1 => "Fine",
        2 => "Normal",
    }
}

define_tag_decoder! {
    kodak_burst_mode,
    type: u8,
    both: {
        0 => "Off",
        1 => "On",
    }
}

define_tag_decoder! {
    kodak_shutter_mode,
    type: u8,
    both: {
        0 => "Auto",
        8 => "Aperture Priority",
        32 => "Manual?",
    }
}

define_tag_decoder! {
    kodak_metering_mode,
    type: u8,
    both: {
        0 => "Multi-segment",
        1 => "Center-weighted average",
        2 => "Spot",
    }
}

define_tag_decoder! {
    kodak_focus_mode_binary,
    type: u8,
    both: {
        0 => "Normal",
        2 => "Macro",
    }
}

define_tag_decoder! {
    kodak_white_balance,
    type: u8,
    both: {
        0 => "Auto",
        1 => "Flash?",
        2 => "Tungsten",
        3 => "Daylight",
        5 => "Auto",
        6 => "Shade",
    }
}

define_tag_decoder! {
    kodak_flash_mode_binary,
    type: u8,
    both: {
        0x00 => "Auto",
        0x01 => "Fill Flash",
        0x02 => "Off",
        0x03 => "Red-Eye",
        0x10 => "Fill Flash",
        0x20 => "Off",
        0x40 => "Red-Eye?",
    }
}

define_tag_decoder! {
    kodak_flash_fired,
    type: u8,
    both: {
        0 => "No",
        1 => "Yes",
    }
}

define_tag_decoder! {
    kodak_color_mode,
    both: {
        0x01 => "B&W",
        0x02 => "Sepia",
        0x03 => "B&W Yellow Filter",
        0x04 => "B&W Red Filter",
        0x20 => "Saturated Color",
        0x40 => "Neutral Color",
        0x100 => "Saturated Color",
        0x200 => "Neutral Color",
        0x2000 => "B&W",
        0x4000 => "Sepia",
    }
}

// Additional Kodak tags from Kodak.pm

define_tag_decoder! {
    kodak_scene_mode,
    both: {
        1 => "Sport",
        3 => "Portrait",
        4 => "Landscape",
        6 => "Beach",
        7 => "Night Portrait",
        8 => "Night Landscape",
        9 => "Snow",
        10 => "Text",
        11 => "Fireworks",
        12 => "Macro",
        13 => "Museum",
        16 => "Children",
        17 => "Program",
        18 => "Aperture Priority",
        19 => "Shutter Priority",
        20 => "Manual",
    }
}

define_tag_decoder! {
    kodak_scene_mode_used,
    both: {
        0 => "Program",
        2 => "Aperture Priority",
        3 => "Shutter Priority",
        4 => "Manual",
        5 => "Portrait",
        6 => "Sport",
        7 => "Children",
        8 => "Museum",
        10 => "High ISO",
        11 => "Text",
        12 => "Macro",
        13 => "Back Light",
        16 => "Landscape",
        17 => "Night Landscape",
        18 => "Night Portrait",
        19 => "Snow",
        20 => "Beach",
        21 => "Fireworks",
        22 => "Sunset",
        23 => "Candlelight",
        28 => "Panorama",
    }
}

define_tag_decoder! {
    kodak_picture_effect,
    both: {
        0 => "None",
        3 => "Monochrome",
        9 => "Kodachrome",
    }
}

define_tag_decoder! {
    kodak_image_rotated,
    type: u8,
    both: {
        0 => "No",
        1 => "Yes",
    }
}

define_tag_decoder! {
    kodak_macro,
    type: u8,
    both: {
        0 => "On",
        1 => "Off",
    }
}

define_tag_decoder! {
    kodak_flash_type5,
    type: u8,
    both: {
        0 => "Auto",
        1 => "On",
        2 => "Off",
        3 => "Red-Eye",
    }
}

// Legacy function aliases for backward compatibility
pub fn decode_quality_exiftool(value: u8) -> &'static str {
    decode_kodak_quality_exiftool(value)
}

pub fn decode_burst_mode_exiftool(value: u8) -> &'static str {
    decode_kodak_burst_mode_exiftool(value)
}

pub fn decode_shutter_mode_exiftool(value: u8) -> &'static str {
    decode_kodak_shutter_mode_exiftool(value)
}

pub fn decode_metering_mode_exiftool(value: u8) -> &'static str {
    decode_kodak_metering_mode_exiftool(value)
}

pub fn decode_focus_mode_binary_exiftool(value: u8) -> &'static str {
    decode_kodak_focus_mode_binary_exiftool(value)
}

pub fn decode_white_balance_exiftool(value: u8) -> &'static str {
    decode_kodak_white_balance_exiftool(value)
}

pub fn decode_flash_mode_binary_exiftool(value: u8) -> &'static str {
    decode_kodak_flash_mode_binary_exiftool(value)
}

pub fn decode_flash_fired_exiftool(value: u8) -> &'static str {
    decode_kodak_flash_fired_exiftool(value)
}

pub fn decode_color_mode_exiftool(value: u16) -> &'static str {
    decode_kodak_color_mode_exiftool(value)
}

/// Parse Kodak maker notes
///
/// Kodak uses multiple makernote formats:
/// - IFD-based format (professional cameras like DCS Pro 14N, DC50, DC120)
/// - Binary formats Type1-11 (various consumer camera series)
///
/// This function attempts to detect the format and parse accordingly.
pub fn parse_kodak_maker_notes(
    data: &[u8],
    endian: Endianness,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    if data.len() < 10 {
        return Ok(HashMap::new());
    }

    // Check for "KDK" header (some Kodak models)
    let ifd_offset = if data.len() >= 8 && &data[0..3] == b"KDK" {
        // KDK header format
        // Skip "KDK" (3 bytes) + null byte (1 byte) + potential padding
        8
    } else {
        // No special header, IFD starts at beginning
        0
    };

    if ifd_offset >= data.len() {
        return Ok(HashMap::new());
    }

    // Try to parse as IFD format
    let parse_result = parse_kodak_ifd(&data[ifd_offset..], endian);

    match parse_result {
        Ok(parsed_tags) if !parsed_tags.is_empty() => {
            // Successfully parsed as IFD
            Ok(parsed_tags)
        }
        _ => {
            // Failed to parse as IFD, try binary format
            // Binary format used by consumer cameras (Type 1)
            if data.len() >= 0x70 {
                parse_kodak_binary_format(data, endian)
            } else {
                // Not enough data for binary format either
                Ok(HashMap::new())
            }
        }
    }
}

/// Parse Kodak IFD-based maker notes
fn parse_kodak_ifd(
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
            .map_err(|_| ExifError::Format("Failed to read Kodak IFD entry count".to_string()))?,
        Endianness::Big => cursor
            .read_u16::<BigEndian>()
            .map_err(|_| ExifError::Format("Failed to read Kodak IFD entry count".to_string()))?,
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
            // Calculate value size in bytes
            let value_size = match tag_type {
                1 => count as usize,      // BYTE
                2 => count as usize,      // ASCII
                3 => count as usize * 2,  // SHORT
                4 => count as usize * 4,  // LONG
                5 => count as usize * 8,  // RATIONAL
                7 => count as usize,      // UNDEFINED
                10 => count as usize * 8, // SRATIONAL
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
                        if let Ok(v) = match endian {
                            Endianness::Little => val_cursor.read_u16::<LittleEndian>(),
                            Endianness::Big => val_cursor.read_u16::<BigEndian>(),
                        } {
                            values.push(v);
                        } else {
                            break;
                        }
                    }
                    ExifValue::Short(values)
                }
                4 => {
                    // LONG
                    let mut values = Vec::new();
                    let mut val_cursor = Cursor::new(&value_bytes);
                    for _ in 0..count {
                        if let Ok(v) = match endian {
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
                        let numerator = match endian {
                            Endianness::Little => val_cursor.read_u32::<LittleEndian>(),
                            Endianness::Big => val_cursor.read_u32::<BigEndian>(),
                        };
                        let denominator = match endian {
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
                10 => {
                    // SRATIONAL
                    let mut values = Vec::new();
                    let mut val_cursor = Cursor::new(&value_bytes);
                    for _ in 0..count {
                        let numerator = match endian {
                            Endianness::Little => val_cursor.read_i32::<LittleEndian>(),
                            Endianness::Big => val_cursor.read_i32::<BigEndian>(),
                        };
                        let denominator = match endian {
                            Endianness::Little => val_cursor.read_i32::<LittleEndian>(),
                            Endianness::Big => val_cursor.read_i32::<BigEndian>(),
                        };

                        if let (Ok(num), Ok(den)) = (numerator, denominator) {
                            values.push((num, den));
                        } else {
                            break;
                        }
                    }
                    ExifValue::SRational(values)
                }
                _ => {
                    // Unsupported type
                    continue;
                }
            };

            tags.insert(
                tag_id,
                MakerNoteTag::new(tag_id, get_kodak_tag_name(tag_id), value),
            );
        }
    }

    Ok(tags)
}

/// Parse Kodak Type 1 binary format maker notes
/// Used by consumer cameras: C360, DX series, Z series, etc.
fn parse_kodak_binary_format(
    data: &[u8],
    _endian: Endianness,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    // Type 1 binary format starts at offset 8 according to ExifTool
    let base_offset = 8;

    if data.len() < base_offset {
        return Ok(tags);
    }

    // Helper function to safely read a byte
    let read_u8 = |offset: usize| -> Option<u8> {
        if base_offset + offset < data.len() {
            Some(data[base_offset + offset])
        } else {
            None
        }
    };

    // Helper function to safely read a u16 (little-endian)
    let read_u16_le = |offset: usize| -> Option<u16> {
        if base_offset + offset + 1 < data.len() {
            Some(u16::from_le_bytes([
                data[base_offset + offset],
                data[base_offset + offset + 1],
            ]))
        } else {
            None
        }
    };

    // Helper function to safely read a string
    let read_string = |offset: usize, len: usize| -> Option<String> {
        if base_offset + offset + len <= data.len() {
            let bytes = &data[base_offset + offset..base_offset + offset + len];
            Some(
                String::from_utf8_lossy(bytes)
                    .trim_end_matches('\0')
                    .to_string(),
            )
        } else {
            None
        }
    };

    // Parse KodakModel (offset 0x00, string[8])
    if let Some(model) = read_string(KODAK_MODEL_OFFSET, 8) {
        if !model.is_empty() {
            tags.insert(
                0xF000,
                MakerNoteTag::new(0xF000, Some("KodakModel"), ExifValue::Ascii(model)),
            );
        }
    }

    // Parse Quality (offset 0x09)
    if let Some(quality) = read_u8(KODAK_QUALITY_OFFSET) {
        if quality > 0 {
            tags.insert(
                0xF001,
                MakerNoteTag::new(
                    0xF001,
                    Some("Quality"),
                    ExifValue::Ascii(decode_quality_exiftool(quality).to_string()),
                ),
            );
        }
    }

    // Parse BurstMode (offset 0x0a)
    if let Some(burst) = read_u8(KODAK_BURST_MODE_OFFSET) {
        tags.insert(
            0xF002,
            MakerNoteTag::new(
                0xF002,
                Some("BurstMode"),
                ExifValue::Ascii(decode_burst_mode_exiftool(burst).to_string()),
            ),
        );
    }

    // Parse ShutterMode (offset 0x1b)
    if let Some(shutter) = read_u8(KODAK_SHUTTER_MODE_OFFSET) {
        tags.insert(
            0xF003,
            MakerNoteTag::new(
                0xF003,
                Some("ShutterMode"),
                ExifValue::Ascii(decode_shutter_mode_exiftool(shutter).to_string()),
            ),
        );
    }

    // Parse MeteringMode (offset 0x1c)
    if let Some(metering) = read_u8(KODAK_METERING_MODE_OFFSET) {
        tags.insert(
            0xF004,
            MakerNoteTag::new(
                0xF004,
                Some("MeteringMode"),
                ExifValue::Ascii(decode_metering_mode_exiftool(metering).to_string()),
            ),
        );
    }

    // Parse FocusMode (offset 0x38)
    if let Some(focus) = read_u8(KODAK_FOCUS_MODE_BINARY_OFFSET) {
        tags.insert(
            0xF005,
            MakerNoteTag::new(
                0xF005,
                Some("FocusMode"),
                ExifValue::Ascii(decode_focus_mode_binary_exiftool(focus).to_string()),
            ),
        );
    }

    // Parse WhiteBalance (offset 0x40)
    if let Some(wb) = read_u8(KODAK_WB_OFFSET) {
        tags.insert(
            0xF006,
            MakerNoteTag::new(
                0xF006,
                Some("WhiteBalance"),
                ExifValue::Ascii(decode_white_balance_exiftool(wb).to_string()),
            ),
        );
    }

    // Parse FlashMode (offset 0x5c)
    if let Some(flash_mode) = read_u8(KODAK_FLASH_MODE_BINARY_OFFSET) {
        tags.insert(
            0xF007,
            MakerNoteTag::new(
                0xF007,
                Some("FlashMode"),
                ExifValue::Ascii(decode_flash_mode_binary_exiftool(flash_mode).to_string()),
            ),
        );
    }

    // Parse FlashFired (offset 0x5d)
    if let Some(flash_fired) = read_u8(KODAK_FLASH_FIRED_OFFSET) {
        tags.insert(
            0xF008,
            MakerNoteTag::new(
                0xF008,
                Some("FlashFired"),
                ExifValue::Ascii(decode_flash_fired_exiftool(flash_fired).to_string()),
            ),
        );
    }

    // Parse ISOSetting (offset 0x5e, int16u)
    if let Some(iso_setting) = read_u16_le(KODAK_ISO_SETTING_OFFSET) {
        let iso_str = if iso_setting == 0 {
            "Auto".to_string()
        } else {
            iso_setting.to_string()
        };
        tags.insert(
            0xF009,
            MakerNoteTag::new(0xF009, Some("ISOSetting"), ExifValue::Ascii(iso_str)),
        );
    }

    // Parse ISO (offset 0x60, int16u)
    if let Some(iso) = read_u16_le(KODAK_ISO_OFFSET) {
        if iso > 0 {
            tags.insert(
                0xF00A,
                MakerNoteTag::new(0xF00A, Some("ISO"), ExifValue::Short(vec![iso])),
            );
        }
    }

    // Parse ColorMode (offset 0x66, int16u)
    if let Some(color_mode) = read_u16_le(KODAK_COLOR_MODE_OFFSET) {
        if color_mode > 0 {
            tags.insert(
                0xF00B,
                MakerNoteTag::new(
                    0xF00B,
                    Some("ColorMode"),
                    ExifValue::Ascii(decode_color_mode_exiftool(color_mode).to_string()),
                ),
            );
        }
    }

    // Parse Sharpness (offset 0x6b, int8s)
    if let Some(sharpness) = read_u8(KODAK_SHARPNESS_OFFSET) {
        // Treat as signed
        let sharpness_i8 = sharpness as i8;
        tags.insert(
            0xF00C,
            MakerNoteTag::new(
                0xF00C,
                Some("Sharpness"),
                ExifValue::Ascii(sharpness_i8.to_string()),
            ),
        );
    }

    Ok(tags)
}
