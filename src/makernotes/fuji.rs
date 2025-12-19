// makernotes/fuji.rs - Fujifilm maker notes parsing

use crate::data_types::{Endianness, ExifValue};
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use byteorder::{BigEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Cursor;

// Fujifilm MakerNote tag IDs
pub const FUJI_VERSION: u16 = 0x0000;
pub const FUJI_SERIAL_NUMBER: u16 = 0x0010;
pub const FUJI_QUALITY: u16 = 0x1000;
pub const FUJI_SHARPNESS: u16 = 0x1001;
pub const FUJI_WHITE_BALANCE: u16 = 0x1002;
pub const FUJI_SATURATION: u16 = 0x1003;
pub const FUJI_CONTRAST: u16 = 0x1004;
pub const FUJI_COLOR_TEMPERATURE: u16 = 0x1005;
pub const FUJI_CONTRAST_DETECTION_AF: u16 = 0x1006;
pub const FUJI_FLASH_MODE: u16 = 0x1010;
pub const FUJI_FLASH_EXPOSURE_COMP: u16 = 0x1011;
pub const FUJI_MACRO: u16 = 0x1020;
pub const FUJI_FOCUS_MODE: u16 = 0x1021;
pub const FUJI_AF_MODE: u16 = 0x1022;
pub const FUJI_FOCUS_PIXEL: u16 = 0x1023;
pub const FUJI_SLOW_SYNC: u16 = 0x1030;
pub const FUJI_PICTURE_MODE: u16 = 0x1031;
pub const FUJI_EXR_AUTO: u16 = 0x1033;
pub const FUJI_EXR_MODE: u16 = 0x1034;
pub const FUJI_AUTO_BRACKETING: u16 = 0x1100;
pub const FUJI_SEQUENCE_NUMBER: u16 = 0x1101;
pub const FUJI_BLUR_WARNING: u16 = 0x1300;
pub const FUJI_FOCUS_WARNING: u16 = 0x1301;
pub const FUJI_EXPOSURE_WARNING: u16 = 0x1302;
pub const FUJI_DYNAMIC_RANGE: u16 = 0x1400;
pub const FUJI_FILM_MODE: u16 = 0x1401;
pub const FUJI_DYNAMIC_RANGE_SETTING: u16 = 0x1402;
pub const FUJI_DEVELOPMENT_DYNAMIC_RANGE: u16 = 0x1403;
pub const FUJI_MIN_FOCAL_LENGTH: u16 = 0x1404;
pub const FUJI_MAX_FOCAL_LENGTH: u16 = 0x1405;
pub const FUJI_MAX_APERTURE_AT_MIN_FOCAL: u16 = 0x1406;
pub const FUJI_MAX_APERTURE_AT_MAX_FOCAL: u16 = 0x1407;
pub const FUJI_FILE_SOURCE: u16 = 0x8000;
pub const FUJI_ORDER_NUMBER: u16 = 0x8002;
pub const FUJI_FRAME_NUMBER: u16 = 0x8003;
pub const FUJI_FACES_DETECTED: u16 = 0x4100;
pub const FUJI_FACE_POSITIONS: u16 = 0x4103;
pub const FUJI_FACE_REC_INFO: u16 = 0x4282;
pub const FUJI_RAW_IMAGE_FULL_SIZE: u16 = 0x0100;
pub const FUJI_RAW_IMAGE_CROP_TOP_LEFT: u16 = 0x0110;
pub const FUJI_RAW_IMAGE_CROPPED_SIZE: u16 = 0x0111;
pub const FUJI_RAW_IMAGE_ASPECT_RATIO: u16 = 0x0115;
pub const FUJI_LENS_MOUNT_TYPE: u16 = 0x1600;
pub const FUJI_RATINGS_INFO: u16 = 0xB211;
pub const FUJI_GE_IMAGE_SIZE: u16 = 0xB212;

/// Get the name of a Fujifilm MakerNote tag
pub fn get_fuji_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        FUJI_VERSION => Some("Version"),
        FUJI_SERIAL_NUMBER => Some("InternalSerialNumber"),
        FUJI_QUALITY => Some("Quality"),
        FUJI_SHARPNESS => Some("Sharpness"),
        FUJI_WHITE_BALANCE => Some("WhiteBalance"),
        FUJI_SATURATION => Some("Saturation"),
        FUJI_CONTRAST => Some("Contrast"),
        FUJI_COLOR_TEMPERATURE => Some("ColorTemperature"),
        FUJI_CONTRAST_DETECTION_AF => Some("ContrastDetectionAF"),
        FUJI_FLASH_MODE => Some("FlashMode"),
        FUJI_FLASH_EXPOSURE_COMP => Some("FlashExposureComp"),
        FUJI_MACRO => Some("Macro"),
        FUJI_FOCUS_MODE => Some("FocusMode"),
        FUJI_AF_MODE => Some("AFMode"),
        FUJI_FOCUS_PIXEL => Some("FocusPixel"),
        FUJI_SLOW_SYNC => Some("SlowSync"),
        FUJI_PICTURE_MODE => Some("PictureMode"),
        FUJI_EXR_AUTO => Some("EXRAuto"),
        FUJI_EXR_MODE => Some("EXRMode"),
        FUJI_AUTO_BRACKETING => Some("AutoBracketing"),
        FUJI_SEQUENCE_NUMBER => Some("SequenceNumber"),
        FUJI_BLUR_WARNING => Some("BlurWarning"),
        FUJI_FOCUS_WARNING => Some("FocusWarning"),
        FUJI_EXPOSURE_WARNING => Some("ExposureWarning"),
        FUJI_DYNAMIC_RANGE => Some("DynamicRange"),
        FUJI_FILM_MODE => Some("FilmMode"),
        FUJI_DYNAMIC_RANGE_SETTING => Some("DynamicRangeSetting"),
        FUJI_DEVELOPMENT_DYNAMIC_RANGE => Some("DevelopmentDynamicRange"),
        FUJI_MIN_FOCAL_LENGTH => Some("MinFocalLength"),
        FUJI_MAX_FOCAL_LENGTH => Some("MaxFocalLength"),
        FUJI_MAX_APERTURE_AT_MIN_FOCAL => Some("MaxApertureAtMinFocal"),
        FUJI_MAX_APERTURE_AT_MAX_FOCAL => Some("MaxApertureAtMaxFocal"),
        FUJI_FILE_SOURCE => Some("FileSource"),
        FUJI_ORDER_NUMBER => Some("OrderNumber"),
        FUJI_FRAME_NUMBER => Some("FrameNumber"),
        FUJI_FACES_DETECTED => Some("FacesDetected"),
        FUJI_FACE_POSITIONS => Some("FacePositions"),
        FUJI_FACE_REC_INFO => Some("FaceRecInfo"),
        FUJI_RAW_IMAGE_FULL_SIZE => Some("RawImageFullSize"),
        FUJI_RAW_IMAGE_CROP_TOP_LEFT => Some("RawImageCropTopLeft"),
        FUJI_RAW_IMAGE_CROPPED_SIZE => Some("RawImageCroppedSize"),
        FUJI_RAW_IMAGE_ASPECT_RATIO => Some("RawImageAspectRatio"),
        FUJI_LENS_MOUNT_TYPE => Some("LensMountType"),
        FUJI_RATINGS_INFO => Some("RatingsInfo"),
        FUJI_GE_IMAGE_SIZE => Some("GEImageSize"),
        _ => None,
    }
}

/// Parse Fujifilm maker notes
pub fn parse_fuji_maker_notes(
    data: &[u8],
    _endian: Endianness,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    if data.len() < 12 {
        return Ok(tags);
    }

    // Fuji maker notes start with "FUJIFILM" header (8 bytes)
    // followed by offset to IFD (4 bytes, big-endian)
    if data.len() >= 12 && &data[0..8] == b"FUJIFILM" {
        // Read IFD offset (big-endian)
        let mut cursor = Cursor::new(&data[8..12]);
        let ifd_offset = cursor
            .read_u32::<BigEndian>()
            .map_err(|_| ExifError::Format("Failed to read Fuji IFD offset".to_string()))?
            as usize;

        // IFD offset is relative to start of maker note data
        if ifd_offset >= data.len() {
            return Ok(tags);
        }

        // Parse IFD
        let ifd_data = &data[ifd_offset..];
        if ifd_data.len() < 2 {
            return Ok(tags);
        }

        let mut cursor = Cursor::new(ifd_data);

        // Read number of entries (always big-endian for Fuji)
        let num_entries = cursor
            .read_u16::<BigEndian>()
            .map_err(|_| ExifError::Format("Failed to read Fuji maker note count".to_string()))?;

        // Parse each entry
        for _ in 0..num_entries {
            if cursor.position() as usize + 12 > ifd_data.len() {
                break;
            }

            let tag_id = match cursor.read_u16::<BigEndian>() {
                Ok(id) => id,
                Err(_) => break,
            };

            let data_type = match cursor.read_u16::<BigEndian>() {
                Ok(dt) => dt,
                Err(_) => break,
            };

            let count = match cursor.read_u32::<BigEndian>() {
                Ok(c) => c,
                Err(_) => break,
            };

            let value_offset = match cursor.read_u32::<BigEndian>() {
                Ok(o) => o,
                Err(_) => break,
            };

            // Parse the value based on type
            // For simplicity, we'll only handle a few common types
            let value = match data_type {
                1 => {
                    // BYTE
                    if count <= 4 {
                        // Value is in the offset field
                        let bytes = value_offset.to_be_bytes();
                        ExifValue::Byte(bytes[..count as usize].to_vec())
                    } else {
                        // Value is at offset
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
                        let string_bytes = &data[offset..offset + count as usize];
                        let s = String::from_utf8_lossy(string_bytes).to_string();
                        ExifValue::Ascii(s)
                    } else {
                        continue;
                    }
                }
                3 => {
                    // SHORT
                    if count == 1 {
                        // Value is in the offset field (first 2 bytes)
                        ExifValue::Short(vec![(value_offset >> 16) as u16])
                    } else if count == 2 {
                        // Both values are in the offset field
                        ExifValue::Short(vec![
                            (value_offset >> 16) as u16,
                            (value_offset & 0xFFFF) as u16,
                        ])
                    } else {
                        // Value is at offset
                        let offset = value_offset as usize;
                        if offset + (count as usize * 2) <= data.len() {
                            let mut values = Vec::new();
                            let mut cursor = Cursor::new(&data[offset..]);
                            for _ in 0..count {
                                if let Ok(v) = cursor.read_u16::<BigEndian>() {
                                    values.push(v);
                                } else {
                                    break;
                                }
                            }
                            ExifValue::Short(values)
                        } else {
                            continue;
                        }
                    }
                }
                4 => {
                    // LONG
                    if count == 1 {
                        ExifValue::Long(vec![value_offset])
                    } else {
                        let offset = value_offset as usize;
                        if offset + (count as usize * 4) <= data.len() {
                            let mut values = Vec::new();
                            let mut cursor = Cursor::new(&data[offset..]);
                            for _ in 0..count {
                                if let Ok(v) = cursor.read_u32::<BigEndian>() {
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
                _ => {
                    // Unsupported type for now
                    continue;
                }
            };

            tags.insert(
                tag_id,
                MakerNoteTag {
                    tag_id,
                    tag_name: get_fuji_tag_name(tag_id),
                    value,
                },
            );
        }
    }

    Ok(tags)
}
