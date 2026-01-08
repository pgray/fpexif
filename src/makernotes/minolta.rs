// makernotes/minolta.rs - Minolta/Konica Minolta maker notes parsing
//
// Reference files:
// - exiftool/lib/Image/ExifTool/Minolta.pm
// - exiv2/src/minoltamn_int.cpp
//
// Minolta/Konica Minolta cameras (Dimage series, Dynax/Maxxum/Alpha DSLRs)
// exited the camera business in 2006, transferring to Sony.
// MakerNote format varies by camera series.

use crate::data_types::{Endianness, ExifValue};
use crate::define_tag_decoder;
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Cursor;

// Main Minolta MakerNote tag IDs
pub const MINOLTA_VERSION: u16 = 0x0000;
pub const MINOLTA_CAMERA_SETTINGS_OLD: u16 = 0x0001;
pub const MINOLTA_CAMERA_SETTINGS: u16 = 0x0003;
pub const MINOLTA_CAMERA_SETTINGS_7D: u16 = 0x0004;
pub const MINOLTA_CAMERA_INFO_A100: u16 = 0x0010;
pub const MINOLTA_IMAGE_STABILIZATION_DATA: u16 = 0x0018;
pub const MINOLTA_WB_INFO_A100: u16 = 0x0020;
pub const MINOLTA_COMPRESSED_IMAGE_SIZE: u16 = 0x0040;
pub const MINOLTA_THUMBNAIL: u16 = 0x0081;
pub const MINOLTA_THUMBNAIL_OFFSET: u16 = 0x0088;
pub const MINOLTA_THUMBNAIL_LENGTH: u16 = 0x0089;
pub const MINOLTA_SCENE_MODE: u16 = 0x0100;
pub const MINOLTA_COLOR_MODE: u16 = 0x0101;
pub const MINOLTA_QUALITY: u16 = 0x0102;
pub const MINOLTA_IMAGE_SIZE: u16 = 0x0103;
pub const MINOLTA_FLASH_EXPOSURE_COMP: u16 = 0x0104;
pub const MINOLTA_TELECONVERTER: u16 = 0x0105;
pub const MINOLTA_IMAGE_STABILIZATION: u16 = 0x0107;
pub const MINOLTA_RAW_AND_JPG_RECORDING: u16 = 0x0109;
pub const MINOLTA_ZONE_MATCHING: u16 = 0x010A;
pub const MINOLTA_COLOR_TEMPERATURE: u16 = 0x010B;
pub const MINOLTA_LENS_ID: u16 = 0x010C;
pub const MINOLTA_COLOR_COMPENSATION_FILTER: u16 = 0x0111;
pub const MINOLTA_WHITE_BALANCE_FINE_TUNE: u16 = 0x0112;
pub const MINOLTA_IMAGE_STABILIZATION_A100: u16 = 0x0113;
pub const MINOLTA_CAMERA_SETTINGS_5D: u16 = 0x0114;
pub const MINOLTA_WHITE_BALANCE: u16 = 0x0115;
pub const MINOLTA_PRINT_IM: u16 = 0x0E00;
pub const MINOLTA_CAMERA_SETTINGS_Z1: u16 = 0x0F00;

// CameraSettings sub-tags (stored within tag 0x0001/0x0003)
pub const MINOLTA_CS_EXPOSURE_MODE: u16 = 1;
pub const MINOLTA_CS_FLASH_MODE: u16 = 2;
pub const MINOLTA_CS_WHITE_BALANCE: u16 = 3;
pub const MINOLTA_CS_IMAGE_SIZE: u16 = 4;
pub const MINOLTA_CS_QUALITY: u16 = 5;
pub const MINOLTA_CS_DRIVE_MODE: u16 = 6;
pub const MINOLTA_CS_METERING_MODE: u16 = 7;
pub const MINOLTA_CS_ISO: u16 = 8;
pub const MINOLTA_CS_EXPOSURE_TIME: u16 = 9;
pub const MINOLTA_CS_F_NUMBER: u16 = 10;
pub const MINOLTA_CS_MACRO_MODE: u16 = 11;
pub const MINOLTA_CS_DIGITAL_ZOOM: u16 = 12;
pub const MINOLTA_CS_EXPOSURE_COMPENSATION: u16 = 13;
pub const MINOLTA_CS_BRACKET_STEP: u16 = 14;
pub const MINOLTA_CS_FOCAL_LENGTH: u16 = 18;
pub const MINOLTA_CS_FOCUS_DISTANCE: u16 = 19;
pub const MINOLTA_CS_FLASH_FIRED: u16 = 20;
pub const MINOLTA_CS_MINOLTA_DATE: u16 = 21;
pub const MINOLTA_CS_MINOLTA_TIME: u16 = 22;
pub const MINOLTA_CS_MAX_APERTURE: u16 = 23;
pub const MINOLTA_CS_SATURATION: u16 = 31;
pub const MINOLTA_CS_CONTRAST: u16 = 32;
pub const MINOLTA_CS_SHARPNESS: u16 = 33;
pub const MINOLTA_CS_SUBJECT_PROGRAM: u16 = 34;
pub const MINOLTA_CS_ISO_SETTING: u16 = 36;
pub const MINOLTA_CS_MODEL_ID: u16 = 37;
pub const MINOLTA_CS_COLOR_MODE: u16 = 40;
pub const MINOLTA_CS_COLOR_FILTER: u16 = 41;
pub const MINOLTA_CS_FOCUS_MODE: u16 = 48;
pub const MINOLTA_CS_FOCUS_AREA: u16 = 49;

/// Get the name of a Minolta maker note tag
pub fn get_minolta_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        MINOLTA_VERSION => Some("MakerNoteVersion"),
        MINOLTA_CAMERA_SETTINGS_OLD => Some("MinoltaCameraSettingsOld"),
        MINOLTA_CAMERA_SETTINGS => Some("MinoltaCameraSettings"),
        MINOLTA_CAMERA_SETTINGS_7D => Some("MinoltaCameraSettings7D"),
        MINOLTA_CAMERA_INFO_A100 => Some("CameraInfoA100"),
        MINOLTA_IMAGE_STABILIZATION_DATA => Some("ImageStabilizationData"),
        MINOLTA_WB_INFO_A100 => Some("WBInfoA100"),
        MINOLTA_COMPRESSED_IMAGE_SIZE => Some("CompressedImageSize"),
        MINOLTA_THUMBNAIL => Some("Thumbnail"),
        MINOLTA_THUMBNAIL_OFFSET => Some("ThumbnailOffset"),
        MINOLTA_THUMBNAIL_LENGTH => Some("ThumbnailLength"),
        MINOLTA_SCENE_MODE => Some("SceneMode"),
        MINOLTA_COLOR_MODE => Some("ColorMode"),
        MINOLTA_QUALITY => Some("Quality"),
        MINOLTA_IMAGE_SIZE => Some("ImageSize"),
        MINOLTA_FLASH_EXPOSURE_COMP => Some("FlashExposureComp"),
        MINOLTA_TELECONVERTER => Some("Teleconverter"),
        MINOLTA_IMAGE_STABILIZATION => Some("ImageStabilization"),
        MINOLTA_RAW_AND_JPG_RECORDING => Some("RawAndJpgRecording"),
        MINOLTA_ZONE_MATCHING => Some("ZoneMatching"),
        MINOLTA_COLOR_TEMPERATURE => Some("ColorTemperature"),
        MINOLTA_LENS_ID => Some("LensID"),
        MINOLTA_COLOR_COMPENSATION_FILTER => Some("ColorCompensationFilter"),
        MINOLTA_WHITE_BALANCE_FINE_TUNE => Some("WhiteBalanceFineTune"),
        MINOLTA_IMAGE_STABILIZATION_A100 => Some("ImageStabilizationA100"),
        MINOLTA_CAMERA_SETTINGS_5D => Some("CameraSettings5D"),
        MINOLTA_WHITE_BALANCE => Some("WhiteBalance"),
        MINOLTA_PRINT_IM => Some("PrintIM"),
        MINOLTA_CAMERA_SETTINGS_Z1 => Some("CameraSettingsZ1"),
        _ => None,
    }
}

/// Get the name of a Minolta CameraSettings sub-tag
pub fn get_minolta_cs_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        MINOLTA_CS_EXPOSURE_MODE => Some("ExposureMode"),
        MINOLTA_CS_FLASH_MODE => Some("FlashMode"),
        MINOLTA_CS_WHITE_BALANCE => Some("WhiteBalance"),
        MINOLTA_CS_IMAGE_SIZE => Some("MinoltaImageSize"),
        MINOLTA_CS_QUALITY => Some("MinoltaQuality"),
        MINOLTA_CS_DRIVE_MODE => Some("DriveMode"),
        MINOLTA_CS_METERING_MODE => Some("MeteringMode"),
        MINOLTA_CS_ISO => Some("ISO"),
        MINOLTA_CS_EXPOSURE_TIME => Some("ExposureTime"),
        MINOLTA_CS_F_NUMBER => Some("FNumber"),
        MINOLTA_CS_MACRO_MODE => Some("MacroMode"),
        MINOLTA_CS_DIGITAL_ZOOM => Some("DigitalZoom"),
        MINOLTA_CS_EXPOSURE_COMPENSATION => Some("ExposureCompensation"),
        MINOLTA_CS_BRACKET_STEP => Some("BracketStep"),
        MINOLTA_CS_FOCAL_LENGTH => Some("FocalLength"),
        MINOLTA_CS_FOCUS_DISTANCE => Some("FocusDistance"),
        MINOLTA_CS_FLASH_FIRED => Some("FlashFired"),
        MINOLTA_CS_MINOLTA_DATE => Some("MinoltaDate"),
        MINOLTA_CS_MINOLTA_TIME => Some("MinoltaTime"),
        MINOLTA_CS_MAX_APERTURE => Some("MaxAperture"),
        MINOLTA_CS_SATURATION => Some("Saturation"),
        MINOLTA_CS_CONTRAST => Some("Contrast"),
        MINOLTA_CS_SHARPNESS => Some("Sharpness"),
        MINOLTA_CS_SUBJECT_PROGRAM => Some("SubjectProgram"),
        MINOLTA_CS_ISO_SETTING => Some("ISOSetting"),
        MINOLTA_CS_MODEL_ID => Some("MinoltaModelID"),
        MINOLTA_CS_FOCUS_MODE => Some("FocusMode"),
        MINOLTA_CS_FOCUS_AREA => Some("FocusArea"),
        MINOLTA_CS_COLOR_MODE => Some("ColorMode"),
        _ => None,
    }
}

// ===== Tag decoders using the define_tag_decoder! macro =====
// Minolta uses u32 type for all tags

define_tag_decoder! {
    minolta_exposure_mode,
    type: u32,
    exiftool: {
        0 => "Program",
        1 => "Aperture Priority",
        2 => "Shutter Priority",
        3 => "Manual",
    },
    exiv2: {
        0 => "Program",
        1 => "Aperture priority",
        2 => "Shutter priority",
        3 => "Manual",
    }
}

define_tag_decoder! {
    minolta_flash_mode,
    type: u32,
    both: {
        0 => "Fill flash",
        1 => "Red-eye reduction",
        2 => "Rear flash sync",
        3 => "Wireless",
        4 => "Off",
    }
}

define_tag_decoder! {
    minolta_white_balance_cs,
    type: u32,
    both: {
        0 => "Auto",
        1 => "Daylight",
        2 => "Cloudy",
        3 => "Tungsten",
        5 => "Custom",
        7 => "Fluorescent",
        8 => "Fluorescent 2",
        11 => "Custom 2",
        12 => "Custom 3",
    }
}

define_tag_decoder! {
    minolta_white_balance,
    type: u32,
    both: {
        0x00 => "Auto",
        0x01 => "Color Temperature/Color Filter",
        0x10 => "Daylight",
        0x20 => "Cloudy",
        0x30 => "Shade",
        0x40 => "Tungsten",
        0x50 => "Flash",
        0x60 => "Fluorescent",
        0x70 => "Custom",
    }
}

define_tag_decoder! {
    minolta_image_size,
    type: u32,
    exiftool: {
        0 => "Full",
        1 => "1600x1200",
        2 => "1280x960",
        3 => "640x480",
        6 => "2080x1560",
        7 => "2560x1920",
        8 => "3264x2176",
    },
    exiv2: {
        0 => "Full size",
        1 => "1600x1200",
        2 => "1280x960",
        3 => "640x480",
        6 => "2080x1560",
        7 => "2560x1920",
        8 => "3264x2176",
    }
}

define_tag_decoder! {
    minolta_quality,
    type: u32,
    exiftool: {
        0 => "Raw",
        1 => "Super Fine",
        2 => "Fine",
        3 => "Standard",
        4 => "Economy",
        5 => "Extra Fine",
    },
    exiv2: {
        0 => "Raw",
        1 => "Super fine",
        2 => "Fine",
        3 => "Standard",
        4 => "Economy",
        5 => "Extra fine",
    }
}

define_tag_decoder! {
    minolta_drive_mode,
    type: u32,
    both: {
        0 => "Single",
        1 => "Continuous",
        2 => "Self-timer",
        4 => "Bracketing",
        5 => "Interval",
        6 => "UHS continuous",
        7 => "HS continuous",
    }
}

define_tag_decoder! {
    minolta_metering_mode,
    type: u32,
    both: {
        0 => "Multi-segment",
        1 => "Center weighted average",
        2 => "Spot",
    }
}

define_tag_decoder! {
    minolta_macro_mode,
    type: u32,
    both: {
        0 => "Off",
        1 => "On",
        2 => "Super Macro",
    }
}

define_tag_decoder! {
    minolta_digital_zoom,
    type: u32,
    both: {
        0 => "Off",
        1 => "Electronic magnification",
        2 => "2x",
    }
}

define_tag_decoder! {
    minolta_bracket_step,
    type: u32,
    both: {
        0 => "1/3 EV",
        1 => "2/3 EV",
        2 => "1 EV",
    }
}

define_tag_decoder! {
    minolta_flash_fired,
    type: u32,
    exiftool: {
        0 => "No",
        1 => "Yes",
    },
    exiv2: {
        0 => "Did not fire",
        1 => "Fired",
    }
}

define_tag_decoder! {
    minolta_sharpness,
    type: u32,
    both: {
        0 => "Hard",
        1 => "Normal",
        2 => "Soft",
    }
}

define_tag_decoder! {
    minolta_subject_program,
    type: u32,
    exiftool: {
        0 => "None",
        1 => "Portrait",
        2 => "Text",
        3 => "Night Portrait",
        4 => "Sunset",
        5 => "Sports",
    },
    exiv2: {
        0 => "None",
        1 => "Portrait",
        2 => "Text",
        3 => "Night portrait",
        4 => "Sunset",
        5 => "Sports action",
    }
}

define_tag_decoder! {
    minolta_iso_setting,
    type: u32,
    both: {
        0 => "100",
        1 => "200",
        2 => "400",
        3 => "800",
        4 => "Auto",
        5 => "64",
    }
}

define_tag_decoder! {
    minolta_model_id,
    type: u32,
    both: {
        0 => "DiMAGE 7, X1, X21 or X31",
        1 => "DiMAGE 5",
        2 => "DiMAGE S304",
        3 => "DiMAGE S404",
        4 => "DiMAGE 7i",
        5 => "DiMAGE 7Hi",
        6 => "DiMAGE A1",
        7 => "DiMAGE A2 or S414",
    }
}

define_tag_decoder! {
    minolta_focus_mode,
    type: u32,
    exiftool: {
        0 => "AF",
        1 => "MF",
    },
    exiv2: {
        0 => "AF",
        1 => "MF",
    }
}

define_tag_decoder! {
    minolta_focus_area,
    type: u32,
    exiftool: {
        0 => "Wide Focus (normal)",
        1 => "Spot Focus",
    },
    exiv2: {
        0 => "Wide focus (normal)",
        1 => "Spot focus",
    }
}

define_tag_decoder! {
    minolta_color_mode,
    type: u32,
    exiftool: {
        0 => "Natural color",
        1 => "Black & White",
        2 => "Vivid color",
        3 => "Solarization",
        4 => "Adobe RGB",
        5 => "Sepia",
        9 => "Natural",
        12 => "Portrait",
        13 => "Natural sRGB",
        14 => "Natural+ sRGB",
        15 => "Landscape",
        16 => "Evening",
        17 => "Night Scene",
        18 => "Night Portrait",
    },
    exiv2: {
        0 => "Natural color",
        1 => "Black & White",
        2 => "Vivid color",
        3 => "Solarization",
        4 => "AdobeRGB",
        5 => "Sepia",
        9 => "Natural",
        12 => "Portrait",
        13 => "Natural sRGB",
        14 => "Natural+ sRGB",
        15 => "Landscape",
        16 => "Evening",
        17 => "Night Scene",
        18 => "Night Portrait",
    }
}

define_tag_decoder! {
    minolta_color_mode_cs,
    type: u32,
    exiftool: {
        0 => "Natural color",
        1 => "Black & White",
        2 => "Vivid color",
        3 => "Solarization",
        4 => "Adobe RGB",
    },
    exiv2: {
        0 => "Natural color",
        1 => "Black and white",
        2 => "Vivid color",
        3 => "Solarization",
        4 => "Adobe RGB",
    }
}

define_tag_decoder! {
    minolta_scene_mode,
    type: u32,
    both: {
        0 => "Standard",
        1 => "Portrait",
        2 => "Text",
        3 => "Night Scene",
        4 => "Sunset",
        5 => "Sports",
        6 => "Landscape",
        7 => "Night Portrait",
        8 => "Macro",
        9 => "Super Macro",
        16 => "Auto",
        17 => "Night View/Portrait",
    }
}

define_tag_decoder! {
    minolta_image_stabilization,
    type: u32,
    both: {
        1 => "Off",
        5 => "On",
    }
}

define_tag_decoder! {
    minolta_zone_matching,
    type: u32,
    both: {
        0 => "ISO Setting Used",
        1 => "High Key",
        2 => "Low Key",
    }
}

define_tag_decoder! {
    minolta_teleconverter,
    type: u32,
    both: {
        0x00 => "None",
        0x04 => "Minolta/Sony AF 1.4x APO (D) (0x04)",
        0x05 => "Minolta/Sony AF 2x APO (D) (0x05)",
        0x48 => "Minolta/Sony AF 2x APO (D)",
        0x50 => "Minolta AF 2x APO II",
        0x60 => "Minolta AF 2x APO",
        0x88 => "Minolta/Sony AF 1.4x APO (D)",
        0x90 => "Minolta AF 1.4x APO II",
        0xa0 => "Minolta AF 1.4x APO",
    }
}

define_tag_decoder! {
    minolta_raw_and_jpg,
    type: u32,
    both: {
        0 => "Off",
        1 => "On",
    }
}

/// Get Minolta/Sony A-mount lens name from lens ID
/// Reference: Minolta.pm %minoltaLensTypes
pub fn get_minolta_lens_name(lens_id: u32) -> Option<&'static str> {
    match lens_id {
        // Low IDs (Sony SAL lenses)
        0 => Some("Minolta AF 28-85mm F3.5-4.5"),
        1 => Some("Minolta AF 80-200mm F2.8 HS-APO G"),
        2 => Some("Minolta AF 28-70mm F2.8 G"),
        3 => Some("Minolta AF 28-80mm F4-5.6"),
        4 => Some("Minolta AF 85mm F1.4G"),
        5 => Some("Minolta AF 35-70mm F3.5-4.5"),
        6 => Some("Minolta AF 24-85mm F3.5-4.5"),
        7 => Some("Minolta AF 100-300mm F4.5-5.6 APO"),
        13 => Some("Minolta AF 75-300mm F4.5-5.6"),
        16 => Some("Minolta AF 17-35mm F3.5 G"),
        19 => Some("Minolta AF 35mm F1.4 G"),
        20 => Some("Minolta/Sony 135mm F2.8 STF"),
        25 => Some("Minolta AF 100-300mm F4.5-5.6 APO (D)"),
        27 => Some("Minolta AF 85mm F1.4 G (D)"),
        28 => Some("Minolta/Sony AF 100mm F2.8 Macro (D)"),
        29 => Some("Minolta/Sony AF 75-300mm F4.5-5.6 (D)"),
        31 => Some("Minolta/Sony AF 50mm F2.8 Macro (D)"),
        32 => Some("Minolta/Sony AF 300mm F2.8 G APO"),
        33 => Some("Minolta/Sony AF 70-200mm F2.8 G"),
        38 => Some("Minolta AF 17-35mm F2.8-4 (D)"),
        39 => Some("Minolta AF 28-75mm F2.8 (D)"),
        40 => Some("Minolta/Sony AF DT 18-70mm F3.5-5.6 (D)"),
        41 => Some("Minolta/Sony AF DT 11-18mm F4.5-5.6 (D)"),
        42 => Some("Minolta/Sony AF DT 18-200mm F3.5-6.3 (D)"),
        // High IDs from %minoltaLensTypes (older Minolta lenses)
        25601 => Some("Minolta AF 100-200mm F4.5"),
        25611 => Some("Minolta AF 75-300mm F4.5-5.6 (or Sigma)"),
        25621 => Some("Minolta AF 50mm F1.4 [New]"),
        25631 => Some("Minolta AF 300mm F2.8 APO (or Sigma)"),
        25641 => Some("Minolta AF 50mm F2.8 Macro (or Sigma)"),
        25651 => Some("Minolta AF 600mm F4 HS-APO G"),
        25721 => Some("Minolta AF 500mm F8 Reflex"),
        25781 => Some("Minolta/Sony AF 16mm F2.8 Fisheye"),
        25811 => Some("Minolta/Sony AF 20mm F2.8"),
        25851 => Some("Minolta AF 35-105mm F3.5-4.5"),
        25858 => Some("Tamron SP AF 90mm F2.5 (172E)"),
        25881 => Some("Minolta AF 70-210mm F3.5-4.5"),
        25891 => Some("Minolta AF 80-200mm F2.8 APO"),
        25911 => Some("Minolta AF 35mm F1.4"),
        25921 => Some("Minolta AF 85mm F1.4 G (D)"),
        25931 => Some("Minolta AF 200mm F2.8 APO"),
        26011 => Some("Minolta AF 35-80mm F4-5.6"),
        26041 => Some("Minolta AF 80-200mm F4.5-5.6"),
        26051 => Some("Minolta AF 35-80mm F4-5.6"),
        26061 => Some("Minolta AF 100mm F2"),
        26071 => Some("Minolta AF 100-300mm F4.5-5.6"),
        26121 => Some("Minolta AF 200mm F2.8 HS-APO G"),
        26131 => Some("Minolta AF 50mm F1.7"),
        26241 => Some("Minolta AF 35-80mm F4-5.6 Power Zoom"),
        _ => None,
    }
}

// Legacy function aliases for backward compatibility
pub fn decode_exposure_mode_exiftool(value: u32) -> &'static str {
    decode_minolta_exposure_mode_exiftool(value)
}

pub fn decode_exposure_mode_exiv2(value: u32) -> &'static str {
    decode_minolta_exposure_mode_exiv2(value)
}

pub fn decode_flash_mode_exiftool(value: u32) -> &'static str {
    decode_minolta_flash_mode_exiftool(value)
}

pub fn decode_flash_mode_exiv2(value: u32) -> &'static str {
    decode_minolta_flash_mode_exiv2(value)
}

pub fn decode_white_balance_cs_exiftool(value: u32) -> &'static str {
    decode_minolta_white_balance_cs_exiftool(value)
}

pub fn decode_white_balance_cs_exiv2(value: u32) -> &'static str {
    decode_minolta_white_balance_cs_exiv2(value)
}

pub fn decode_white_balance_exiftool(value: u32) -> &'static str {
    decode_minolta_white_balance_exiftool(value)
}

pub fn decode_white_balance_exiv2(value: u32) -> &'static str {
    decode_minolta_white_balance_exiv2(value)
}

pub fn decode_image_size_exiftool(value: u32) -> &'static str {
    decode_minolta_image_size_exiftool(value)
}

pub fn decode_image_size_exiv2(value: u32) -> &'static str {
    decode_minolta_image_size_exiv2(value)
}

pub fn decode_quality_exiftool(value: u32) -> &'static str {
    decode_minolta_quality_exiftool(value)
}

pub fn decode_quality_exiv2(value: u32) -> &'static str {
    decode_minolta_quality_exiv2(value)
}

pub fn decode_drive_mode_exiftool(value: u32) -> &'static str {
    decode_minolta_drive_mode_exiftool(value)
}

pub fn decode_drive_mode_exiv2(value: u32) -> &'static str {
    decode_minolta_drive_mode_exiv2(value)
}

pub fn decode_metering_mode_exiftool(value: u32) -> &'static str {
    decode_minolta_metering_mode_exiftool(value)
}

pub fn decode_metering_mode_exiv2(value: u32) -> &'static str {
    decode_minolta_metering_mode_exiv2(value)
}

pub fn decode_macro_mode_exiftool(value: u32) -> &'static str {
    decode_minolta_macro_mode_exiftool(value)
}

pub fn decode_digital_zoom_exiftool(value: u32) -> &'static str {
    decode_minolta_digital_zoom_exiftool(value)
}

pub fn decode_digital_zoom_exiv2(value: u32) -> &'static str {
    decode_minolta_digital_zoom_exiv2(value)
}

pub fn decode_bracket_step_exiftool(value: u32) -> &'static str {
    decode_minolta_bracket_step_exiftool(value)
}

pub fn decode_flash_fired_exiftool(value: u32) -> &'static str {
    decode_minolta_flash_fired_exiftool(value)
}

pub fn decode_flash_fired_exiv2(value: u32) -> &'static str {
    decode_minolta_flash_fired_exiv2(value)
}

pub fn decode_sharpness_exiftool(value: u32) -> &'static str {
    decode_minolta_sharpness_exiftool(value)
}

pub fn decode_sharpness_exiv2(value: u32) -> &'static str {
    decode_minolta_sharpness_exiv2(value)
}

pub fn decode_contrast_exiftool(value: u32) -> &'static str {
    decode_minolta_sharpness_exiftool(value)
}

pub fn decode_contrast_exiv2(value: u32) -> &'static str {
    decode_minolta_sharpness_exiv2(value)
}

pub fn decode_subject_program_exiftool(value: u32) -> &'static str {
    decode_minolta_subject_program_exiftool(value)
}

pub fn decode_subject_program_exiv2(value: u32) -> &'static str {
    decode_minolta_subject_program_exiv2(value)
}

pub fn decode_iso_setting_exiftool(value: u32) -> &'static str {
    decode_minolta_iso_setting_exiftool(value)
}

pub fn decode_model_id_exiftool(value: u32) -> &'static str {
    decode_minolta_model_id_exiftool(value)
}

pub fn decode_focus_mode_exiftool(value: u32) -> &'static str {
    decode_minolta_focus_mode_exiftool(value)
}

pub fn decode_focus_mode_exiv2(value: u32) -> &'static str {
    decode_minolta_focus_mode_exiv2(value)
}

pub fn decode_focus_area_exiftool(value: u32) -> &'static str {
    decode_minolta_focus_area_exiftool(value)
}

pub fn decode_focus_area_exiv2(value: u32) -> &'static str {
    decode_minolta_focus_area_exiv2(value)
}

pub fn decode_color_mode_exiftool(value: u32) -> &'static str {
    decode_minolta_color_mode_exiftool(value)
}

pub fn decode_color_mode_exiv2(value: u32) -> &'static str {
    decode_minolta_color_mode_exiv2(value)
}

pub fn decode_color_mode_cs_exiftool(value: u32) -> &'static str {
    decode_minolta_color_mode_cs_exiftool(value)
}

pub fn decode_color_mode_cs_exiv2(value: u32) -> &'static str {
    decode_minolta_color_mode_cs_exiv2(value)
}

pub fn decode_scene_mode_exiftool(value: u32) -> &'static str {
    decode_minolta_scene_mode_exiftool(value)
}

pub fn decode_image_stabilization_exiftool(value: u32) -> &'static str {
    decode_minolta_image_stabilization_exiftool(value)
}

pub fn decode_image_stabilization_exiv2(value: u32) -> &'static str {
    decode_minolta_image_stabilization_exiv2(value)
}

pub fn decode_zone_matching_exiftool(value: u32) -> &'static str {
    decode_minolta_zone_matching_exiftool(value)
}

/// Parse Minolta CameraSettings sub-IFD (stored in tag 0x0001 or 0x0003)
fn parse_camera_settings(data: &[u8], _endian: Endianness) -> HashMap<u16, MakerNoteTag> {
    let mut tags = HashMap::new();

    // CameraSettings is stored as int32u array (big-endian)
    // Each index is a 4-byte value
    if data.len() < 4 {
        return tags;
    }

    let num_entries = data.len() / 4;

    for i in 0..num_entries {
        let offset = i * 4;
        if offset + 4 > data.len() {
            break;
        }

        // Always big-endian according to ExifTool
        let value = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);

        let tag_id = i as u16;

        // Decode specific tags
        let decoded_value = match tag_id {
            MINOLTA_CS_EXPOSURE_MODE => Some(decode_exposure_mode_exiftool(value).to_string()),
            MINOLTA_CS_FLASH_MODE => Some(decode_flash_mode_exiftool(value).to_string()),
            MINOLTA_CS_WHITE_BALANCE => Some(decode_white_balance_cs_exiftool(value).to_string()),
            MINOLTA_CS_IMAGE_SIZE => Some(decode_image_size_exiftool(value).to_string()),
            MINOLTA_CS_QUALITY => Some(decode_quality_exiftool(value).to_string()),
            MINOLTA_CS_DRIVE_MODE => Some(decode_drive_mode_exiftool(value).to_string()),
            MINOLTA_CS_METERING_MODE => Some(decode_metering_mode_exiftool(value).to_string()),
            MINOLTA_CS_MACRO_MODE => Some(decode_macro_mode_exiftool(value).to_string()),
            MINOLTA_CS_DIGITAL_ZOOM => Some(decode_digital_zoom_exiftool(value).to_string()),
            MINOLTA_CS_BRACKET_STEP => Some(decode_bracket_step_exiftool(value).to_string()),
            MINOLTA_CS_FLASH_FIRED => Some(decode_flash_fired_exiftool(value).to_string()),
            MINOLTA_CS_SHARPNESS => Some(decode_sharpness_exiftool(value).to_string()),
            // Saturation and Contrast use ValueConv: $val - 3 (or -5 for A2)
            // We use -3 as the default since we don't have model info here
            MINOLTA_CS_SATURATION => Some(format!("{}", value as i32 - 3)),
            MINOLTA_CS_CONTRAST => Some(format!("{}", value as i32 - 3)),
            MINOLTA_CS_SUBJECT_PROGRAM => Some(decode_subject_program_exiftool(value).to_string()),
            MINOLTA_CS_ISO_SETTING => Some(decode_iso_setting_exiftool(value).to_string()),
            MINOLTA_CS_MODEL_ID => Some(decode_model_id_exiftool(value).to_string()),
            MINOLTA_CS_FOCUS_MODE => Some(decode_focus_mode_exiftool(value).to_string()),
            MINOLTA_CS_FOCUS_AREA => Some(decode_focus_area_exiftool(value).to_string()),
            MINOLTA_CS_COLOR_MODE => Some(decode_color_mode_cs_exiftool(value).to_string()),
            // MinoltaDate: packed as YYYY<<16 | MM<<8 | DD
            MINOLTA_CS_MINOLTA_DATE => {
                let year = value >> 16;
                let month = (value >> 8) & 0xff;
                let day = value & 0xff;
                if year > 0 {
                    Some(format!("{:04}:{:02}:{:02}", year, month, day))
                } else {
                    None
                }
            }
            // MinoltaTime: packed as HH<<16 | MM<<8 | SS
            MINOLTA_CS_MINOLTA_TIME => {
                let hour = value >> 16;
                let minute = (value >> 8) & 0xff;
                let second = value & 0xff;
                if hour > 0 || minute > 0 || second > 0 {
                    Some(format!("{:02}:{:02}:{:02}", hour, minute, second))
                } else {
                    None
                }
            }
            // MaxAperture: 2^((val-8)/16)
            MINOLTA_CS_MAX_APERTURE => {
                if value > 0 {
                    let aperture = 2.0_f64.powf((value as f64 - 8.0) / 16.0);
                    Some(format!("{:.1}", aperture))
                } else {
                    None
                }
            }
            // FocusDistance: val/1000, output as "X.X m" or "inf"
            MINOLTA_CS_FOCUS_DISTANCE => {
                if value == 0 {
                    Some("inf".to_string())
                } else {
                    let distance = value as f64 / 1000.0;
                    Some(format!("{:.1} m", distance))
                }
            }
            _ => None,
        };

        let exif_value = if let Some(s) = decoded_value {
            ExifValue::Ascii(s)
        } else {
            ExifValue::Long(vec![value])
        };

        tags.insert(
            tag_id,
            MakerNoteTag {
                tag_id,
                tag_name: get_minolta_cs_tag_name(tag_id),
                value: exif_value,
            },
        );
    }

    tags
}

/// Parse Minolta maker notes
pub fn parse_minolta_maker_notes(
    data: &[u8],
    endian: Endianness,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    if data.len() < 10 {
        return Ok(tags);
    }

    // Determine offset based on header type
    // Some Minolta files have no header, some have various headers
    let ifd_offset = if data.len() >= 8 && data.starts_with(b"MLT0") {
        // MakerNote version header "MLT0" (4 bytes, not null-terminated)
        // IFD starts right after
        4
    } else if data.len() >= 10 && data.starts_with(b"MINOLTA\0") {
        // Some cameras use "MINOLTA\0" header (8 bytes)
        // Followed by TIFF header at offset 8
        let tiff_endian = if data.len() > 10 {
            match &data[8..10] {
                b"II" => Endianness::Little,
                b"MM" => Endianness::Big,
                _ => endian,
            }
        } else {
            endian
        };

        // IFD offset is at bytes 12-15 (relative to offset 8)
        if data.len() > 16 {
            let ifd_off = match tiff_endian {
                Endianness::Little => u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
                Endianness::Big => u32::from_be_bytes([data[12], data[13], data[14], data[15]]),
            };
            8 + ifd_off as usize
        } else {
            return Ok(tags);
        }
    } else {
        // No header, IFD starts at beginning
        0
    };

    if ifd_offset >= data.len() {
        return Ok(tags);
    }

    let mut cursor = Cursor::new(&data[ifd_offset..]);

    // Read number of entries
    let num_entries = match endian {
        Endianness::Little => cursor.read_u16::<LittleEndian>().map_err(|_| {
            ExifError::Format("Failed to read Minolta maker note count".to_string())
        })?,
        Endianness::Big => cursor.read_u16::<BigEndian>().map_err(|_| {
            ExifError::Format("Failed to read Minolta maker note count".to_string())
        })?,
    };

    // Parse IFD entries
    for _ in 0..num_entries {
        if cursor.position() as usize + 12 > data[ifd_offset..].len() {
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

                    // Apply decoders for single-value LONG tags
                    if values.len() == 1 {
                        let v = values[0];
                        let decoded = match tag_id {
                            MINOLTA_SCENE_MODE => Some(decode_scene_mode_exiftool(v).to_string()),
                            MINOLTA_COLOR_MODE => Some(decode_color_mode_exiftool(v).to_string()),
                            MINOLTA_QUALITY => Some(decode_quality_exiftool(v).to_string()),
                            MINOLTA_WHITE_BALANCE => {
                                Some(decode_white_balance_exiftool(v).to_string())
                            }
                            MINOLTA_IMAGE_STABILIZATION => {
                                Some(decode_image_stabilization_exiftool(v).to_string())
                            }
                            MINOLTA_ZONE_MATCHING => {
                                Some(decode_zone_matching_exiftool(v).to_string())
                            }
                            MINOLTA_TELECONVERTER => {
                                Some(decode_minolta_teleconverter_exiftool(v).to_string())
                            }
                            MINOLTA_RAW_AND_JPG_RECORDING => {
                                Some(decode_minolta_raw_and_jpg_exiftool(v).to_string())
                            }
                            MINOLTA_LENS_ID => get_minolta_lens_name(v).map(|s| s.to_string()),
                            _ => None,
                        };

                        if let Some(s) = decoded {
                            ExifValue::Ascii(s)
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
                    // UNDEFINED - Check if this is a CameraSettings sub-IFD
                    if tag_id == MINOLTA_CAMERA_SETTINGS || tag_id == MINOLTA_CAMERA_SETTINGS_OLD {
                        // Parse CameraSettings as a sub-structure
                        let cs_tags = parse_camera_settings(&value_bytes, endian);

                        // Store the sub-tags in the main tags HashMap with a special prefix
                        // We'll use tag_id in the 0x1000-0x1FFF range to avoid conflicts
                        for (cs_tag_id, cs_tag) in cs_tags {
                            let composite_tag_id = 0x1000 + cs_tag_id;
                            tags.insert(composite_tag_id, cs_tag);
                        }

                        // Also store the raw UNDEFINED value
                        ExifValue::Undefined(value_bytes)
                    } else if tag_id == MINOLTA_VERSION {
                        // MakerNoteVersion is a string like "MLT0"
                        let s = String::from_utf8_lossy(&value_bytes)
                            .trim_end_matches('\0')
                            .to_string();
                        ExifValue::Ascii(s)
                    } else {
                        ExifValue::Undefined(value_bytes)
                    }
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
                    tag_name: get_minolta_tag_name(tag_id),
                    value,
                },
            );
        }
    }

    Ok(tags)
}
