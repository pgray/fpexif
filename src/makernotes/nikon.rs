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
pub const NIKON_HIGH_ISO_NOISE_REDUCTION: u16 = 0x00B1;
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
pub const NIKON_DATE_STAMP_MODE: u16 = 0x009D;
pub const NIKON_SERIAL_NUMBER_2: u16 = 0x00A0;
pub const NIKON_IMAGE_DATA_SIZE: u16 = 0x00A2;
pub const NIKON_IMAGE_COUNT: u16 = 0x00A5;
pub const NIKON_DELETED_IMAGE_COUNT: u16 = 0x00A6;
pub const NIKON_SHUTTER_COUNT: u16 = 0x00A7;

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
        NIKON_HIGH_ISO_NOISE_REDUCTION => Some("HighISONoiseReduction"),
        NIKON_DATE_STAMP_MODE => Some("DateStampMode"),
        NIKON_SERIAL_NUMBER_2 => Some("SerialNumber"),
        NIKON_IMAGE_DATA_SIZE => Some("ImageDataSize"),
        NIKON_IMAGE_COUNT => Some("ImageCount"),
        NIKON_DELETED_IMAGE_COUNT => Some("DeletedImageCount"),
        NIKON_SHUTTER_COUNT => Some("ShutterCount"),
        _ => None,
    }
}

/// Get Nikon lens name from composite lens ID
/// Nikon lens IDs are 8-byte composite values formatted as "XX XX XX XX XX XX XX XX"
/// Based on ExifTool's nikonLensIDs database
pub fn get_nikon_lens_name(lens_id: &str) -> Option<&'static str> {
    match lens_id.to_uppercase().as_str() {
        // Popular AF and AF-D lenses
        "01 58 50 50 14 14 02 00" => Some("AF Nikkor 50mm f/1.8"),
        "01 58 50 50 14 14 05 00" => Some("AF Nikkor 50mm f/1.8"),
        "02 42 44 5C 2A 34 02 00" => Some("AF Zoom-Nikkor 35-70mm f/3.3-4.5"),
        "03 48 5C 81 30 30 02 00" => Some("AF Zoom-Nikkor 70-210mm f/4"),
        "04 48 3C 3C 24 24 03 00" => Some("AF Nikkor 28mm f/2.8"),
        "05 54 50 50 0C 0C 04 00" => Some("AF Nikkor 50mm f/1.4"),
        "06 54 53 53 24 24 06 00" => Some("AF Micro-Nikkor 55mm f/2.8"),
        "09 48 37 37 24 24 04 00" => Some("AF Nikkor 24mm f/2.8"),
        "0A 48 8E 8E 24 24 03 00" => Some("AF Nikkor 300mm f/2.8 IF-ED"),
        "0B 48 7C 7C 24 24 05 00" => Some("AF Nikkor 180mm f/2.8 IF-ED"),
        "0F 58 50 50 14 14 05 00" => Some("AF Nikkor 50mm f/1.8 N"),
        "10 48 8E 8E 30 30 08 00" => Some("AF Nikkor 300mm f/4 IF-ED"),
        "11 48 44 5C 24 24 08 00" => Some("AF Zoom-Nikkor 35-70mm f/2.8"),
        "15 4C 62 62 14 14 0C 00" => Some("AF Nikkor 85mm f/1.8"),
        "1A 54 44 44 18 18 11 00" => Some("AF Nikkor 35mm f/2"),
        "1C 48 30 30 24 24 12 00" => Some("AF Nikkor 20mm f/2.8"),
        "1E 54 56 56 24 24 13 00" => Some("AF Micro-Nikkor 60mm f/2.8"),
        "1F 54 6A 6A 24 24 14 00" => Some("AF Micro-Nikkor 105mm f/2.8"),
        "22 48 72 72 18 18 16 00" => Some("AF DC-Nikkor 135mm f/2"),
        // AF-D lenses
        "24 48 60 80 24 24 1A 02" => Some("AF Zoom-Nikkor 80-200mm f/2.8D ED"),
        "31 54 56 56 24 24 25 02" => Some("AF Micro-Nikkor 60mm f/2.8D"),
        "32 54 6A 6A 24 24 35 02" => Some("AF Micro-Nikkor 105mm f/2.8D"),
        "33 48 2D 2D 24 24 31 02" => Some("AF Nikkor 18mm f/2.8D"),
        "34 48 29 29 24 24 32 02" => Some("AF Fisheye Nikkor 16mm f/2.8D"),
        "36 48 37 37 24 24 34 02" => Some("AF Nikkor 24mm f/2.8D"),
        "37 48 30 30 24 24 36 02" => Some("AF Nikkor 20mm f/2.8D"),
        "38 4C 62 62 14 14 37 02" => Some("AF Nikkor 85mm f/1.8D"),
        "42 54 44 44 18 18 44 02" => Some("AF Nikkor 35mm f/2D"),
        "43 54 50 50 0C 0C 46 02" => Some("AF Nikkor 50mm f/1.4D"),
        "4A 54 62 62 0C 0C 4D 02" => Some("AF Nikkor 85mm f/1.4D IF"),
        "4E 48 72 72 18 18 51 02" => Some("AF DC-Nikkor 135mm f/2D"),
        // AF-S lenses
        "48 48 8E 8E 24 24 4B 02" => Some("AF-S Nikkor 300mm f/2.8D IF-ED"),
        "5D 48 3C 5C 24 24 63 02" => Some("AF-S Zoom-Nikkor 28-70mm f/2.8D IF-ED"),
        "5E 48 60 80 24 24 64 02" => Some("AF-S Zoom-Nikkor 80-200mm f/2.8D IF-ED"),
        "63 48 2B 44 24 24 68 02" => Some("AF-S Nikkor 17-35mm f/2.8D IF-ED"),
        "6A 48 8E 8E 30 30 70 02" => Some("AF-S Nikkor 300mm f/4D IF-ED"),
        "6B 48 24 24 24 24 71 02" => Some("AF Nikkor ED 14mm f/2.8D"),
        // AF-S G lenses
        "77 48 5C 80 24 24 7B 0E" => Some("AF-S VR Zoom-Nikkor 70-200mm f/2.8G IF-ED"),
        "78 48 5C 80 24 24 7C 0E" => Some("AF-S VR Zoom-Nikkor 70-200mm f/2.8G IF-ED II"),
        "79 40 11 11 2C 2C 7D 06" => Some("AF-S Fisheye Nikkor 8-15mm f/3.5-4.5E ED"),
        "7A 48 5C 80 24 24 7E 0E" => Some("AF-S VR Zoom-Nikkor 70-200mm f/2.8E FL ED VR"),
        "8A 54 6A 6A 24 24 A3 0E" => Some("AF-S VR Micro-Nikkor 105mm f/2.8G IF-ED"),
        "8B 40 2D 80 2C 3C A4 0E" => Some("AF-S DX VR Zoom-Nikkor 18-200mm f/3.5-5.6G IF-ED"),
        "8C 40 2D 53 2C 3C A5 06" => Some("AF-S DX Zoom-Nikkor 18-55mm f/3.5-5.6G ED"),
        "8D 44 5C 8E 34 3C A6 06" => Some("AF-S VR Zoom-Nikkor 70-300mm f/4.5-5.6G IF-ED"),
        "8E 3C 2B 5C 24 30 A7 0E" => Some("AF-S Zoom-Nikkor 17-70mm f/2.8-4G IF-ED"),
        "8F 40 2D 72 2C 3C A8 0E" => Some("AF-S DX Zoom-Nikkor 18-135mm f/3.5-5.6G IF-ED"),
        "90 3B 53 80 30 3C A9 0E" => Some("AF-S DX VR Zoom-Nikkor 55-200mm f/4-5.6G IF-ED"),
        "92 48 24 37 24 24 AB 0E" => Some("AF-S Zoom-Nikkor 14-24mm f/2.8G ED"),
        "93 48 37 5C 24 24 AC 0E" => Some("AF-S Zoom-Nikkor 24-70mm f/2.8G ED"),
        "94 40 2D 53 2C 3C AE 0E" => Some("AF-S DX Zoom-Nikkor 18-55mm f/3.5-5.6G VR"),
        "95 00 37 37 2C 2C AF 06" => Some("PC-E Nikkor 24mm f/3.5D ED"),
        "96 48 98 98 24 24 B0 0E" => Some("AF-S VR Nikkor 400mm f/2.8G ED"),
        "97 3C A0 A0 30 30 B1 0E" => Some("AF-S VR Nikkor 500mm f/4G ED"),
        "98 3C A6 A6 30 30 B2 0E" => Some("AF-S VR Nikkor 600mm f/4G ED"),
        "99 40 29 62 2C 3C B3 0E" => Some("AF-S DX VR Zoom-Nikkor 16-85mm f/3.5-5.6G ED"),
        "9A 40 2D 53 2C 3C B4 0E" => Some("AF-S DX Zoom-Nikkor 18-55mm f/3.5-5.6G VR II"),
        "9B 54 4C 4C 24 24 B5 02" => Some("PC-E Micro Nikkor 45mm f/2.8D ED"),
        "9C 54 56 56 24 24 B6 06" => Some("AF-S Micro Nikkor 60mm f/2.8G ED"),
        "9D 00 62 62 24 24 B7 0E" => Some("PC-E Micro Nikkor 85mm f/2.8D"),
        "9E 40 2D 6A 2C 3C B8 0E" => Some("AF-S DX VR Zoom-Nikkor 18-105mm f/3.5-5.6G ED"),
        "9F 58 44 44 14 14 B9 06" => Some("AF-S DX Nikkor 35mm f/1.8G"),
        "A0 54 50 50 0C 0C BA 06" => Some("AF-S Nikkor 50mm f/1.4G"),
        "A1 40 18 37 2C 34 BB 06" => Some("AF-S DX Nikkor 10-24mm f/3.5-4.5G ED"),
        "A2 48 5C 80 24 24 BC 0E" => Some("AF-S Nikkor 70-200mm f/2.8G ED VR II"),
        "A3 3C 29 44 30 30 BD 0E" => Some("AF-S Nikkor 16-35mm f/4G ED VR"),
        "A4 54 37 37 0C 0C BE 06" => Some("AF-S Nikkor 24mm f/1.4G ED"),
        "A5 40 3C 8E 2C 3C BF 0E" => Some("AF-S Nikkor 28-300mm f/3.5-5.6G ED VR"),
        "A6 48 8E 8E 24 24 C0 0E" => Some("AF-S Nikkor 300mm f/2.8G ED VR II"),
        "A7 4C 2D 50 24 24 C1 06" => Some("AF-S DX Nikkor 18-50mm f/2.8G ED"),
        "A8 48 80 98 30 30 C2 0E" => Some("AF-S Zoom-Nikkor 200-400mm f/4G IF-ED VR"),
        "A9 54 80 80 18 18 C3 0E" => Some("AF-S Nikkor 200mm f/2G ED VR II"),
        "AA 3C 37 6E 30 30 C4 0E" => Some("AF-S Nikkor 24-120mm f/4G ED VR"),
        "AB 3C A0 A0 30 30 C5 0E" => Some("AF-S Nikkor 500mm f/4G ED VR"),
        "AC 38 53 8E 34 3C C6 0E" => Some("AF-S DX Nikkor 55-300mm f/4.5-5.6G ED VR"),
        "AD 3C 2D 8E 2C 3C C7 0E" => Some("AF-S DX Nikkor 18-300mm f/3.5-5.6G ED VR"),
        "AE 54 62 62 0C 0C C8 06" => Some("AF-S Nikkor 85mm f/1.4G"),
        "AF 54 44 44 0C 0C C9 06" => Some("AF-S Nikkor 35mm f/1.4G"),
        "B0 4C 50 50 14 14 CA 06" => Some("AF-S Nikkor 50mm f/1.8G"),
        "B1 48 48 48 24 24 CB 06" => Some("AF-S DX Micro Nikkor 40mm f/2.8G"),
        // AF-P and modern lenses
        "B2 48 5C 80 30 30 CC 0E" => Some("AF-S Nikkor 70-200mm f/4G ED VR"),
        "B3 4C 62 62 14 14 CD 06" => Some("AF-S Nikkor 85mm f/1.8G"),
        "B4 40 37 62 2C 34 CE 0E" => Some("AF-S Nikkor 24-85mm f/3.5-4.5G ED VR"),
        "B5 4C 3C 3C 14 14 CF 06" => Some("AF-S Nikkor 28mm f/1.8G"),
        "B6 3C B0 B0 3C 3C D0 0E" => Some("AF-S VR Nikkor 800mm f/5.6E FL ED"),
        "B7 44 60 98 34 3C D1 0E" => Some("AF-S Nikkor 80-400mm f/4.5-5.6G ED VR"),
        "B8 40 2D 44 2C 34 D2 0E" => Some("AF-S DX Nikkor 18-35mm f/3.5-4.5G ED"),
        // Z-mount lenses (Nikon Z)
        "01 00 00 00 00 00 00 00" => Some("Nikon Z Lens"),
        _ => None,
    }
}

/// Decode Nikon ASCII tag values to human-readable strings
fn decode_nikon_ascii_value(tag_id: u16, value: &str) -> String {
    match tag_id {
        NIKON_QUALITY => match value.trim() {
            "RAW" => "RAW",
            "FINE" => "Fine",
            "NORMAL" => "Normal",
            "BASIC" => "Basic",
            "RAW+FINE" | "RAW + FINE" => "RAW + Fine",
            "RAW+NORMAL" | "RAW + NORMAL" => "RAW + Normal",
            "RAW+BASIC" | "RAW + BASIC" => "RAW + Basic",
            _ => value.trim(),
        }
        .to_string(),
        NIKON_WHITE_BALANCE => match value.trim() {
            "AUTO" | "AUTO1" | "AUTO2" => "Auto",
            "SUNNY" | "DIRECT SUNLIGHT" => "Daylight",
            "SHADE" => "Shade",
            "CLOUDY" => "Cloudy",
            "TUNGSTEN" | "INCANDESCENT" => "Tungsten",
            "FLUORESCENT" => "Fluorescent",
            "FLASH" => "Flash",
            "PRESET" => "Preset",
            _ => value.trim(),
        }
        .to_string(),
        NIKON_FOCUS_MODE => match value.trim() {
            "AF-S" => "AF-S",
            "AF-C" => "AF-C",
            "AF-A" => "AF-A",
            "MF" | "MANUAL" => "Manual",
            _ => value.trim(),
        }
        .to_string(),
        NIKON_SHARPNESS => match value.trim() {
            "AUTO" => "Auto",
            "NORMAL" => "Normal",
            "LOW" => "Low",
            "MED.L" | "MEDIUM LOW" => "Medium Low",
            "MED.H" | "MEDIUM HIGH" => "Medium High",
            "HIGH" => "High",
            "NONE" => "None",
            _ => value.trim(),
        }
        .to_string(),
        NIKON_COLOR_MODE => value.trim().to_string(),
        _ => value.trim().to_string(),
    }
}

/// Decode Active D-Lighting value
fn decode_active_d_lighting(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "Low",
        2 => "Normal",
        3 => "High",
        4 => "Extra High",
        5 => "Auto",
        _ => "Unknown",
    }
}

/// Decode Color Space value
fn decode_color_space(value: u16) -> &'static str {
    match value {
        1 => "sRGB",
        2 => "Adobe RGB",
        _ => "Unknown",
    }
}

/// Decode Vignette Control value
fn decode_vignette_control(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "Low",
        2 => "Normal",
        3 => "High",
        _ => "Unknown",
    }
}

/// Decode High ISO Noise Reduction value (tag 0x00b1)
fn decode_high_iso_noise_reduction(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "Minimal",
        2 => "Low",
        3 => "Medium Low",
        4 => "Normal",
        5 => "Medium High",
        6 => "High",
        _ => "Unknown",
    }
}

/// Decode Date Stamp Mode value (tag 0x009d)
fn decode_date_stamp_mode(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "Date & Time",
        2 => "Date",
        3 => "Date Counter",
        _ => "Unknown",
    }
}

/// Nikon decryption lookup tables (from ExifTool)
const NIKON_XLAT_0: [u8; 256] = [
    0xc1, 0xbf, 0x6d, 0x0d, 0x59, 0xc5, 0x13, 0x9d, 0x83, 0x61, 0x6b, 0x4f, 0xc7, 0x7f, 0x3d, 0x3d,
    0x53, 0x59, 0xe3, 0xc7, 0xe9, 0x2f, 0x95, 0xa7, 0x95, 0x1f, 0xdf, 0x7f, 0x2b, 0x29, 0xc7, 0x0d,
    0xdf, 0x07, 0xef, 0x71, 0x89, 0x3d, 0x13, 0x3d, 0x3b, 0x13, 0xfb, 0x0d, 0x89, 0xc1, 0x65, 0x1f,
    0xb3, 0x0d, 0x6b, 0x29, 0xe3, 0xfb, 0xef, 0xa3, 0x6b, 0x47, 0x7f, 0x95, 0x35, 0xa7, 0x47, 0x4f,
    0xc7, 0xf1, 0x59, 0x95, 0x35, 0x11, 0x29, 0x61, 0xf1, 0x3d, 0xb3, 0x2b, 0x0d, 0x43, 0x89, 0xc1,
    0x9d, 0x9d, 0x89, 0x65, 0xf1, 0xe9, 0xdf, 0xbf, 0x3d, 0x7f, 0x53, 0x97, 0xe5, 0xe9, 0x95, 0x17,
    0x1d, 0x3d, 0x8b, 0xfb, 0xc7, 0xe3, 0x67, 0xa7, 0x07, 0xf1, 0x71, 0xa7, 0x53, 0xb5, 0x29, 0x89,
    0xe5, 0x2b, 0xa7, 0x17, 0x29, 0xe9, 0x4f, 0xc5, 0x65, 0x6d, 0x6b, 0xef, 0x0d, 0x89, 0x49, 0x2f,
    0xb3, 0x43, 0x53, 0x65, 0x1d, 0x49, 0xa3, 0x13, 0x89, 0x59, 0xef, 0x6b, 0xef, 0x65, 0x1d, 0x0b,
    0x59, 0x13, 0xe3, 0x4f, 0x9d, 0xb3, 0x29, 0x43, 0x2b, 0x07, 0x1d, 0x95, 0x59, 0x59, 0x47, 0xfb,
    0xe5, 0xe9, 0x61, 0x47, 0x2f, 0x35, 0x7f, 0x17, 0x7f, 0xef, 0x7f, 0x95, 0x95, 0x71, 0xd3, 0xa3,
    0x0b, 0x71, 0xa3, 0xad, 0x0b, 0x3b, 0xb5, 0xfb, 0xa3, 0xbf, 0x4f, 0x83, 0x1d, 0xad, 0xe9, 0x2f,
    0x71, 0x65, 0xa3, 0xe5, 0x07, 0x35, 0x3d, 0x0d, 0xb5, 0xe9, 0xe5, 0x47, 0x3b, 0x9d, 0xef, 0x35,
    0xa3, 0xbf, 0xb3, 0xdf, 0x53, 0xd3, 0x97, 0x53, 0x49, 0x71, 0x07, 0x35, 0x61, 0x71, 0x2f, 0x43,
    0x2f, 0x11, 0xdf, 0x17, 0x97, 0xfb, 0x95, 0x3b, 0x7f, 0x6b, 0xd3, 0x25, 0xbf, 0xad, 0xc7, 0xc5,
    0xc5, 0xb5, 0x8b, 0xef, 0x2f, 0xd3, 0x07, 0x6b, 0x25, 0x49, 0x95, 0x25, 0x49, 0x6d, 0x71, 0xc7,
];

const NIKON_XLAT_1: [u8; 256] = [
    0xa7, 0xbc, 0xc9, 0xad, 0x91, 0xdf, 0x85, 0xe5, 0xd4, 0x78, 0xd5, 0x17, 0x46, 0x7c, 0x29, 0x4c,
    0x4d, 0x03, 0xe9, 0x25, 0x68, 0x11, 0x86, 0xb3, 0xbd, 0xf7, 0x6f, 0x61, 0x22, 0xa2, 0x26, 0x34,
    0x2a, 0xbe, 0x1e, 0x46, 0x14, 0x68, 0x9d, 0x44, 0x18, 0xc2, 0x40, 0xf4, 0x7e, 0x5f, 0x1b, 0xad,
    0x0b, 0x94, 0xb6, 0x67, 0xb4, 0x0b, 0xe1, 0xea, 0x95, 0x9c, 0x66, 0xdc, 0xe7, 0x5d, 0x6c, 0x05,
    0xda, 0xd5, 0xdf, 0x7a, 0xef, 0xf6, 0xdb, 0x1f, 0x82, 0x4c, 0xc0, 0x68, 0x47, 0xa1, 0xbd, 0xee,
    0x39, 0x50, 0x56, 0x4a, 0xdd, 0xdf, 0xa5, 0xf8, 0xc6, 0xda, 0xca, 0x90, 0xca, 0x01, 0x42, 0x9d,
    0x8b, 0x0c, 0x73, 0x43, 0x75, 0x05, 0x94, 0xde, 0x24, 0xb3, 0x80, 0x34, 0xe5, 0x2c, 0xdc, 0x9b,
    0x3f, 0xca, 0x33, 0x45, 0xd0, 0xdb, 0x5f, 0xf5, 0x52, 0xc3, 0x21, 0xda, 0xe2, 0x22, 0x72, 0x6b,
    0x3e, 0xd0, 0x5b, 0xa8, 0x87, 0x8c, 0x06, 0x5d, 0x0f, 0xdd, 0x09, 0x19, 0x93, 0xd0, 0xb9, 0xfc,
    0x8b, 0x0f, 0x84, 0x60, 0x33, 0x1c, 0x9b, 0x45, 0xf1, 0xf0, 0xa3, 0x94, 0x3a, 0x12, 0x77, 0x33,
    0x4d, 0x44, 0x78, 0x28, 0x3c, 0x9e, 0xfd, 0x65, 0x57, 0x16, 0x94, 0x6b, 0xfb, 0x59, 0xd0, 0xc8,
    0x22, 0x36, 0xdb, 0xd2, 0x63, 0x98, 0x43, 0xa1, 0x04, 0x87, 0x86, 0xf7, 0xa6, 0x26, 0xbb, 0xd6,
    0x59, 0x4d, 0xbf, 0x6a, 0x2e, 0xaa, 0x2b, 0xef, 0xe6, 0x78, 0xb6, 0x4e, 0xe0, 0x2f, 0xdc, 0x7c,
    0xbe, 0x57, 0x19, 0x32, 0x7e, 0x2a, 0xd0, 0xb8, 0xba, 0x29, 0x00, 0x3c, 0x52, 0x7d, 0xa8, 0x49,
    0x3b, 0x2d, 0xeb, 0x25, 0x49, 0xfa, 0xa3, 0xaa, 0x39, 0xa7, 0xc5, 0xa7, 0x50, 0x11, 0x36, 0xfb,
    0xc6, 0x67, 0x4a, 0xf5, 0xa5, 0x12, 0x65, 0x7e, 0xb0, 0xdf, 0xaf, 0x4e, 0xb3, 0x61, 0x7f, 0x2f,
];

/// Decrypt Nikon encrypted data
/// serial: Serial number (last byte used as index into XLAT_0)
/// count: ShutterCount (XOR of all 4 bytes used as index into XLAT_1)
/// data: Data to decrypt (in-place modification)
/// start: Starting offset for decryption
fn nikon_decrypt(serial: u32, count: u32, data: &mut [u8], start: usize) {
    if data.is_empty() || start >= data.len() {
        return;
    }

    // Initialize decryption parameters
    let key =
        ((count & 0xff) ^ ((count >> 8) & 0xff) ^ ((count >> 16) & 0xff) ^ ((count >> 24) & 0xff))
            as u8;
    let ci = NIKON_XLAT_0[(serial & 0xff) as usize];
    let mut cj = NIKON_XLAT_1[key as usize];
    let mut ck: u8 = 0x60;

    // Decrypt data starting at offset
    for byte in data[start..].iter_mut() {
        cj = cj.wrapping_add(ci.wrapping_mul(ck));
        ck = ck.wrapping_add(1);
        *byte ^= cj;
    }
}

/// Get ShutterCount offset for a given ShotInfo version and data size
/// Returns (offset, is_little_endian) or None if not supported
fn get_shot_info_shutter_offset(version: &str, data_len: usize) -> Option<(usize, bool)> {
    // Version-specific handling based on ExifTool logic
    match version {
        // D40
        "0209" => Some((0x2c, false)),
        // D80 - no ShutterCount in ShotInfo
        "0208" => None,
        // D90
        "0213" => Some((0x2c, false)),
        // D3/D300 variants - differentiate by size
        "0210" => match data_len {
            5399 => Some((0x2c, false)),        // D3a
            5408 | 5412 => Some((0x2c, false)), // D3b
            5291 => Some((0x2c, false)),        // D300a
            5303 => Some((0x2c, false)),        // D300b
            _ => Some((0x2c, false)),           // Default for unknown D3/D300 variants
        },
        // D3X
        "0214" => Some((0x2c, false)),
        // D3S
        "0218" => Some((0x2c, false)),
        // D5000
        "0215" => Some((0x2c, false)),
        // D300S
        "0216" => Some((0x2c, false)),
        // D700
        "0212" => Some((0x2c, false)),
        // D7000
        "0220" => Some((0x2c, false)),
        // D5100 - no ShutterCount
        "0221" => None,
        // D800
        "0222" => Some((0x5c, false)),
        // D5200 - no ShutterCount
        "0226" => None,
        // D810
        "0233" => Some((0x100, true)),
        // D7500
        "0242" => None, // Offset unknown
        // D850
        "0243" => Some((0x1dc, true)),
        // D780
        "0245" => Some((0x220, true)),
        // Unknown version
        _ => None,
    }
}

/// Parse ShotInfo and extract ShutterCount if available
/// serial: Camera serial number (for decryption key)
/// shutter_count: ShutterCount from tag 0x00a7 (for decryption key)
/// data: Raw ShotInfo data
/// Returns: ShutterCount from ShotInfo if successfully decrypted
pub fn parse_shot_info_shutter_count(serial: u32, shutter_count: u32, data: &[u8]) -> Option<u32> {
    // First 4 bytes are version string (ASCII, not encrypted)
    if data.len() < 8 {
        return None;
    }

    // Get version string
    let version = std::str::from_utf8(&data[0..4]).ok()?;

    // Get offset and endianness for this version
    let (offset, is_little_endian) = get_shot_info_shutter_offset(version, data.len())?;

    // Offset is relative to start of encrypted data (byte 4)
    // So actual offset in decrypted data is offset
    let decrypt_start = 4;
    let value_offset = offset;

    // Check if we have enough data
    if decrypt_start + value_offset + 4 > data.len() {
        return None;
    }

    // Make a copy of the data for decryption
    let mut decrypted = data.to_vec();

    // Decrypt starting at byte 4
    nikon_decrypt(serial, shutter_count, &mut decrypted, decrypt_start);

    // Read the ShutterCount value at the specified offset (relative to decrypt start)
    let pos = decrypt_start + value_offset;
    if pos + 4 > decrypted.len() {
        return None;
    }

    let count = if is_little_endian {
        u32::from_le_bytes([
            decrypted[pos],
            decrypted[pos + 1],
            decrypted[pos + 2],
            decrypted[pos + 3],
        ])
    } else {
        u32::from_be_bytes([
            decrypted[pos],
            decrypted[pos + 1],
            decrypted[pos + 2],
            decrypted[pos + 3],
        ])
    };

    Some(count)
}

/// Decode Lens Type bitfield
/// Bit 0 = MF, Bit 1 = D, Bit 2 = G, Bit 3 = VR, Bit 4 = 1, Bit 6 = E, Bit 7 = AF-P
fn decode_lens_type(value: u8) -> String {
    let mut features = Vec::new();

    if value & 0x01 != 0 {
        features.push("MF");
    }
    if value & 0x02 != 0 {
        features.push("D");
    }
    if value & 0x04 != 0 {
        features.push("G");
    }
    if value & 0x08 != 0 {
        features.push("VR");
    }
    if value & 0x10 != 0 {
        features.push("1");
    }
    if value & 0x40 != 0 {
        features.push("E");
    }
    if value & 0x80 != 0 {
        features.push("AF-P");
    }

    if features.is_empty() {
        "Unknown".to_string()
    } else {
        features.join(" ")
    }
}

/// Format lens info from 4 RATIONAL values
/// [MinFocalLength, MaxFocalLength, MaxApertureAtMinFocal, MaxApertureAtMaxFocal]
fn format_lens_info(values: &[(u32, u32)]) -> Option<String> {
    if values.len() != 4 {
        return None;
    }

    let min_focal = if values[0].1 != 0 {
        values[0].0 as f64 / values[0].1 as f64
    } else {
        return None;
    };

    let max_focal = if values[1].1 != 0 {
        values[1].0 as f64 / values[1].1 as f64
    } else {
        return None;
    };

    let min_aperture = if values[2].1 != 0 {
        values[2].0 as f64 / values[2].1 as f64
    } else {
        return None;
    };

    let max_aperture = if values[3].1 != 0 {
        values[3].0 as f64 / values[3].1 as f64
    } else {
        return None;
    };

    // Format the lens description
    if (min_focal - max_focal).abs() < 0.1 {
        // Prime lens
        Some(format!("{:.0}mm f/{:.1}", min_focal, min_aperture))
    } else {
        // Zoom lens
        if (min_aperture - max_aperture).abs() < 0.1 {
            // Constant aperture
            Some(format!(
                "{:.0}-{:.0}mm f/{:.1}",
                min_focal, max_focal, min_aperture
            ))
        } else {
            // Variable aperture
            Some(format!(
                "{:.0}-{:.0}mm f/{:.1}-{:.1}",
                min_focal, max_focal, min_aperture, max_aperture
            ))
        }
    }
}

/// Parse Nikon maker notes
pub fn parse_nikon_maker_notes(
    data: &[u8],
    endian: Endianness,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    // Nikon maker notes often start with "Nikon\0" header
    if data.len() < 18 {
        return Ok(tags);
    }

    let base_offset;
    let ifd_offset;
    let maker_endian;

    // Check for Nikon Type 3 header: "Nikon\0" + version + TIFF header
    if data.starts_with(b"Nikon\0") {
        // Structure: "Nikon\0" (6 bytes) + version (4 bytes) + TIFF header
        base_offset = 10; // Start of TIFF header (after "Nikon\0" + version)

        // Read endianness from TIFF header
        if data.len() < base_offset + 8 {
            return Ok(tags);
        }

        // Check endianness marker at base_offset
        maker_endian = if &data[base_offset..base_offset + 2] == b"MM" {
            Endianness::Big
        } else if &data[base_offset..base_offset + 2] == b"II" {
            Endianness::Little
        } else {
            endian // Fallback to provided endianness
        };

        // Read IFD offset from TIFF header (4 bytes at offset base_offset + 4)
        let mut cursor = Cursor::new(&data[base_offset + 4..]);
        let ifd_relative_offset = match maker_endian {
            Endianness::Little => cursor.read_u32::<LittleEndian>(),
            Endianness::Big => cursor.read_u32::<BigEndian>(),
        }
        .map_err(|_| ExifError::Format("Failed to read Nikon IFD offset".to_string()))?
            as usize;

        // IFD offset is relative to the TIFF header
        ifd_offset = base_offset + ifd_relative_offset;
    } else {
        // No Nikon header, use the data as-is
        base_offset = 0;
        maker_endian = endian;
        ifd_offset = 0;
    }

    if ifd_offset >= data.len() {
        return Ok(tags);
    }

    let mut cursor = Cursor::new(&data[ifd_offset..]);

    // Read number of entries
    let num_entries = match maker_endian {
        Endianness::Little => cursor
            .read_u16::<LittleEndian>()
            .map_err(|_| ExifError::Format("Failed to read Nikon maker note count".to_string()))?,
        Endianness::Big => cursor
            .read_u16::<BigEndian>()
            .map_err(|_| ExifError::Format("Failed to read Nikon maker note count".to_string()))?,
    };

    // Parse IFD entries
    for _ in 0..num_entries {
        if cursor.position() as usize + 12 > data[ifd_offset..].len() {
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
                // Value at offset (relative to TIFF header for Nikon Type 3)
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
                    let bytes = value_bytes[..count as usize].to_vec();
                    // Apply decoder for lens type
                    if tag_id == NIKON_LENS_TYPE && !bytes.is_empty() {
                        ExifValue::Ascii(decode_lens_type(bytes[0]))
                    } else {
                        ExifValue::Byte(bytes)
                    }
                }
                2 => {
                    // ASCII
                    let s = String::from_utf8_lossy(&value_bytes[..count as usize])
                        .trim_end_matches('\0')
                        .to_string();
                    // Apply value decoders for specific tags
                    let decoded = decode_nikon_ascii_value(tag_id, &s);
                    ExifValue::Ascii(decoded)
                }
                3 => {
                    // SHORT
                    let mut values = Vec::new();
                    let mut cursor = Cursor::new(&value_bytes);
                    for _ in 0..count {
                        if let Ok(v) = match maker_endian {
                            Endianness::Little => cursor.read_u16::<LittleEndian>(),
                            Endianness::Big => cursor.read_u16::<BigEndian>(),
                        } {
                            values.push(v);
                        } else {
                            break;
                        }
                    }
                    // Special handling for ISO tag
                    if tag_id == NIKON_ISO_SETTING && values.len() >= 2 {
                        let iso = if values[1] > 0 { values[1] } else { values[0] };
                        ExifValue::Ascii(iso.to_string())
                    } else if values.len() == 1 {
                        // Apply value decoders for single SHORT values
                        let v = values[0];
                        let decoded = match tag_id {
                            NIKON_ACTIVE_D_LIGHTING => {
                                Some(decode_active_d_lighting(v).to_string())
                            }
                            NIKON_COLOR_SPACE => Some(decode_color_space(v).to_string()),
                            NIKON_VIGNETTE_CONTROL => Some(decode_vignette_control(v).to_string()),
                            NIKON_HIGH_ISO_NOISE_REDUCTION => {
                                Some(decode_high_iso_noise_reduction(v).to_string())
                            }
                            NIKON_DATE_STAMP_MODE => Some(decode_date_stamp_mode(v).to_string()),
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
                    let mut cursor = Cursor::new(&value_bytes);
                    for _ in 0..count {
                        if let Ok(v) = match maker_endian {
                            Endianness::Little => cursor.read_u32::<LittleEndian>(),
                            Endianness::Big => cursor.read_u32::<BigEndian>(),
                        } {
                            values.push(v);
                        } else {
                            break;
                        }
                    }
                    // Special handling for ShutterCount
                    if tag_id == NIKON_SHUTTER_COUNT && values.len() == 1 {
                        let v = values[0];
                        if v == 4294965247 {
                            ExifValue::Ascii("n/a".to_string())
                        } else {
                            ExifValue::Long(values)
                        }
                    } else {
                        ExifValue::Long(values)
                    }
                }
                5 => {
                    // RATIONAL
                    let mut values = Vec::new();
                    let mut cursor = Cursor::new(&value_bytes);
                    for _ in 0..count {
                        let numerator = match maker_endian {
                            Endianness::Little => cursor.read_u32::<LittleEndian>(),
                            Endianness::Big => cursor.read_u32::<BigEndian>(),
                        };
                        let denominator = match maker_endian {
                            Endianness::Little => cursor.read_u32::<LittleEndian>(),
                            Endianness::Big => cursor.read_u32::<BigEndian>(),
                        };

                        if let (Ok(num), Ok(den)) = (numerator, denominator) {
                            values.push((num, den));
                        } else {
                            break;
                        }
                    }

                    // Apply decoder for lens info
                    if tag_id == NIKON_LENS {
                        if let Some(formatted) = format_lens_info(&values) {
                            ExifValue::Ascii(formatted)
                        } else {
                            ExifValue::Rational(values)
                        }
                    } else {
                        ExifValue::Rational(values)
                    }
                }
                7 => {
                    // UNDEFINED - keep as binary
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
                    tag_name: get_nikon_tag_name(tag_id),
                    value,
                },
            );
        }
    }

    Ok(tags)
}
