// makernotes/phaseone.rs - Phase One maker notes parsing
//
// Reference files:
// - exiftool/lib/Image/ExifTool/PhaseOne.pm
//
// Phase One cameras (professional medium format digital backs) use a proprietary
// IFD-based format with a custom header structure. The format includes both the
// main PhaseOne IFD (16 bytes per entry) and a SensorCalibration sub-IFD (12 bytes per entry).
//
// Note: exiv2 does not have Phase One MakerNote support, so we only implement
// ExifTool-compatible output.

use crate::data_types::{Endianness, ExifValue};
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Cursor;

// Phase One Main IFD tag IDs (from PhaseOne.pm)
pub const PHASEONE_CAMERA_ORIENTATION: u16 = 0x0100;
pub const PHASEONE_SERIAL_NUMBER: u16 = 0x0102;
pub const PHASEONE_ISO: u16 = 0x0105;
pub const PHASEONE_COLOR_MATRIX1: u16 = 0x0106;
pub const PHASEONE_WB_RGB_LEVELS: u16 = 0x0107;
pub const PHASEONE_SENSOR_WIDTH: u16 = 0x0108;
pub const PHASEONE_SENSOR_HEIGHT: u16 = 0x0109;
pub const PHASEONE_SENSOR_LEFT_MARGIN: u16 = 0x010a;
pub const PHASEONE_SENSOR_TOP_MARGIN: u16 = 0x010b;
pub const PHASEONE_IMAGE_WIDTH: u16 = 0x010c;
pub const PHASEONE_IMAGE_HEIGHT: u16 = 0x010d;
pub const PHASEONE_RAW_FORMAT: u16 = 0x010e;
pub const PHASEONE_RAW_DATA: u16 = 0x010f;
pub const PHASEONE_SENSOR_CALIBRATION: u16 = 0x0110;
pub const PHASEONE_DATE_TIME_ORIGINAL: u16 = 0x0112;
pub const PHASEONE_IMAGE_NUMBER: u16 = 0x0113;
pub const PHASEONE_SOFTWARE: u16 = 0x0203;
pub const PHASEONE_SYSTEM: u16 = 0x0204;
pub const PHASEONE_SENSOR_TEMPERATURE: u16 = 0x0210;
pub const PHASEONE_SENSOR_TEMPERATURE2: u16 = 0x0211;
pub const PHASEONE_UNKNOWN_DATE: u16 = 0x0212;
pub const PHASEONE_STRIP_OFFSETS: u16 = 0x021c;
pub const PHASEONE_BLACK_LEVEL: u16 = 0x021d;
pub const PHASEONE_SPLIT_COLUMN: u16 = 0x0222;
pub const PHASEONE_BLACK_LEVEL_DATA: u16 = 0x0223;
pub const PHASEONE_COLOR_MATRIX2: u16 = 0x0226;
pub const PHASEONE_AF_ADJUSTMENT: u16 = 0x0267;
pub const PHASEONE_SEQUENCE_ID: u16 = 0x0262;
pub const PHASEONE_SEQUENCE_KIND: u16 = 0x0263;
pub const PHASEONE_SEQUENCE_FRAME_NUMBER: u16 = 0x0264;
pub const PHASEONE_SEQUENCE_FRAME_COUNT: u16 = 0x0265;
pub const PHASEONE_FIRMWARE_VERSIONS: u16 = 0x0301;
pub const PHASEONE_SHUTTER_SPEED_VALUE: u16 = 0x0400;
pub const PHASEONE_APERTURE_VALUE: u16 = 0x0401;
pub const PHASEONE_EXPOSURE_COMPENSATION: u16 = 0x0402;
pub const PHASEONE_FOCAL_LENGTH: u16 = 0x0403;
pub const PHASEONE_CAMERA_MODEL: u16 = 0x0410;
pub const PHASEONE_LENS_MODEL: u16 = 0x0412;
pub const PHASEONE_MAX_APERTURE_VALUE: u16 = 0x0414;
pub const PHASEONE_MIN_APERTURE_VALUE: u16 = 0x0415;
pub const PHASEONE_VIEWFINDER: u16 = 0x0455;

// SensorCalibration sub-IFD tag IDs
pub const PHASEONE_SENSOR_DEFECTS: u16 = 0x0400;
pub const PHASEONE_ALL_COLOR_FLAT_FIELD1: u16 = 0x0401;
pub const PHASEONE_SENSOR_CAL_SERIAL_NUMBER: u16 = 0x0407;
pub const PHASEONE_RED_BLUE_FLAT_FIELD: u16 = 0x040b;
pub const PHASEONE_ALL_COLOR_FLAT_FIELD2: u16 = 0x0410;
pub const PHASEONE_ALL_COLOR_FLAT_FIELD3: u16 = 0x0416;
pub const PHASEONE_LINEARIZATION_COEFFICIENTS1: u16 = 0x0419;
pub const PHASEONE_LINEARIZATION_COEFFICIENTS2: u16 = 0x041a;

/// Get the name of a Phase One maker note tag
pub fn get_phaseone_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        PHASEONE_CAMERA_ORIENTATION => Some("CameraOrientation"),
        PHASEONE_SERIAL_NUMBER => Some("SerialNumber"),
        PHASEONE_ISO => Some("ISO"),
        PHASEONE_COLOR_MATRIX1 => Some("ColorMatrix1"),
        PHASEONE_WB_RGB_LEVELS => Some("WB_RGBLevels"),
        PHASEONE_SENSOR_WIDTH => Some("SensorWidth"),
        PHASEONE_SENSOR_HEIGHT => Some("SensorHeight"),
        PHASEONE_SENSOR_LEFT_MARGIN => Some("SensorLeftMargin"),
        PHASEONE_SENSOR_TOP_MARGIN => Some("SensorTopMargin"),
        PHASEONE_IMAGE_WIDTH => Some("ImageWidth"),
        PHASEONE_IMAGE_HEIGHT => Some("ImageHeight"),
        PHASEONE_RAW_FORMAT => Some("RawFormat"),
        PHASEONE_RAW_DATA => Some("RawData"),
        PHASEONE_SENSOR_CALIBRATION => Some("SensorCalibration"),
        PHASEONE_DATE_TIME_ORIGINAL => Some("DateTimeOriginal"),
        PHASEONE_IMAGE_NUMBER => Some("ImageNumber"),
        PHASEONE_SOFTWARE => Some("Software"),
        PHASEONE_SYSTEM => Some("System"),
        PHASEONE_SENSOR_TEMPERATURE => Some("SensorTemperature"),
        PHASEONE_SENSOR_TEMPERATURE2 => Some("SensorTemperature2"),
        PHASEONE_UNKNOWN_DATE => Some("UnknownDate"),
        PHASEONE_STRIP_OFFSETS => Some("StripOffsets"),
        PHASEONE_BLACK_LEVEL => Some("BlackLevel"),
        PHASEONE_SPLIT_COLUMN => Some("SplitColumn"),
        PHASEONE_BLACK_LEVEL_DATA => Some("BlackLevelData"),
        PHASEONE_COLOR_MATRIX2 => Some("ColorMatrix2"),
        PHASEONE_AF_ADJUSTMENT => Some("AFAdjustment"),
        PHASEONE_SEQUENCE_ID => Some("SequenceID"),
        PHASEONE_SEQUENCE_KIND => Some("SequenceKind"),
        PHASEONE_SEQUENCE_FRAME_NUMBER => Some("SequenceFrameNumber"),
        PHASEONE_SEQUENCE_FRAME_COUNT => Some("SequenceFrameCount"),
        PHASEONE_FIRMWARE_VERSIONS => Some("FirmwareVersions"),
        PHASEONE_SHUTTER_SPEED_VALUE => Some("ShutterSpeedValue"),
        PHASEONE_APERTURE_VALUE => Some("ApertureValue"),
        PHASEONE_EXPOSURE_COMPENSATION => Some("ExposureCompensation"),
        PHASEONE_FOCAL_LENGTH => Some("FocalLength"),
        PHASEONE_CAMERA_MODEL => Some("CameraModel"),
        PHASEONE_LENS_MODEL => Some("LensModel"),
        PHASEONE_MAX_APERTURE_VALUE => Some("MaxApertureValue"),
        PHASEONE_MIN_APERTURE_VALUE => Some("MinApertureValue"),
        PHASEONE_VIEWFINDER => Some("Viewfinder"),
        _ => None,
    }
}

/// Decode CameraOrientation value (tag 0x0100)
pub fn decode_camera_orientation_exiftool(value: u32) -> &'static str {
    match value & 0x03 {
        0 => "Horizontal (normal)",
        1 => "Rotate 90 CW",
        2 => "Rotate 270 CW",
        3 => "Rotate 180",
        _ => "Unknown",
    }
}

/// Decode RawFormat value (tag 0x010e)
pub fn decode_raw_format_exiftool(value: u32) -> &'static str {
    match value {
        0 => "Uncompressed",
        1 => "RAW 1",
        2 => "RAW 2",
        3 => "IIQ L",
        5 => "IIQ S",
        6 => "IIQ Sv2",
        8 => "IIQ L16",
        _ => "Unknown",
    }
}

/// Decode SequenceKind value (tag 0x0263)
pub fn decode_sequence_kind_exiftool(value: u32) -> &'static str {
    match value {
        0 => "Bracketing: Shutter Speed",
        1 => "Bracketing: Aperture",
        2 => "Bracketing: ISO",
        3 => "Hyperfocal",
        4 => "Time Lapse",
        5 => "HDR",
        6 => "Focus Stacking",
        _ => "Unknown",
    }
}

/// Parse Phase One maker notes
///
/// Phase One uses a proprietary IFD format with a special header:
/// - Bytes 0-3: "IIII" (little-endian) or "MMMM" (big-endian)
/// - Bytes 4-7: ".waR" or "Raw." signature
/// - Bytes 8-11: Offset to IFD start (32-bit integer)
///
/// Each IFD entry is 16 bytes:
/// - Tag ID (4 bytes)
/// - Format size (4 bytes): 1=string, 2=int16s, 4=int32s
/// - Data size (4 bytes)
/// - Value or offset (4 bytes)
pub fn parse_phaseone_maker_notes(
    data: &[u8],
    endian: Endianness,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    if data.len() < 12 {
        return Ok(HashMap::new());
    }

    // Check for Phase One header signature
    // Pattern: IIII.waR (little-endian) or MMMMRaw. (big-endian)
    // where the 5th byte can be any character
    let header_sig = &data[5..8];
    let is_phaseone = match endian {
        Endianness::Little => header_sig == b"waR",
        Endianness::Big => header_sig == b"aw.",
    };

    if !is_phaseone {
        return Ok(HashMap::new());
    }

    // Read IFD offset from header
    let mut cursor = Cursor::new(&data[8..12]);
    let ifd_offset = match endian {
        Endianness::Little => cursor.read_u32::<LittleEndian>(),
        Endianness::Big => cursor.read_u32::<BigEndian>(),
    }
    .map_err(|_| ExifError::Format("Failed to read PhaseOne IFD offset".to_string()))?
        as usize;

    if ifd_offset + 8 > data.len() {
        return Ok(HashMap::new());
    }

    // Read number of entries
    let mut cursor = Cursor::new(&data[ifd_offset..]);
    let num_entries = match endian {
        Endianness::Little => cursor.read_u32::<LittleEndian>(),
        Endianness::Big => cursor.read_u32::<BigEndian>(),
    }
    .map_err(|_| ExifError::Format("Failed to read PhaseOne entry count".to_string()))?;

    // Sanity check
    if !(2..=300).contains(&num_entries) {
        return Ok(HashMap::new());
    }

    let ifd_end = ifd_offset + 8 + (16 * num_entries as usize);
    if ifd_end > data.len() {
        return Ok(HashMap::new());
    }

    let mut tags = HashMap::new();

    // Parse each IFD entry (16 bytes each)
    for i in 0..num_entries {
        let entry_offset = ifd_offset + 8 + (i as usize * 16);
        if entry_offset + 16 > data.len() {
            break;
        }

        let mut entry_cursor = Cursor::new(&data[entry_offset..entry_offset + 16]);

        // Read entry fields
        let tag_id = match endian {
            Endianness::Little => entry_cursor.read_u32::<LittleEndian>(),
            Endianness::Big => entry_cursor.read_u32::<BigEndian>(),
        }
        .ok()
        .unwrap_or(0) as u16;

        let format_size = match endian {
            Endianness::Little => entry_cursor.read_u32::<LittleEndian>(),
            Endianness::Big => entry_cursor.read_u32::<BigEndian>(),
        }
        .ok()
        .unwrap_or(0);

        let data_size = match endian {
            Endianness::Little => entry_cursor.read_u32::<LittleEndian>(),
            Endianness::Big => entry_cursor.read_u32::<BigEndian>(),
        }
        .ok()
        .unwrap_or(0) as usize;

        let value_or_offset = match endian {
            Endianness::Little => entry_cursor.read_u32::<LittleEndian>(),
            Endianness::Big => entry_cursor.read_u32::<BigEndian>(),
        }
        .ok()
        .unwrap_or(0);

        // Skip invalid entries
        if data_size == 0 || data_size > 0x7fffffff {
            continue;
        }

        // Determine format string based on format_size
        let format_str = match format_size {
            1 => "string",
            2 => "int16s",
            4 => "int32s",
            _ => "undef",
        };

        // Get value bytes
        let value_bytes = if data_size <= 4 {
            // Inline value
            match endian {
                Endianness::Little => value_or_offset.to_le_bytes()[..data_size].to_vec(),
                Endianness::Big => value_or_offset.to_be_bytes()[4 - data_size..].to_vec(),
            }
        } else {
            // Value at offset
            let abs_offset = value_or_offset as usize;
            if abs_offset + data_size <= data.len() {
                data[abs_offset..abs_offset + data_size].to_vec()
            } else {
                continue;
            }
        };

        // Parse value based on format
        let value = parse_phaseone_value(&value_bytes, format_str, endian, tag_id);

        if let Some(v) = value {
            tags.insert(
                tag_id,
                MakerNoteTag::new(tag_id, get_phaseone_tag_name(tag_id), v),
            );
        }
    }

    Ok(tags)
}

/// Parse Phase One value based on format string
fn parse_phaseone_value(
    bytes: &[u8],
    format: &str,
    endian: Endianness,
    tag_id: u16,
) -> Option<ExifValue> {
    match format {
        "string" => {
            let s = String::from_utf8_lossy(bytes)
                .trim_end_matches('\0')
                .to_string();
            Some(ExifValue::Ascii(s))
        }
        "int16s" => {
            let mut values = Vec::new();
            let mut cursor = Cursor::new(bytes);
            while cursor.position() < bytes.len() as u64 {
                if let Ok(v) = match endian {
                    Endianness::Little => cursor.read_i16::<LittleEndian>(),
                    Endianness::Big => cursor.read_i16::<BigEndian>(),
                } {
                    values.push(v);
                } else {
                    break;
                }
            }
            if values.is_empty() {
                None
            } else {
                Some(ExifValue::SShort(values))
            }
        }
        "int32s" => {
            let mut values = Vec::new();
            let mut cursor = Cursor::new(bytes);
            while cursor.position() < bytes.len() as u64 {
                if let Ok(v) = match endian {
                    Endianness::Little => cursor.read_i32::<LittleEndian>(),
                    Endianness::Big => cursor.read_i32::<BigEndian>(),
                } {
                    values.push(v);
                } else {
                    break;
                }
            }

            // Special handling for specific tags
            if values.len() == 1 {
                let v = values[0];
                match tag_id {
                    PHASEONE_CAMERA_ORIENTATION => {
                        return Some(ExifValue::Ascii(
                            decode_camera_orientation_exiftool(v as u32).to_string(),
                        ));
                    }
                    PHASEONE_RAW_FORMAT => {
                        return Some(ExifValue::Ascii(
                            decode_raw_format_exiftool(v as u32).to_string(),
                        ));
                    }
                    PHASEONE_SEQUENCE_KIND => {
                        return Some(ExifValue::Ascii(
                            decode_sequence_kind_exiftool(v as u32).to_string(),
                        ));
                    }
                    PHASEONE_ISO
                    | PHASEONE_SENSOR_WIDTH
                    | PHASEONE_SENSOR_HEIGHT
                    | PHASEONE_SENSOR_LEFT_MARGIN
                    | PHASEONE_SENSOR_TOP_MARGIN
                    | PHASEONE_IMAGE_WIDTH
                    | PHASEONE_IMAGE_HEIGHT
                    | PHASEONE_IMAGE_NUMBER
                    | PHASEONE_BLACK_LEVEL
                    | PHASEONE_SPLIT_COLUMN
                    | PHASEONE_SEQUENCE_FRAME_NUMBER
                    | PHASEONE_SEQUENCE_FRAME_COUNT => {
                        // Return as Long for numeric display
                        return Some(ExifValue::Long(vec![v as u32]));
                    }
                    _ => {}
                }
            }

            if values.is_empty() {
                None
            } else {
                Some(ExifValue::SLong(values))
            }
        }
        _ => Some(ExifValue::Undefined(bytes.to_vec())),
    }
}
