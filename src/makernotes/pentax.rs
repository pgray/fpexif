// makernotes/pentax.rs - Pentax maker notes parsing
//
// Reference files:
// - exiftool/lib/Image/ExifTool/Pentax.pm
// - exiv2/src/pentaxmn_int.cpp

use crate::data_types::{Endianness, ExifValue};
use crate::define_tag_decoder;
use crate::define_wrapper;
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
pub const PENTAX_NOISE_REDUCTION: u16 = 0x0049;
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
pub const PENTAX_SERIAL_NUMBER: u16 = 0x0229;
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
        PENTAX_IMAGE_SIZE => Some("PentaxImageSize"),
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
        PENTAX_NOISE_REDUCTION => Some("NoiseReduction"),
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
        PENTAX_SERIAL_NUMBER => Some("SerialNumber"),
        PENTAX_PIXEL_SHIFT_INFO => Some("PixelShiftInfo"),
        PENTAX_AF_POINT_INFO => Some("AFPointInfo"),
        PENTAX_HDR_INFO => Some("HDRInfo"),
        PENTAX_TEMPERATURE_INFO => Some("TemperatureInfo"),
        PENTAX_SERIAL_NUMBER_2 => Some("SerialNumber2"),
        _ => None,
    }
}

// Tag decoders using the define_tag_decoder! macro

define_tag_decoder! {
    pentax_quality,
    exiftool: {
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
    },
    exiv2: {
        0 => "Good",
        1 => "Better",
        2 => "Best",
        3 => "TIFF",
        4 => "RAW",
        5 => "Premium",
        65535 => "n/a",
    }
}

define_tag_decoder! {
    pentax_focus_mode,
    exiftool: {
        0x00 => "Normal",
        0x01 => "Macro",
        0x02 => "Infinity",
        0x03 => "Manual",
        0x04 => "Super Macro",
        0x05 => "Pan Focus",
        0x06 => "Auto-area",
        0x07 => "Zone Select",
        0x08 => "Select",
        0x09 => "Pinpoint",
        0x0a => "Tracking",
        0x0b => "Continuous",
        0x0c => "Snap",
        0x10 => "AF-S (Focus-priority)",
        0x11 => "AF-C (Focus-priority)",
        0x12 => "AF-A (Focus-priority)",
        0x20 => "Contrast-detect (Focus-priority)",
        0x21 => "Tracking Contrast-detect (Focus-priority)",
        0x110 => "AF-S (Release-priority)",
        0x111 => "AF-C (Release-priority)",
        0x112 => "AF-A (Release-priority)",
        0x120 => "Contrast-detect (Release-priority)",
    },
    exiv2: {
        0 => "Normal",
        1 => "Macro",
        2 => "Infinity",
        3 => "Manual",
        4 => "Super Macro",
        5 => "Pan Focus",
        16 => "AF-S",
        17 => "AF-C",
        18 => "AF-A",
        32 => "Contrast-detect",
        33 => "Tracking Contrast-detect",
        288 => "Face Detect",
    }
}

define_tag_decoder! {
    pentax_metering_mode,
    exiftool: {
        0 => "Multi-segment",
        1 => "Center-weighted average",
        2 => "Spot",
        6 => "Highlight",
    },
    exiv2: {
        0 => "Multi Segment",
        1 => "Center Weighted",
        2 => "Spot",
    }
}

define_tag_decoder! {
    pentax_white_balance,
    exiftool: {
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
    },
    exiv2: {
        0 => "Auto",
        1 => "Daylight",
        2 => "Shade",
        3 => "Fluorescent",
        4 => "Tungsten",
        5 => "Manual",
        6 => "DaylightFluorescent",
        7 => "DaywhiteFluorescent",
        8 => "WhiteFluorescent",
        9 => "Flash",
        10 => "Cloudy",
        15 => "Color Temperature Enhancement",
        17 => "Kelvin",
        65534 => "Unknown",
        65535 => "User Selected",
    }
}

define_tag_decoder! {
    pentax_flash_mode,
    exiftool: {
        0x0000 => "Auto, Did not fire",
        0x0001 => "Off, Did not fire",
        0x0002 => "On, Did not fire",
        0x0003 => "Auto, Did not fire, Red-eye reduction",
        0x0005 => "On, Did not fire, Wireless (Master)",
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
    },
    exiv2: {
        0x000 => "Auto, Did not fire",
        0x001 => "Off, Did not fire",
        0x002 => "Off, Did not fire",
        0x003 => "Auto, Did not fire, Red-eye reduction",
        0x005 => "On. Did not fire. Wireless (Master)",
        0x100 => "Auto, Fired",
        0x102 => "On, Fired",
        0x103 => "Auto, Fired, Red-eye reduction",
        0x104 => "On, Red-eye reduction",
        0x105 => "On, Wireless (Master)",
        0x106 => "On, Wireless (Control)",
        0x108 => "On, Soft",
        0x109 => "On, Slow-sync",
        0x10a => "On, Slow-sync, Red-eye reduction",
        0x10b => "On, Trailing-curtain Sync",
    }
}

define_tag_decoder! {
    pentax_saturation,
    exiftool: {
        0 => "-2 (low)",
        1 => "0 (normal)",
        2 => "+2 (high)",
        3 => "-1 (medium low)",
        4 => "+1 (medium high)",
        5 => "-3 (very low)",
        6 => "+3 (very high)",
        7 => "-4 (minimum)",
        8 => "+4 (maximum)",
        65535 => "None",
    },
    exiv2: {
        0 => "Low",
        1 => "Normal",
        2 => "High",
        3 => "Med Low",
        4 => "Med High",
        5 => "Very Low",
        6 => "Very High",
        7 => "-4",
        8 => "+4",
        65535 => "None",
    }
}

define_tag_decoder! {
    pentax_contrast,
    exiftool: {
        0 => "-2 (low)",
        1 => "0 (normal)",
        2 => "+2 (high)",
        3 => "-1 (medium low)",
        4 => "+1 (medium high)",
        5 => "-3 (very low)",
        6 => "+3 (very high)",
        7 => "-4 (minimum)",
        8 => "+4 (maximum)",
        65535 => "n/a",
    },
    exiv2: {
        0 => "Low",
        1 => "Normal",
        2 => "High",
        3 => "Med Low",
        4 => "Med High",
        5 => "Very Low",
        6 => "Very High",
        7 => "-4",
        8 => "+4",
    }
}

define_tag_decoder! {
    pentax_sharpness,
    exiftool: {
        0 => "-2 (soft)",
        1 => "0 (normal)",
        2 => "+2 (hard)",
        3 => "-1 (medium soft)",
        4 => "+1 (medium hard)",
        5 => "-3 (very soft)",
        6 => "+3 (very hard)",
        7 => "-4 (minimum)",
        8 => "+4 (maximum)",
    },
    exiv2: {
        0 => "Soft",
        1 => "Normal",
        2 => "Hard",
        3 => "Med Soft",
        4 => "Med Hard",
        5 => "Very Soft",
        6 => "Very Hard",
        7 => "-4",
        8 => "+4",
    }
}

define_tag_decoder! {
    pentax_drive_mode,
    exiftool: {
        0 => "Single-frame",
        1 => "Continuous",
        2 => "Continuous (Lo)",
        3 => "Burst",
        4 => "Continuous (Medium)",
        5 => "Continuous (Low)",
        255 => "Video",
    },
    exiv2: {
        0 => "Single-frame",
        1 => "Continuous",
        2 => "Continuous (Hi)",
        3 => "Burst",
        255 => "Video",
    }
}

define_tag_decoder! {
    pentax_color_space,
    both: {
        0 => "sRGB",
        1 => "Adobe RGB",
    }
}

// Additional tags from Pentax.pm and pentaxmn_int.cpp

define_tag_decoder! {
    pentax_picture_mode,
    exiftool: {
        0 => "Program",
        1 => "Shutter Speed Priority",
        2 => "Program AE",
        3 => "Manual",
        5 => "Portrait",
        6 => "Landscape",
        8 => "Sport",
        9 => "Night Scene",
        11 => "Soft",
        12 => "Surf & Snow",
        13 => "Candlelight",
        14 => "Autumn",
        15 => "Macro",
        17 => "Fireworks",
        18 => "Text",
        19 => "Panorama",
        20 => "3-D",
        21 => "Black & White",
        22 => "Sepia",
        23 => "Red",
        24 => "Pink",
        25 => "Purple",
        26 => "Blue",
        27 => "Green",
        28 => "Yellow",
        30 => "Self Portrait",
        31 => "Illustrations",
        33 => "Digital Filter",
        35 => "Night Scene Portrait",
        37 => "Museum",
        38 => "Food",
        39 => "Underwater",
        40 => "Green Mode",
        49 => "Light Pet",
        50 => "Dark Pet",
        51 => "Medium Pet",
        53 => "Underwater",
        54 => "Candlelight",
        55 => "Natural Skin Tone",
        56 => "Synchro Sound Record",
        58 => "Frame Composite",
        60 => "Kids",
        61 => "Blur Reduction",
        63 => "Panorama 2",
        65 => "Half-length Portrait",
        80 => "Scene Mode",
        81 => "Shutter Priority",
        82 => "Aperture Priority",
        83 => "Program",
        85 => "Portrait",
        221 => "Video",
        255 => "Digital Filter?",
    },
    exiv2: {
        0 => "Program",
        1 => "Hi-speed Program",
        2 => "DOF Program",
        3 => "MTF Program",
        4 => "Standard",
        5 => "Portrait",
        6 => "Landscape",
        7 => "Macro",
        8 => "Sport",
        9 => "Night Scene Portrait",
        10 => "No Flash",
        11 => "Night Scene",
        12 => "Surf & Snow",
        13 => "Text",
        14 => "Sunset",
        15 => "Kids",
        16 => "Pet",
        17 => "Candlelight",
        18 => "Museum",
    }
}

define_tag_decoder! {
    pentax_image_tone,
    both: {
        0 => "Natural",
        1 => "Bright",
        2 => "Portrait",
        3 => "Landscape",
        4 => "Vibrant",
        5 => "Monochrome",
        6 => "Muted",
        7 => "Reversal Film",
        8 => "Bleach Bypass",
        9 => "Radiant",
        10 => "Cross Processing",
        11 => "Flat",
    }
}

define_tag_decoder! {
    pentax_dynamic_range_expansion,
    both: {
        0 => "Off",
        1 => "On",
    }
}

define_tag_decoder! {
    pentax_high_iso_noise_reduction,
    exiftool: {
        0 => "Off",
        1 => "Weakest",
        2 => "Weak",
        3 => "Strong",
        4 => "Custom",
    },
    exiv2: {
        0 => "Off",
        1 => "Weakest",
        2 => "Weak",
        3 => "Strong",
        4 => "Custom",
    }
}

define_tag_decoder! {
    pentax_shake_reduction,
    both: {
        0 => "Off",
        1 => "On",
        4 => "On (4)",
        5 => "On but Disabled",
        6 => "On (Video)",
        7 => "On (Composition Adjust)",
        15 => "On (IBIS Only)",
        39 => "On (S-R Auto)",
    }
}

define_tag_decoder! {
    pentax_af_point_selected,
    both: {
        0 => "None",
        1 => "Upper-left",
        2 => "Top",
        3 => "Upper-right",
        4 => "Left",
        5 => "Mid-left",
        6 => "Center",
        7 => "Mid-right",
        8 => "Right",
        9 => "Lower-left",
        10 => "Bottom",
        11 => "Lower-right",
        65531 => "AF Select",
        65532 => "Face Detect AF",
        65533 => "Automatic Tracking AF",
        65534 => "Fixed Center",
        65535 => "Auto",
    }
}

define_tag_decoder! {
    pentax_image_processing,
    both: {
        0 => "Unprocessed",
        1 => "Resized",
        2 => "Cropped",
        3 => "Color Filter",
        4 => "Digital Filter",
        16 => "Frame Synthesis?",
    }
}

// Legacy function aliases for backward compatibility
define_wrapper!(decode_quality_exiftool, decode_pentax_quality_exiftool, u16);
define_wrapper!(
    decode_focus_mode_exiftool,
    decode_pentax_focus_mode_exiftool,
    u16
);
define_wrapper!(
    decode_metering_mode_exiftool,
    decode_pentax_metering_mode_exiftool,
    u16
);
define_wrapper!(
    decode_white_balance_exiftool,
    decode_pentax_white_balance_exiftool,
    u16
);
define_wrapper!(
    decode_flash_mode_exiftool,
    decode_pentax_flash_mode_exiftool,
    u16
);
define_wrapper!(
    decode_saturation_exiftool,
    decode_pentax_saturation_exiftool,
    u16
);
define_wrapper!(
    decode_contrast_exiftool,
    decode_pentax_contrast_exiftool,
    u16
);
define_wrapper!(
    decode_sharpness_exiftool,
    decode_pentax_sharpness_exiftool,
    u16
);
define_wrapper!(
    decode_drive_mode_exiftool,
    decode_pentax_drive_mode_exiftool,
    u16
);
define_wrapper!(
    decode_color_space_exiftool,
    decode_pentax_color_space_exiftool,
    u16
);

/// Get the name of a Pentax camera model from its ID
/// Reference: Pentax.pm %pentaxModelID
pub fn get_pentax_model_name(model_id: u32) -> Option<&'static str> {
    match model_id {
        0x0000d => Some("Optio 330/430"),
        0x12926 => Some("Optio 230"),
        0x12958 => Some("Optio 330GS"),
        0x12962 => Some("Optio 450/550"),
        0x1296c => Some("Optio S"),
        0x12971 => Some("Optio S V1.01"),
        0x12994 => Some("*ist D"),
        0x129b2 => Some("Optio 33L"),
        0x129bc => Some("Optio 33LF"),
        0x129c6 => Some("Optio 33WR/43WR/555"),
        0x129d5 => Some("Optio S4"),
        0x12a02 => Some("Optio MX"),
        0x12a0c => Some("Optio S40"),
        0x12a16 => Some("Optio S4i"),
        0x12a34 => Some("Optio 30"),
        0x12a52 => Some("Optio S30"),
        0x12a66 => Some("Optio 750Z"),
        0x12a70 => Some("Optio SV"),
        0x12a75 => Some("Optio SVi"),
        0x12a7a => Some("Optio X"),
        0x12a8e => Some("Optio S5i"),
        0x12a98 => Some("Optio S50"),
        0x12aa2 => Some("*ist DS"),
        0x12ab6 => Some("Optio MX4"),
        0x12ac0 => Some("Optio S5n"),
        0x12aca => Some("Optio WP"),
        0x12afc => Some("Optio S55"),
        0x12b10 => Some("Optio S5z"),
        0x12b1a => Some("*ist DL"),
        0x12b24 => Some("Optio S60"),
        0x12b2e => Some("Optio S45"),
        0x12b38 => Some("Optio S6"),
        0x12b4c => Some("Optio WPi"),
        0x12b56 => Some("BenQ DC X600"),
        0x12b60 => Some("*ist DS2"),
        0x12b62 => Some("Samsung GX-1S"),
        0x12b6a => Some("Optio A10"),
        0x12b7e => Some("*ist DL2"),
        0x12b80 => Some("Samsung GX-1L"),
        0x12b9c => Some("K100D"),
        0x12b9d => Some("K110D"),
        0x12ba2 => Some("K100D Super"),
        0x12bb0 => Some("Optio T10/T20"),
        0x12be2 => Some("Optio W10"),
        0x12bf6 => Some("Optio M10"),
        0x12c1e => Some("K10D"),
        0x12c20 => Some("Samsung GX10"),
        0x12c28 => Some("Optio S7"),
        0x12c2d => Some("Optio L20"),
        0x12c32 => Some("Optio M20"),
        0x12c3c => Some("Optio W20"),
        0x12c46 => Some("Optio A20"),
        0x12c78 => Some("Optio E30"),
        0x12c7d => Some("Optio E35"),
        0x12c82 => Some("Optio T30"),
        0x12c8c => Some("Optio M30"),
        0x12c91 => Some("Optio L30"),
        0x12c96 => Some("Optio W30"),
        0x12ca0 => Some("Optio A30"),
        0x12cb4 => Some("Optio E40"),
        0x12cbe => Some("Optio M40"),
        0x12cc3 => Some("Optio L40"),
        0x12cc5 => Some("Optio L36"),
        0x12cc8 => Some("Optio Z10"),
        0x12cd2 => Some("K20D"),
        0x12cd4 => Some("Samsung GX20"),
        0x12cdc => Some("Optio S10"),
        0x12ce6 => Some("Optio A40"),
        0x12cf0 => Some("Optio V10"),
        0x12cfa => Some("K200D"),
        0x12d04 => Some("Optio S12"),
        0x12d0e => Some("Optio E50"),
        0x12d18 => Some("Optio M50"),
        0x12d22 => Some("Optio L50"),
        0x12d2c => Some("Optio V20"),
        0x12d40 => Some("Optio W60"),
        0x12d4a => Some("Optio M60"),
        0x12d68 => Some("Optio E60/M90"),
        0x12d72 => Some("K2000"),
        0x12d73 => Some("K-m"),
        0x12d86 => Some("Optio P70"),
        0x12d90 => Some("Optio L70"),
        0x12d9a => Some("Optio E70"),
        0x12dae => Some("X70"),
        0x12db8 => Some("K-7"),
        0x12dcc => Some("Optio W80"),
        0x12dea => Some("Optio P80"),
        0x12df4 => Some("Optio WS80"),
        0x12dfe => Some("K-x"),
        0x12e08 => Some("645D"),
        0x12e12 => Some("Optio E80"),
        0x12e30 => Some("Optio W90"),
        0x12e3a => Some("Optio I-10"),
        0x12e44 => Some("Optio H90"),
        0x12e4e => Some("Optio E90"),
        0x12e58 => Some("X90"),
        0x12e6c => Some("K-r"),
        0x12e76 => Some("K-5"),
        0x12e8a => Some("Optio RS1000/RS1500"),
        0x12e94 => Some("Optio RZ10"),
        0x12e9e => Some("Optio LS1000"),
        0x12ebc => Some("Optio WG-1 GPS"),
        0x12ed0 => Some("Optio S1"),
        0x12ee4 => Some("Q"),
        0x12ef8 => Some("K-01"),
        0x12f0c => Some("Optio RZ18"),
        0x12f16 => Some("Optio VS20"),
        0x12f2a => Some("Optio WG-2 GPS"),
        0x12f48 => Some("Optio LS465"),
        0x12f52 => Some("K-30"),
        0x12f5c => Some("X-5"),
        0x12f66 => Some("Q10"),
        0x12f70 => Some("K-5 II"),
        0x12f71 => Some("K-5 II s"),
        0x12f7a => Some("Q7"),
        0x12f84 => Some("MX-1"),
        0x12f8e => Some("WG-3 GPS"),
        0x12f98 => Some("WG-3"),
        0x12fa2 => Some("WG-10"),
        0x12fb6 => Some("K-50"),
        0x12fc0 => Some("K-3"),
        0x12fca => Some("K-500"),
        0x12fe8 => Some("WG-4"),
        0x12fde => Some("WG-4 GPS"),
        0x13006 => Some("WG-20"),
        0x13010 => Some("645Z"),
        0x1301a => Some("K-S1"),
        0x13024 => Some("K-S2"),
        0x1302e => Some("Q-S1"),
        0x13056 => Some("WG-30"),
        0x1307e => Some("WG-30W"),
        0x13088 => Some("WG-5 GPS"),
        0x13092 => Some("K-1"),
        0x1309c => Some("K-3 II"),
        0x131f0 => Some("WG-M2"),
        0x1320e => Some("GR III"),
        0x13222 => Some("K-70"),
        0x1322c => Some("KP"),
        0x13240 => Some("K-1 Mark II"),
        0x13254 => Some("K-3 Mark III"),
        0x13290 => Some("WG-70"),
        0x1329a => Some("GR IIIx"),
        0x132b8 => Some("KF"),
        0x132d6 => Some("K-3 Mark III Monochrome"),
        0x132e0 => Some("GR IV"),
        _ => None,
    }
}

/// Get the name of a Pentax lens from series and model ID
/// Reference: Pentax.pm %pentaxLensTypes
pub fn get_pentax_lens_name(series: u8, model: u8) -> Option<&'static str> {
    match (series, model) {
        (0, 0) => Some("M-42 or No Lens"),
        (1, 0) => Some("K or M Lens"),
        (2, 0) => Some("A Series Lens"),
        (3, 0) => Some("Sigma"),
        (3, 17) => Some("smc PENTAX-FA SOFT 85mm F2.8"),
        (3, 18) => Some("smc PENTAX-F 1.7X AF ADAPTER"),
        (3, 19) => Some("smc PENTAX-F 24-50mm F4"),
        (3, 20) => Some("smc PENTAX-F 35-80mm F4-5.6"),
        (3, 21) => Some("smc PENTAX-F 80-200mm F4.7-5.6"),
        (3, 22) => Some("smc PENTAX-F FISH-EYE 17-28mm F3.5-4.5"),
        (3, 23) => Some("smc PENTAX-F 100-300mm F4.5-5.6 or Sigma Lens"),
        (3, 24) => Some("smc PENTAX-F 35-135mm F3.5-4.5"),
        (3, 25) => Some("smc PENTAX-F 35-105mm F4-5.6 or Sigma or Tokina Lens"),
        (3, 26) => Some("smc PENTAX-F* 250-600mm F5.6 ED[IF]"),
        (3, 27) => Some("smc PENTAX-F 28-80mm F3.5-4.5 or Tokina Lens"),
        (3, 28) => Some("smc PENTAX-F 35-70mm F3.5-4.5 or Tokina Lens"),
        (3, 29) => Some("PENTAX-F 28-80mm F3.5-4.5 or Sigma or Tokina Lens"),
        (3, 30) => Some("PENTAX-F 70-200mm F4-5.6"),
        (3, 31) => Some("smc PENTAX-F 70-210mm F4-5.6 or Tokina or Takumar Lens"),
        (3, 32) => Some("smc PENTAX-F 50mm F1.4"),
        (3, 33) => Some("smc PENTAX-F 50mm F1.7"),
        (3, 34) => Some("smc PENTAX-F 135mm F2.8 [IF]"),
        (3, 35) => Some("smc PENTAX-F 28mm F2.8"),
        (3, 36) => Some("Sigma 20mm F1.8 EX DG Aspherical RF"),
        (3, 38) => Some("smc PENTAX-F* 300mm F4.5 ED[IF]"),
        (3, 39) => Some("smc PENTAX-F* 600mm F4 ED[IF]"),
        (3, 40) => Some("smc PENTAX-F Macro 100mm F2.8"),
        (3, 41) => Some("smc PENTAX-F Macro 50mm F2.8 or Sigma Lens"),
        (3, 50) => Some("smc PENTAX-FA 28-70mm F4 AL"),
        (3, 51) => Some("Sigma 28mm F1.8 EX DG Aspherical Macro"),
        (3, 52) => Some("smc PENTAX-FA 28-200mm F3.8-5.6 AL[IF] or Tamron Lens"),
        (3, 53) => Some("smc PENTAX-FA 28-80mm F3.5-5.6 AL"),
        (3, 247) => Some("smc PENTAX-DA FISH-EYE 10-17mm F3.5-4.5 ED[IF]"),
        (3, 248) => Some("smc PENTAX-DA 12-24mm F4 ED AL[IF]"),
        (3, 250) => Some("smc PENTAX-DA 50-200mm F4-5.6 ED"),
        (3, 251) => Some("smc PENTAX-DA 40mm F2.8 Limited"),
        (3, 252) => Some("smc PENTAX-DA 18-55mm F3.5-5.6 AL"),
        (3, 253) => Some("smc PENTAX-DA 14mm F2.8 ED[IF]"),
        (3, 254) => Some("smc PENTAX-DA 16-45mm F4 ED AL"),
        (3, 255) => Some("Sigma Lens (3 255)"),
        (4, 1) => Some("smc PENTAX-FA SOFT 28mm F2.8"),
        (4, 2) => Some("smc PENTAX-FA 80-320mm F4.5-5.6"),
        (4, 3) => Some("smc PENTAX-FA 43mm F1.9 Limited"),
        (4, 6) => Some("smc PENTAX-FA 35-80mm F4-5.6"),
        (4, 12) => Some("smc PENTAX-FA 50mm F1.4"),
        (4, 15) => Some("smc PENTAX-FA 28-105mm F4-5.6 [IF]"),
        (4, 23) => Some("smc PENTAX-FA 20-35mm F4 AL"),
        (4, 24) => Some("smc PENTAX-FA 77mm F1.8 Limited"),
        (4, 28) => Some("smc PENTAX-FA 35mm F2 AL"),
        (4, 34) => Some("smc PENTAX-FA 24-90mm F3.5-4.5 AL[IF]"),
        (4, 35) => Some("smc PENTAX-FA 100-300mm F4.7-5.8"),
        (4, 38) => Some("smc PENTAX-FA 28-105mm F3.2-4.5 AL[IF]"),
        (4, 39) => Some("smc PENTAX-FA 31mm F1.8 AL Limited"),
        (4, 43) => Some("smc PENTAX-FA 28-90mm F3.5-5.6"),
        (4, 44) => Some("smc PENTAX-FA J 75-300mm F4.5-5.8 AL"),
        (4, 46) => Some("smc PENTAX-FA J 28-80mm F3.5-5.6 AL"),
        (4, 47) => Some("smc PENTAX-FA J 18-35mm F4-5.6 AL"),
        (4, 51) => Some("smc PENTAX-D FA 50mm F2.8 Macro"),
        (4, 52) => Some("smc PENTAX-D FA 100mm F2.8 Macro"),
        (4, 214) => Some("smc PENTAX-DA 35mm F2.4 AL"),
        (4, 229) => Some("smc PENTAX-DA 18-55mm F3.5-5.6 AL II"),
        (4, 230) => Some("Tamron SP AF 17-50mm F2.8 XR Di II"),
        (4, 231) => Some("smc PENTAX-DA 18-250mm F3.5-6.3 ED AL [IF]"),
        (4, 243) => Some("smc PENTAX-DA 70mm F2.4 Limited"),
        (4, 244) => Some("smc PENTAX-DA 21mm F3.2 AL Limited"),
        (4, 247) => Some("smc PENTAX-DA FISH-EYE 10-17mm F3.5-4.5 ED[IF]"),
        (4, 248) => Some("smc PENTAX-DA 12-24mm F4 ED AL [IF]"),
        (4, 250) => Some("smc PENTAX-DA 50-200mm F4-5.6 ED"),
        (4, 251) => Some("smc PENTAX-DA 40mm F2.8 Limited"),
        (4, 252) => Some("smc PENTAX-DA 18-55mm F3.5-5.6 AL"),
        (4, 253) => Some("smc PENTAX-DA 14mm F2.8 ED[IF]"),
        (4, 254) => Some("smc PENTAX-DA 16-45mm F4 ED AL"),
        (5, 1) => Some("smc PENTAX-FA* 24mm F2 AL[IF]"),
        (5, 2) => Some("smc PENTAX-FA 28mm F2.8 AL"),
        (5, 3) => Some("smc PENTAX-FA 50mm F1.7"),
        (5, 4) => Some("smc PENTAX-FA 50mm F1.4"),
        (5, 5) => Some("smc PENTAX-FA* 600mm F4 ED[IF]"),
        (5, 6) => Some("smc PENTAX-FA* 300mm F4.5 ED[IF]"),
        (5, 7) => Some("smc PENTAX-FA 135mm F2.8 [IF]"),
        (5, 8) => Some("smc PENTAX-FA Macro 50mm F2.8"),
        (5, 9) => Some("smc PENTAX-FA Macro 100mm F2.8"),
        (5, 10) => Some("smc PENTAX-FA* 85mm F1.4 [IF]"),
        (5, 11) => Some("smc PENTAX-FA* 200mm F2.8 ED[IF]"),
        (5, 12) => Some("smc PENTAX-FA 28-80mm F3.5-4.7"),
        (5, 13) => Some("smc PENTAX-FA 70-200mm F4-5.6"),
        (5, 14) => Some("smc PENTAX-FA* 250-600mm F5.6 ED[IF]"),
        (5, 15) => Some("smc PENTAX-FA 28-105mm F4-5.6"),
        (5, 16) => Some("smc PENTAX-FA 100-300mm F4.5-5.6"),
        (6, 1) => Some("smc PENTAX-FA* 85mm F1.4 [IF]"),
        (6, 2) => Some("smc PENTAX-FA* 200mm F2.8 ED[IF]"),
        (6, 3) => Some("smc PENTAX-FA* 300mm F2.8 ED[IF]"),
        (6, 4) => Some("smc PENTAX-FA* 28-70mm F2.8 AL"),
        (6, 5) => Some("smc PENTAX-FA* 80-200mm F2.8 ED[IF]"),
        (6, 9) => Some("smc PENTAX-FA 20mm F2.8"),
        (6, 10) => Some("smc PENTAX-FA* 400mm F5.6 ED[IF]"),
        (6, 14) => Some("smc PENTAX-FA* Macro 200mm F4 ED[IF]"),
        (7, 0) => Some("smc PENTAX-DA 21mm F3.2 AL Limited"),
        (7, 58) => Some("smc PENTAX-D FA Macro 100mm F2.8 WR"),
        (7, 201) => Some("smc Pentax-DA L 50-200mm F4-5.6 ED WR"),
        (7, 202) => Some("smc PENTAX-DA L 18-55mm F3.5-5.6 AL WR"),
        (7, 203) => Some("HD PENTAX-DA 55-300mm F4-5.8 ED WR"),
        (7, 204) => Some("HD PENTAX-DA 15mm F4 ED AL Limited"),
        (7, 205) => Some("HD PENTAX-DA 35mm F2.8 Macro Limited"),
        (7, 206) => Some("HD PENTAX-DA 70mm F2.4 Limited"),
        (7, 207) => Some("HD PENTAX-DA 21mm F3.2 ED AL Limited"),
        (7, 208) => Some("HD PENTAX-DA 40mm F2.8 Limited"),
        (7, 212) => Some("smc PENTAX-DA 50mm F1.8"),
        (7, 213) => Some("smc PENTAX-DA 40mm F2.8 XS"),
        (7, 214) => Some("smc PENTAX-DA 35mm F2.4 AL"),
        (7, 216) => Some("smc PENTAX-DA L 55-300mm F4-5.8 ED"),
        (7, 217) => Some("smc PENTAX-DA 50-200mm F4-5.6 ED WR"),
        (7, 218) => Some("smc PENTAX-DA 18-55mm F3.5-5.6 AL WR"),
        (7, 221) => Some("smc PENTAX-DA L 50-200mm F4-5.6 ED"),
        (7, 222) => Some("smc PENTAX-DA L 18-55mm F3.5-5.6"),
        (7, 223) => Some("Samsung/Schneider D-XENON 18-55mm F3.5-5.6 II"),
        (7, 224) => Some("smc PENTAX-DA 15mm F4 ED AL Limited"),
        (7, 229) => Some("smc PENTAX-DA 18-55mm F3.5-5.6 AL II"),
        (7, 230) => Some("Tamron AF 17-50mm F2.8 XR Di-II LD (Model A16)"),
        (7, 231) => Some("smc PENTAX-DA 18-250mm F3.5-6.3 ED AL [IF]"),
        (7, 233) => Some("smc PENTAX-DA 35mm F2.8 Macro Limited"),
        (7, 234) => Some("smc PENTAX-DA* 300mm F4 ED [IF] SDM (SDM unused)"),
        (7, 235) => Some("smc PENTAX-DA* 200mm F2.8 ED [IF] SDM (SDM unused)"),
        (7, 236) => Some("smc PENTAX-DA 55-300mm F4-5.8 ED"),
        (7, 238) => Some("Tamron AF 18-250mm F3.5-6.3 Di II LD Aspherical [IF] Macro"),
        (7, 241) => Some("smc PENTAX-DA* 50-135mm F2.8 ED [IF] SDM (SDM unused)"),
        (7, 242) => Some("smc PENTAX-DA* 16-50mm F2.8 ED AL [IF] SDM (SDM unused)"),
        (7, 243) => Some("smc PENTAX-DA 70mm F2.4 Limited"),
        (7, 244) => Some("smc PENTAX-DA 21mm F3.2 AL Limited"),
        (8, 0) => Some("Sigma 50-150mm F2.8 II APO EX DC HSM"),
        (8, 3) => Some("Sigma 18-125mm F3.8-5.6 DC HSM"),
        (8, 4) => Some("Sigma 50mm F1.4 EX DG HSM"),
        (8, 59) => Some("HD PENTAX-D FA 150-450mm F4.5-5.6 ED DC AW"),
        (8, 60) => Some("HD PENTAX-D FA* 70-200mm F2.8 ED DC AW"),
        (8, 61) => Some("HD PENTAX-D FA 28-105mm F3.5-5.6 ED DC WR"),
        (8, 62) => Some("HD PENTAX-D FA 24-70mm F2.8 ED SDM WR"),
        (8, 63) => Some("HD PENTAX-D FA 15-30mm F2.8 ED SDM WR"),
        (8, 64) => Some("HD PENTAX-D FA* 50mm F1.4 SDM AW"),
        (8, 65) => Some("HD PENTAX-D FA 70-210mm F4 ED SDM WR"),
        (8, 195) => Some("HD PENTAX DA* 16-50mm F2.8 ED PLM AW"),
        (8, 196) => Some("HD PENTAX-DA* 11-18mm F2.8 ED DC AW"),
        (8, 197) => Some("HD PENTAX-DA 55-300mm F4.5-6.3 ED PLM WR RE"),
        (8, 198) => Some("smc PENTAX-DA L 18-50mm F4-5.6 DC WR RE"),
        (8, 199) => Some("HD PENTAX-DA 18-50mm F4-5.6 DC WR RE"),
        (8, 200) => Some("HD PENTAX-DA 16-85mm F3.5-5.6 ED DC WR"),
        (8, 209) => Some("HD PENTAX-DA 20-40mm F2.8-4 ED Limited DC WR"),
        (8, 210) => Some("smc PENTAX-DA 18-270mm F3.5-6.3 ED SDM"),
        (8, 211) => Some("HD PENTAX-DA 560mm F5.6 ED AW"),
        (8, 215) => Some("smc PENTAX-DA 18-135mm F3.5-5.6 ED AL [IF] DC WR"),
        (8, 226) => Some("smc PENTAX-DA* 55mm F1.4 SDM"),
        (8, 227) => Some("smc PENTAX-DA* 60-250mm F4 [IF] SDM"),
        (8, 232) => Some("smc PENTAX-DA 17-70mm F4 AL [IF] SDM"),
        (8, 234) => Some("smc PENTAX-DA* 300mm F4 ED [IF] SDM"),
        (8, 235) => Some("smc PENTAX-DA* 200mm F2.8 ED [IF] SDM"),
        (8, 241) => Some("smc PENTAX-DA* 50-135mm F2.8 ED [IF] SDM"),
        (8, 242) => Some("smc PENTAX-DA* 16-50mm F2.8 ED AL [IF] SDM"),
        (8, 255) => Some("Sigma Lens (8 255)"),
        (9, 3) => Some("HD PENTAX-FA 43mm F1.9 Limited"),
        (9, 24) => Some("HD PENTAX-FA 77mm F1.8 Limited"),
        (9, 39) => Some("HD PENTAX-FA 31mm F1.8 AL Limited"),
        (9, 247) => Some("HD PENTAX-DA FISH-EYE 10-17mm F3.5-4.5 ED [IF]"),
        (21, 1) => Some("01 Standard Prime 8.5mm F1.9"),
        (21, 2) => Some("02 Standard Zoom 5-15mm F2.8-4.5"),
        (21, 6) => Some("06 Telephoto Zoom 15-45mm F2.8"),
        (22, 3) => Some("03 Fish-eye 3.2mm F5.6"),
        (22, 4) => Some("04 Toy Lens Wide 6.3mm F7.1"),
        (22, 5) => Some("05 Toy Lens Telephoto 18mm F8"),
        (31, 1) => Some("18.3mm F2.8"),
        (31, 4) => Some("26.1mm F2.8"),
        (31, 5) => Some("26.1mm F2.8 GT-2 TC"),
        (31, 8) => Some("18.3mm F2.8"),
        _ => None,
    }
}

/// Decode Pentax firmware version (XOR encrypted)
/// Reference: Pentax.pm %pentaxFirmwareID
pub fn decode_firmware_version(data: &[u8]) -> String {
    if data.len() < 4 {
        return format_bytes_as_hex(data);
    }
    // XOR each byte with 0xFF to decrypt
    let decoded: Vec<u8> = data.iter().map(|b| b ^ 0xff).collect();
    format!(
        "{}.{:02}.{:02}.{:02}",
        decoded[0], decoded[1], decoded[2], decoded[3]
    )
}

/// Format bytes as hex string for unknown data
fn format_bytes_as_hex(data: &[u8]) -> String {
    data.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Decode Pentax DriveMode multi-value tag
/// Reference: Pentax.pm tag 0x0034
pub fn decode_drive_mode_multi(data: &[u16]) -> String {
    if data.is_empty() {
        return "Unknown".to_string();
    }

    let mut parts = Vec::new();

    // First value: drive mode
    if !data.is_empty() {
        let mode = match data[0] {
            0 => "Single-frame",
            1 => "Continuous",
            2 => "Continuous (Lo)",
            3 => "Burst",
            4 => "Continuous (Medium)",
            5 => "Continuous (Lo)",
            255 => "Video",
            _ => "Unknown",
        };
        parts.push(mode.to_string());
    }

    // Second value: self-timer
    if data.len() > 1 {
        let timer = match data[1] {
            0 => "No Timer",
            1 => "Self-timer (12 s)",
            2 => "Self-timer (2 s)",
            _ => "Timer Unknown",
        };
        parts.push(timer.to_string());
    }

    // Third value: shutter release button
    if data.len() > 2 {
        let shutter = match data[2] {
            0 => "Shutter Button",
            1 => "Remote Control (3 s delay)",
            2 => "Remote Control",
            _ => "Shutter Unknown",
        };
        parts.push(shutter.to_string());
    }

    // Fourth value: exposure bracket
    if data.len() > 3 {
        let exposure = match data[3] {
            0 => "Single Exposure",
            1 => "Multiple Exposure",
            _ => "Exposure Unknown",
        };
        parts.push(exposure.to_string());
    }

    parts.join("; ")
}

/// Decode Pentax WorldTimeLocation
pub fn decode_world_time_location(value: u16) -> &'static str {
    match value {
        0 => "Hometown",
        1 => "Destination",
        _ => "Unknown",
    }
}

/// Decode Pentax DST (Daylight Saving Time)
pub fn decode_dst(value: u16) -> &'static str {
    match value {
        0 => "No",
        1 => "Yes",
        _ => "Unknown",
    }
}

/// Decode Pentax City code
/// Reference: Pentax.pm %pentaxCities
pub fn decode_city(value: u32) -> Option<&'static str> {
    match value {
        0 => Some("Pago Pago"),
        1 => Some("Honolulu"),
        2 => Some("Anchorage"),
        3 => Some("Vancouver"),
        4 => Some("San Francisco"),
        5 => Some("Los Angeles"),
        6 => Some("Calgary"),
        7 => Some("Denver"),
        8 => Some("Mexico City"),
        9 => Some("Chicago"),
        10 => Some("Miami"),
        11 => Some("Toronto"),
        12 => Some("New York"),
        13 => Some("Santiago"),
        14 => Some("Caracus"),
        15 => Some("Halifax"),
        16 => Some("Buenos Aires"),
        17 => Some("Sao Paulo"),
        18 => Some("Rio de Janeiro"),
        19 => Some("Madrid"),
        20 => Some("London"),
        21 => Some("Paris"),
        22 => Some("Milan"),
        23 => Some("Rome"),
        24 => Some("Berlin"),
        25 => Some("Johannesburg"),
        26 => Some("Istanbul"),
        27 => Some("Cairo"),
        28 => Some("Jerusalem"),
        29 => Some("Moscow"),
        30 => Some("Jeddah"),
        31 => Some("Tehran"),
        32 => Some("Dubai"),
        33 => Some("Karachi"),
        34 => Some("Kabul"),
        35 => Some("Male"),
        36 => Some("Delhi"),
        37 => Some("Colombo"),
        38 => Some("Kathmandu"),
        39 => Some("Dacca"),
        40 => Some("Yangon"),
        41 => Some("Bangkok"),
        42 => Some("Kuala Lumpur"),
        43 => Some("Vientiane"),
        44 => Some("Singapore"),
        45 => Some("Phnom Penh"),
        46 => Some("Ho Chi Minh"),
        47 => Some("Jakarta"),
        48 => Some("Hong Kong"),
        49 => Some("Perth"),
        50 => Some("Beijing"),
        51 => Some("Shanghai"),
        52 => Some("Manila"),
        53 => Some("Taipei"),
        54 => Some("Seoul"),
        55 => Some("Adelaide"),
        56 => Some("Tokyo"),
        57 => Some("Guam"),
        58 => Some("Sydney"),
        59 => Some("Noumea"),
        60 => Some("Wellington"),
        61 => Some("Auckland"),
        62 => Some("Lima"),
        63 => Some("Dakar"),
        64 => Some("Algiers"),
        65 => Some("Helsinki"),
        66 => Some("Athens"),
        67 => Some("Nairobi"),
        68 => Some("Amsterdam"),
        69 => Some("Stockholm"),
        70 => Some("Lisbon"),
        71 => Some("Copenhagen"),
        72 => Some("Warsaw"),
        73 => Some("Prague"),
        74 => Some("Budapest"),
        _ => None,
    }
}

/// Decode DriveMode from bytes (when stored as BYTE type)
/// Reference: Pentax.pm tag 0x0034
pub fn decode_drive_mode_from_bytes(data: &[u8]) -> String {
    if data.is_empty() {
        return "Unknown".to_string();
    }

    let mut parts = Vec::new();

    // First value: drive mode
    let mode = match data[0] {
        0 => "Single-frame",
        1 => "Continuous",
        2 => "Continuous (Lo)",
        3 => "Burst",
        4 => "Continuous (Medium)",
        5 => "Continuous (Lo)",
        255 => "Video",
        _ => "Unknown",
    };
    parts.push(mode.to_string());

    // Second value: self-timer
    if data.len() > 1 {
        let timer = match data[1] {
            0 => "No Timer",
            1 => "Self-timer (12 s)",
            2 => "Self-timer (2 s)",
            _ => "Timer Unknown",
        };
        parts.push(timer.to_string());
    }

    // Third value: shutter release button
    if data.len() > 2 {
        let shutter = match data[2] {
            0 => "Shutter Button",
            1 => "Remote Control (3 s delay)",
            2 => "Remote Control",
            _ => "Shutter Unknown",
        };
        parts.push(shutter.to_string());
    }

    // Fourth value: exposure bracket
    if data.len() > 3 {
        let exposure = match data[3] {
            0 => "Single Exposure",
            1 => "Multiple Exposure",
            _ => "Exposure Unknown",
        };
        parts.push(exposure.to_string());
    }

    parts.join("; ")
}

/// Decode Pentax Date from bytes
/// Reference: Pentax.pm tag 0x0006 - Year is int16u in MM (big-endian) byte order
pub fn decode_date_from_bytes(data: &[u8]) -> Option<String> {
    if data.len() >= 4 {
        // Year is big-endian u16, month and day are single bytes
        let year = ((data[0] as u16) << 8) | (data[1] as u16);
        let month = data[2];
        let day = data[3];
        if year > 0 && year < 3000 && (1..=12).contains(&month) && (1..=31).contains(&day) {
            return Some(format!("{:04}:{:02}:{:02}", year, month, day));
        }
    }
    None
}

/// Decode Pentax Time from bytes
/// Reference: Pentax.pm tag 0x0007 - 3 bytes for HH:MM:SS
pub fn decode_time_from_bytes(data: &[u8]) -> Option<String> {
    if data.len() >= 3 {
        let hour = data[0];
        let minute = data[1];
        let second = data[2];
        if hour < 24 && minute < 60 && second < 60 {
            return Some(format!("{:02}:{:02}:{:02}", hour, minute, second));
        }
    }
    None
}

/// Decode Pentax WhiteBalanceMode
/// Reference: Pentax.pm tag 0x001A
pub fn decode_white_balance_mode(value: u16) -> Option<&'static str> {
    match value {
        1 => Some("Auto (Daylight)"),
        2 => Some("Auto (Shade)"),
        3 => Some("Auto (Flash)"),
        4 => Some("Auto (Tungsten)"),
        6 => Some("Auto (Daylight Fluorescent)"),
        7 => Some("Auto (Day White Fluorescent)"),
        8 => Some("Auto (White Fluorescent)"),
        10 => Some("Auto (Cloudy)"),
        0xfffe => Some("Unknown"),
        0xffff => Some("User-Selected"),
        _ => None,
    }
}

/// Decode Pentax AutoBracketing
/// Reference: Pentax.pm tag 0x0018
pub fn decode_auto_bracketing(values: &[u16]) -> String {
    if values.is_empty() {
        return "Unknown".to_string();
    }

    // First value: EV bracket step
    let ev = values[0];

    // Single value case
    if values.len() == 1 {
        // When single value is 0, ExifTool outputs just "0"
        if ev == 0 {
            return "0".to_string();
        }
        let ev_val = if ev < 10 {
            ev as f32 / 3.0
        } else if ev < 20 {
            ev as f32 - 9.5
        } else if ev & 0x1000 != 0 {
            (ev & 0x0fff) as f32 / 2.0
        } else if ev & 0x2000 != 0 {
            (ev & 0x0fff) as f32 / 3.0
        } else {
            return format!("{}", ev);
        };
        return format!("{:.1}", ev_val);
    }

    // Two value case - format with EV and extended bracket
    let mut parts = Vec::new();

    let ev_str = if ev == 0 {
        "0 EV".to_string()
    } else if ev < 10 {
        format!("{:.1} EV", ev as f32 / 3.0)
    } else if ev < 20 {
        format!("{:.1} EV", ev as f32 - 9.5)
    } else if ev & 0x1000 != 0 {
        format!("{:.1} EV", (ev & 0x0fff) as f32 / 2.0)
    } else if ev & 0x2000 != 0 {
        format!("{:.1} EV", (ev & 0x0fff) as f32 / 3.0)
    } else {
        format!("{} EV", ev)
    };
    parts.push(ev_str);

    // Second value: extended bracket mode
    let ext = values[1];
    let ext_type = (ext >> 8) as u8;
    let ext_level = ext & 0xff;
    let ext_str = if ext == 0 {
        "No Extended Bracket".to_string()
    } else {
        let type_name = match ext_type {
            1 => "WB-BA",
            2 => "WB-GM",
            3 => "Saturation",
            4 => "Sharpness",
            5 => "Contrast",
            6 => "Hue",
            7 => "HighLowKey",
            _ => "Unknown",
        };
        format!("{}+{}", type_name, ext_level)
    };
    parts.push(ext_str);

    parts.join(", ")
}

/// Decode Pentax PictureMode multi-value tag (0x0033)
/// Reference: Pentax.pm tag 0x0033
pub fn decode_picture_mode_multi(data: &[u8]) -> String {
    if data.is_empty() {
        return "Unknown".to_string();
    }

    // First two bytes form the mode key
    let mode_key = if data.len() >= 2 {
        (data[0], data[1])
    } else {
        (data[0], 0)
    };

    let mode = match mode_key {
        (0, 0) => "Program",
        (0, 1) => "Hi-speed Program",
        (0, 2) => "DOF Program",
        (0, 3) => "MTF Program",
        (0, 4) => "Standard",
        (0, 5) => "Portrait",
        (0, 6) => "Landscape",
        (0, 7) => "Macro",
        (0, 8) => "Sport",
        (0, 9) => "Night Scene Portrait",
        (0, 10) => "No Flash",
        (0, 11) => "Night Scene",
        (0, 12) => "Surf & Snow",
        (0, 13) => "Text",
        (0, 14) => "Sunset",
        (0, 15) => "Kids",
        (0, 16) => "Pet",
        (0, 17) => "Candlelight",
        (0, 18) => "Museum",
        (0, 19) => "Food",
        (0, 20) => "Stage Lighting",
        (0, 21) => "Night Snap",
        (0, 23) => "Blue Sky",
        (0, 24) => "Sunset",
        (0, 26) => "Night Scene HDR",
        (0, 27) => "HDR",
        (0, 28) => "Quick Macro",
        (0, 29) => "Forest",
        (0, 30) => "Backlight Silhouette",
        (0, 31) => "Max. Aperture Priority",
        (0, 32) => "DOF",
        (1, 4) => "Auto PICT (Standard)",
        (1, 5) => "Auto PICT (Portrait)",
        (1, 6) => "Auto PICT (Landscape)",
        (1, 7) => "Auto PICT (Macro)",
        (1, 8) => "Auto PICT (Sport)",
        (2, 0) => "Program (HyP)",
        (2, 1) => "Hi-speed Program (HyP)",
        (2, 2) => "DOF Program (HyP)",
        (2, 3) => "MTF Program (HyP)",
        (2, 22) => "Shallow DOF (HyP)",
        (3, 0) => "Green Mode",
        (4, 0) => "Shutter Speed Priority",
        (5, 0) => "Aperture Priority",
        (6, 0) => "Program Tv Shift",
        (7, 0) => "Program Av Shift",
        (8, 0) => "Manual",
        (9, 0) => "Bulb",
        (10, 0) => "Aperture Priority, Off-Auto-Aperture",
        (11, 0) => "Manual, Off-Auto-Aperture",
        (12, 0) => "Bulb, Off-Auto-Aperture",
        (13, 0) => "Shutter & Aperture Priority AE",
        (14, 0) => "Shutter Priority AE",
        (15, 0) => "Sensitivity Priority AE",
        (16, 0) => "Flash X-Sync Speed AE",
        (17, 0) => "Flash X-Sync Speed",
        (18, 0) => "Auto Program (Normal)",
        (18, 1) => "Auto Program (Hi-speed)",
        (18, 2) => "Auto Program (DOF)",
        (18, 3) => "Auto Program (MTF)",
        (18, 22) => "Auto Program (Shallow DOF)",
        (19, 0) => "Astrotracer",
        (20, 22) => "Blur Control",
        (24, 0) => "Aperture Priority (Adv.Hyp)",
        (25, 0) => "Manual Exposure (Adv.Hyp)",
        (26, 0) => "Shutter and Aperture Priority (TAv)",
        (249, 0) => "Movie (TAv)",
        (250, 0) => "Movie (TAv, Auto Aperture)",
        (251, 0) => "Movie (Manual)",
        (252, 0) => "Movie (Manual, Auto Aperture)",
        (253, 0) => "Movie (Av)",
        (254, 0) => "Movie (Av, Auto Aperture)",
        (255, 0) => "Movie (P, Auto Aperture)",
        (255, 4) => "Video (4)",
        _ => return format!("{} {}", data[0], if data.len() > 1 { data[1] } else { 0 }),
    };

    // Third value is EV step size
    if data.len() >= 3 {
        let ev_steps = match data[2] {
            0 => "1/2 EV steps",
            1 => "1/3 EV steps",
            _ => "",
        };
        if !ev_steps.is_empty() {
            return format!("{}; {}", mode, ev_steps);
        }
    }

    mode.to_string()
}

/// Decode Pentax AFPointSelected multi-value
pub fn decode_af_point_selected_multi(data: &[u16]) -> String {
    if data.is_empty() {
        return "Unknown".to_string();
    }

    let point = match data[0] {
        0 => "None",
        1 => "Upper-left",
        2 => "Top",
        3 => "Upper-right",
        4 => "Left",
        5 => "Mid-left",
        6 => "Center",
        7 => "Mid-right",
        8 => "Right",
        9 => "Lower-left",
        10 => "Bottom",
        11 => "Lower-right",
        0xfffa => "Auto 2",
        0xfffb => "AF Select",
        0xfffc => "Face Detect AF",
        0xfffd => "Automatic Tracking AF",
        0xfffe => "Fixed Center",
        0xffff => "Auto",
        _ => return format!("{}", data[0]),
    };

    // If there's a second value, it's the AF area mode
    // Value 0 is "Single Point", 1 is "Expanded Area"
    if data.len() > 1 {
        let area = match data[1] {
            0 => "Single Point",
            1 => "Expanded Area",
            _ => "",
        };
        if !area.is_empty() {
            return format!("{}; {}", point, area);
        }
    }

    point.to_string()
}

/// Decode Pentax ImageSize from 1 or 2 values
/// Reference: Pentax.pm tag 0x0009
pub fn decode_image_size(values: &[u16]) -> Option<String> {
    if values.is_empty() {
        return None;
    }

    // Try compound key first (2 values)
    if values.len() >= 2 {
        match (values[0], values[1]) {
            (0, 0) => return Some("2304x1728".to_string()),
            (4, 0) => return Some("1600x1200".to_string()),
            (5, 0) => return Some("2048x1536".to_string()),
            (8, 0) => return Some("2560x1920".to_string()),
            (32, 2) => return Some("960x640".to_string()),
            (33, 2) => return Some("1152x768".to_string()),
            (34, 2) => return Some("1536x1024".to_string()),
            (35, 1) => return Some("2400x1600".to_string()),
            (36, 0) => return Some("3008x2008 or 3040x2024".to_string()),
            (37, 0) => return Some("3008x2000".to_string()),
            _ => {}
        }
    }

    // Single value lookup
    let v = values[0];
    let size = match v {
        0 => "640x480",
        1 => "Full",
        2 => "1024x768",
        3 => "1280x960",
        4 => "1600x1200",
        5 => "2048x1536",
        8 => "2560x1920 or 2304x1728",
        9 => "3072x2304",
        10 => "3264x2448",
        19 => "320x240",
        20 => "2288x1712",
        21 => "2592x1944",
        22 => "2304x1728 or 2592x1944",
        23 => "3056x2296",
        25 => "2816x2212 or 2816x2112",
        27 => "3648x2736",
        29 => "4000x3000",
        30 => "4288x3216",
        31 => "4608x3456",
        129 => "1920x1080",
        135 => "4608x2592",
        257 => "3216x3216",
        _ => return None,
    };

    Some(size.to_string())
}

/// Decrypt Pentax ShutterCount using Date and Time values
/// Algorithm from ExifTool Pentax.pm CryptShutterCount
/// The encryption is: value XOR date XOR (0xFFFFFFFF - time)
fn decrypt_shutter_count(encrypted: &[u8], date_bytes: &[u8], time_bytes: &[u8]) -> Option<u32> {
    // Need exactly 4 bytes for encrypted value
    if encrypted.len() != 4 {
        return None;
    }
    // Need at least 4 bytes for date
    if date_bytes.len() < 4 {
        return None;
    }
    // Need at least 3 bytes for time (padded to 4 with null)
    if time_bytes.len() < 3 {
        return None;
    }

    // Get encrypted value as big-endian u32
    let val = u32::from_be_bytes([encrypted[0], encrypted[1], encrypted[2], encrypted[3]]);

    // Get date as big-endian u32
    let date = u32::from_be_bytes([date_bytes[0], date_bytes[1], date_bytes[2], date_bytes[3]]);

    // Get time as big-endian u32 (pad with null byte if only 3 bytes)
    let time = if time_bytes.len() >= 4 {
        u32::from_be_bytes([time_bytes[0], time_bytes[1], time_bytes[2], time_bytes[3]])
    } else {
        u32::from_be_bytes([time_bytes[0], time_bytes[1], time_bytes[2], 0])
    };

    // Decrypt: val XOR date XOR (0xFFFFFFFF - time)
    let decrypted = val ^ date ^ (0xFFFFFFFF_u32.wrapping_sub(time));
    Some(decrypted)
}

/// Parse Pentax maker notes
///
/// # Arguments
/// * `data` - The maker note data (contents of MakerNote tag)
/// * `endian` - Default byte order
/// * `tiff_data` - Optional full TIFF/EXIF data for resolving TIFF-relative offsets
/// * `tiff_offset` - Offset of TIFF header within the full data
pub fn parse_pentax_maker_notes(
    data: &[u8],
    endian: Endianness,
    tiff_data: Option<&[u8]>,
    tiff_offset: usize,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    // Store raw Date and Time bytes for ShutterCount decryption
    let mut raw_date_bytes: Option<Vec<u8>> = None;
    let mut raw_time_bytes: Option<Vec<u8>> = None;
    let mut raw_shutter_count_bytes: Option<Vec<u8>> = None;

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
                // Value at offset - for Pentax AOC format, offsets are TIFF-relative
                // First try reading from tiff_data if available
                let value_bytes_opt: Option<Vec<u8>> = if let Some(tiff) = tiff_data {
                    // Value offset is relative to TIFF header
                    let tiff_off = tiff_offset + value_offset as usize;
                    if tiff_off + value_size <= tiff.len() {
                        Some(tiff[tiff_off..tiff_off + value_size].to_vec())
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Fall back to reading from MakerNote data (relative to base_offset)
                if let Some(bytes) = value_bytes_opt {
                    bytes
                } else {
                    let abs_offset = base_offset + value_offset as usize;
                    if abs_offset + value_size <= data.len() {
                        data[abs_offset..abs_offset + value_size].to_vec()
                    } else {
                        continue;
                    }
                }
            };

            // Parse the value based on type
            let value = match tag_type {
                1 => {
                    // BYTE
                    let bytes = value_bytes[..(count as usize).min(value_bytes.len())].to_vec();

                    // Decode PictureMode (0x0033) which is 3 bytes
                    if tag_id == PENTAX_PICTURE_MODE && bytes.len() >= 2 {
                        ExifValue::Ascii(decode_picture_mode_multi(&bytes))
                    } else if tag_id == PENTAX_LENS_TYPE && bytes.len() >= 2 {
                        // LensType is encoded as series + model
                        if let Some(lens_name) = get_pentax_lens_name(bytes[0], bytes[1]) {
                            ExifValue::Ascii(lens_name.to_string())
                        } else {
                            ExifValue::Byte(bytes)
                        }
                    } else if tag_id == PENTAX_DRIVE_MODE && bytes.len() >= 2 {
                        // DriveMode can be stored as bytes (4 bytes typically)
                        ExifValue::Ascii(decode_drive_mode_from_bytes(&bytes))
                    } else {
                        ExifValue::Byte(bytes)
                    }
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
                    // First handle multi-value tags
                    let decoded_multi: Option<String> = match tag_id {
                        PENTAX_DRIVE_MODE if values.len() > 1 => {
                            Some(decode_drive_mode_multi(&values))
                        }
                        PENTAX_AF_POINT_SELECTED => Some(decode_af_point_selected_multi(&values)),
                        PENTAX_AUTO_BRACKETING if !values.is_empty() => {
                            Some(decode_auto_bracketing(&values))
                        }
                        PENTAX_RAW_IMAGE_SIZE if values.len() == 2 => {
                            // Format as WIDTHxHEIGHT
                            Some(format!("{}x{}", values[0], values[1]))
                        }
                        PENTAX_SENSOR_SIZE if values.len() == 2 => {
                            // Values are in 1/1000 mm, format as "WIDTH x HEIGHT mm"
                            let width_mm = values[0] as f64 / 500.0;
                            let height_mm = values[1] as f64 / 500.0;
                            Some(format!("{:.3} x {:.3} mm", width_mm, height_mm))
                        }
                        PENTAX_IMAGE_SIZE => decode_image_size(&values),
                        _ => None,
                    };

                    if let Some(s) = decoded_multi {
                        ExifValue::Ascii(s)
                    } else if values.len() == 1 {
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
                            PENTAX_WORLD_TIME_LOCATION => {
                                Some(decode_world_time_location(v).to_string())
                            }
                            PENTAX_HOMETOWN_DST | PENTAX_DESTINATION_DST => {
                                Some(decode_dst(v).to_string())
                            }
                            PENTAX_HOMETOWN_CITY | PENTAX_DESTINATION_CITY => {
                                decode_city(v as u32).map(|s| s.to_string())
                            }
                            PENTAX_WHITE_BALANCE_MODE => {
                                decode_white_balance_mode(v).map(|s| s.to_string())
                            }
                            // EffectiveLV: ValueConv => '$val/1024' per Pentax.pm
                            PENTAX_EFFECTIVE_LV => {
                                // Treat as signed int16 for negative values
                                let signed_val = v as i16;
                                let lv = signed_val as f64 / 1024.0;
                                Some(format!("{:.1}", lv))
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

                    // Decode PentaxModelID and city codes
                    if tag_id == PENTAX_MODEL_ID && values.len() == 1 {
                        if let Some(model_name) = get_pentax_model_name(values[0]) {
                            ExifValue::Ascii(model_name.to_string())
                        } else {
                            ExifValue::Long(values)
                        }
                    } else if (tag_id == PENTAX_HOMETOWN_CITY || tag_id == PENTAX_DESTINATION_CITY)
                        && values.len() == 1
                    {
                        if let Some(city_name) = decode_city(values[0]) {
                            ExifValue::Ascii(city_name.to_string())
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
                    // Decode firmware versions with XOR decryption
                    if (tag_id == PENTAX_DSP_FIRMWARE_VERSION
                        || tag_id == PENTAX_CPU_FIRMWARE_VERSION)
                        && value_bytes.len() >= 4
                    {
                        ExifValue::Ascii(decode_firmware_version(&value_bytes))
                    } else if tag_id == PENTAX_DATE && value_bytes.len() >= 4 {
                        // Store raw bytes for ShutterCount decryption
                        raw_date_bytes = Some(value_bytes.clone());
                        // Date is stored as big-endian year (2 bytes) + month + day
                        if let Some(date_str) = decode_date_from_bytes(&value_bytes) {
                            ExifValue::Ascii(date_str)
                        } else {
                            ExifValue::Undefined(value_bytes)
                        }
                    } else if tag_id == PENTAX_TIME && value_bytes.len() >= 3 {
                        // Store raw bytes for ShutterCount decryption
                        raw_time_bytes = Some(value_bytes.clone());
                        // Time is stored as hour + minute + second
                        if let Some(time_str) = decode_time_from_bytes(&value_bytes) {
                            ExifValue::Ascii(time_str)
                        } else {
                            ExifValue::Undefined(value_bytes)
                        }
                    } else if tag_id == PENTAX_SHUTTER_COUNT && value_bytes.len() == 4 {
                        // Store for later decryption (needs Date and Time)
                        raw_shutter_count_bytes = Some(value_bytes.clone());
                        // Temporarily store as undefined, will be decrypted later
                        ExifValue::Undefined(value_bytes)
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
                MakerNoteTag::new(tag_id, get_pentax_tag_name(tag_id), value),
            );
        }
    }

    // Post-process: Decrypt ShutterCount if we have Date and Time
    if let (Some(shutter_bytes), Some(date_bytes), Some(time_bytes)) =
        (&raw_shutter_count_bytes, &raw_date_bytes, &raw_time_bytes)
    {
        if let Some(decrypted) = decrypt_shutter_count(shutter_bytes, date_bytes, time_bytes) {
            tags.insert(
                PENTAX_SHUTTER_COUNT,
                MakerNoteTag::new(
                    PENTAX_SHUTTER_COUNT,
                    Some("ShutterCount"),
                    ExifValue::Ascii(decrypted.to_string()),
                ),
            );
        }
    }

    Ok(tags)
}
