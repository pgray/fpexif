// makernotes/fuji.rs - Fujifilm maker notes parsing

use crate::data_types::{Endianness, ExifValue};
use crate::define_tag_decoder;
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Cursor;

// exiv2 group name
pub const EXIV2_GROUP_FUJIFILM: &str = "Fujifilm";

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
pub const FUJI_PRIORITY_SETTINGS: u16 = 0x102b;
pub const FUJI_FOCUS_SETTINGS: u16 = 0x102d;
pub const FUJI_AFC_SETTINGS: u16 = 0x102e;
pub const FUJI_SLOW_SYNC: u16 = 0x1030;

// Virtual tag IDs for PrioritySettings sub-fields (0x102b)
pub const FUJI_AF_S_PRIORITY: u16 = 0xf02b; // Virtual: AF-SPriority
pub const FUJI_AF_C_PRIORITY: u16 = 0xf02c; // Virtual: AF-CPriority

// Virtual tag IDs for FocusSettings sub-fields (0x102d)
pub const FUJI_FOCUS_MODE_2: u16 = 0xf02d; // Virtual: FocusMode2
pub const FUJI_PRE_AF: u16 = 0xf02e; // Virtual: PreAF
pub const FUJI_AF_AREA_MODE: u16 = 0xf02f; // Virtual: AFAreaMode
pub const FUJI_AF_AREA_POINT_SIZE: u16 = 0xf030; // Virtual: AFAreaPointSize
pub const FUJI_AF_AREA_ZONE_SIZE: u16 = 0xf031; // Virtual: AFAreaZoneSize

// Virtual tag IDs for AFCSettings sub-fields (0x102e)
pub const FUJI_AFC_TRACKING_SENS: u16 = 0xf032; // Virtual: AF-CTrackingSensitivity
pub const FUJI_AFC_SPEED_TRACK_SENS: u16 = 0xf033; // Virtual: AF-CSpeedTrackingSensitivity
pub const FUJI_AFC_ZONE_AREA_SWITCHING: u16 = 0xf034; // Virtual: AF-CZoneAreaSwitching
pub const FUJI_PICTURE_MODE: u16 = 0x1031;
pub const FUJI_EXPOSURE_COUNT: u16 = 0x1032;
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
pub const FUJI_AUTO_DYNAMIC_RANGE: u16 = 0x140b;
pub const FUJI_FILE_SOURCE: u16 = 0x8000;
pub const FUJI_ORDER_NUMBER: u16 = 0x8002;
pub const FUJI_FRAME_NUMBER: u16 = 0x8003;
pub const FUJI_FACES_DETECTED: u16 = 0x4100;
pub const FUJI_FACE_POSITIONS: u16 = 0x4103;
pub const FUJI_NUM_FACE_ELEMENTS: u16 = 0x4200;
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
pub const FUJI_DIGITAL_ZOOM: u16 = 0x1044;
pub const FUJI_LENS_MODULATION_OPTIMIZER: u16 = 0x1045;
pub const FUJI_COLOR_CHROME_EFFECT: u16 = 0x1048;
pub const FUJI_GRAIN_EFFECT_ROUGHNESS: u16 = 0x1047;
pub const FUJI_GRAIN_EFFECT_SIZE: u16 = 0x104C;
pub const FUJI_CROP_MODE: u16 = 0x104D;
pub const FUJI_COLOR_CHROME_FX_BLUE: u16 = 0x104E;
pub const FUJI_SHUTTER_TYPE: u16 = 0x1050;
pub const FUJI_PANORAMA_DIRECTION: u16 = 0x1154;
pub const FUJI_ADVANCED_FILTER: u16 = 0x1201;
pub const FUJI_COLOR_MODE: u16 = 0x1210;
pub const FUJI_IMAGE_STABILIZATION: u16 = 0x1422;
pub const FUJI_SCENE_RECOGNITION: u16 = 0x1425;
pub const FUJI_IMAGE_GENERATION: u16 = 0x1436;
pub const FUJI_WHITE_BALANCE_FINE_TUNE: u16 = 0x100A;
pub const FUJI_FLASH_FIRING: u16 = 0x1008;
pub const FUJI_IMAGE_HEIGHT: u16 = 0x1009;
pub const FUJI_IMAGE_WIDTH: u16 = 0x1007;
pub const FUJI_OUTPUT_IMAGE_SIZE: u16 = 0x1304;
pub const FUJI_CONTINUOUS_DRIVE: u16 = 0x1103;
pub const FUJI_VIDEO_MODE: u16 = 0x1303;
pub const FUJI_LENS_MOUNT_TYPE: u16 = 0x1600;
pub const FUJI_RATING: u16 = 0x1431;
pub const FUJI_IMAGE_COUNT: u16 = 0x1438;
pub const FUJI_FLICKER_REDUCTION: u16 = 0x1446;
pub const FUJI_FUJI_MODEL: u16 = 0x1447;
pub const FUJI_FUJI_MODEL2: u16 = 0x1448;
pub const FUJI_COMPOSITE_IMAGE_MODE: u16 = 0x1150;
pub const FUJI_COMPOSITE_IMAGE_COUNT1: u16 = 0x1151;
pub const FUJI_COMPOSITE_IMAGE_COUNT2: u16 = 0x1152;
pub const FUJI_RELATIVE_EXPOSURE: u16 = 0x9200;
pub const FUJI_RAW_EXPOSURE_BIAS: u16 = 0x9650;
pub const FUJI_RATINGS_INFO: u16 = 0xB211;
pub const FUJI_GE_IMAGE_SIZE: u16 = 0xB212;

// WB_GRGBLevels tags (int16u[4] in GRGB order)
pub const FUJI_WB_GRGB_LEVELS_AUTO: u16 = 0x2000;
pub const FUJI_WB_GRGB_LEVELS_DAYLIGHT: u16 = 0x2100;
pub const FUJI_WB_GRGB_LEVELS_CLOUDY: u16 = 0x2200;
pub const FUJI_WB_GRGB_LEVELS_DAYLIGHT_FLUOR: u16 = 0x2300;
pub const FUJI_WB_GRGB_LEVELS_DAY_WHITE_FLUOR: u16 = 0x2301;
pub const FUJI_WB_GRGB_LEVELS_WHITE_FLUOR: u16 = 0x2302;
pub const FUJI_WB_GRGB_LEVELS_WARM_WHITE_FLUOR: u16 = 0x2310;
pub const FUJI_WB_GRGB_LEVELS_LIVING_ROOM_WARM_WHITE_FLUOR: u16 = 0x2311;
pub const FUJI_WB_GRGB_LEVELS_TUNGSTEN: u16 = 0x2400;
pub const FUJI_WB_GRGB_LEVELS: u16 = 0x2FF0;

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
        // Note: 0x100B is skipped when value is 256 (handled in parser)
        FUJI_NOISE_REDUCTION => Some("NoiseReduction_Old"),
        // 0x100E is the preferred NoiseReduction tag for newer cameras
        FUJI_HIGH_ISO_NOISE_REDUCTION => Some("NoiseReduction"),
        FUJI_CLARITY => Some("Clarity"),
        FUJI_FLASH_MODE => Some("FujiFlashMode"),
        FUJI_FLASH_EXPOSURE_COMP => Some("FlashExposureComp"),
        FUJI_MACRO => Some("Macro"),
        FUJI_FOCUS_MODE => Some("FocusMode"),
        FUJI_AF_MODE => Some("AFMode"),
        FUJI_FOCUS_PIXEL => Some("FocusPixel"),
        // Virtual sub-field tags from PrioritySettings (0x102b)
        FUJI_AF_S_PRIORITY => Some("AF-SPriority"),
        FUJI_AF_C_PRIORITY => Some("AF-CPriority"),
        // Virtual sub-field tags from FocusSettings (0x102d)
        FUJI_FOCUS_MODE_2 => Some("FocusMode2"),
        FUJI_PRE_AF => Some("PreAF"),
        FUJI_AF_AREA_MODE => Some("AFAreaMode"),
        FUJI_AF_AREA_POINT_SIZE => Some("AFAreaPointSize"),
        FUJI_AF_AREA_ZONE_SIZE => Some("AFAreaZoneSize"),
        // Virtual sub-field tags from AFCSettings (0x102e)
        FUJI_AFC_TRACKING_SENS => Some("AF-CTrackingSensitivity"),
        FUJI_AFC_SPEED_TRACK_SENS => Some("AF-CSpeedTrackingSensitivity"),
        FUJI_AFC_ZONE_AREA_SWITCHING => Some("AF-CZoneAreaSwitching"),
        FUJI_SLOW_SYNC => Some("SlowSync"),
        FUJI_PICTURE_MODE => Some("PictureMode"),
        FUJI_EXPOSURE_COUNT => Some("ExposureCount"),
        FUJI_EXR_AUTO => Some("EXRAuto"),
        FUJI_EXR_MODE => Some("EXRMode"),
        FUJI_SHADOW_TONE => Some("ShadowTone"),
        FUJI_HIGHLIGHT_TONE => Some("HighlightTone"),
        FUJI_DIGITAL_ZOOM => Some("DigitalZoom"),
        FUJI_LENS_MODULATION_OPTIMIZER => Some("LensModulationOptimizer"),
        FUJI_COLOR_CHROME_EFFECT => Some("ColorChromeEffect"),
        FUJI_GRAIN_EFFECT_SIZE => Some("GrainEffectSize"),
        FUJI_CROP_MODE => Some("CropMode"),
        FUJI_COLOR_CHROME_FX_BLUE => Some("ColorChromeFXBlue"),
        FUJI_SHUTTER_TYPE => Some("ShutterType"),
        FUJI_GRAIN_EFFECT_ROUGHNESS => Some("GrainEffectRoughness"),
        FUJI_WHITE_BALANCE_FINE_TUNE => Some("WhiteBalanceFineTune"),
        FUJI_FLASH_FIRING => Some("FlashFiring"),
        // These are undocumented Fuji MakerNote tags, not standard TIFF ImageWidth/Height
        // ExifTool outputs them as FujiFilm_0x1007 and FujiFilm_0x1009
        // We skip naming them to avoid collision with standard tags
        // FUJI_IMAGE_HEIGHT => Some("FujiImageHeight"),
        // FUJI_IMAGE_WIDTH => Some("FujiImageWidth"),
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
        FUJI_COLOR_MODE => Some("ColorMode"),
        FUJI_DYNAMIC_RANGE => Some("DynamicRange"),
        FUJI_FILM_MODE => Some("FilmMode"),
        FUJI_DYNAMIC_RANGE_SETTING => Some("DynamicRangeSetting"),
        FUJI_DEVELOPMENT_DYNAMIC_RANGE => Some("DevelopmentDynamicRange"),
        FUJI_MIN_FOCAL_LENGTH => Some("MinFocalLength"),
        FUJI_MAX_FOCAL_LENGTH => Some("MaxFocalLength"),
        FUJI_MAX_APERTURE_AT_MIN_FOCAL => Some("MaxApertureAtMinFocal"),
        FUJI_MAX_APERTURE_AT_MAX_FOCAL => Some("MaxApertureAtMaxFocal"),
        FUJI_AUTO_DYNAMIC_RANGE => Some("AutoDynamicRange"),
        FUJI_IMAGE_STABILIZATION => Some("ImageStabilization"),
        FUJI_SCENE_RECOGNITION => Some("SceneRecognition"),
        FUJI_IMAGE_GENERATION => Some("ImageGeneration"),
        FUJI_FILE_SOURCE => Some("FileSource"),
        FUJI_ORDER_NUMBER => Some("OrderNumber"),
        FUJI_FRAME_NUMBER => Some("FrameNumber"),
        FUJI_FACES_DETECTED => Some("FacesDetected"),
        FUJI_FACE_POSITIONS => Some("FacePositions"),
        FUJI_NUM_FACE_ELEMENTS => Some("NumFaceElements"),
        FUJI_FACE_REC_INFO => Some("FaceRecInfo"),
        FUJI_RAW_IMAGE_FULL_SIZE => Some("RawImageFullSize"),
        FUJI_RAW_IMAGE_CROP_TOP_LEFT => Some("RawImageCropTopLeft"),
        FUJI_RAW_IMAGE_CROPPED_SIZE => Some("RawImageCroppedSize"),
        FUJI_RAW_IMAGE_ASPECT_RATIO => Some("RawImageAspectRatio"),
        FUJI_LENS_MOUNT_TYPE => Some("LensMountType"),
        FUJI_RATING => Some("Rating"),
        FUJI_IMAGE_COUNT => Some("ImageCount"),
        FUJI_FLICKER_REDUCTION => Some("FlickerReduction"),
        FUJI_FUJI_MODEL => Some("FujiModel"),
        FUJI_FUJI_MODEL2 => Some("FujiModel2"),
        FUJI_COMPOSITE_IMAGE_MODE => Some("CompositeImageMode"),
        FUJI_COMPOSITE_IMAGE_COUNT1 => Some("CompositeImageCount1"),
        FUJI_COMPOSITE_IMAGE_COUNT2 => Some("CompositeImageCount2"),
        FUJI_RELATIVE_EXPOSURE => Some("RelativeExposure"),
        FUJI_RAW_EXPOSURE_BIAS => Some("RawExposureBias"),
        FUJI_RATINGS_INFO => Some("RatingsInfo"),
        FUJI_GE_IMAGE_SIZE => Some("GEImageSize"),
        // WB_GRGBLevels tags
        FUJI_WB_GRGB_LEVELS_AUTO => Some("WB_GRGBLevelsAuto"),
        FUJI_WB_GRGB_LEVELS_DAYLIGHT => Some("WB_GRGBLevelsDaylight"),
        FUJI_WB_GRGB_LEVELS_CLOUDY => Some("WB_GRGBLevelsCloudy"),
        FUJI_WB_GRGB_LEVELS_DAYLIGHT_FLUOR => Some("WB_GRGBLevelsDaylightFluor"),
        FUJI_WB_GRGB_LEVELS_DAY_WHITE_FLUOR => Some("WB_GRGBLevelsDayWhiteFluor"),
        FUJI_WB_GRGB_LEVELS_WHITE_FLUOR => Some("WB_GRGBLevelsWhiteFluorescent"),
        FUJI_WB_GRGB_LEVELS_WARM_WHITE_FLUOR => Some("WB_GRGBLevelsWarmWhiteFluor"),
        FUJI_WB_GRGB_LEVELS_LIVING_ROOM_WARM_WHITE_FLUOR => {
            Some("WB_GRGBLevelsLivingRoomWarmWhiteFluor")
        }
        FUJI_WB_GRGB_LEVELS_TUNGSTEN => Some("WB_GRGBLevelsTungsten"),
        FUJI_WB_GRGB_LEVELS => Some("WB_GRGBLevels"),
        _ => None,
    }
}

// FilmMode (tag 0x1401): FujiFilm.pm / fujimn_int.cpp fujiFilmMode[]
define_tag_decoder! {
    film_mode,
    exiftool: {
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
        0x701 => "Eterna Bleach Bypass",
        0x800 => "Classic Negative",
        0x801 => "Acros",
        0x802 => "Acros+Ye Filter",
        0x803 => "Acros+R Filter",
        0x804 => "Acros+G Filter",
        0x900 => "Bleach Bypass",
        0xa00 => "Nostalgic Neg",
        0xb00 => "Reala ACE",
    },
    exiv2: {
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
    }
}

// DynamicRange (tag 0x1400): FujiFilm.pm / fujimn_int.cpp
define_tag_decoder! {
    dynamic_range,
    both: {
        1 => "Standard",
        3 => "Wide",
    }
}

// WhiteBalance (tag 0x1002): FujiFilm.pm / fujimn_int.cpp fujiWhiteBalance[]
define_tag_decoder! {
    white_balance,
    exiftool: {
        0 => "Auto",
        1 => "Auto (white priority)",
        2 => "Auto (ambiance priority)",
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
        4080 => "Kelvin",
    },
    exiv2: {
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
    }
}

// Sharpness (tag 0x1001): FujiFilm.pm / fujimn_int.cpp fujiSharpness[]
define_tag_decoder! {
    sharpness,
    exiftool: {
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
    },
    exiv2: {
        0 => "-4 (softest)",
        1 => "-3 (very soft)",
        2 => "-2 (soft)",
        3 => "0 (normal)",
        4 => "+2 (hard)",
        5 => "+3 (very hard)",
        6 => "+4 (hardest)",
        130 => "-1 (medium soft)",
        132 => "+1 (medium hard)",
    }
}

// Contrast (tag 0x1004): FujiFilm.pm / fujimn_int.cpp
define_tag_decoder! {
    contrast,
    both: {
        0x0 => "Normal",
        0x080 => "Medium High",
        0x100 => "High",
        0x180 => "Medium Low",
        0x200 => "Low",
        0x8000 => "Film Simulation",
    }
}

// Saturation (tag 0x1003): FujiFilm.pm / fujimn_int.cpp fujiColor[]
define_tag_decoder! {
    saturation,
    exiftool: {
        0x0 => "0 (normal)",
        0x80 => "+1 (medium high)",
        0x100 => "+2 (high)",
        0xc0 => "+3 (very high)",
        0xe0 => "+4 (highest)",
        0x180 => "-1 (medium low)",
        0x200 => "Low",
        0x300 => "None (B&W)",
        0x301 => "B&W Red Filter",
        0x302 => "B&W Yellow Filter",
        0x303 => "B&W Green Filter",
        0x310 => "B&W Sepia",
        0x400 => "-2 (low)",
        0x4c0 => "-3 (very low)",
        0x4e0 => "-4 (lowest)",
        0x500 => "Acros",
        0x501 => "Acros Red Filter",
        0x502 => "Acros Yellow Filter",
        0x503 => "Acros Green Filter",
        0x8000 => "Film Simulation",
        0xffff => "n/a",
    },
    exiv2: {
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
    }
}

// Macro (tag 0x1020): FujiFilm.pm / fujimn_int.cpp (fujiOffOn[])
define_tag_decoder! {
    fuji_macro,
    both: {
        0 => "Off",
        1 => "On",
    }
}

// FocusMode (tag 0x1021): FujiFilm.pm / fujimn_int.cpp
define_tag_decoder! {
    focus_mode,
    both: {
        0 => "Auto",
        1 => "Manual",
        65535 => "Movie",
    }
}

// AFMode (tag 0x1022): FujiFilm.pm / fujimn_int.cpp fujiFocusArea[]
define_tag_decoder! {
    af_mode,
    exiftool: {
        0 => "No",
        1 => "Single Point",
        256 => "Zone",
        512 => "Wide/Tracking",
        768 => "Wide/Tracking (PriorityFace)",
    },
    exiv2: {
        0 => "Wide",
        1 => "Single Point",
        256 => "Zone",
        512 => "Tracking",
    }
}

// SlowSync (tag 0x1030): FujiFilm.pm / fujimn_int.cpp (fujiOffOn[])
define_tag_decoder! {
    slow_sync,
    both: {
        0 => "Off",
        1 => "On",
    }
}

// FlashMode (tag 0x1010): FujiFilm.pm fujiFlashMode[] / fujimn_int.cpp
define_tag_decoder! {
    flash_mode,
    exiftool: {
        0 => "Auto",
        1 => "On",
        2 => "Off",
        3 => "Red-eye reduction",
        4 => "External",
        16 => "Commander",
        0x8000 => "Not Attached",
        0x8120 => "TTL",
        0x8320 => "TTL Auto - Did not fire",
        0x9840 => "Manual",
        0x9860 => "Flash Commander",
        0x9880 => "Multi-flash",
        0xa920 => "1st Curtain (front)",
        0xaa20 => "TTL Slow - 1st Curtain (front)",
        0xab20 => "TTL Auto - 1st Curtain (front)",
        0xad20 => "TTL - Red-eye Flash - 1st Curtain (front)",
        0xae20 => "TTL Slow - Red-eye Flash - 1st Curtain (front)",
        0xaf20 => "TTL Auto - Red-eye Flash - 1st Curtain (front)",
        0xc920 => "2nd Curtain (rear)",
        0xca20 => "TTL Slow - 2nd Curtain (rear)",
        0xcb20 => "TTL Auto - 2nd Curtain (rear)",
        0xcd20 => "TTL - Red-eye Flash - 2nd Curtain (rear)",
        0xce20 => "TTL Slow - Red-eye Flash - 2nd Curtain (rear)",
        0xcf20 => "TTL Auto - Red-eye Flash - 2nd Curtain (rear)",
        0xe920 => "High Speed Sync (HSS)",
    },
    exiv2: {
        0x0000 => "Auto",
        0x0001 => "On",
        0x0002 => "Off",
        0x0003 => "Red-eye reduction",
        0x0004 => "External",
        0x0010 => "Commander",
        0x8000 => "No flash",
        0x8120 => "TTL",
    }
}

// AutoBracketing (tag 0x1100): FujiFilm.pm / fujimn_int.cpp fujiContinuous[]
define_tag_decoder! {
    auto_bracketing,
    exiftool: {
        0 => "Off",
        1 => "On",
        2 => "Pre-shot",
    },
    exiv2: {
        0 => "Off",
        1 => "On",
        2 => "Pre-shot/No flash & flash",
        6 => "Pixel Shift",
    }
}

// BlurWarning (tag 0x1300): FujiFilm.pm / fujimn_int.cpp
define_tag_decoder! {
    blur_warning,
    exiftool: {
        0 => "None",
        1 => "Blur Warning",
    },
    exiv2: {
        0 => "Off",
        1 => "On",
    }
}

// FocusWarning (tag 0x1301): FujiFilm.pm / fujimn_int.cpp
define_tag_decoder! {
    focus_warning,
    exiftool: {
        0 => "Good",
        1 => "Out of focus",
    },
    exiv2: {
        0 => "Off",
        1 => "On",
    }
}

// ExposureWarning (tag 0x1302): FujiFilm.pm / fujimn_int.cpp
define_tag_decoder! {
    exposure_warning,
    exiftool: {
        0 => "Good",
        1 => "Bad exposure",
    },
    exiv2: {
        0 => "Off",
        1 => "On",
    }
}

// PictureMode (tag 0x1031): FujiFilm.pm / fujimn_int.cpp fujiPictureMode[]
define_tag_decoder! {
    picture_mode,
    exiftool: {
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
    },
    exiv2: {
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
    }
}

// DynamicRangeSetting (tag 0x1402): FujiFilm.pm / fujimn_int.cpp fujiDynamicRangeSetting[]
define_tag_decoder! {
    dynamic_range_setting,
    exiftool: {
        0 => "Auto",
        1 => "Manual",
        256 => "Standard (100%)",
        512 => "Wide1 (230%)",
        513 => "Wide2 (400%)",
        32768 => "Film Simulation",
    },
    exiv2: {
        0 => "Auto",
        1 => "Manual",
        256 => "Standard (100%)",
        512 => "Wide mode 1 (230%)",
        513 => "Wide mode 2 (400%)",
        32768 => "Film simulation mode",
    }
}

// EXRAuto (tag 0x1033): FujiFilm.pm
define_tag_decoder! {
    exr_auto,
    both: {
        0 => "Auto",
        1 => "Manual",
    }
}

// EXRMode (tag 0x1034): FujiFilm.pm
define_tag_decoder! {
    exr_mode,
    both: {
        0x100 => "HR (High Resolution)",
        0x200 => "SN (Signal to Noise priority)",
        0x300 => "DR (Dynamic Range priority)",
    }
}

/// Decode InternalSerialNumber to human-readable format
/// Converts hex-encoded body number and date to readable format
/// e.g., "FFDT21804365     59333030373413110124F03021264A" ->
///       "FFDT21804365     Y30074 2013:11:01 24F03021264A"
/// e.g., "FC  B2595279     5933313935341412173EA330218443" ->
///       "FC  B2595279     Y31954 2014:12:17 3EA330218443"
pub fn decode_internal_serial_number(raw: &str) -> String {
    let trimmed = raw.trim_end_matches('\0').trim_end();

    // Try to decode the hex portion
    // Pattern: prefix (with trailing spaces) + hex(starting with 59 = 'Y') + yymmdd + suffix(12 chars)
    // The hex portion starts AFTER the trailing spaces in the prefix

    // Find the last occurrence of multiple consecutive spaces, then look for "59" after that
    // This avoids matching "59" that appears within the prefix (e.g., "B2595279")
    let mut search_start = 0;
    if let Some(space_pos) = trimmed.rfind("  ") {
        // Find the end of the space sequence
        search_start = space_pos;
        while search_start < trimmed.len() && trimmed.as_bytes().get(search_start) == Some(&b' ') {
            search_start += 1;
        }
    }

    // Look for "59" (hex for 'Y') starting from after the spaces
    if let Some(relative_pos) = trimmed[search_start..].find("59") {
        let hex_start = search_start + relative_pos;
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

// NoiseReduction (tag 0x100B): FujiFilm.pm / fujimn_int.cpp
define_tag_decoder! {
    noise_reduction,
    both: {
        64 => "Low",
        128 => "Normal",
        256 => "n/a",
    }
}

// HighIsoNoiseReduction (tag 0x100E): fujimn_int.cpp fujiNoiseReduction[]
define_tag_decoder! {
    high_iso_noise_reduction,
    exiftool: {
        0x000 => "0 (normal)",
        0x100 => "+2 (strong)",
        0x180 => "+1 (medium strong)",
        0x1c0 => "+3 (very strong)",
        0x1e0 => "+4 (strongest)",
        0x200 => "-2 (weak)",
        0x280 => "-1 (medium weak)",
        0x2c0 => "-3 (very weak)",
        0x2e0 => "-4 (weakest)",
    },
    exiv2: {
        0 => "0 (normal)",
        256 => "+2 (strong)",
        384 => "+1 (medium strong)",
        448 => "+3 (very strong)",
        480 => "+4 (strongest)",
        512 => "-2 (weak)",
        640 => "-1 (medium weak)",
        704 => "-3 (very weak)",
        736 => "-4 (weakest)",
    }
}

// Clarity (tag 0x100F): fujimn_int.cpp
define_tag_decoder! {
    clarity,
    type: i16,
    both: {
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
    }
}

// ShadowTone/HighlightTone (tags 0x1040/0x1041): fujimn_int.cpp
define_tag_decoder! {
    shadow_highlight_tone,
    type: i16,
    both: {
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
    }
}

// ColorChromeEffect/ColorChromeFXBlue - fujimn_int.cpp
define_tag_decoder! {
    off_weak_strong,
    both: {
        0 => "Off",
        32 => "Weak",
        64 => "Strong",
    }
}

// GrainEffectSize (tag 0x104c): FujiFilm.pm
define_tag_decoder! {
    grain_effect_size,
    both: {
        0 => "Off",
        16 => "Small",
        32 => "Large",
    }
}

// ShutterType (tag 0x1050): FujiFilm.pm / fujimn_int.cpp
define_tag_decoder! {
    shutter_type,
    both: {
        0 => "Mechanical",
        1 => "Electronic",
        2 => "Electronic (long shutter speed)",
        3 => "Electronic Front Curtain",
    }
}

// CropMode (tag 0x104D): FujiFilm.pm / fujimn_int.cpp
define_tag_decoder! {
    crop_mode,
    exiftool: {
        0 => "n/a",
        1 => "Full-frame on GFX",
        2 => "Sports Finder Mode",
        4 => "Electronic Shutter 1.25x Crop",
        8 => "Digital Tele-Conv",
    },
    exiv2: {
        0 => "None",
        1 => "Full frame",
        2 => "Sports Finder Mode",
        4 => "Electronic Shutter 1.25x Crop",
    }
}

// PanoramaDirection (tag 0x1154): fujimn_int.cpp
define_tag_decoder! {
    panorama_direction,
    both: {
        1 => "Right",
        2 => "Up",
        3 => "Left",
        4 => "Down",
    }
}

// AdvancedFilter (tag 0x1201): fujimn_int.cpp
define_tag_decoder! {
    advanced_filter,
    type: u32,
    both: {
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
    }
}

// ColorMode (tag 0x1210): FujiFilm.pm / fujimn_int.cpp
define_tag_decoder! {
    color_mode,
    exiftool: {
        0x00 => "Standard",
        0x10 => "Chrome",
        0x30 => "B & W",
    },
    exiv2: {
        0 => "Standard",
        16 => "Chrome",
        48 => "Black & white",
    }
}

// SceneRecognition (tag 0x1425): fujimn_int.cpp
define_tag_decoder! {
    scene_recognition,
    both: {
        0x000 => "Unrecognized",
        0x100 => "Portrait Image",
        0x103 => "Night Portrait",
        0x105 => "Backlit Portrait",
        0x200 => "Landscape Image",
        0x300 => "Night Scene",
        0x400 => "Macro",
    }
}

// ImageGeneration (tag 0x1436): FujiFilm.pm / fujimn_int.cpp
define_tag_decoder! {
    image_generation,
    both: {
        0 => "Original Image",
        1 => "Re-developed from RAW",
    }
}

// CompositeImageMode (tag 0x1150): FujiFilm.pm / fujimn_int.cpp
define_tag_decoder! {
    composite_image_mode,
    type: u32,
    both: {
        0 => "n/a",
        1 => "Pro Low-light",
        2 => "Pro Focus",
        32 => "Panorama",
        128 => "HDR",
        1024 => "Multi-exposure",
    }
}

// ShadowTone/HighlightTone ExifTool (tags 0x1040/0x1041): FujiFilm.pm
// For values not in the table, ExifTool calculates: -val / 16
pub fn decode_shadow_highlight_tone_ext_exiftool(value: i32) -> String {
    match value {
        -64 => "+4 (hardest)".to_string(),
        -48 => "+3 (very hard)".to_string(),
        -32 => "+2 (hard)".to_string(),
        -16 => "+1 (medium hard)".to_string(),
        0 => "0 (normal)".to_string(),
        16 => "-1 (medium soft)".to_string(),
        32 => "-2 (soft)".to_string(),
        // For other values, calculate: -val / 16
        _ => {
            let result = -value as f64 / 16.0;
            if result == result.floor() {
                format!("{}", result as i32)
            } else {
                format!("{}", result)
            }
        }
    }
}

pub fn decode_shadow_highlight_tone_ext_exiv2(value: i32) -> String {
    decode_shadow_highlight_tone_ext_exiftool(value)
}

// LensModulationOptimizer (tag 0x1045): FujiFilm.pm
define_tag_decoder! {
    lens_modulation_optimizer,
    type: u32,
    both: {
        0 => "Off",
        1 => "On",
    }
}

/// Decode ImageStabilization (tag 0x1422) - 3 values: type, mode, param
/// ExifTool format: "Type; Mode; Param"
pub fn decode_image_stabilization_exiftool(values: &[u32]) -> String {
    if values.len() < 3 {
        return values
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ");
    }

    // First value: stabilization type
    let is_type = match values[0] {
        0 => "None".to_string(),
        1 => "Optical".to_string(),
        2 => "Sensor-shift".to_string(),
        3 => "OIS Lens".to_string(),
        258 => "IBIS/OIS + DIS".to_string(),
        512 => "Digital".to_string(),
        v => format!("Unknown ({})", v),
    };

    let is_mode = match values[1] {
        0 => "Off",
        1 => "On (mode 1, continuous)",
        2 => "On (mode 2, shooting only)",
        3 => "On (mode 3, panning)",
        _ => "Unknown",
    };

    format!("{}; {}; {}", is_type, is_mode, values[2])
}

// NoiseReduction full range (tag 0x100B or 0x100E): FujiFilm.pm
define_tag_decoder! {
    noise_reduction_full,
    both: {
        0x000 => "0 (normal)",
        0x040 => "Low",
        0x080 => "Normal",
        0x100 => "+2 (strong)",
        0x180 => "+1 (medium strong)",
        0x1c0 => "+3 (very strong)",
        0x1e0 => "+4 (strongest)",
        0x200 => "-2 (weak)",
        0x280 => "-1 (medium weak)",
        0x2c0 => "-3 (very weak)",
        0x2e0 => "-4 (weakest)",
    }
}

/// Check if a tag is a WB_GRGBLevels variant
fn is_wb_grgb_levels_tag(tag_id: u16) -> bool {
    matches!(
        tag_id,
        FUJI_WB_GRGB_LEVELS_AUTO
            | FUJI_WB_GRGB_LEVELS_DAYLIGHT
            | FUJI_WB_GRGB_LEVELS_CLOUDY
            | FUJI_WB_GRGB_LEVELS_DAYLIGHT_FLUOR
            | FUJI_WB_GRGB_LEVELS_DAY_WHITE_FLUOR
            | FUJI_WB_GRGB_LEVELS_WHITE_FLUOR
            | FUJI_WB_GRGB_LEVELS_WARM_WHITE_FLUOR
            | FUJI_WB_GRGB_LEVELS_LIVING_ROOM_WARM_WHITE_FLUOR
            | FUJI_WB_GRGB_LEVELS_TUNGSTEN
            | FUJI_WB_GRGB_LEVELS
    )
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
                3 | 8 => {
                    // SHORT (3) or SSHORT (8)
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

                    // Handle PrioritySettings (0x102b) which expands to multiple sub-tags
                    if tag_id == FUJI_PRIORITY_SETTINGS && values.len() == 1 {
                        let v = values[0];
                        // AF-SPriority: bits 0-3 (mask 0x000f)
                        let af_s_priority = v & 0x000f;
                        let af_s_str = match af_s_priority {
                            1 => "Release",
                            2 => "Focus",
                            _ => "Unknown",
                        };
                        tags.insert(
                            FUJI_AF_S_PRIORITY,
                            MakerNoteTag::new(
                                FUJI_AF_S_PRIORITY,
                                Some("AF-SPriority"),
                                ExifValue::Ascii(af_s_str.to_string()),
                            ),
                        );

                        // AF-CPriority: bits 4-7 (mask 0x00f0)
                        let af_c_priority = (v >> 4) & 0x0f;
                        let af_c_str = match af_c_priority {
                            1 => "Release",
                            2 => "Focus",
                            _ => "Unknown",
                        };
                        tags.insert(
                            FUJI_AF_C_PRIORITY,
                            MakerNoteTag::new(
                                FUJI_AF_C_PRIORITY,
                                Some("AF-CPriority"),
                                ExifValue::Ascii(af_c_str.to_string()),
                            ),
                        );

                        continue; // Skip normal tag insertion
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
                            FUJI_MACRO => Some(decode_fuji_macro_exiftool(v).to_string()),
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
                            FUJI_FLASH_MODE => Some(decode_flash_mode_exiftool(v).to_string()),
                            FUJI_SHUTTER_TYPE => Some(decode_shutter_type_exiftool(v).to_string()),
                            FUJI_IMAGE_GENERATION => {
                                Some(decode_image_generation_exiftool(v).to_string())
                            }
                            FUJI_NOISE_REDUCTION => {
                                // Tag 0x100B - older style (skip handled below for value 0x100)
                                Some(decode_noise_reduction_exiftool(v).to_string())
                            }
                            FUJI_HIGH_ISO_NOISE_REDUCTION => {
                                // Tag 0x100E - newer cameras use this, value 0 = "0 (normal)"
                                Some(decode_noise_reduction_full_exiftool(v).to_string())
                            }
                            FUJI_COLOR_CHROME_EFFECT => {
                                Some(decode_off_weak_strong_exiftool(v).to_string())
                            }
                            FUJI_GRAIN_EFFECT_SIZE => {
                                Some(decode_grain_effect_size_exiftool(v).to_string())
                            }
                            FUJI_COLOR_CHROME_FX_BLUE => {
                                Some(decode_off_weak_strong_exiftool(v).to_string())
                            }
                            FUJI_CROP_MODE => Some(decode_crop_mode_exiftool(v).to_string()),
                            FUJI_COLOR_MODE => Some(decode_color_mode_exiftool(v).to_string()),
                            FUJI_IMAGE_COUNT => {
                                // Apply mask 0x7fff
                                let count = v & 0x7fff;
                                Some(count.to_string())
                            }
                            FUJI_COMPOSITE_IMAGE_COUNT1 | FUJI_COMPOSITE_IMAGE_COUNT2 => {
                                Some(v.to_string())
                            }
                            _ => None,
                        };

                        // Check for special skip condition (NoiseReduction 0x100B with value 256)
                        if tag_id == FUJI_NOISE_REDUCTION && values.len() == 1 && values[0] == 0x100
                        {
                            continue; // Skip this tag entirely, 0x100E will be used
                        }

                        if let Some(s) = decoded {
                            ExifValue::Ascii(s)
                        } else {
                            ExifValue::Short(values)
                        }
                    } else if values.len() == 2 && tag_id == FUJI_FOCUS_PIXEL {
                        // Special formatting for FocusPixel: "X Y" space-separated
                        let formatted = format!("{} {}", values[0], values[1]);
                        ExifValue::Ascii(formatted)
                    } else if tag_id == FUJI_IMAGE_STABILIZATION && values.len() >= 3 {
                        // ImageStabilization: format as "Type; Mode; Param"
                        // Convert shorts to u32 for the decode function
                        let long_vals: Vec<u32> = values.iter().map(|&v| v as u32).collect();
                        ExifValue::Ascii(decode_image_stabilization_exiftool(&long_vals))
                    } else if is_wb_grgb_levels_tag(tag_id) && values.len() >= 4 {
                        // WB_GRGBLevels tags: format as space-separated values "G R G B"
                        let formatted =
                            format!("{} {} {} {}", values[0], values[1], values[2], values[3]);
                        ExifValue::Ascii(formatted)
                    } else if tag_id == FUJI_RAW_IMAGE_ASPECT_RATIO && values.len() == 2 {
                        // RawImageAspectRatio: format as "width:height" (reversed from storage order)
                        let formatted = format!("{}:{}", values[1], values[0]);
                        ExifValue::Ascii(formatted)
                    } else {
                        ExifValue::Short(values)
                    }
                }
                4 | 9 => {
                    // LONG (4) or SLONG (9)
                    let raw_value = if count == 1 {
                        value_offset
                    } else {
                        let offset = value_offset as usize;
                        if offset + 4 <= data.len() {
                            let mut cursor = Cursor::new(&data[offset..]);
                            cursor.read_u32::<LittleEndian>().unwrap_or(0)
                        } else {
                            continue;
                        }
                    };

                    // Handle compound binary tags that expand to multiple sub-tags
                    if tag_id == FUJI_FOCUS_SETTINGS {
                        // FocusSettings (0x102d) - extract bit fields
                        // FocusMode2: bits 0-3
                        let focus_mode2 = (raw_value & 0x0000000f) as u16;
                        let focus_mode2_str = match focus_mode2 {
                            0 => "AF-M",
                            1 => "AF-S",
                            2 => "AF-C",
                            _ => "Unknown",
                        };
                        tags.insert(
                            FUJI_FOCUS_MODE_2,
                            MakerNoteTag::new(
                                FUJI_FOCUS_MODE_2,
                                Some("FocusMode2"),
                                ExifValue::Ascii(focus_mode2_str.to_string()),
                            ),
                        );

                        // PreAF: bits 4-7 (mask 0x00f0)
                        let pre_af = ((raw_value >> 4) & 0x0f) as u16;
                        let pre_af_str = match pre_af {
                            0 => "Off",
                            1 => "On",
                            _ => "Unknown",
                        };
                        tags.insert(
                            FUJI_PRE_AF,
                            MakerNoteTag::new(
                                FUJI_PRE_AF,
                                Some("PreAF"),
                                ExifValue::Ascii(pre_af_str.to_string()),
                            ),
                        );

                        // AFAreaMode: bits 8-11 (mask 0x0f00)
                        let af_area_mode = ((raw_value >> 8) & 0x0f) as u16;
                        let af_area_mode_str = match af_area_mode {
                            0 => "Single Point",
                            1 => "Zone",
                            2 => "Wide/Tracking",
                            _ => "Unknown",
                        };
                        tags.insert(
                            FUJI_AF_AREA_MODE,
                            MakerNoteTag::new(
                                FUJI_AF_AREA_MODE,
                                Some("AFAreaMode"),
                                ExifValue::Ascii(af_area_mode_str.to_string()),
                            ),
                        );

                        // AFAreaPointSize: bits 12-15 (mask 0xf000)
                        let af_point_size = ((raw_value >> 12) & 0x0f) as u16;
                        let af_point_size_str = if af_point_size == 0 {
                            "n/a".to_string()
                        } else {
                            af_point_size.to_string()
                        };
                        tags.insert(
                            FUJI_AF_AREA_POINT_SIZE,
                            MakerNoteTag::new(
                                FUJI_AF_AREA_POINT_SIZE,
                                Some("AFAreaPointSize"),
                                ExifValue::Ascii(af_point_size_str),
                            ),
                        );

                        // AFAreaZoneSize: bits 16-23 (mask 0xff0000)
                        let af_zone_size = ((raw_value >> 16) & 0xff) as u16;
                        let af_zone_size_str = if af_zone_size == 0 {
                            "n/a".to_string()
                        } else {
                            // Decode as "width x height" per ExifTool formula
                            let w = af_zone_size & 0x0f;
                            let h = af_zone_size >> 5;
                            format!("{} x {}", w, h)
                        };
                        tags.insert(
                            FUJI_AF_AREA_ZONE_SIZE,
                            MakerNoteTag::new(
                                FUJI_AF_AREA_ZONE_SIZE,
                                Some("AFAreaZoneSize"),
                                ExifValue::Ascii(af_zone_size_str),
                            ),
                        );

                        continue; // Skip normal tag insertion
                    }

                    if tag_id == FUJI_AFC_SETTINGS {
                        // AFCSettings (0x102e) - extract bit fields
                        // AF-CTrackingSensitivity: bits 0-3 (values 0-4)
                        let tracking_sens = (raw_value & 0x000f) as u16;
                        tags.insert(
                            FUJI_AFC_TRACKING_SENS,
                            MakerNoteTag::new(
                                FUJI_AFC_TRACKING_SENS,
                                Some("AF-CTrackingSensitivity"),
                                ExifValue::Ascii(tracking_sens.to_string()),
                            ),
                        );

                        // AF-CSpeedTrackingSensitivity: bits 4-7 (values 0-2)
                        let speed_sens = ((raw_value >> 4) & 0x0f) as u16;
                        tags.insert(
                            FUJI_AFC_SPEED_TRACK_SENS,
                            MakerNoteTag::new(
                                FUJI_AFC_SPEED_TRACK_SENS,
                                Some("AF-CSpeedTrackingSensitivity"),
                                ExifValue::Ascii(speed_sens.to_string()),
                            ),
                        );

                        // AF-CZoneAreaSwitching: bits 8-11
                        let zone_switching = ((raw_value >> 8) & 0x0f) as u16;
                        let zone_switching_str = match zone_switching {
                            0 => "Front",
                            1 => "Auto",
                            2 => "Center",
                            _ => "Unknown",
                        };
                        tags.insert(
                            FUJI_AFC_ZONE_AREA_SWITCHING,
                            MakerNoteTag::new(
                                FUJI_AFC_ZONE_AREA_SWITCHING,
                                Some("AF-CZoneAreaSwitching"),
                                ExifValue::Ascii(zone_switching_str.to_string()),
                            ),
                        );

                        continue; // Skip normal tag insertion
                    }

                    // Decode specific LONG tags
                    let decoded = match tag_id {
                        FUJI_SHADOW_TONE | FUJI_HIGHLIGHT_TONE => {
                            // These are signed values
                            let signed_val = raw_value as i32;
                            Some(decode_shadow_highlight_tone_ext_exiftool(signed_val).to_string())
                        }
                        FUJI_LENS_MODULATION_OPTIMIZER => {
                            Some(decode_lens_modulation_optimizer_exiftool(raw_value).to_string())
                        }
                        FUJI_DIGITAL_ZOOM => {
                            // DigitalZoom = value / 8
                            let zoom = raw_value as f64 / 8.0;
                            Some(format!("{}", zoom))
                        }
                        FUJI_COLOR_CHROME_EFFECT | FUJI_COLOR_CHROME_FX_BLUE => {
                            // These can be LONG (int32s) on some cameras - use same decoder as SHORT
                            Some(decode_off_weak_strong_exiftool(raw_value as u16).to_string())
                        }
                        FUJI_FLICKER_REDUCTION => {
                            // Extract on/off from bits - ((val >> 8) & 0x0f) == 1 means On
                            let is_on = ((raw_value >> 8) & 0x0f) == 1;
                            let state = if is_on { "On" } else { "Off" };
                            Some(format!("{} (0x{:04x})", state, raw_value))
                        }
                        FUJI_COMPOSITE_IMAGE_MODE => {
                            Some(decode_composite_image_mode_exiftool(raw_value).to_string())
                        }
                        FUJI_GRAIN_EFFECT_ROUGHNESS => {
                            // Same decoder as ColorChromeEffect - uses int32s
                            Some(decode_off_weak_strong_exiftool(raw_value as u16).to_string())
                        }
                        _ => None,
                    };

                    if let Some(s) = decoded {
                        ExifValue::Ascii(s)
                    } else if count == 1 {
                        ExifValue::Long(vec![raw_value])
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

                            // Check for multi-value LONG/SLONG tags that need special formatting
                            if tag_id == FUJI_WHITE_BALANCE_FINE_TUNE && values.len() == 2 {
                                // WhiteBalanceFineTune: "Red +X, Blue +Y" format (stored as SLONG)
                                let red = values[0] as i32;
                                let blue = values[1] as i32;
                                let red_str = if red >= 0 {
                                    format!("+{}", red)
                                } else {
                                    format!("{}", red)
                                };
                                let blue_str = if blue >= 0 {
                                    format!("+{}", blue)
                                } else {
                                    format!("{}", blue)
                                };
                                ExifValue::Ascii(format!("Red {}, Blue {}", red_str, blue_str))
                            } else {
                                ExifValue::Long(values)
                            }
                        } else {
                            continue;
                        }
                    }
                }
                5 => {
                    // RATIONAL (unsigned)
                    let offset = value_offset as usize;
                    if offset + 8 <= data.len() {
                        let mut cursor = Cursor::new(&data[offset..]);
                        let num = cursor.read_u32::<LittleEndian>().unwrap_or(0);
                        let den = cursor.read_u32::<LittleEndian>().unwrap_or(1);
                        if den != 0 {
                            let val = num as f64 / den as f64;
                            // Format as integer if it's a whole number
                            if val == val.floor() {
                                ExifValue::Ascii(format!("{}", val as i64))
                            } else {
                                ExifValue::Ascii(format!("{}", val))
                            }
                        } else {
                            ExifValue::Rational(vec![(num, den)])
                        }
                    } else {
                        continue;
                    }
                }
                7 => {
                    // UNDEFINED - for Version tag (0x0000)
                    let offset = value_offset as usize;
                    if count <= 4 {
                        // Value is in the offset field (little-endian bytes)
                        let bytes = value_offset.to_le_bytes();
                        let s = String::from_utf8_lossy(&bytes[..count as usize]).to_string();
                        ExifValue::Ascii(s.trim_end_matches('\0').to_string())
                    } else if offset + count as usize <= data.len() {
                        let bytes = &data[offset..offset + count as usize];
                        let s = String::from_utf8_lossy(bytes).to_string();
                        ExifValue::Ascii(s.trim_end_matches('\0').to_string())
                    } else {
                        continue;
                    }
                }
                10 => {
                    // SRATIONAL (signed rational)
                    let offset = value_offset as usize;
                    if offset + 8 <= data.len() {
                        let mut cursor = Cursor::new(&data[offset..]);
                        let num = cursor.read_i32::<LittleEndian>().unwrap_or(0);
                        let den = cursor.read_i32::<LittleEndian>().unwrap_or(1);
                        if den != 0 {
                            let val = num as f64 / den as f64;
                            // RelativeExposure: apply log2 conversion like ExifTool
                            if tag_id == FUJI_RELATIVE_EXPOSURE {
                                let converted = if val > 0.0 {
                                    val.ln() / 2.0_f64.ln()
                                } else {
                                    0.0
                                };
                                // Format: 0 for zero, else +/- with 1 decimal
                                if converted == 0.0 {
                                    ExifValue::Ascii("0".to_string())
                                } else {
                                    ExifValue::Ascii(format!("{:+.1}", converted))
                                }
                            // Format as integer if it's a whole number
                            } else if val == val.floor() {
                                ExifValue::Ascii(format!("{}", val as i64))
                            } else {
                                ExifValue::Ascii(format!("{:.1}", val))
                            }
                        } else {
                            ExifValue::SRational(vec![(num, den)])
                        }
                    } else {
                        continue;
                    }
                }
                _ => {
                    // Unsupported type for now
                    continue;
                }
            };

            let tag_name = get_fuji_tag_name(tag_id);
            let tag = if let Some(name) = tag_name {
                MakerNoteTag::with_exiv2(
                    tag_id,
                    tag_name,
                    value.clone(),
                    value,
                    EXIV2_GROUP_FUJIFILM,
                    name,
                )
            } else {
                MakerNoteTag::new(tag_id, tag_name, value)
            };
            tags.insert(tag_id, tag);
        }
    }

    Ok(tags)
}
