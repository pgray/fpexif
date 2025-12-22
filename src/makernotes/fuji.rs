// makernotes/fuji.rs - Fujifilm maker notes parsing

use crate::data_types::{Endianness, ExifValue};
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use byteorder::{LittleEndian, ReadBytesExt};
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
pub const FUJI_NOISE_REDUCTION: u16 = 0x100B;
pub const FUJI_HIGH_ISO_NOISE_REDUCTION: u16 = 0x100E;
pub const FUJI_CLARITY: u16 = 0x100F;
pub const FUJI_SHADOW_TONE: u16 = 0x1040;
pub const FUJI_HIGHLIGHT_TONE: u16 = 0x1041;
pub const FUJI_COLOR_CHROME_EFFECT: u16 = 0x1048;
pub const FUJI_GRAIN_EFFECT_SIZE: u16 = 0x104C;
pub const FUJI_CROP_MODE: u16 = 0x104D;
pub const FUJI_COLOR_CHROME_FX_BLUE: u16 = 0x104E;
pub const FUJI_SHUTTER_TYPE: u16 = 0x1050;
pub const FUJI_PANORAMA_DIRECTION: u16 = 0x1154;
pub const FUJI_ADVANCED_FILTER: u16 = 0x1201;
pub const FUJI_FINE_PIX_COLOR: u16 = 0x1210;
pub const FUJI_SCENE_RECOGNITION: u16 = 0x1425;
pub const FUJI_IMAGE_GENERATION: u16 = 0x1436;
pub const FUJI_GRAIN_EFFECT_ROUGHNESS: u16 = 0x104B;
pub const FUJI_WHITE_BALANCE_FINE_TUNE: u16 = 0x100A;
pub const FUJI_FLASH_FIRING: u16 = 0x1008;
pub const FUJI_IMAGE_HEIGHT: u16 = 0x1009;
pub const FUJI_IMAGE_WIDTH: u16 = 0x1007;
pub const FUJI_OUTPUT_IMAGE_SIZE: u16 = 0x1304;
pub const FUJI_CONTINUOUS_DRIVE: u16 = 0x1103;
pub const FUJI_VIDEO_MODE: u16 = 0x1303;
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
        FUJI_NOISE_REDUCTION => Some("NoiseReduction"),
        FUJI_HIGH_ISO_NOISE_REDUCTION => Some("HighIsoNoiseReduction"),
        FUJI_CLARITY => Some("Clarity"),
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
        FUJI_SHADOW_TONE => Some("ShadowTone"),
        FUJI_HIGHLIGHT_TONE => Some("HighlightTone"),
        FUJI_COLOR_CHROME_EFFECT => Some("ColorChromeEffect"),
        FUJI_GRAIN_EFFECT_SIZE => Some("GrainEffectSize"),
        FUJI_CROP_MODE => Some("CropMode"),
        FUJI_COLOR_CHROME_FX_BLUE => Some("ColorChromeFXBlue"),
        FUJI_SHUTTER_TYPE => Some("ShutterType"),
        FUJI_GRAIN_EFFECT_ROUGHNESS => Some("GrainEffectRoughness"),
        FUJI_WHITE_BALANCE_FINE_TUNE => Some("WhiteBalanceFineTune"),
        FUJI_FLASH_FIRING => Some("FlashFiring"),
        FUJI_IMAGE_HEIGHT => Some("ImageHeight"),
        FUJI_IMAGE_WIDTH => Some("ImageWidth"),
        FUJI_OUTPUT_IMAGE_SIZE => Some("OutputImageSize"),
        FUJI_CONTINUOUS_DRIVE => Some("ContinuousDrive"),
        FUJI_VIDEO_MODE => Some("VideoMode"),
        FUJI_AUTO_BRACKETING => Some("AutoBracketing"),
        FUJI_SEQUENCE_NUMBER => Some("SequenceNumber"),
        FUJI_BLUR_WARNING => Some("BlurWarning"),
        FUJI_FOCUS_WARNING => Some("FocusWarning"),
        FUJI_EXPOSURE_WARNING => Some("ExposureWarning"),
        FUJI_PANORAMA_DIRECTION => Some("PanoramaDirection"),
        FUJI_ADVANCED_FILTER => Some("AdvancedFilter"),
        FUJI_FINE_PIX_COLOR => Some("FinePixColor"),
        FUJI_DYNAMIC_RANGE => Some("DynamicRange"),
        FUJI_FILM_MODE => Some("FilmMode"),
        FUJI_DYNAMIC_RANGE_SETTING => Some("DynamicRangeSetting"),
        FUJI_DEVELOPMENT_DYNAMIC_RANGE => Some("DevelopmentDynamicRange"),
        FUJI_MIN_FOCAL_LENGTH => Some("MinFocalLength"),
        FUJI_MAX_FOCAL_LENGTH => Some("MaxFocalLength"),
        FUJI_MAX_APERTURE_AT_MIN_FOCAL => Some("MaxApertureAtMinFocal"),
        FUJI_MAX_APERTURE_AT_MAX_FOCAL => Some("MaxApertureAtMaxFocal"),
        FUJI_SCENE_RECOGNITION => Some("SceneRecognition"),
        FUJI_IMAGE_GENERATION => Some("ImageGeneration"),
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

/// Decode FilmMode value (tag 0x1401) - ExifTool format
/// Values based on ExifTool FujiFilm.pm reference
pub fn decode_film_mode_exiftool(value: u16) -> &'static str {
    match value {
        0x000 => "F0/Standard (Provia)",
        0x100 => "F1/Studio Portrait",
        0x110 => "F1a/Studio Portrait Enhanced Saturation",
        0x120 => "F1b/Studio Portrait Smooth Skin Tone (Astia)",
        0x130 => "F1c/Studio Portrait Increased Sharpness",
        0x200 => "F2/Fujichrome (Velvia)",
        0x300 => "F3/Studio Portrait Ex",
        0x400 => "F4/Velvia",
        0x500 => "Pro Neg. Std",
        0x501 => "Pro Neg. Hi",
        0x600 => "Classic Chrome",
        0x700 => "Eterna",
        0x800 => "Classic Negative",
        0x900 => "Bleach Bypass",
        0xa00 => "Nostalgic Neg",
        0xb00 => "Reala ACE",
        // Acros variants (in Saturation tag 0x1003, but sometimes reported here)
        0x801 => "Acros",
        0x802 => "Acros+Ye Filter",
        0x803 => "Acros+R Filter",
        0x804 => "Acros+G Filter",
        // Eterna variants
        0x701 => "Eterna Bleach Bypass",
        _ => "Unknown",
    }
}

/// Decode FilmMode value (tag 0x1401) - exiv2 format
/// Values based on exiv2 fujimn_int.cpp fujiFilmMode[]
pub fn decode_film_mode_exiv2(value: u16) -> &'static str {
    match value {
        0 => "PROVIA (F0/Standard)",
        256 => "F1/Studio Portrait",
        272 => "F1a/Studio Portrait Enhanced Saturation",
        288 => "ASTIA (F1b/Studio Portrait Smooth Skin Tone)",
        304 => "F1c/Studio Portrait Increased Sharpness",
        512 => "Velvia (F2/Fujichrome)",
        768 => "F3/Studio Portrait Ex",
        1024 => "F4/Velvia",
        1280 => "PRO Neg. Std",
        1281 => "PRO Neg. Hi",
        1536 => "CLASSIC CHROME",
        1792 => "ETERNA",
        2048 => "CLASSIC Neg.",
        2304 => "ETERNA Bleach Bypass",
        2560 => "Nostalgic Neg.",
        2816 => "REALA ACE",
        _ => "Unknown",
    }
}

/// Decode DynamicRange value (tag 0x1400)
/// Identical between ExifTool and exiv2
pub fn decode_dynamic_range_exiftool(value: u16) -> &'static str {
    match value {
        1 => "Standard",
        3 => "Wide",
        _ => "Unknown",
    }
}
// decode_dynamic_range_exiv2 - same as exiftool, no separate function needed

/// Decode WhiteBalance value (tag 0x1002) - ExifTool format
pub fn decode_white_balance_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        1 => "Auto (White Priority)",
        2 => "Auto (Ambience Priority)",
        256 => "Daylight",
        512 => "Cloudy",
        768 => "Daylight Fluorescent",
        769 => "Day White Fluorescent",
        770 => "White Fluorescent",
        771 => "Warm White Fluorescent",
        772 => "Living Room Warm White Fluorescent",
        1024 => "Incandescent",
        1280 => "Flash",
        1536 => "Underwater",
        3840 => "Custom",
        3841 => "Custom2",
        3842 => "Custom3",
        3843 => "Custom4",
        3844 => "Custom5",
        6144 => "Kelvin",
        _ => "Unknown",
    }
}

/// Decode WhiteBalance value (tag 0x1002) - exiv2 format
/// Values based on exiv2 fujimn_int.cpp fujiWhiteBalance[]
pub fn decode_white_balance_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        1 => "Auto White Priority",
        2 => "Auto Ambience Priority",
        256 => "Daylight",
        512 => "Cloudy",
        768 => "Fluorescent (daylight)",
        769 => "Fluorescent (warm white)",
        770 => "Fluorescent (cool white)",
        1024 => "Incandescent",
        1536 => "Underwater",
        3480 => "Custom",
        3840 => "Custom 1",
        3841 => "Custom 2",
        3842 => "Custom 3",
        3843 => "Custom 4",
        3844 => "Custom 5",
        4080 => "Kelvin",
        _ => "Unknown",
    }
}

/// Decode Sharpness value (tag 0x1001) - ExifTool format
pub fn decode_sharpness_exiftool(value: u16) -> &'static str {
    match value {
        0x00 => "-4 (softest)",
        0x01 => "-3 (very soft)",
        0x02 => "-2 (soft)",
        0x03 => "0 (normal)",
        0x04 => "+2 (hard)",
        0x05 => "+3 (very hard)",
        0x06 => "+4 (hardest)",
        0x82 => "-1 (medium soft)",
        0x84 => "+1 (medium hard)",
        0x8000 => "Film Simulation",
        0xffff => "n/a",
        _ => "Unknown",
    }
}

/// Decode Sharpness value (tag 0x1001) - exiv2 format
/// Values based on exiv2 fujimn_int.cpp fujiSharpness[]
pub fn decode_sharpness_exiv2(value: u16) -> &'static str {
    match value {
        0 => "-4 (softest)",
        1 => "-3 (very soft)",
        2 => "-2 (soft)",
        3 => "0 (normal)",
        4 => "+2 (hard)",
        5 => "+3 (very hard)",
        6 => "+4 (hardest)",
        130 => "-1 (medium soft)",
        132 => "+1 (medium hard)",
        _ => "Unknown",
    }
}

/// Decode Contrast value (tag 0x1004) - ExifTool format
pub fn decode_contrast_exiftool(value: u16) -> &'static str {
    match value {
        0x0 => "Normal",
        0x080 => "Medium High",
        0x100 => "High",
        0x180 => "Medium Low",
        0x200 => "Low",
        0x8000 => "Film Simulation",
        _ => "Unknown",
    }
}

/// Decode Saturation value (tag 0x1003) - ExifTool format
pub fn decode_saturation_exiftool(value: u16) -> &'static str {
    match value {
        0x0 => "0 (normal)",
        0x80 => "+1 (medium high)",
        0x100 => "+2 (high)",
        0x180 => "+3 (very high)",
        0x200 => "+4 (highest)",
        0x300 => "+4.5",
        0x301 => "Acros",
        0x302 => "Acros+Ye Filter",
        0x303 => "Acros+R Filter",
        0x304 => "Acros+G Filter",
        0x8000 => "Film Simulation",
        0xffff => "n/a",
        _ => "Unknown",
    }
}

/// Decode Saturation value (tag 0x1003) - exiv2 format
/// Values based on exiv2 fujimn_int.cpp fujiColor[]
pub fn decode_saturation_exiv2(value: u16) -> &'static str {
    match value {
        0 => "0 (normal)",
        128 => "+1 (medium high)",
        192 => "+3 (very high)",
        224 => "+4 (highest)",
        256 => "+2 (high)",
        384 => "-1 (medium low)",
        512 => "-2 (low)",
        768 => "Monochrome",
        769 => "Monochrome + R Filter",
        770 => "Monochrome + Ye Filter",
        771 => "Monochrome + G Filter",
        784 => "Sepia",
        1024 => "-2 (low)",
        1216 => "-3 (very low)",
        1248 => "-4 (lowest)",
        1280 => "ACROS",
        1281 => "ACROS + R Filter",
        1282 => "ACROS + Ye Filter",
        1283 => "ACROS + G Filter",
        32768 => "Film Simulation",
        _ => "Unknown",
    }
}

/// Decode Macro value (tag 0x1020)
/// Identical between ExifTool and exiv2 (uses fujiOffOn[])
pub fn decode_macro_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        _ => "Unknown",
    }
}
// decode_macro_exiv2 - same as exiftool, no separate function needed

/// Decode FocusMode value (tag 0x1021)
/// Identical between ExifTool and exiv2
pub fn decode_focus_mode_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        1 => "Manual",
        65535 => "Movie",
        _ => "Unknown",
    }
}
// decode_focus_mode_exiv2 - same as exiftool, no separate function needed

/// Decode AFMode value (tag 0x1022) - ExifTool format
pub fn decode_af_mode_exiftool(value: u16) -> &'static str {
    match value {
        0 => "No",
        1 => "Single Point",
        256 => "Zone",
        512 => "Wide/Tracking",
        768 => "Wide/Tracking (PriorityFace)",
        _ => "Unknown",
    }
}

/// Decode AFMode value (tag 0x1022) - exiv2 format
/// Values based on exiv2 fujimn_int.cpp fujiFocusArea[]
pub fn decode_af_mode_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Wide",
        1 => "Single Point",
        256 => "Zone",
        512 => "Tracking",
        _ => "Unknown",
    }
}

/// Decode SlowSync value (tag 0x1030)
/// Identical between ExifTool and exiv2 (uses fujiOffOn[])
pub fn decode_slow_sync_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        _ => "Unknown",
    }
}
// decode_slow_sync_exiv2 - same as exiftool, no separate function needed

/// Decode AutoBracketing value (tag 0x1100) - ExifTool format
pub fn decode_auto_bracketing_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        2 => "Pre-shot",
        _ => "Unknown",
    }
}

/// Decode AutoBracketing value (tag 0x1100) - exiv2 format
/// Values based on exiv2 fujimn_int.cpp fujiContinuous[]
pub fn decode_auto_bracketing_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        2 => "Pre-shot/No flash & flash",
        6 => "Pixel Shift",
        _ => "Unknown",
    }
}

/// Decode BlurWarning value (tag 0x1300) - ExifTool format
pub fn decode_blur_warning_exiftool(value: u16) -> &'static str {
    match value {
        0 => "None",
        1 => "Blur Warning",
        _ => "Unknown",
    }
}

/// Decode BlurWarning value (tag 0x1300) - exiv2 format
/// Values based on exiv2 fujimn_int.cpp fujiOffOn[]
pub fn decode_blur_warning_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        _ => "Unknown",
    }
}

/// Decode FocusWarning value (tag 0x1301) - ExifTool format
pub fn decode_focus_warning_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Good",
        1 => "Out of focus",
        _ => "Unknown",
    }
}

/// Decode FocusWarning value (tag 0x1301) - exiv2 format
/// Values based on exiv2 fujimn_int.cpp fujiOffOn[]
pub fn decode_focus_warning_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        _ => "Unknown",
    }
}

/// Decode ExposureWarning value (tag 0x1302) - ExifTool format
pub fn decode_exposure_warning_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Good",
        1 => "Bad exposure",
        _ => "Unknown",
    }
}

/// Decode ExposureWarning value (tag 0x1302) - exiv2 format
/// Values based on exiv2 fujimn_int.cpp fujiOffOn[]
pub fn decode_exposure_warning_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        _ => "Unknown",
    }
}

/// Decode PictureMode value (tag 0x1031) - ExifTool format
pub fn decode_picture_mode_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        1 => "Portrait",
        2 => "Landscape",
        3 => "Macro",
        4 => "Sports",
        5 => "Night Scene",
        6 => "Program AE",
        7 => "Natural Light",
        8 => "Anti-blur",
        9 => "Beach & Snow",
        10 => "Sunset",
        11 => "Museum",
        12 => "Party",
        13 => "Flower",
        14 => "Text",
        256 => "Aperture-priority AE",
        512 => "Shutter speed priority AE",
        768 => "Manual",
        _ => "Unknown",
    }
}

/// Decode PictureMode value (tag 0x1031) - exiv2 format
/// Values based on exiv2 fujimn_int.cpp fujiPictureMode[]
pub fn decode_picture_mode_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        1 => "Portrait",
        2 => "Landscape",
        3 => "Macro",
        4 => "Sports",
        5 => "Night scene",
        6 => "Program AE",
        7 => "Natural light",
        8 => "Anti-blur",
        9 => "Beach & Snow",
        10 => "Sunset",
        11 => "Museum",
        12 => "Party",
        13 => "Flower",
        14 => "Text",
        15 => "Natural Light & Flash",
        16 => "Beach",
        17 => "Snow",
        18 => "Fireworks",
        19 => "Underwater",
        20 => "Portrait with Skin Correction",
        22 => "Panorama",
        23 => "Night (tripod)",
        24 => "Pro Low-light",
        25 => "Pro Focus",
        26 => "Portrait 2",
        27 => "Dog Face Detection",
        28 => "Cat Face Detection",
        48 => "HDR",
        64 => "Advanced Filter",
        256 => "Aperture-priority AE",
        512 => "Shutter speed priority AE",
        768 => "Manual",
        _ => "Unknown",
    }
}

/// Decode DynamicRangeSetting value (tag 0x1402) - ExifTool format
pub fn decode_dynamic_range_setting_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        1 => "Manual",
        _ => "Unknown",
    }
}

/// Decode DynamicRangeSetting value (tag 0x1402) - exiv2 format
/// Values based on exiv2 fujimn_int.cpp fujiDynamicRangeSetting[]
pub fn decode_dynamic_range_setting_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        1 => "Manual",
        256 => "Standard (100%)",
        512 => "Wide mode 1 (230%)",
        513 => "Wide mode 2 (400%)",
        32768 => "Film simulation mode",
        _ => "Unknown",
    }
}

/// Decode EXRAuto value (tag 0x1033) - ExifTool format
pub fn decode_exr_auto_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        1 => "Manual",
        _ => "Unknown",
    }
}

/// Decode EXRMode value (tag 0x1034) - ExifTool format
pub fn decode_exr_mode_exiftool(value: u16) -> &'static str {
    match value {
        0x100 => "HR (High Resolution)",
        0x200 => "SN (Signal to Noise priority)",
        0x300 => "DR (Dynamic Range priority)",
        _ => "Unknown",
    }
}

/// Decode InternalSerialNumber to human-readable format
/// Converts hex-encoded body number and date to readable format
/// e.g., "FFDT21804365     59333030373413110124F03021264A" ->
///       "FFDT21804365     Y30074 2013:11:01 24F03021264A"
pub fn decode_internal_serial_number(raw: &str) -> String {
    let trimmed = raw.trim_end_matches('\0').trim_end();

    // Try to decode the hex portion
    // Pattern: prefix + hex(starting with 59 = 'Y') + yymmdd + suffix(12 chars)
    // The hex portion typically starts after spaces and begins with "59" (ASCII 'Y')

    // Find where hex portion starts - look for "59" which is 'Y' in ASCII
    if let Some(hex_start) = trimmed.find("59") {
        // Check that this looks like a hex sequence
        let prefix = &trimmed[..hex_start];
        let rest = &trimmed[hex_start..];

        // rest should be: hex_body + yymmdd + suffix(12)
        // Minimum: some hex + 6 digits date + 12 suffix = at least 20 chars
        if rest.len() >= 18 {
            let suffix_start = rest.len() - 12;
            let date_start = suffix_start - 6;

            // Check if date portion looks like yymmdd (all digits)
            let date_portion = &rest[date_start..suffix_start];
            if date_portion.chars().all(|c| c.is_ascii_digit()) && date_start > 0 {
                let hex_portion = &rest[..date_start];
                let suffix = &rest[suffix_start..];

                // Try to decode hex portion to ASCII
                if hex_portion.len().is_multiple_of(2)
                    && hex_portion.chars().all(|c| c.is_ascii_hexdigit())
                {
                    let mut decoded_body = String::new();
                    for i in (0..hex_portion.len()).step_by(2) {
                        if let Ok(byte) = u8::from_str_radix(&hex_portion[i..i + 2], 16) {
                            if byte.is_ascii_graphic() || byte == b' ' {
                                decoded_body.push(byte as char);
                            } else {
                                // Non-printable, return original
                                return trimmed.to_string();
                            }
                        } else {
                            return trimmed.to_string();
                        }
                    }

                    // Parse date: yymmdd
                    let yy: u16 = date_portion[0..2].parse().unwrap_or(0);
                    let mm = &date_portion[2..4];
                    let dd = &date_portion[4..6];
                    let year = if yy < 70 { 2000 + yy } else { 1900 + yy };

                    return format!(
                        "{}{} {}:{}:{} {}",
                        prefix, decoded_body, year, mm, dd, suffix
                    );
                }
            }
        }
    }

    // Fallback: return as-is
    trimmed.to_string()
}

/// Decode NoiseReduction value (tag 0x100B) - ExifTool/exiv2 format
pub fn decode_noise_reduction_exiftool(value: u16) -> &'static str {
    match value {
        64 => "Low",
        128 => "Normal",
        256 => "n/a",
        _ => "Unknown",
    }
}

/// Decode HighIsoNoiseReduction value (tag 0x100E) - exiv2 format
pub fn decode_high_iso_noise_reduction_exiv2(value: u16) -> &'static str {
    match value {
        0 => "0 (normal)",
        256 => "+2 (strong)",
        384 => "+1 (medium strong)",
        448 => "+3 (very strong)",
        480 => "+4 (strongest)",
        512 => "-2 (weak)",
        640 => "-1 (medium weak)",
        704 => "-3 (very weak)",
        736 => "-4 (weakest)",
        _ => "Unknown",
    }
}

/// Decode Clarity value (tag 0x100F) - exiv2 format
pub fn decode_clarity_exiv2(value: i16) -> &'static str {
    match value {
        -5000 => "-5",
        -4000 => "-4",
        -3000 => "-3",
        -2000 => "-2",
        -1000 => "-1",
        0 => "0",
        1000 => "+1",
        2000 => "+2",
        3000 => "+3",
        4000 => "+4",
        5000 => "+5",
        _ => "Unknown",
    }
}

/// Decode ShadowTone/HighlightTone value (tags 0x1040/0x1041) - exiv2 format
pub fn decode_shadow_highlight_tone_exiv2(value: i16) -> &'static str {
    match value {
        -64 => "+4",
        -56 => "+3.5",
        -48 => "+3",
        -40 => "+2.5",
        -32 => "+2",
        -24 => "+1.5",
        -16 => "+1",
        -8 => "+0.5",
        0 => "0",
        8 => "-0.5",
        16 => "-1",
        24 => "-1.5",
        32 => "-2",
        _ => "Unknown",
    }
}

/// Decode ColorChromeEffect/GrainEffectSize/ColorChromeFXBlue value - exiv2 format
pub fn decode_off_weak_strong_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Off",
        32 => "Weak",
        64 => "Strong",
        _ => "Unknown",
    }
}

/// Decode ShutterType value (tag 0x1050) - exiv2 format
pub fn decode_shutter_type_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Mechanical",
        1 => "Electronic",
        2 => "Electronic (long shutter speed)",
        3 => "Electronic Front Curtain",
        _ => "Unknown",
    }
}

/// Decode CropMode value (tag 0x104D) - exiv2 format
pub fn decode_crop_mode_exiv2(value: u16) -> &'static str {
    match value {
        0 => "None",
        1 => "Full frame",
        2 => "Sports Finder Mode",
        4 => "Electronic Shutter 1.25x Crop",
        _ => "Unknown",
    }
}

/// Decode PanoramaDirection value (tag 0x1154) - exiv2 format
pub fn decode_panorama_direction_exiv2(value: u16) -> &'static str {
    match value {
        1 => "Right",
        2 => "Up",
        3 => "Left",
        4 => "Down",
        _ => "Unknown",
    }
}

/// Decode AdvancedFilter value (tag 0x1201) - exiv2 format
pub fn decode_advanced_filter_exiv2(value: u32) -> &'static str {
    match value {
        0x10000 => "Pop Color",
        0x20000 => "Hi Key",
        0x30000 => "Toy Camera",
        0x40000 => "Miniature",
        0x50000 => "Dynamic Tone",
        0x60001 => "Partial Color Red",
        0x60002 => "Partial Color Yellow",
        0x60003 => "Partial Color Green",
        0x60004 => "Partial Color Blue",
        0x60005 => "Partial Color Orange",
        0x60006 => "Partial Color Purple",
        0x70000 => "Soft Focus",
        0x90000 => "Low Key",
        _ => "Unknown",
    }
}

/// Decode FinePixColor value (tag 0x1210) - exiv2 format
pub fn decode_fine_pix_color_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Standard",
        16 => "Chrome",
        48 => "Black & white",
        _ => "Unknown",
    }
}

/// Decode SceneRecognition value (tag 0x1425) - exiv2 format
pub fn decode_scene_recognition_exiv2(value: u16) -> &'static str {
    match value {
        0x000 => "Unrecognized",
        0x100 => "Portrait Image",
        0x103 => "Night Portrait",
        0x105 => "Backlit Portrait",
        0x200 => "Landscape Image",
        0x300 => "Night Scene",
        0x400 => "Macro",
        _ => "Unknown",
    }
}

/// Decode ImageGeneration value (tag 0x1436) - exiv2 format
pub fn decode_image_generation_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Original Image",
        1 => "Re-developed from RAW",
        _ => "Unknown",
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
    // followed by offset to IFD (4 bytes, little-endian)
    if data.len() >= 12 && &data[0..8] == b"FUJIFILM" {
        // Read IFD offset (little-endian)
        let mut cursor = Cursor::new(&data[8..12]);
        let ifd_offset = cursor
            .read_u32::<LittleEndian>()
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

        // Read number of entries (always little-endian for Fuji)
        let num_entries = cursor
            .read_u16::<LittleEndian>()
            .map_err(|_| ExifError::Format("Failed to read Fuji maker note count".to_string()))?;

        // Parse each entry
        for _ in 0..num_entries {
            if cursor.position() as usize + 12 > ifd_data.len() {
                break;
            }

            let tag_id = match cursor.read_u16::<LittleEndian>() {
                Ok(id) => id,
                Err(_) => break,
            };

            let data_type = match cursor.read_u16::<LittleEndian>() {
                Ok(dt) => dt,
                Err(_) => break,
            };

            let count = match cursor.read_u32::<LittleEndian>() {
                Ok(c) => c,
                Err(_) => break,
            };

            let value_offset = match cursor.read_u32::<LittleEndian>() {
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
                        // Special handling for InternalSerialNumber
                        if tag_id == FUJI_SERIAL_NUMBER {
                            ExifValue::Ascii(decode_internal_serial_number(&s))
                        } else {
                            ExifValue::Ascii(s)
                        }
                    } else {
                        continue;
                    }
                }
                3 => {
                    // SHORT
                    let mut values = Vec::new();
                    if count == 1 {
                        // Value is in the offset field - lower 16 bits (little-endian)
                        values.push((value_offset & 0xFFFF) as u16);
                    } else if count == 2 {
                        // Both values are in the offset field - first is low 16 bits, second is high 16 bits
                        values.push((value_offset & 0xFFFF) as u16);
                        values.push((value_offset >> 16) as u16);
                    } else {
                        // Value is at offset
                        let offset = value_offset as usize;
                        if offset + (count as usize * 2) <= data.len() {
                            let mut cursor = Cursor::new(&data[offset..]);
                            for _ in 0..count {
                                if let Ok(v) = cursor.read_u16::<LittleEndian>() {
                                    values.push(v);
                                } else {
                                    break;
                                }
                            }
                        } else {
                            continue;
                        }
                    }

                    // Apply value decoders for single-value SHORT tags
                    if values.len() == 1 {
                        let v = values[0];
                        let decoded = match tag_id {
                            FUJI_FILM_MODE => Some(decode_film_mode_exiftool(v).to_string()),
                            FUJI_DYNAMIC_RANGE => {
                                Some(decode_dynamic_range_exiftool(v).to_string())
                            }
                            FUJI_WHITE_BALANCE => {
                                Some(decode_white_balance_exiftool(v).to_string())
                            }
                            FUJI_SHARPNESS => Some(decode_sharpness_exiftool(v).to_string()),
                            FUJI_SATURATION => Some(decode_saturation_exiftool(v).to_string()),
                            FUJI_CONTRAST => Some(decode_contrast_exiftool(v).to_string()),
                            FUJI_MACRO => Some(decode_macro_exiftool(v).to_string()),
                            FUJI_FOCUS_MODE => Some(decode_focus_mode_exiftool(v).to_string()),
                            FUJI_AF_MODE => Some(decode_af_mode_exiftool(v).to_string()),
                            FUJI_SLOW_SYNC => Some(decode_slow_sync_exiftool(v).to_string()),
                            FUJI_PICTURE_MODE => Some(decode_picture_mode_exiftool(v).to_string()),
                            FUJI_AUTO_BRACKETING => {
                                Some(decode_auto_bracketing_exiftool(v).to_string())
                            }
                            FUJI_BLUR_WARNING => Some(decode_blur_warning_exiftool(v).to_string()),
                            FUJI_FOCUS_WARNING => {
                                Some(decode_focus_warning_exiftool(v).to_string())
                            }
                            FUJI_EXPOSURE_WARNING => {
                                Some(decode_exposure_warning_exiftool(v).to_string())
                            }
                            FUJI_DYNAMIC_RANGE_SETTING => {
                                Some(decode_dynamic_range_setting_exiftool(v).to_string())
                            }
                            FUJI_EXR_AUTO => Some(decode_exr_auto_exiftool(v).to_string()),
                            FUJI_EXR_MODE => Some(decode_exr_mode_exiftool(v).to_string()),
                            _ => None,
                        };

                        if let Some(s) = decoded {
                            ExifValue::Ascii(s)
                        } else {
                            ExifValue::Short(values)
                        }
                    } else if values.len() == 2 && tag_id == FUJI_FOCUS_PIXEL {
                        // Special formatting for FocusPixel: "X Y" space-separated
                        let formatted = format!("{} {}", values[0], values[1]);
                        ExifValue::Ascii(formatted)
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
                        if offset + (count as usize * 4) <= data.len() {
                            let mut values = Vec::new();
                            let mut cursor = Cursor::new(&data[offset..]);
                            for _ in 0..count {
                                if let Ok(v) = cursor.read_u32::<LittleEndian>() {
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
