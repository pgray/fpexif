// makernotes/panasonic.rs - Panasonic/Lumix maker notes parsing

use crate::data_types::{Endianness, ExifValue};
use crate::define_tag_decoder;
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Cursor;

// exiv2 group names
pub const EXIV2_GROUP_PANASONIC: &str = "Panasonic";
pub const EXIV2_GROUP_PANASONIC_RAW: &str = "PanasonicRaw";

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
pub const PANA_IMAGE_WIDTH: u16 = 0x004B;
pub const PANA_IMAGE_HEIGHT: u16 = 0x004C;
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
pub const PANA_LANDMARK: u16 = 0x006F;
pub const PANA_INTELLIGENT_RESOLUTION: u16 = 0x0070;
pub const PANA_CLEAR_RETOUCH: u16 = 0x007C;
pub const PANA_PHOTO_STYLE: u16 = 0x0089;
pub const PANA_HDR_SHOT: u16 = 0x0093;
pub const PANA_SHUTTER_TYPE: u16 = 0x009F;
pub const PANA_FLASH_CURTAIN: u16 = 0x0048;
pub const PANA_TOUCH_AE: u16 = 0x00AE;
pub const PANA_SATURATION: u16 = 0x0040;
pub const PANA_SHARPNESS: u16 = 0x0041;
pub const PANA_FILM_MODE: u16 = 0x0042;
pub const PANA_JPEG_QUALITY: u16 = 0x0043;
pub const PANA_COLOR_TEMP_KELVIN: u16 = 0x0044;
pub const PANA_BRACKET_SETTINGS: u16 = 0x0045;
pub const PANA_WB_SHIFT_AB: u16 = 0x0046;
pub const PANA_WB_SHIFT_GM: u16 = 0x0047;
pub const PANA_LONG_EXPOSURE_NR: u16 = 0x0049;
pub const PANA_AF_POINT_POSITION: u16 = 0x004D;
pub const PANA_FACE_DETECT_INFO: u16 = 0x004E;
pub const PANA_WHITE_BALANCE_BIAS: u16 = 0x0023;
pub const PANA_WB_RED_LEVEL: u16 = 0x8004;
pub const PANA_WB_GREEN_LEVEL: u16 = 0x8005;
pub const PANA_WB_BLUE_LEVEL: u16 = 0x8006;
pub const PANA_SCENE_MODE: u16 = 0x8001;
pub const PANA_VIDEO_FRAME_RATE: u16 = 0x0027;
pub const PANA_MAKERNOTE_VERSION: u16 = 0x8000;
pub const PANA_HIGHLIGHT_WARNING: u16 = 0x8002;
pub const PANA_DARK_FOCUS_ENVIRONMENT: u16 = 0x8003;
pub const PANA_TEXT_STAMP_3: u16 = 0x8008;
pub const PANA_TEXT_STAMP_4: u16 = 0x8009;
pub const PANA_BABY_AGE_2: u16 = 0x8010;
pub const PANA_LENS_FIRMWARE_VERSION: u16 = 0x0060;
pub const PANA_TITLE: u16 = 0x0065;
pub const PANA_BABY_NAME: u16 = 0x0066;
pub const PANA_LOCATION: u16 = 0x0067;
pub const PANA_MERGED_IMAGES: u16 = 0x0076;
pub const PANA_BURST_SPEED: u16 = 0x0077;
pub const PANA_INTELLIGENT_D_RANGE: u16 = 0x0079;
pub const PANA_INTERNAL_ND_FILTER: u16 = 0x009D;

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
        PANA_EXIF_VERSION => Some("PanasonicExifVersion"),
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
        PANA_ADVANCED_SCENE_MODE => Some("AdvancedSceneType"),
        PANA_TEXT_STAMP_2 => Some("TextStamp2"),
        PANA_FACE_DETECTED => Some("FacesDetected"),
        PANA_IMAGE_WIDTH => Some("PanasonicImageWidth"),
        PANA_IMAGE_HEIGHT => Some("PanasonicImageHeight"),
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
        PANA_SATURATION => Some("Saturation"),
        PANA_SHARPNESS => Some("Sharpness"),
        PANA_FILM_MODE => Some("FilmMode"),
        PANA_JPEG_QUALITY => Some("JPEGQuality"),
        PANA_COLOR_TEMP_KELVIN => Some("ColorTempKelvin"),
        PANA_BRACKET_SETTINGS => Some("BracketSettings"),
        PANA_WB_SHIFT_AB => Some("WBShiftAB"),
        PANA_WB_SHIFT_GM => Some("WBShiftGM"),
        PANA_LONG_EXPOSURE_NR => Some("LongExposureNoiseReduction"),
        PANA_AF_POINT_POSITION => Some("AFPointPosition"),
        PANA_FACE_DETECT_INFO => Some("FaceDetInfo"),
        PANA_WHITE_BALANCE_BIAS => Some("WhiteBalanceBias"),
        PANA_WB_RED_LEVEL => Some("WBRedLevel"),
        PANA_WB_GREEN_LEVEL => Some("WBGreenLevel"),
        PANA_WB_BLUE_LEVEL => Some("WBBlueLevel"),
        PANA_SCENE_MODE => Some("SceneMode"),
        PANA_VIDEO_FRAME_RATE => Some("VideoFrameRate"),
        PANA_MAKERNOTE_VERSION => Some("MakerNoteVersion"),
        PANA_HIGHLIGHT_WARNING => Some("HighlightWarning"),
        PANA_DARK_FOCUS_ENVIRONMENT => Some("DarkFocusEnvironment"),
        PANA_TEXT_STAMP_3 => Some("TextStamp"),
        PANA_TEXT_STAMP_4 => Some("TextStamp"),
        PANA_BABY_AGE_2 => Some("BabyAge"),
        PANA_LENS_FIRMWARE_VERSION => Some("LensFirmwareVersion"),
        PANA_TITLE => Some("Title"),
        PANA_BABY_NAME => Some("BabyName"),
        PANA_LOCATION => Some("Location"),
        PANA_MERGED_IMAGES => Some("MergedImages"),
        PANA_BURST_SPEED => Some("BurstSpeed"),
        PANA_INTELLIGENT_D_RANGE => Some("IntelligentD-Range"),
        PANA_INTERNAL_ND_FILTER => Some("InternalNDFilter"),
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
        11 => "Manual 2",
        12 => "Shade",
        13 => "Kelvin",
        14 => "Manual 3",
        15 => "Manual 4",
        19 => "Auto (cool)",
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
        16 => "Self Portrait",
        17 => "Economy",
        18 => "Fireworks",
        19 => "Party",
        20 => "Snow",
        21 => "Night Scenery",
        22 => "Food",
        23 => "Baby",
        24 => "Soft Skin",
        25 => "Candlelight",
        26 => "Starry Night",
        27 => "High Sensitivity",
        28 => "Panorama Assist",
        29 => "Underwater",
        30 => "Beach",
        31 => "Aerial Photo",
        32 => "Sunset",
        33 => "Pet",
        34 => "Intelligent ISO",
        35 => "Clipboard",
        36 => "High Speed Continuous Shooting",
        37 => "Intelligent Auto",
        39 => "Multi-aspect",
        41 => "Transform",
        42 => "Flash Burst",
        43 => "Pin Hole",
        44 => "Film Grain",
        45 => "My Color",
        46 => "Photo Frame",
        48 => "Movie",
        51 => "HDR",
        52 => "Peripheral Defocus",
        55 => "Handheld Night Shot",
        57 => "3D",
        59 => "Creative Control",
        60 => "Intelligent Auto Plus",
        62 => "Panorama",
        63 => "Glass Through",
        64 => "HDR",
        66 => "Digital Filter",
        67 => "Clear Portrait",
        68 => "Silky Skin",
        69 => "Backlit Softness",
        70 => "Clear in Backlight",
        71 => "Relaxing Tone",
        72 => "Sweet Child's Face",
        73 => "Distinct Scenery",
        74 => "Bright Blue Sky",
        75 => "Romantic Sunset Glow",
        76 => "Vivid Sunset Glow",
        77 => "Glistening Water",
        78 => "Clear Nightscape",
        79 => "Cool Night Sky",
        80 => "Warm Glowing Nightscape",
        81 => "Artistic Nightscape",
        82 => "Glittering Illuminations",
        83 => "Clear Night Portrait",
        84 => "Soft Image of a Flower",
        85 => "Appetizing Food",
        86 => "Cute Dessert",
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
// Note: Uses different mappings for G2, GF1, GF2, GF3, GF5, GF6 models
define_tag_decoder! {
    contrast_mode,
    exiftool: {
        0 => "Normal",
        1 => "Low",
        2 => "High",
        5 => "Normal 2",
        6 => "Medium Low",
        7 => "Medium High",
        13 => "High Dynamic",
        24 => "Dynamic Range (film-like)",
        46 => "Match Filter Effects Toy",
        55 => "Match Photo Style L. Monochrome",
        256 => "Low",
        272 => "Standard",
        288 => "High",
    },
    exiv2: {
        0 => "Normal",
        1 => "Low",
        2 => "High",
        5 => "Normal 2",
        6 => "Medium low",
        7 => "Medium high",
        13 => "High Dynamic",
        24 => "Dynamic Range (film-like)",
        46 => "Match Filter Effects Toy",
        55 => "Match Photo Style L. Monochrome",
        256 => "Low",
        272 => "Standard",
        288 => "High",
    }
}

// ContrastMode for GF1, G2, GF2, GF3, GF5, GF6 models (different mapping)
define_tag_decoder! {
    contrast_mode_gf,
    exiftool: {
        0 => "-2",
        1 => "-1",
        2 => "Normal",
        3 => "+1",
        4 => "+2",
        5 => "Normal 2",
        7 => "Nature (Color Film)",
        9 => "Expressive",
        12 => "Smooth (Color Film) or Pure (My Color)",
        17 => "Dynamic (B&W Film)",
        22 => "Smooth (B&W Film)",
        25 => "High Dynamic",
        26 => "Retro",
        27 => "Dynamic (Color Film)",
        28 => "Low Key",
        29 => "Toy Effect",
        32 => "Vibrant (Color Film) or Expressive (My Color)",
        33 => "Elegant (My Color)",
        37 => "Nostalgic (Color Film)",
        41 => "Dynamic Art (My Color)",
        42 => "Retro (My Color)",
        45 => "Cinema",
        47 => "Dynamic Mono",
        50 => "Impressive Art",
        51 => "Cross Process",
        100 => "High Dynamic 2",
        101 => "Retro 2",
    },
    exiv2: {
        0 => "-2",
        1 => "-1",
        2 => "Normal",
        3 => "+1",
        4 => "+2",
        5 => "Normal 2",
        7 => "Nature (Color Film)",
        9 => "Expressive",
        12 => "Smooth (Color Film) or Pure (My Color)",
        17 => "Dynamic (B&W Film)",
        22 => "Smooth (B&W Film)",
        25 => "High Dynamic",
        26 => "Retro",
        27 => "Dynamic (Color Film)",
        28 => "Low Key",
        29 => "Toy Effect",
        32 => "Vibrant (Color Film) or Expressive (My Color)",
        33 => "Elegant (My Color)",
        37 => "Nostalgic (Color Film)",
        41 => "Dynamic Art (My Color)",
        42 => "Retro (My Color)",
        45 => "Cinema",
        47 => "Dynamic Mono",
        50 => "Impressive Art",
        51 => "Cross Process",
        100 => "High Dynamic 2",
        101 => "Retro 2",
    }
}

/// Decode ContrastMode with model-specific handling
/// Based on ExifTool Panasonic.pm - some models output raw value, others decode
pub fn decode_contrast_mode_with_model(value: u16, model: Option<&str>) -> String {
    // ExifTool uses different handling based on model:
    // 1. GF series + G2: Uses -2,-1,Normal,... mapping
    // 2. G1, L1, L10, LC80, FX10, TZ10, ZS7, DC-*: Output raw value (no decoding)
    // 3. Other models: Use Normal,Low,High,... mapping

    let model_str = model.unwrap_or("");
    let model_upper = model_str.to_uppercase();

    // Check if this is a model that outputs raw values (no decoding)
    // NOTE: Use exact matching to avoid false positives (e.g., DMC-TZ10 vs DMC-TZ100)
    let is_raw_output_model = model_upper == "DMC-G1"
        || model_upper == "DMC-L1"
        || model_upper == "DMC-L10"
        || model_upper == "DMC-LC80"
        || model_upper == "DMC-FX10"
        || model_upper == "DMC-TZ10"
        || model_upper == "DMC-ZS7"
        || model_upper.starts_with("DC-");

    // Check if this is a GF/G2 series camera (DMC-GF1, DMC-GF2, etc. or DMC-G2)
    let is_gf_model = model_upper.starts_with("DMC-GF") || model_upper == "DMC-G2";

    if is_raw_output_model {
        // Output raw value like ExifTool
        value.to_string()
    } else if is_gf_model {
        decode_contrast_mode_gf_exiftool(value).to_string()
    } else {
        decode_contrast_mode_exiftool(value).to_string()
    }
}

// BurstMode (tag 0x002A): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    burst_mode,
    exiftool: {
        0 => "Off",
        1 => "On",
        2 => "Auto Exposure Bracketing (AEB)",
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

// ClearRetouch (tag 0x007C): Panasonic.pm / panasonicmn_int.cpp
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

// FlashCurtain (tag 0x0048): Panasonic.pm / panasonicmn_int.cpp
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

/// Decode AFAreaMode from 2-byte array (tag 0x000F)
/// Format is "byte0 byte1" -> description string
pub fn decode_af_area_mode_pair(byte0: u8, byte1: u8) -> &'static str {
    match (byte0, byte1) {
        (0, 1) => "9-area",
        (0, 16) => "3-area (high speed)",
        (0, 23) => "23-area",
        (0, 49) => "49-area",
        (0, 225) => "225-area",
        (1, 0) => "Spot Focusing",
        (1, 1) => "5-area",
        (16, 0) => "1-area",
        (16, 16) => "1-area (high speed)",
        (16, 32) => "1-area +",
        (17, 0) => "Full Area",
        (32, 0) => "Tracking",
        (32, 1) => "3-area (left)",
        (32, 2) => "3-area (center)",
        (32, 3) => "3-area (right)",
        (32, 16) => "Zone",
        (32, 18) => "Zone (horizontal/vertical)",
        (64, 0) => "Face Detect",
        (64, 1) => "Face Detect + 3-area (left)",
        (64, 2) => "Face Detect + 3-area (center)",
        (64, 3) => "Face Detect + 3-area (right)",
        (64, 4) => "Face Detect + 1-area",
        (128, 0) => "Pinpoint focus",
        (240, 0) => "Tracking",
        _ => "Unknown",
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

// FilmMode (tag 0x0042): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    film_mode,
    exiftool: {
        0 => "n/a",
        1 => "Standard (color)",
        2 => "Dynamic (color)",
        3 => "Nature (color)",
        4 => "Smooth (color)",
        5 => "Standard (B&W)",
        6 => "Dynamic (B&W)",
        7 => "Smooth (B&W)",
        10 => "Nostalgic",
        11 => "Vibrant",
    },
    exiv2: {
        1 => "Standard (color)",
        2 => "Dynamic (color)",
        3 => "Nature (color)",
        4 => "Smooth (color)",
        5 => "Standard (B&W)",
        6 => "Dynamic (B&W)",
        7 => "Smooth (B&W)",
        10 => "Nostalgic",
        11 => "Vibrant",
    }
}

// JPEGQuality (tag 0x0043): Panasonic.pm (exiv2 doesn't decode this)
define_tag_decoder! {
    jpeg_quality,
    both: {
        0 => "n/a (Movie)",
        2 => "High",
        3 => "Standard",
        6 => "Very High",
        255 => "n/a (RAW only)",
    }
}

// BracketSettings (tag 0x0045): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    bracket_settings,
    exiftool: {
        0 => "No Bracket",
        1 => "3 Images, Sequence 0/-/+",
        2 => "3 Images, Sequence -/0/+",
        3 => "5 Images, Sequence 0/-/+",
        4 => "5 Images, Sequence -/0/+",
        5 => "7 Images, Sequence 0/-/+",
        6 => "7 Images, Sequence -/0/+",
    },
    exiv2: {
        0 => "No Bracket",
        1 => "3 images, Sequence 0/-/+",
        2 => "3 images, Sequence -/0/+",
        3 => "5 images, Sequence 0/-/+",
        4 => "5 images, Sequence -/0/+",
        5 => "7 images, Sequence 0/-/+",
        6 => "7 images, Sequence -/0/+",
    }
}

// LongExposureNoiseReduction (tag 0x0049): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    long_exposure_nr,
    both: {
        1 => "Off",
        2 => "On",
    }
}

// HighlightWarning (tag 0x8002): Panasonic.pm
define_tag_decoder! {
    highlight_warning,
    both: {
        0 => "Disabled",
        1 => "No",
        2 => "Yes",
    }
}

// DarkFocusEnvironment (tag 0x8003): Panasonic.pm
define_tag_decoder! {
    dark_focus_environment,
    both: {
        1 => "No",
        2 => "Yes",
    }
}

// SceneMode (tag 0x8001): Panasonic.pm / panasonicmn_int.cpp (uses same values as ShootingMode)
define_tag_decoder! {
    scene_mode,
    exiftool: {
        0 => "Off",
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
        16 => "Self Portrait",
        17 => "Economy",
        18 => "Fireworks",
        19 => "Party",
        20 => "Snow",
        21 => "Night Scenery",
        22 => "Food",
        23 => "Baby",
        24 => "Soft Skin",
        25 => "Candlelight",
        26 => "Starry Night",
        27 => "High Sensitivity",
        28 => "Panorama Assist",
        29 => "Underwater",
        30 => "Beach",
        31 => "Aerial Photo",
        32 => "Sunset",
        33 => "Pet",
        34 => "Intelligent ISO",
        35 => "Clipboard",
        36 => "High Speed Continuous Shooting",
        37 => "Intelligent Auto",
        39 => "Multi-aspect",
        41 => "Transform",
        42 => "Flash Burst",
        43 => "Pin Hole",
        44 => "Film Grain",
        45 => "My Color",
        46 => "Photo Frame",
        48 => "Movie",
        51 => "HDR",
        52 => "Peripheral Defocus",
        55 => "Handheld Night Shot",
        57 => "3D",
        59 => "Creative Control",
        60 => "Intelligent Auto Plus",
        62 => "Panorama",
        63 => "Glass Through",
        64 => "HDR",
        66 => "Digital Filter",
        67 => "Clear Portrait",
        68 => "Silky Skin",
        69 => "Backlit Softness",
        70 => "Clear in Backlight",
        71 => "Relaxing Tone",
        72 => "Sweet Child's Face",
        73 => "Distinct Scenery",
        74 => "Bright Blue Sky",
        75 => "Romantic Sunset Glow",
        76 => "Vivid Sunset Glow",
        77 => "Glistening Water",
        78 => "Clear Nightscape",
        79 => "Cool Night Sky",
    },
    exiv2: {
        0 => "Off",
        1 => "Normal",
        2 => "Portrait",
        3 => "Scenery",
        4 => "Sports",
        5 => "Night portrait",
        6 => "Program",
        7 => "Aperture priority",
        8 => "Shutter-speed priority",
        9 => "Macro",
        10 => "Spot",
        11 => "Manual",
        12 => "Movie preview",
        13 => "Panning",
        14 => "Simple",
        15 => "Color effects",
        16 => "Self Portrait",
        17 => "Economy",
        18 => "Fireworks",
        19 => "Party",
        20 => "Snow",
        21 => "Night scenery",
        22 => "Food",
        23 => "Baby",
        24 => "Soft skin",
        25 => "Candlelight",
        26 => "Starry night",
        27 => "High sensitivity",
        28 => "Panorama assist",
        29 => "Underwater",
        30 => "Beach",
        31 => "Aerial photo",
        32 => "Sunset",
        33 => "Pet",
        34 => "Intelligent ISO",
        35 => "Clipboard",
        36 => "High speed continuous shooting",
        37 => "Intelligent auto",
        39 => "Multi-aspect",
        41 => "Transform",
        42 => "Flash Burst",
        43 => "Pin Hole",
        44 => "Film Grain",
        45 => "My Color",
        46 => "Photo Frame",
        51 => "HDR",
        55 => "Handheld Night Shot",
        57 => "3D",
    }
}

// IntelligentD-Range (tag 0x0079): Panasonic.pm / panasonicmn_int.cpp
define_tag_decoder! {
    intelligent_d_range,
    both: {
        0 => "Off",
        1 => "Low",
        2 => "Standard",
        3 => "High",
    }
}

/// Parse Panasonic maker notes
///
/// # Arguments
/// * `data` - The maker note data (contents of MakerNote tag)
/// * `endian` - Byte order
/// * `model` - Camera model string (used for model-specific formatting)
/// * `tiff_data` - Optional full TIFF/EXIF data for resolving absolute offsets
/// * `tiff_offset` - Offset of TIFF header within the full data
pub fn parse_panasonic_maker_notes(
    data: &[u8],
    endian: Endianness,
    model: Option<&str>,
    tiff_data: Option<&[u8]>,
    tiff_offset: usize,
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
                    } else if count == 2 && bytes.len() >= 2 {
                        // Handle 2-byte tags like AFAreaMode
                        if tag_id == PANA_AF_AREA_MODE {
                            ExifValue::Ascii(
                                decode_af_area_mode_pair(bytes[0], bytes[1]).to_string(),
                            )
                        } else {
                            ExifValue::Byte(bytes)
                        }
                    } else if tag_id == PANA_CITY || tag_id == PANA_LANDMARK {
                        // City/Landmark: string fields, show empty for invalid data (like ExifTool)
                        let trimmed: Vec<u8> =
                            bytes.iter().copied().take_while(|&b| b != 0).collect();
                        if trimmed.is_empty() || trimmed.iter().all(|&b| b == 0) {
                            ExifValue::Ascii(String::new())
                        } else if let Ok(s) = std::str::from_utf8(&trimmed) {
                            ExifValue::Ascii(s.to_string())
                        } else {
                            // Invalid UTF-8 - treat as empty
                            ExifValue::Ascii(String::new())
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

                        // Special handling for BabyAge
                        if (tag_id == PANA_BABY_AGE || tag_id == PANA_BABY_AGE_2)
                            && (s == "9999:99:99 00:00:00" || s.is_empty())
                        {
                            ExifValue::Ascii("(not set)".to_string())
                        } else {
                            ExifValue::Ascii(s)
                        }
                    } else {
                        // Panasonic uses offsets relative to TIFF header
                        let abs_offset = tiff_offset + value_offset as usize;
                        let string_data = if let Some(tiff) = tiff_data {
                            if abs_offset + count as usize <= tiff.len() {
                                Some(&tiff[abs_offset..abs_offset + count as usize])
                            } else {
                                None
                            }
                        } else {
                            // Fallback to maker note data
                            let offset = value_offset as usize;
                            if offset + count as usize <= data.len() {
                                Some(&data[offset..offset + count as usize])
                            } else {
                                None
                            }
                        };

                        if let Some(bytes) = string_data {
                            let s = String::from_utf8_lossy(bytes)
                                .trim_end_matches('\0')
                                .to_string();

                            // Special handling for BabyAge
                            if (tag_id == PANA_BABY_AGE || tag_id == PANA_BABY_AGE_2)
                                && (s == "9999:99:99 00:00:00" || s.is_empty())
                            {
                                ExifValue::Ascii("(not set)".to_string())
                            } else {
                                ExifValue::Ascii(s)
                            }
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
                                // Model-specific decoding for GF/G2 series
                                let decoded = decode_contrast_mode_with_model(v, model);
                                if decoded == "Unknown" {
                                    // ExifTool has PrintHex flag - show hex for unknown values
                                    Some(format!("Unknown (0x{:x})", v))
                                } else {
                                    Some(decoded)
                                }
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
                            PANA_PROGRAM_ISO => {
                                // Special values for ProgramISO
                                if v == 65534 {
                                    Some("Intelligent ISO".to_string())
                                } else if v == 65535 {
                                    Some("n/a".to_string())
                                } else {
                                    None // Pass through raw value
                                }
                            }
                            PANA_TRAVEL_DAY => {
                                // 65535 means "n/a" for TravelDay
                                if v == 65535 {
                                    Some("n/a".to_string())
                                } else {
                                    None
                                }
                            }
                            PANA_FILM_MODE => Some(decode_film_mode_exiftool(v).to_string()),
                            PANA_JPEG_QUALITY => Some(decode_jpeg_quality_exiftool(v).to_string()),
                            PANA_BRACKET_SETTINGS => {
                                Some(decode_bracket_settings_exiftool(v).to_string())
                            }
                            PANA_LONG_EXPOSURE_NR => {
                                Some(decode_long_exposure_nr_exiftool(v).to_string())
                            }
                            PANA_SCENE_MODE => Some(decode_scene_mode_exiftool(v).to_string()),
                            PANA_VIDEO_FRAME_RATE => {
                                if v == 0 {
                                    Some("n/a".to_string())
                                } else {
                                    None
                                }
                            }
                            PANA_HIGHLIGHT_WARNING => {
                                Some(decode_highlight_warning_exiftool(v).to_string())
                            }
                            PANA_DARK_FOCUS_ENVIRONMENT => {
                                Some(decode_dark_focus_environment_exiftool(v).to_string())
                            }
                            PANA_INTELLIGENT_D_RANGE => {
                                Some(decode_intelligent_d_range_exiftool(v).to_string())
                            }
                            PANA_BURST_SPEED => {
                                // BurstSpeed is images per second, raw value
                                None
                            }
                            PANA_FACE_DETECTED => {
                                // Output raw numeric value to match exiftool -json
                                // (exiftool sometimes outputs "No"/"Yes" but usually outputs numeric)
                                None // Pass through raw value
                            }
                            PANA_TEXT_STAMP_3 | PANA_TEXT_STAMP_4 => {
                                // Additional TextStamp tags with same values
                                Some(decode_text_stamp_exiftool(v).to_string())
                            }
                            PANA_SATURATION | PANA_SHARPNESS | PANA_CONTRAST => {
                                // These are signed values that use printParameter formatting
                                // Convert to signed and format as +/- values
                                let signed_v = v as i16;
                                if signed_v == 0 {
                                    Some("Normal".to_string())
                                } else if signed_v > 0 {
                                    Some(format!("+{}", signed_v))
                                } else {
                                    Some(format!("{}", signed_v))
                                }
                            }
                            PANA_WB_SHIFT_AB | PANA_WB_SHIFT_GM => {
                                // These are signed shift values
                                let signed_v = v as i16;
                                Some(format!("{}", signed_v))
                            }
                            PANA_WHITE_BALANCE_BIAS => {
                                // WhiteBalanceBias is signed, divided by 3
                                let signed_v = v as i16;
                                let bias = signed_v as f64 / 3.0;
                                // Format as fraction
                                if bias.fract() == 0.0 {
                                    if bias > 0.0 {
                                        Some(format!("+{}", bias as i32))
                                    } else {
                                        Some(format!("{}", bias as i32))
                                    }
                                } else if bias > 0.0 {
                                    Some(format!("+{:.2}", bias))
                                } else {
                                    Some(format!("{:.2}", bias))
                                }
                            }
                            PANA_FLASH_BIAS => {
                                // FlashBias is in 1/3 EV steps
                                // Raw value -1 → "-1/3", raw value 3 → "1", raw value 0 → "0"
                                let signed_v = v as i16;
                                if signed_v == 0 {
                                    Some("0".to_string())
                                } else if signed_v % 3 == 0 {
                                    // Whole EV value
                                    let ev = signed_v / 3;
                                    if ev > 0 {
                                        Some(format!("+{}", ev))
                                    } else {
                                        Some(format!("{}", ev))
                                    }
                                } else {
                                    // Fractional value - output as "n/3"
                                    if signed_v > 0 {
                                        Some(format!("+{}/3", signed_v))
                                    } else {
                                        Some(format!("{}/3", signed_v))
                                    }
                                }
                            }
                            PANA_ACCELEROMETER_X | PANA_ACCELEROMETER_Y | PANA_ACCELEROMETER_Z => {
                                // Accelerometer values are signed
                                let signed_v = v as i16;
                                Some(format!("{}", signed_v))
                            }
                            PANA_ROLL_ANGLE => {
                                // RollAngle: signed value divided by 10 for degrees
                                let signed_v = v as i16;
                                let degrees = signed_v as f64 / 10.0;
                                // Format without decimal if value is integer
                                if degrees.fract() == 0.0 {
                                    Some(format!("{}", degrees as i32))
                                } else {
                                    Some(format!("{:.1}", degrees))
                                }
                            }
                            PANA_PITCH_ANGLE => {
                                // PitchAngle: negative of signed value divided by 10
                                let signed_v = v as i16;
                                let degrees = -(signed_v as f64) / 10.0;
                                // Format without decimal if value is integer
                                if degrees.fract() == 0.0 {
                                    Some(format!("{}", degrees as i32))
                                } else {
                                    Some(format!("{:.1}", degrees))
                                }
                            }
                            PANA_WB_RED_LEVEL | PANA_WB_GREEN_LEVEL | PANA_WB_BLUE_LEVEL => {
                                // WB levels are stored as x*4, divide by 4 for ExifTool compatibility
                                Some((v / 4).to_string())
                            }
                            PANA_LANDMARK => {
                                // Landmark: 0 means empty/not set
                                if v == 0 {
                                    Some(String::new())
                                } else {
                                    None
                                }
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
                        // Check for special decoding
                        if tag_id == PANA_TIME_SINCE_POWER_ON {
                            // Format as "[DD days ]HH:MM:SS.ss"
                            let centiseconds = value_offset;
                            let total_seconds = centiseconds as f64 / 100.0;

                            let mut val = total_seconds;
                            let mut prefix = String::new();

                            // Handle days if >= 24 hours
                            if val >= 24.0 * 3600.0 {
                                let days = (val / (24.0 * 3600.0)) as u32;
                                prefix = format!("{} days ", days);
                                val -= days as f64 * 24.0 * 3600.0;
                            }

                            let hours = (val / 3600.0) as u32;
                            val -= hours as f64 * 3600.0;
                            let minutes = (val / 60.0) as u32;
                            val -= minutes as f64 * 60.0;

                            let formatted =
                                format!("{}{:02}:{:02}:{:05.2}", prefix, hours, minutes, val);
                            ExifValue::Ascii(formatted)
                        } else if tag_id == PANA_LANDMARK && value_offset == 0 {
                            // Landmark: 0 means empty/not set
                            ExifValue::Ascii(String::new())
                        } else {
                            ExifValue::Long(vec![value_offset])
                        }
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
                    // Panasonic offsets are relative to TIFF header, not maker note start
                    let abs_offset = tiff_offset + value_offset as usize;
                    let data_slice = if let Some(tiff) = tiff_data {
                        if abs_offset + value_size <= tiff.len() {
                            Some(&tiff[abs_offset..abs_offset + value_size])
                        } else {
                            None
                        }
                    } else {
                        // Fallback to maker note data for older behavior
                        let offset = value_offset as usize;
                        if offset + value_size <= data.len() {
                            Some(&data[offset..offset + value_size])
                        } else {
                            None
                        }
                    };

                    if let Some(slice) = data_slice {
                        let mut values = Vec::new();
                        let mut cursor = Cursor::new(slice);
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

                        // Special formatting for InternalNDFilter
                        if tag_id == PANA_INTERNAL_ND_FILTER && values.len() == 1 {
                            let (num, den) = values[0];
                            if den != 0 {
                                let nd_value = num as f64 / den as f64;
                                // Format as decimal
                                ExifValue::Ascii(format!("{}", nd_value))
                            } else {
                                ExifValue::Rational(values)
                            }
                        }
                        // Special formatting for AFPointPosition
                        else if tag_id == PANA_AF_POINT_POSITION && values.len() == 2 {
                            let (num1, den1) = values[0];
                            let (num2, den2) = values[1];

                            // Check for special "none" value (0xFFFFFFFF/256 0xFFFFFFFF/256)
                            // Raw numerators are 4294967295 (0xFFFFFFFF), which divided by 256
                            // gives approximately 16777216
                            if num1 == 0xFFFFFFFF
                                && den1 == 256
                                && num2 == 0xFFFFFFFF
                                && den2 == 256
                            {
                                ExifValue::Ascii("none".to_string())
                            }
                            // Check for "n/a" value (4294967295/1024)
                            else if num1 == 0xFFFFFFFF && den1 == 1024 {
                                ExifValue::Ascii("n/a".to_string())
                            } else {
                                // Format as "X Y" with up to 2 decimal places, trim trailing zeros
                                let x = if den1 != 0 {
                                    num1 as f64 / den1 as f64
                                } else {
                                    0.0
                                };
                                let y = if den2 != 0 {
                                    num2 as f64 / den2 as f64
                                } else {
                                    0.0
                                };
                                // Format and trim trailing zeros like ExifTool
                                let x_str = format!("{:.2}", x)
                                    .trim_end_matches('0')
                                    .trim_end_matches('.')
                                    .to_string();
                                let y_str = format!("{:.2}", y)
                                    .trim_end_matches('0')
                                    .trim_end_matches('.')
                                    .to_string();
                                ExifValue::Ascii(format!("{} {}", x_str, y_str))
                            }
                        } else {
                            ExifValue::Rational(values)
                        }
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
                    if count <= 4 {
                        // Inline value for small undefined
                        let bytes = match endian {
                            Endianness::Little => value_offset.to_le_bytes(),
                            Endianness::Big => value_offset.to_be_bytes(),
                        };
                        // Special handling for FirmwareVersion (4 bytes -> "x.x.x.x")
                        if tag_id == PANA_FIRMWARE_VERSION && count == 4 {
                            let version_str =
                                format!("{}.{}.{}.{}", bytes[0], bytes[1], bytes[2], bytes[3]);
                            ExifValue::Ascii(version_str)
                        } else if tag_id == PANA_LENS_FIRMWARE_VERSION && count == 4 {
                            // LensFirmwareVersion: 4 bytes formatted as "x.x.x.x" (same as FirmwareVersion)
                            let version_str =
                                format!("{}.{}.{}.{}", bytes[0], bytes[1], bytes[2], bytes[3]);
                            ExifValue::Ascii(version_str)
                        } else if tag_id == PANA_MAKERNOTE_VERSION && count == 4 {
                            // MakerNoteVersion: ASCII bytes "0130" -> "0130"
                            if let Ok(s) = std::str::from_utf8(&bytes[..4]) {
                                ExifValue::Ascii(s.to_string())
                            } else {
                                ExifValue::Undefined(bytes[..4].to_vec())
                            }
                        } else if tag_id == PANA_EXIF_VERSION && count == 4 {
                            // PanasonicExifVersion: ASCII bytes "0330" -> "0330"
                            if let Ok(s) = std::str::from_utf8(&bytes[..4]) {
                                ExifValue::Ascii(s.to_string())
                            } else {
                                ExifValue::Undefined(bytes[..4].to_vec())
                            }
                        } else {
                            ExifValue::Undefined(bytes[..count as usize].to_vec())
                        }
                    } else {
                        // Panasonic offsets are relative to TIFF header, not maker note start
                        let abs_offset = tiff_offset + value_offset as usize;
                        let value_bytes = if let Some(tiff) = tiff_data {
                            if abs_offset + count as usize <= tiff.len() {
                                Some(&tiff[abs_offset..abs_offset + count as usize])
                            } else {
                                None
                            }
                        } else {
                            // Fallback to maker note data
                            let offset = value_offset as usize;
                            if offset + count as usize <= data.len() {
                                Some(&data[offset..offset + count as usize])
                            } else {
                                None
                            }
                        };

                        if let Some(value_bytes) = value_bytes {
                            // MakerNoteVersion: ASCII bytes -> string
                            if tag_id == PANA_MAKERNOTE_VERSION && count == 4 {
                                if let Ok(s) = std::str::from_utf8(value_bytes) {
                                    ExifValue::Ascii(s.to_string())
                                } else {
                                    ExifValue::Undefined(value_bytes.to_vec())
                                }
                            } else if tag_id == PANA_EXIF_VERSION && count == 4 {
                                // PanasonicExifVersion: ASCII bytes -> string
                                if let Ok(s) = std::str::from_utf8(value_bytes) {
                                    ExifValue::Ascii(s.to_string())
                                } else {
                                    ExifValue::Undefined(value_bytes.to_vec())
                                }
                            } else if tag_id == PANA_LANDMARK
                                || tag_id == PANA_CITY
                                || tag_id == PANA_TITLE
                                || tag_id == PANA_BABY_NAME
                                || tag_id == PANA_LOCATION
                            {
                                // String fields that show empty when all zeros or invalid
                                // ExifTool returns empty for these when they don't contain valid text
                                let trimmed: Vec<u8> = value_bytes
                                    .iter()
                                    .copied()
                                    .take_while(|&b| b != 0)
                                    .collect();
                                if trimmed.is_empty() || trimmed.iter().all(|&b| b == 0) {
                                    ExifValue::Ascii(String::new())
                                } else if let Ok(s) = std::str::from_utf8(&trimmed) {
                                    ExifValue::Ascii(s.to_string())
                                } else {
                                    // Invalid UTF-8 - treat as empty (like ExifTool)
                                    ExifValue::Ascii(String::new())
                                }
                            } else if tag_id == PANA_INTERNAL_SERIAL_NUMBER {
                                // InternalSerialNumber: decode factory code, date, and serial number
                                // Format: XYZ14100603820 -> "(XYZ) 2014:10:06 no. 0382"
                                // Data may have trailing garbage after null byte, so extract before null
                                let null_pos = value_bytes.iter().position(|&b| b == 0);
                                let serial_bytes = match null_pos {
                                    Some(pos) => &value_bytes[..pos],
                                    None => value_bytes,
                                };
                                if let Ok(s) = std::str::from_utf8(serial_bytes) {
                                    // Pattern: [A-Z][0-9A-Z]{2}(\d{2})(\d{2})(\d{2})(\d{4})
                                    if s.len() >= 13 {
                                        let factory = &s[0..3];
                                        if let (Ok(yr), Ok(mon), Ok(day), Ok(num)) = (
                                            s[3..5].parse::<u32>(),
                                            s[5..7].parse::<u32>(),
                                            s[7..9].parse::<u32>(),
                                            s[9..13].parse::<u32>(),
                                        ) {
                                            let year = if yr < 70 { 2000 + yr } else { 1900 + yr };
                                            ExifValue::Ascii(format!(
                                                "({}) {:04}:{:02}:{:02} no. {:04}",
                                                factory, year, mon, day, num
                                            ))
                                        } else {
                                            ExifValue::Ascii(s.to_string())
                                        }
                                    } else {
                                        ExifValue::Ascii(s.to_string())
                                    }
                                } else {
                                    ExifValue::Undefined(value_bytes.to_vec())
                                }
                            } else {
                                ExifValue::Undefined(value_bytes.to_vec())
                            }
                        } else {
                            continue;
                        }
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

            let tag_name = get_panasonic_tag_name(tag_id);
            let tag = if let Some(name) = tag_name {
                MakerNoteTag::with_exiv2(
                    tag_id,
                    tag_name,
                    value.clone(),
                    value,
                    EXIV2_GROUP_PANASONIC,
                    name,
                )
            } else {
                MakerNoteTag::new(tag_id, tag_name, value)
            };
            tags.insert(tag_id, tag);
        }
    }

    // Post-processing: Compute composite AdvancedSceneMode from SceneMode and AdvancedSceneType
    // ExifTool's AdvancedSceneMode is derived from these two tags
    if let (Some(scene_mode_tag), Some(adv_type_tag)) = (
        tags.get(&PANA_SCENE_MODE),
        tags.get(&PANA_ADVANCED_SCENE_MODE),
    ) {
        // Get the SceneMode value (either decoded string or raw value)
        let scene_mode_val = match &scene_mode_tag.value {
            ExifValue::Short(v) if !v.is_empty() => Some(v[0]),
            ExifValue::Ascii(s) => {
                // Try to reverse-lookup the scene mode from its decoded string
                // For now, check known decoded values
                match s.as_str() {
                    "Off" => Some(0),
                    "Normal" => Some(1),
                    "Portrait" => Some(2),
                    "Scenery" => Some(3),
                    "Sports" => Some(4),
                    "Night Portrait" => Some(5),
                    "Program" => Some(6),
                    "Aperture Priority" => Some(7),
                    "Shutter Priority" => Some(8),
                    "Macro" => Some(9),
                    "Spot" => Some(10),
                    "Manual" => Some(11),
                    "Movie Preview" => Some(12),
                    "Panning" => Some(13),
                    "Simple" => Some(14),
                    "Color Effects" => Some(15),
                    "Self Portrait" => Some(16),
                    "Economy" => Some(17),
                    "Fireworks" => Some(18),
                    "Party" => Some(19),
                    "Snow" => Some(20),
                    "Night Scenery" => Some(21),
                    "Food" => Some(22),
                    "Baby" => Some(23),
                    "Soft Skin" => Some(24),
                    "Candlelight" => Some(25),
                    "Starry Night" => Some(26),
                    "High Sensitivity" => Some(27),
                    "Panorama Assist" => Some(28),
                    "Underwater" => Some(29),
                    "Beach" => Some(30),
                    "Aerial Photo" => Some(31),
                    "Sunset" => Some(32),
                    "Pet" => Some(33),
                    "Intelligent ISO" => Some(34),
                    "Clipboard" => Some(35),
                    "High Speed Continuous Shooting" => Some(36),
                    "Intelligent Auto" => Some(37),
                    "Multi-aspect" => Some(39),
                    "Transform" => Some(41),
                    "Flash Burst" => Some(42),
                    "Pin Hole" => Some(43),
                    "Film Grain" => Some(44),
                    "My Color" => Some(45),
                    "Photo Frame" => Some(46),
                    "HDR" => Some(51),
                    "Handheld Night Shot" => Some(55),
                    "3D" => Some(57),
                    "Creative Control" => Some(59),
                    "Intelligent Auto Plus" => Some(60),
                    "Panorama" => Some(62),
                    "Glass Through" => Some(63),
                    "Photo Style" => Some(90),
                    _ => None,
                }
            }
            _ => None,
        };

        let adv_type_val = match &adv_type_tag.value {
            ExifValue::Short(v) if !v.is_empty() => Some(v[0]),
            _ => None,
        };

        if let (Some(scene), Some(adv_type)) = (scene_mode_val, adv_type_val) {
            let composite_value = compute_advanced_scene_mode(scene, adv_type);
            // Update the AdvancedSceneMode tag with the computed composite value
            let value = ExifValue::Ascii(composite_value);
            tags.insert(
                PANA_ADVANCED_SCENE_MODE,
                MakerNoteTag::with_exiv2(
                    PANA_ADVANCED_SCENE_MODE,
                    Some("AdvancedSceneMode"),
                    value.clone(),
                    value,
                    EXIV2_GROUP_PANASONIC,
                    "AdvancedSceneMode",
                ),
            );
        }
    }

    Ok(tags)
}

/// Compute the composite AdvancedSceneMode from SceneMode and AdvancedSceneType
/// Based on ExifTool's Panasonic.pm composite tag logic
fn compute_advanced_scene_mode(scene_mode: u16, adv_type: u16) -> String {
    // First check specific (SceneMode, AdvancedType) pairs
    match (scene_mode, adv_type) {
        (0, 1) => return "Off".to_string(),
        (2, 2) => return "Outdoor Portrait".to_string(),
        (2, 3) => return "Indoor Portrait".to_string(),
        (2, 4) => return "Creative Portrait".to_string(),
        (3, 2) => return "Nature".to_string(),
        (3, 3) => return "Architecture".to_string(),
        (3, 4) => return "Creative Scenery".to_string(),
        (4, 2) => return "Outdoor Sports".to_string(),
        (4, 3) => return "Indoor Sports".to_string(),
        (4, 4) => return "Creative Sports".to_string(),
        (9, 2) => return "Flower".to_string(),
        (9, 3) => return "Objects".to_string(),
        (9, 4) => return "Creative Macro".to_string(),
        (18, 1) => return "High Sensitivity".to_string(),
        (20, 1) => return "Fireworks".to_string(),
        (21, 2) => return "Illuminations".to_string(),
        (21, 4) => return "Creative Night Scenery".to_string(),
        (26, 1) => return "High-speed Burst (shot 1)".to_string(),
        (27, 1) => return "High-speed Burst (shot 2)".to_string(),
        (29, 1) => return "Snow".to_string(),
        (30, 1) => return "Starry Sky".to_string(),
        (31, 1) => return "Beach".to_string(),
        (36, 1) => return "High-speed Burst (shot 3)".to_string(),
        (39, 1) => return "Aerial Photo / Underwater / Multi-aspect".to_string(),
        (45, 2) => return "Cinema".to_string(),
        (45, 7) => return "Expressive".to_string(),
        (45, 8) => return "Retro".to_string(),
        (45, 9) => return "Pure".to_string(),
        (45, 10) => return "Elegant".to_string(),
        (45, 12) => return "Monochrome".to_string(),
        (45, 13) => return "Dynamic Art".to_string(),
        (45, 14) => return "Silhouette".to_string(),
        (51, 2) => return "HDR Art".to_string(),
        (51, 3) => return "HDR B&W".to_string(),
        (59, 1) => return "Expressive".to_string(),
        (59, 2) => return "Retro".to_string(),
        (59, 3) => return "High Key".to_string(),
        (59, 4) => return "Sepia".to_string(),
        (59, 5) => return "High Dynamic".to_string(),
        (59, 6) => return "Miniature".to_string(),
        (59, 9) => return "Low Key".to_string(),
        (59, 10) => return "Toy Effect".to_string(),
        (59, 11) => return "Dynamic Monochrome".to_string(),
        (59, 12) => return "Soft".to_string(),
        (66, 1) => return "Impressive Art".to_string(),
        (66, 2) => return "Cross Process".to_string(),
        (66, 3) => return "Color Select".to_string(),
        (66, 4) => return "Star".to_string(),
        (90, 3) => return "Old Days".to_string(),
        (90, 4) => return "Sunshine".to_string(),
        (90, 5) => return "Bleach Bypass".to_string(),
        (90, 6) => return "Toy Pop".to_string(),
        (90, 7) => return "Fantasy".to_string(),
        (90, 8) => return "Monochrome".to_string(),
        (90, 9) => return "Rough Monochrome".to_string(),
        (90, 10) => return "Silky Monochrome".to_string(),
        (92, 1) => return "Handheld Night Shot".to_string(),
        _ => {}
    }

    // If no specific pair matches, use the generic logic
    let scene_name = decode_scene_mode_exiftool(scene_mode);
    if scene_name != "Unknown" && scene_name != "Off" {
        if adv_type == 1 {
            return scene_name.to_string();
        } else if adv_type == 5 {
            return format!("{} (intelligent auto)", scene_name);
        } else if adv_type == 7 {
            return format!("{} (intelligent auto plus)", scene_name);
        }
    }

    // Fallback
    format!("Unknown ({} {})", scene_mode, adv_type)
}
