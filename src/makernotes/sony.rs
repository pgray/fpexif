// makernotes/sony.rs - Sony maker notes parsing

use crate::data_types::{Endianness, ExifValue};
use crate::define_tag_decoder;
use crate::errors::ExifError;
use crate::makernotes::minolta;
use crate::makernotes::MakerNoteTag;
use std::collections::HashMap;

// exiv2 group names for Sony sub-IFDs
// Note: exiv2 uses Sony1 for older cameras (A100) and Sony2 for newer cameras (A200+)
// We default to Sony2 as it's more common in modern cameras
pub const EXIV2_GROUP_SONY1: &str = "Sony1";
pub const EXIV2_GROUP_SONY2: &str = "Sony2";
pub const EXIV2_GROUP_SONY1_CS: &str = "Sony1Cs";
pub const EXIV2_GROUP_SONY2_CS: &str = "Sony2Cs";
pub const EXIV2_GROUP_SONY1_CS2: &str = "Sony1Cs2";
pub const EXIV2_GROUP_SONY2_CS2: &str = "Sony2Cs2";
pub const EXIV2_GROUP_SONY_MISC1: &str = "SonyMisc1";
pub const EXIV2_GROUP_SONY_SINFO1: &str = "SonySInfo1";

/// Map Sony sub-IFD field names to exiv2 group and tag name
/// Returns (exiv2_group, exiv2_name) for the given field name and parent tag
/// Note: Most Sony cameras use Sony2/Sony2Cs groups (A200+), only A100 uses Sony1
pub fn get_exiv2_sony_subfield(
    parent_tag: u16,
    field_name: &str,
) -> Option<(&'static str, &'static str)> {
    match parent_tag {
        SONY_CAMERA_SETTINGS => match field_name {
            // Use Sony2Cs as most cameras are A200+
            "Sharpness" => Some((EXIV2_GROUP_SONY2_CS, "Sharpness")),
            "Contrast" => Some((EXIV2_GROUP_SONY2_CS, "Contrast")),
            "Saturation" => Some((EXIV2_GROUP_SONY2_CS, "Saturation")),
            "DriveMode" => Some((EXIV2_GROUP_SONY2_CS, "DriveMode")),
            "ExposureProgram" => Some((EXIV2_GROUP_SONY2_CS, "ExposureProgram")),
            "FocusMode" => Some((EXIV2_GROUP_SONY2_CS, "FocusMode")),
            "AFAreaMode" => Some((EXIV2_GROUP_SONY2_CS, "AFAreaMode")),
            "AFPointSelected" => Some((EXIV2_GROUP_SONY2_CS, "AFPointSelected")),
            "LocalAFAreaPoint" => Some((EXIV2_GROUP_SONY2_CS, "LocalAFAreaPoint")),
            "MeteringMode" => Some((EXIV2_GROUP_SONY2_CS, "MeteringMode")),
            "ISOSetting" => Some((EXIV2_GROUP_SONY2_CS, "ISOSetting")),
            "DynamicRangeOptimizer" => Some((EXIV2_GROUP_SONY2_CS, "DynamicRangeOptimizerMode")),
            "DynamicRangeOptimizerMode" => {
                Some((EXIV2_GROUP_SONY2_CS, "DynamicRangeOptimizerMode"))
            }
            "DynamicRangeOptimizerLevel" => {
                Some((EXIV2_GROUP_SONY2_CS, "DynamicRangeOptimizerLevel"))
            }
            "CreativeStyle" => Some((EXIV2_GROUP_SONY2_CS, "CreativeStyle")),
            "ColorMode" => Some((EXIV2_GROUP_SONY2_CS, "ColorMode")),
            "ColorSpace" => Some((EXIV2_GROUP_SONY2_CS, "ColorSpace")),
            "FlashControl" => Some((EXIV2_GROUP_SONY2_CS, "FlashControl")),
            "WhiteBalanceFineTune" => Some((EXIV2_GROUP_SONY2_CS, "WhiteBalanceFineTune")),
            "Brightness" => Some((EXIV2_GROUP_SONY2_CS, "Brightness")),
            "ZoneMatchingValue" => Some((EXIV2_GROUP_SONY2_CS, "ZoneMatchingValue")),
            "PrioritySetupShutterRelease" => {
                Some((EXIV2_GROUP_SONY2_CS, "PrioritySetupShutterRelease"))
            }
            "AFIlluminator" => Some((EXIV2_GROUP_SONY2_CS, "AFIlluminator")),
            "AFWithShutter" => Some((EXIV2_GROUP_SONY2_CS, "AFWithShutter")),
            "LongExposureNoiseReduction" => {
                Some((EXIV2_GROUP_SONY2_CS, "LongExposureNoiseReduction"))
            }
            "HighISONoiseReduction" => Some((EXIV2_GROUP_SONY2_CS, "HighISONoiseReduction")),
            "ImageStyle" => Some((EXIV2_GROUP_SONY2_CS, "ImageStyle")),
            "ImageStabilization" => Some((EXIV2_GROUP_SONY2_CS, "ImageStabilization")),
            "Rotation" => Some((EXIV2_GROUP_SONY2_CS, "Rotation")),
            "SonyImageSize" => Some((EXIV2_GROUP_SONY2_CS, "SonyImageSize")),
            "AspectRatio" => Some((EXIV2_GROUP_SONY2_CS, "AspectRatio")),
            "Quality" => Some((EXIV2_GROUP_SONY2_CS, "Quality")),
            "ExposureLevelIncrements" => Some((EXIV2_GROUP_SONY2_CS, "ExposureLevelIncrements")),
            _ => None,
        },
        SONY_SHOT_INFO => match field_name {
            "FaceInfoOffset" => Some((EXIV2_GROUP_SONY_SINFO1, "FaceInfoOffset")),
            "SonyDateTime" => Some((EXIV2_GROUP_SONY_SINFO1, "SonyDateTime")),
            "SonyImageHeight" => Some((EXIV2_GROUP_SONY_SINFO1, "SonyImageHeight")),
            "SonyImageWidth" => Some((EXIV2_GROUP_SONY_SINFO1, "SonyImageWidth")),
            "FacesDetected" => Some((EXIV2_GROUP_SONY_SINFO1, "FacesDetected")),
            "MetaVersion" => Some((EXIV2_GROUP_SONY_SINFO1, "MetaVersion")),
            _ => None,
        },
        _ => None,
    }
}

// Common Sony MakerNote tag IDs
pub const SONY_CAMERA_INFO: u16 = 0x0010;
pub const SONY_FOCUS_INFO: u16 = 0x0020;
pub const SONY_IMAGE_QUALITY: u16 = 0x0102;
pub const SONY_FLASH_EXPOSURE_COMP: u16 = 0x0104;
pub const SONY_TELECONVERTER: u16 = 0x0105;
pub const SONY_WHITE_BALANCE_FINE_TUNE: u16 = 0x0112;
pub const SONY_CAMERA_SETTINGS: u16 = 0x0114;
pub const SONY_WHITE_BALANCE: u16 = 0x0115;
pub const SONY_EXTRA_INFO: u16 = 0x0116;
pub const SONY_PRINT_IM: u16 = 0x0E00;
pub const SONY_MULTI_BURST_MODE: u16 = 0x1000;
pub const SONY_MULTI_BURST_IMAGE_WIDTH: u16 = 0x1001;
pub const SONY_MULTI_BURST_IMAGE_HEIGHT: u16 = 0x1002;
pub const SONY_PANORAMA: u16 = 0x1003;
pub const SONY_PREVIEW_IMAGE: u16 = 0x2001;
pub const SONY_RATING: u16 = 0x2002;
pub const SONY_CONTRAST: u16 = 0x2004;
pub const SONY_SATURATION: u16 = 0x2005;
pub const SONY_SHARPNESS: u16 = 0x2006;
pub const SONY_BRIGHTNESS: u16 = 0x2007;
pub const SONY_LONG_EXPOSURE_NOISE_REDUCTION: u16 = 0x2008;
pub const SONY_HIGH_ISO_NOISE_REDUCTION: u16 = 0x2009;
pub const SONY_HDR: u16 = 0x200A;
pub const SONY_MULTI_FRAME_NOISE_REDUCTION: u16 = 0x200B;
pub const SONY_PICTURE_EFFECT: u16 = 0x200E;
pub const SONY_SOFT_SKIN_EFFECT: u16 = 0x200F;
pub const SONY_VIGNETTING_CORRECTION: u16 = 0x2011;
pub const SONY_LATERAL_CHROMATIC_ABERRATION: u16 = 0x2012;
pub const SONY_DISTORTION_CORRECTION: u16 = 0x2013;
pub const SONY_WB_SHIFT_AB: u16 = 0x2014;
pub const SONY_WB_SHIFT_GM: u16 = 0x2015;
pub const SONY_AUTO_PORTRAIT_FRAMED: u16 = 0x2016;
pub const SONY_FOCUS_MODE: u16 = 0x201B;
pub const SONY_AF_POINT_SELECTED: u16 = 0x201E;
pub const SONY_SHOT_INFO: u16 = 0x3000;
pub const SONY_FILE_FORMAT: u16 = 0xB000;
pub const SONY_SONY_MODEL_ID: u16 = 0xB001;
pub const SONY_CREATIVE_STYLE: u16 = 0xB020;
pub const SONY_COLOR_TEMPERATURE: u16 = 0xB021;
pub const SONY_COLOR_COMPENSATION_FILTER: u16 = 0xB022;
pub const SONY_SCENE_MODE: u16 = 0xB023;
pub const SONY_ZONE_MATCHING: u16 = 0xB024;
pub const SONY_DYNAMIC_RANGE_OPTIMIZER: u16 = 0xB025;
pub const SONY_IMAGE_STABILIZATION: u16 = 0xB026;
pub const SONY_LENS_ID: u16 = 0xB027;
pub const SONY_MINOLTA_MAKER_NOTE: u16 = 0xB028;
pub const SONY_COLOR_MODE: u16 = 0xB029;
pub const SONY_LENS_SPEC: u16 = 0xB02A;
pub const SONY_FULL_IMAGE_SIZE: u16 = 0xB02B;
pub const SONY_PREVIEW_IMAGE_SIZE: u16 = 0xB02C;
pub const SONY_MACRO: u16 = 0xB040;
pub const SONY_EXPOSURE_MODE: u16 = 0xB041;
pub const SONY_FOCUS_MODE_2: u16 = 0xB042;
pub const SONY_AF_MODE: u16 = 0xB043;
pub const SONY_AF_ILLUMINATOR: u16 = 0xB044;
pub const SONY_QUALITY_2: u16 = 0xB047;
pub const SONY_FLASH_LEVEL: u16 = 0xB048;
pub const SONY_RELEASE_MODE: u16 = 0xB049;
pub const SONY_SEQUENCE_NUMBER: u16 = 0xB04A;
pub const SONY_ANTI_BLUR: u16 = 0xB04B;
pub const SONY_FOCUS_MODE_3: u16 = 0xB04E;
pub const SONY_DYNAMIC_RANGE_OPTIMIZER_2: u16 = 0xB04F;
pub const SONY_HIGH_ISO_NOISE_REDUCTION_2: u16 = 0xB050;
pub const SONY_INTELLIGENT_AUTO: u16 = 0xB052;
pub const SONY_WHITE_BALANCE_2: u16 = 0xB054;
// Additional Sony Tags for 50% coverage
pub const SONY_FLASH_ACTION: u16 = 0x2017;
pub const SONY_EFCS: u16 = 0x201A;
pub const SONY_AF_AREA_MODE_SETTING: u16 = 0x201C;
pub const SONY_FLEXIBLE_SPOT_POSITION: u16 = 0x201D;
pub const SONY_AF_POINTS_USED: u16 = 0x2020;
pub const SONY_AF_TRACKING: u16 = 0x2021;
pub const SONY_FOCAL_PLANE_AF_POINTS_USED: u16 = 0x2022;
pub const SONY_MULTI_FRAME_NR_EFFECT: u16 = 0x2023;
pub const SONY_WB_SHIFT_AB_GM_PRECISE: u16 = 0x2026;
pub const SONY_FOCUS_LOCATION: u16 = 0x2027;
pub const SONY_VARIABLE_LOW_PASS_FILTER: u16 = 0x2028;
pub const SONY_RAW_FILE_TYPE: u16 = 0x2029;
pub const SONY_PRIORITY_SET_IN_AWB: u16 = 0x202B;
pub const SONY_METERING_MODE_2: u16 = 0x202C;
pub const SONY_EXPOSURE_STANDARD_ADJUSTMENT: u16 = 0x202D;
pub const SONY_QUALITY_3: u16 = 0x202E;
pub const SONY_PIXEL_SHIFT_INFO: u16 = 0x202F;
pub const SONY_SERIAL_NUMBER: u16 = 0x2031;
pub const SONY_SHADOWS: u16 = 0x2032;
pub const SONY_HIGHLIGHTS: u16 = 0x2033;
pub const SONY_FADE: u16 = 0x2034;
pub const SONY_SHARPNESS_RANGE: u16 = 0x2035;
pub const SONY_CLARITY: u16 = 0x2036;
pub const SONY_FOCUS_FRAME_SIZE: u16 = 0x2037;
pub const SONY_JPEG_HEIF_SWITCH: u16 = 0x2039;
pub const SONY_FACE_DETECTION: u16 = 0x0021;
pub const SONY_SMILE_SHUTTER: u16 = 0x0022;
pub const SONY_FOCUS_DISTANCE: u16 = 0x0206;
pub const SONY_NOISE_REDUCTION_2: u16 = 0x200C;
pub const SONY_WB_RG_BG_LEVELS: u16 = 0x2024;

// CameraSettings sub-tag storage IDs (A200/A300/A350/A700/A850/A900)
// These are virtual tag IDs for sub-tags extracted from the CameraSettings binary blob
// Base offset 0xC000 to avoid collision with real Sony tags
pub const CS_DRIVE_MODE: u16 = 0xC004; // Offset 0x04 in CameraSettings
pub const CS_WHITE_BALANCE_FINE_TUNE: u16 = 0xC006; // Offset 0x06 in CameraSettings
pub const CS_FOCUS_MODE: u16 = 0xC010; // Offset 0x10 in CameraSettings
pub const CS_AF_AREA_MODE: u16 = 0xC011; // Offset 0x11 in CameraSettings
pub const CS_LOCAL_AF_AREA_POINT: u16 = 0xC012; // Offset 0x12 in CameraSettings
pub const CS_METERING_MODE: u16 = 0xC015; // Offset 0x15 in CameraSettings
pub const CS_ISO_SETTING: u16 = 0xC016; // Offset 0x16 in CameraSettings
pub const CS_DYNAMIC_RANGE_OPTIMIZER_MODE: u16 = 0xC017; // Offset 0x17 in CameraSettings
pub const CS_DYNAMIC_RANGE_OPTIMIZER_LEVEL: u16 = 0xC018; // Offset 0x18 in CameraSettings
pub const CS_CREATIVE_STYLE: u16 = 0xC01A; // Offset 0x1a in CameraSettings
pub const CS_SHARPNESS: u16 = 0xC01C; // Offset 0x1c in CameraSettings
pub const CS_CONTRAST: u16 = 0xC01D; // Offset 0x1d in CameraSettings
pub const CS_SATURATION: u16 = 0xC01E; // Offset 0x1e in CameraSettings
pub const CS_ZONE_MATCHING_VALUE: u16 = 0xC01F; // Offset 0x1f in CameraSettings
pub const CS_BRIGHTNESS: u16 = 0xC022; // Offset 0x22 in CameraSettings
pub const CS_FLASH_CONTROL: u16 = 0xC023; // Offset 0x23 in CameraSettings
pub const CS_PRIORITY_SETUP_SHUTTER_RELEASE: u16 = 0xC025; // Offset 0x25 in CameraSettings
pub const CS_AF_ILLUMINATOR: u16 = 0xC029; // Offset 0x29 in CameraSettings
pub const CS_AF_WITH_SHUTTER: u16 = 0xC02A; // Offset 0x2a in CameraSettings
pub const CS_LONG_EXPOSURE_NR: u16 = 0xC02B; // Offset 0x2b in CameraSettings
pub const CS_HIGH_ISO_NR: u16 = 0xC02C; // Offset 0x2c in CameraSettings
pub const CS_IMAGE_STYLE: u16 = 0xC02D; // Offset 0x2d in CameraSettings
pub const CS_EXPOSURE_PROGRAM: u16 = 0xC03C; // Offset 0x3c in CameraSettings
pub const CS_IMAGE_STABILIZATION: u16 = 0xC03D; // Offset 0x3d in CameraSettings
pub const CS_ROTATION: u16 = 0xC03F; // Offset 0x3f in CameraSettings
pub const CS_SONY_IMAGE_SIZE: u16 = 0xC054; // Offset 0x54 in CameraSettings
pub const CS_ASPECT_RATIO: u16 = 0xC055; // Offset 0x55 in CameraSettings
pub const CS_QUALITY: u16 = 0xC056; // Offset 0x56 in CameraSettings
pub const CS_EXPOSURE_LEVEL_INCREMENTS: u16 = 0xC058; // Offset 0x58 in CameraSettings

// MRWInfo tag for A100 and older cameras - contains MinoltaRaw RIF structure
pub const SONY_MRW_INFO: u16 = 0x7250;

// RIF (Requested Image Format) sub-tags - extracted from MRWInfo binary blob
pub const RIF_SATURATION: u16 = 0xD001; // Offset 1 in RIF structure
pub const RIF_CONTRAST: u16 = 0xD002; // Offset 2 in RIF structure
pub const RIF_SHARPNESS: u16 = 0xD003; // Offset 3 in RIF structure

// Encrypted subdirectory tags
pub const SONY_TAG_2010: u16 = 0x2010;
pub const SONY_TAG_9050: u16 = 0x9050;
pub const SONY_TAG_9400: u16 = 0x9400;
pub const SONY_TAG_9401: u16 = 0x9401;
pub const SONY_TAG_9402: u16 = 0x9402;
pub const SONY_TAG_9403: u16 = 0x9403;
pub const SONY_TAG_9404: u16 = 0x9404;
pub const SONY_TAG_9405: u16 = 0x9405;
pub const SONY_TAG_9406: u16 = 0x9406;

/// Get the name of a Sony MakerNote tag
pub fn get_sony_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        SONY_CAMERA_INFO => Some("CameraInfo"),
        SONY_FOCUS_INFO => Some("FocusInfo"),
        SONY_IMAGE_QUALITY => Some("ImageQuality"),
        SONY_FLASH_EXPOSURE_COMP => Some("FlashExposureComp"),
        SONY_TELECONVERTER => Some("Teleconverter"),
        SONY_WHITE_BALANCE => Some("WhiteBalance"),
        SONY_CAMERA_SETTINGS => Some("CameraSettings"),
        SONY_RATING => Some("Rating"),
        SONY_CONTRAST => Some("Contrast"),
        SONY_SATURATION => Some("Saturation"),
        SONY_SHARPNESS => Some("Sharpness"),
        SONY_BRIGHTNESS => Some("Brightness"),
        SONY_LONG_EXPOSURE_NOISE_REDUCTION => Some("LongExposureNoiseReduction"),
        SONY_HIGH_ISO_NOISE_REDUCTION => Some("HighISONoiseReduction"),
        SONY_HDR => Some("HDR"),
        SONY_MULTI_FRAME_NOISE_REDUCTION => Some("MultiFrameNoiseReduction"),
        SONY_PICTURE_EFFECT => Some("PictureEffect"),
        SONY_VIGNETTING_CORRECTION => Some("VignettingCorrection"),
        SONY_DISTORTION_CORRECTION => Some("DistortionCorrectionSetting"),
        SONY_FOCUS_MODE => Some("FocusMode"),
        SONY_AF_POINT_SELECTED => Some("AFPointSelected"),
        SONY_SONY_MODEL_ID => Some("SonyModelID"),
        SONY_CREATIVE_STYLE => Some("CreativeStyle"),
        SONY_COLOR_TEMPERATURE => Some("ColorTemperature"),
        SONY_SCENE_MODE => Some("SceneMode"),
        SONY_DYNAMIC_RANGE_OPTIMIZER => Some("DynamicRangeOptimizer"),
        SONY_IMAGE_STABILIZATION => Some("ImageStabilization"),
        SONY_LENS_ID => Some("LensID"),
        SONY_LENS_SPEC => Some("LensSpec"),
        SONY_MACRO => Some("Macro"),
        SONY_EXPOSURE_MODE => Some("ExposureMode"),
        SONY_AF_MODE => Some("AFMode"),
        SONY_QUALITY_2 => Some("JPEGQuality"),
        SONY_FLASH_LEVEL => Some("FlashLevel"),
        SONY_RELEASE_MODE => Some("ReleaseMode"),
        SONY_ANTI_BLUR => Some("AntiBlur"),
        SONY_INTELLIGENT_AUTO => Some("IntelligentAuto"),
        SONY_WHITE_BALANCE_2 => Some("WhiteBalance2"),
        SONY_FLASH_ACTION => Some("FlashAction"),
        SONY_EFCS => Some("ElectronicFrontCurtainShutter"),
        SONY_AF_AREA_MODE_SETTING => Some("AFAreaModeSetting"),
        SONY_FLEXIBLE_SPOT_POSITION => Some("FlexibleSpotPosition"),
        SONY_AF_POINTS_USED => Some("AFPointsUsed"),
        SONY_AF_TRACKING => Some("AFTracking"),
        SONY_FOCAL_PLANE_AF_POINTS_USED => Some("FocalPlaneAFPointsUsed"),
        SONY_MULTI_FRAME_NR_EFFECT => Some("MultiFrameNREffect"),
        SONY_WB_SHIFT_AB_GM_PRECISE => Some("WBShiftABGMPrecise"),
        SONY_FOCUS_LOCATION => Some("FocusLocation"),
        SONY_VARIABLE_LOW_PASS_FILTER => Some("VariableLowPassFilter"),
        SONY_RAW_FILE_TYPE => Some("RAWFileType"),
        SONY_PRIORITY_SET_IN_AWB => Some("PrioritySetInAWB"),
        SONY_METERING_MODE_2 => Some("MeteringMode2"),
        SONY_EXPOSURE_STANDARD_ADJUSTMENT => Some("ExposureStandardAdjustment"),
        SONY_QUALITY_3 => Some("Quality3"),
        SONY_PIXEL_SHIFT_INFO => Some("PixelShiftInfo"),
        SONY_SERIAL_NUMBER => Some("SerialNumber"),
        SONY_SHADOWS => Some("Shadows"),
        SONY_HIGHLIGHTS => Some("Highlights"),
        SONY_FADE => Some("Fade"),
        SONY_SHARPNESS_RANGE => Some("SharpnessRange"),
        SONY_CLARITY => Some("Clarity"),
        SONY_FOCUS_FRAME_SIZE => Some("FocusFrameSize"),
        SONY_JPEG_HEIF_SWITCH => Some("JPEGHEIFSwitch"),
        SONY_WHITE_BALANCE_FINE_TUNE => Some("WhiteBalanceFineTune"),
        SONY_SOFT_SKIN_EFFECT => Some("SoftSkinEffect"),
        SONY_LATERAL_CHROMATIC_ABERRATION => Some("LateralChromaticAberration"),
        SONY_WB_SHIFT_AB => Some("WBShiftAB"),
        SONY_WB_SHIFT_GM => Some("WBShiftGM"),
        SONY_AUTO_PORTRAIT_FRAMED => Some("AutoPortraitFramed"),
        SONY_COLOR_COMPENSATION_FILTER => Some("ColorCompensationFilter"),
        SONY_ZONE_MATCHING => Some("ZoneMatching"),
        SONY_COLOR_MODE => Some("ColorMode"),
        SONY_FULL_IMAGE_SIZE => Some("FullImageSize"),
        SONY_FILE_FORMAT => Some("FileFormat"),
        SONY_AF_ILLUMINATOR => Some("AFIlluminator"),
        SONY_FOCUS_MODE_2 => Some("FocusMode"),
        SONY_DYNAMIC_RANGE_OPTIMIZER_2 => Some("DynamicRangeOptimizer2"),
        SONY_HIGH_ISO_NOISE_REDUCTION_2 => Some("HighISONoiseReduction2"),
        SONY_FOCUS_MODE_3 => Some("FocusMode"),
        SONY_SEQUENCE_NUMBER => Some("SequenceNumber"),
        SONY_FACE_DETECTION => Some("FaceDetection"),
        SONY_SMILE_SHUTTER => Some("SmileShutter"),
        SONY_FOCUS_DISTANCE => Some("FocusDistance"),
        SONY_NOISE_REDUCTION_2 => Some("NoiseReduction2"),
        SONY_WB_RG_BG_LEVELS => Some("WBRGBGLevels"),
        // Synthetic Tag9050 subdirectory field IDs
        0x9000 => Some("SonyMaxAperture"),
        0x9001 => Some("SonyMinAperture"),
        0x9020 => Some("Shutter"),
        0x9031 => Some("FlashStatus"),
        0x9032 => Some("ShutterCount"),
        0x903A => Some("SonyExposureTime"),
        0x903C => Some("SonyFNumber"),
        0x903F => Some("ReleaseMode2"),
        0x907C => Some("InternalSerialNumber"),
        0x90F0 => Some("InternalSerialNumber"),
        0x9105 => Some("LensMount"),
        0x9106 => Some("LensFormat"),
        0x9107 => Some("LensType2"),
        0x9109 => Some("LensType"),
        // CameraSettings sub-tags (virtual IDs)
        CS_SHARPNESS => Some("Sharpness"),
        CS_CONTRAST => Some("Contrast"),
        CS_SATURATION => Some("Saturation"),
        // MRWInfo tag
        SONY_MRW_INFO => Some("MRWInfo"),
        // RIF sub-tags (from MRWInfo binary blob)
        RIF_SATURATION => Some("Saturation"),
        RIF_CONTRAST => Some("Contrast"),
        RIF_SHARPNESS => Some("Sharpness"),
        _ => None,
    }
}

/// Get Sony camera model name from model ID (tag 0xB001)
/// Based on ExifTool's SonyModelID database
pub fn get_sony_model_name(model_id: u16) -> Option<&'static str> {
    match model_id {
        // DSC compact cameras
        2 => Some("DSC-R1"),
        297 => Some("DSC-RX100"),
        298 => Some("DSC-RX1"),
        308 => Some("DSC-RX100M2"),
        309 => Some("DSC-RX10"),
        310 => Some("DSC-RX1R"),
        317 => Some("DSC-RX100M3"),
        341 => Some("DSC-RX100M4"),
        342 => Some("DSC-RX10M2"),
        344 => Some("DSC-RX1RM2"),
        355 => Some("DSC-RX10M3"),
        356 => Some("DSC-RX100M5"),
        364 => Some("DSC-RX0"),
        365 => Some("DSC-RX10M4"),
        366 => Some("DSC-RX100M6"),
        367 => Some("DSC-HX99"),
        369 => Some("DSC-RX100M5A"),
        372 => Some("DSC-RX0M2"),
        373 => Some("DSC-HX95"),
        374 => Some("DSC-RX100M7"),
        389 => Some("ZV-1F"),
        395 => Some("ZV-1M2"),
        // DSLR-A series
        256 => Some("DSLR-A100"),
        257 => Some("DSLR-A900"),
        258 => Some("DSLR-A700"),
        259 => Some("DSLR-A200"),
        260 => Some("DSLR-A350"),
        261 => Some("DSLR-A300"),
        263 => Some("DSLR-A380/A390"),
        264 => Some("DSLR-A330"),
        265 => Some("DSLR-A230"),
        266 => Some("DSLR-A290"),
        269 => Some("DSLR-A850"),
        273 => Some("DSLR-A550"),
        274 => Some("DSLR-A500"),
        275 => Some("DSLR-A450"),
        282 => Some("DSLR-A560"),
        283 => Some("DSLR-A580"),
        // SLT-A series
        280 => Some("SLT-A33"),
        281 => Some("SLT-A55 / SLT-A55V"),
        285 => Some("SLT-A35"),
        286 => Some("SLT-A65 / SLT-A65V"),
        287 => Some("SLT-A77 / SLT-A77V"),
        291 => Some("SLT-A37"),
        292 => Some("SLT-A57"),
        294 => Some("SLT-A99 / SLT-A99V"),
        303 => Some("SLT-A58"),
        // NEX series
        278 => Some("NEX-5"),
        279 => Some("NEX-3"),
        284 => Some("NEX-C3"),
        288 => Some("NEX-5N"),
        289 => Some("NEX-7"),
        290 => Some("NEX-VG20E"),
        293 => Some("NEX-F3"),
        295 => Some("NEX-6"),
        296 => Some("NEX-5R"),
        299 => Some("NEX-VG900"),
        300 => Some("NEX-VG30E"),
        305 => Some("NEX-3N"),
        307 => Some("NEX-5T"),
        // ILCE (Alpha) series
        302 => Some("ILCE-3000/3500"),
        306 => Some("ILCE-7"),
        311 => Some("ILCE-7R"),
        312 => Some("ILCE-6000"),
        313 => Some("ILCE-5000"),
        318 => Some("ILCE-7S"),
        339 => Some("ILCE-5100"),
        340 => Some("ILCE-7M2"),
        346 => Some("ILCE-QX1"),
        347 => Some("ILCE-7RM2"),
        350 => Some("ILCE-7SM2"),
        357 => Some("ILCE-6300"),
        358 => Some("ILCE-9"),
        360 => Some("ILCE-6500"),
        362 => Some("ILCE-7RM3"),
        363 => Some("ILCE-7M3"),
        371 => Some("ILCE-6400"),
        375 => Some("ILCE-7RM4"),
        376 => Some("ILCE-9M2"),
        378 => Some("ILCE-6600"),
        379 => Some("ILCE-6100"),
        381 => Some("ILCE-7C"),
        383 => Some("ILCE-7SM3"),
        384 => Some("ILCE-1"),
        386 => Some("ILCE-7RM3A"),
        387 => Some("ILCE-7RM4A"),
        388 => Some("ILCE-7M4"),
        390 => Some("ILCE-7RM5"),
        392 => Some("ILCE-9M3"),
        394 => Some("ILCE-6700"),
        396 => Some("ILCE-7CR"),
        397 => Some("ILCE-7CM2"),
        400 => Some("ILCE-1M2"),
        407 => Some("ILCE-7M5"),
        // ILCA (A-mount) series
        319 => Some("ILCA-77M2"),
        353 => Some("ILCA-68"),
        354 => Some("ILCA-99M2"),
        // Cinema/Pro Video
        385 => Some("ILME-FX3"),
        391 => Some("ILME-FX30"),
        406 => Some("ILME-FX2"),
        // ZV Vlog cameras
        380 => Some("ZV-1"),
        382 => Some("ZV-E10"),
        393 => Some("ZV-E1"),
        399 => Some("ZV-E10M2"),
        _ => None,
    }
}

/// Get Sony/Minolta lens name from lens type ID
/// Based on ExifTool's minoltaLensTypes and sonyLensTypes2 databases
pub fn get_sony_lens_name(lens_id: u32) -> Option<&'static str> {
    match lens_id {
        // Minolta AF lenses (inherited by Sony A-mount)
        0 => Some("Minolta AF 28-85mm F3.5-4.5"),
        1 => Some("Minolta AF 80-200mm F2.8 HS-APO G"),
        2 => Some("Minolta AF 28-70mm F2.8 G"),
        3 => Some("Minolta AF 28-80mm F4-5.6"),
        4 => Some("Minolta AF 85mm F1.4G"),
        5 => Some("Minolta AF 35-70mm F3.5-4.5"),
        6 => Some("Minolta AF 24-85mm F3.5-4.5"),
        7 => Some("Minolta AF 100-300mm F4.5-5.6 APO"),
        8 => Some("Minolta AF 70-210mm F4.5-5.6"),
        9 => Some("Minolta AF 50mm F3.5 Macro"),
        10 => Some("Minolta AF 28-105mm F3.5-4.5"),
        11 => Some("Minolta AF 300mm F4 HS-APO G"),
        12 => Some("Minolta AF 100mm F2.8 Soft Focus"),
        13 => Some("Minolta AF 75-300mm F4.5-5.6"),
        14 => Some("Minolta AF 100-400mm F4.5-6.7 APO"),
        15 => Some("Minolta AF 400mm F4.5 HS-APO G"),
        16 => Some("Minolta AF 17-35mm F3.5 G"),
        17 => Some("Minolta AF 20-35mm F3.5-4.5"),
        18 => Some("Minolta AF 28-80mm F3.5-5.6 II"),
        19 => Some("Minolta AF 35mm F1.4 G"),
        20 => Some("Minolta/Sony 135mm F2.8 STF"),
        22 => Some("Minolta AF 35-80mm F4-5.6 II"),
        23 => Some("Minolta AF 200mm F4 Macro APO G"),
        24 => Some("Minolta/Sony AF 24-105mm F3.5-4.5 (D)"),
        25 => Some("Minolta AF 100-300mm F4.5-5.6 APO (D)"),
        27 => Some("Minolta AF 85mm F1.4 G (D)"),
        28 => Some("Minolta/Sony AF 100mm F2.8 Macro (D)"),
        29 => Some("Minolta/Sony AF 75-300mm F4.5-5.6 (D)"),
        30 => Some("Minolta AF 28-80mm F3.5-5.6 (D)"),
        31 => Some("Minolta/Sony AF 50mm F2.8 Macro (D)"),
        32 => Some("Minolta/Sony AF 300mm F2.8 G APO"),
        33 => Some("Minolta/Sony AF 70-200mm F2.8 G"),
        35 => Some("Minolta AF 85mm F1.4 G (D) Limited"),
        36 => Some("Minolta AF 28-100mm F3.5-5.6 (D)"),
        38 => Some("Minolta AF 17-35mm F2.8-4 (D)"),
        39 => Some("Minolta AF 28-75mm F2.8 (D)"),
        40 => Some("Minolta/Sony AF DT 18-70mm F3.5-5.6 (D)"),
        41 => Some("Minolta/Sony AF DT 11-18mm F4.5-5.6 (D)"),
        42 => Some("Minolta/Sony AF DT 18-200mm F3.5-6.3 (D)"),
        43 => Some("Sony 35mm F1.4 G (SAL35F14G)"),
        44 => Some("Sony 50mm F1.4 (SAL50F14)"),
        45 => Some("Carl Zeiss Planar T* 85mm F1.4 ZA (SAL85F14Z)"),
        46 => Some("Carl Zeiss Vario-Sonnar T* DT 16-80mm F3.5-4.5 ZA (SAL1680Z)"),
        47 => Some("Carl Zeiss Sonnar T* 135mm F1.8 ZA (SAL135F18Z)"),
        48 => Some("Carl Zeiss Vario-Sonnar T* 24-70mm F2.8 ZA SSM (SAL2470Z)"),
        49 => Some("Sony DT 55-200mm F4-5.6 (SAL55200)"),
        50 => Some("Sony DT 18-250mm F3.5-6.3 (SAL18250)"),
        51 => Some("Sony DT 16-105mm F3.5-5.6 (SAL16105)"),
        52 => Some("Sony 70-300mm F4.5-5.6 G SSM (SAL70300G)"),
        53 => Some("Sony 70-400mm F4-5.6 G SSM (SAL70400G)"),
        54 => Some("Carl Zeiss Vario-Sonnar T* 16-35mm F2.8 ZA SSM (SAL1635Z)"),
        55 => Some("Sony DT 18-55mm F3.5-5.6 SAM (SAL1855)"),
        56 => Some("Sony DT 55-200mm F4-5.6 SAM (SAL55200-2)"),
        57 => Some("Sony DT 50mm F1.8 SAM (SAL50F18)"),
        58 => Some("Sony DT 30mm F2.8 Macro SAM (SAL30M28)"),
        59 => Some("Sony 28-75mm F2.8 SAM (SAL2875)"),
        60 => Some("Carl Zeiss Distagon T* 24mm F2 ZA SSM (SAL24F20Z)"),
        61 => Some("Sony 85mm F2.8 SAM (SAL85F28)"),
        62 => Some("Sony DT 35mm F1.8 SAM (SAL35F18)"),
        63 => Some("Sony DT 16-50mm F2.8 SSM (SAL1650)"),
        64 => Some("Sony 500mm F4 G SSM (SAL500F40G)"),
        65 => Some("Sony DT 18-135mm F3.5-5.6 SAM (SAL18135)"),
        66 => Some("Sony 300mm F2.8 G SSM II (SAL300F28G2)"),
        67 => Some("Sony 70-200mm F2.8 G SSM II (SAL70200G2)"),
        68 => Some("Sony DT 55-300mm F4.5-5.6 SAM (SAL55300)"),
        69 => Some("Sony 70-400mm F4-5.6 G SSM II (SAL70400G2)"),
        70 => Some("Carl Zeiss Planar T* 50mm F1.4 ZA SSM (SAL50F14Z)"),
        // Third-party and older Minolta lenses (high IDs from %minoltaLensTypes)
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
        // Sony E-mount lenses (use sonyLensTypes2 format with high IDs)
        // Note: ExifTool outputs lens names without (SELxxxx) suffixes
        32784 => Some("Sony E 16mm F2.8"),
        32785 => Some("Sony E 18-55mm F3.5-5.6 OSS"),
        32786 => Some("Sony E 55-210mm F4.5-6.3 OSS"),
        32787 => Some("Sony E 18-200mm F3.5-6.3 OSS"),
        32788 => Some("Sony E 30mm F3.5 Macro"),
        32789 => Some("Sony E 24mm F1.8 ZA"),
        32790 => Some("Sony E 50mm F1.8 OSS"),
        32791 => Some("Sony E 16-50mm F3.5-5.6 PZ OSS"),
        32792 => Some("Sony E 10-18mm F4 OSS"),
        32793 => Some("Sony E PZ 18-105mm F4 G OSS"),
        32794 => Some("Sony E 20mm F2.8"),
        32795 => Some("Sony E 35mm F1.8 OSS"),
        32796 => Some("Sony E PZ 18-200mm F3.5-6.3 OSS"),
        32797 => Some("Sony FE 35mm F2.8 ZA"),
        32798 => Some("Sony FE 24-70mm F4 ZA OSS"),
        32799 => Some("Sony FE 55mm F1.8 ZA"),
        32800 => Some("Sony FE 70-200mm F4 G OSS"),
        32801 => Some("Sony FE 28-70mm F3.5-5.6 OSS"),
        32802 => Some("Sony FE 16-35mm F4 ZA OSS"),
        32803 => Some("Sony FE 90mm F2.8 Macro G OSS"),
        32807 => Some("Sony E 18-200mm F3.5-6.3 OSS LE"),
        32808 => Some("Sony E 50mm F1.8 OSS"),
        32813 => Some("Sony FE 28mm F2"),
        32814 => Some("Sony FE 35mm F1.4 ZA"),
        32815 => Some("Sony FE 24-240mm F3.5-6.3 OSS"),
        32816 => Some("Sony FE 28-135mm F4 G PZ OSS"),
        32817 => Some("Sony FE PZ 28-135mm F4 G OSS"),
        32820 => Some("Sony FE 21mm F2.8"),
        32821 => Some("Sony FE 16mm F3.5 Fisheye"),
        32826 => Some("Sony FE 85mm F1.4 GM"),
        32827 => Some("Sony FE 50mm F1.4 ZA"),
        32829 => Some("Sony FE 70-300mm F4.5-5.6 G OSS"),
        32830 => Some("Sony FE 100mm F2.8 STF GM OSS"),
        32831 => Some("Sony FE 50mm F2.8 Macro"),
        32832 => Some("Sony FE 85mm F1.8"),
        33072 => Some("Sony FE 70-200mm F2.8 GM OSS"),
        33073 => Some("Sony FE 24-70mm F2.8 GM"),
        33076 => Some("Sony FE 100-400mm F4.5-5.6 GM OSS"),
        33077 => Some("Sony FE 12-24mm F4 G"),
        33079 => Some("Sony FE 16-35mm F2.8 GM"),
        33080 => Some("Sony FE 400mm F2.8 GM OSS"),
        33081 => Some("Sony FE 24mm F1.4 GM"),
        33082 => Some("Sony FE 135mm F1.8 GM"),
        33083 => Some("Sony FE 200-600mm F5.6-6.3 G OSS"),
        33084 => Some("Sony FE 600mm F4 GM OSS"),
        33085 => Some("Sony FE 20mm F1.8 G"),
        33086 => Some("Sony FE 35mm F1.8"),
        33088 => Some("Sony FE 12-24mm F2.8 GM"),
        33089 => Some("Sony FE 50mm F1.2 GM"),
        33090 => Some("Sony FE 14mm F1.8 GM"),
        33091 => Some("Sony FE 35mm F1.4 GM"),
        33092 => Some("Sony FE 24mm F2.8 G"),
        33093 => Some("Sony FE 40mm F2.5 G"),
        33094 => Some("Sony FE 50mm F2.5 G"),
        33095 => Some("Sony FE 70-200mm F2.8 GM OSS II"),
        33096 => Some("Sony FE 24-70mm F2.8 GM II"),
        33097 => Some("Sony FE 16-35mm F2.8 GM II"),
        // 65535 = No lens or E-mount/T-mount/other non-A-mount lens
        65535 => Some("E-Mount, T-Mount, Other Lens or no lens"),
        _ => None,
    }
}

/// Read u16 with given endianness
fn read_u16(data: &[u8], endian: Endianness) -> u16 {
    match endian {
        Endianness::Little => u16::from_le_bytes([data[0], data[1]]),
        Endianness::Big => u16::from_be_bytes([data[0], data[1]]),
    }
}

/// Read u32 with given endianness
fn read_u32(data: &[u8], endian: Endianness) -> u32 {
    match endian {
        Endianness::Little => u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
        Endianness::Big => u32::from_be_bytes([data[0], data[1], data[2], data[3]]),
    }
}

/// Read i32 with given endianness
fn read_i32(data: &[u8], endian: Endianness) -> i32 {
    match endian {
        Endianness::Little => i32::from_le_bytes([data[0], data[1], data[2], data[3]]),
        Endianness::Big => i32::from_be_bytes([data[0], data[1], data[2], data[3]]),
    }
}

/// Decipher Sony encrypted data (Tag9050, Tag2010, Tag94xx)
/// Sony uses a simple substitution cipher based on cubic modular arithmetic
/// Reference: exiv2/src/sonymn_int.cpp sonyTagCipher()
fn decipher_sony_data(data: &[u8]) -> Vec<u8> {
    // Build decryption lookup table: code[(i³) % 249] = i for i in 0..249
    let mut code = [0u8; 256];
    for i in 0u32..249 {
        let idx = ((i * i * i) % 249) as usize;
        code[idx] = i as u8;
    }
    // Values 249-255 stay the same
    for i in 249u8..=255 {
        code[i as usize] = i;
    }

    // Apply substitution to each byte
    data.iter().map(|&b| code[b as usize]).collect()
}

/// Decode CreativeStyle value (tag 0xB020) - ExifTool format
/// Note: exiv2 uses string-based lookup, not integer codes
pub fn decode_creative_style_exiftool(value: u16) -> &'static str {
    match value {
        0 => "None",
        1 => "Standard",
        2 => "Vivid",
        3 => "Portrait",
        4 => "Landscape",
        5 => "Sunset",
        6 => "Night",
        8 => "B&W",
        9 => "Clear",
        10 => "Deep",
        11 => "Light",
        12 => "Autumn",
        13 => "Neutral",
        14 => "Sepia",
        15 => "Night View/Portrait",
        16 => "Soft",
        17 => "Spring",
        18 => "Summer",
        19 => "Winter",
        20 => "Portrait (Adobe RGB)",
        21 => "Landscape (Adobe RGB)",
        22 => "Night Scene",
        23 => "Flowers",
        24 => "Saloon",
        25 => "Illustration",
        26 => "High Contrast Mono",
        27 => "Retro Photo",
        _ => "Unknown",
    }
}
// decode_creative_style_exiv2 - exiv2 uses string-based lookup (StringTagDetails)

// ExposureMode (tag 0xB041): Sony.pm / sonymn_int.cpp
// ExifTool tag 0xB041 mapping (NOT SceneMode!)
define_tag_decoder! {
    exposure_mode,
    exiftool: {
        0 => "Program AE",
        1 => "Portrait",
        2 => "Beach",
        3 => "Sports",
        4 => "Snow",
        5 => "Landscape",
        6 => "Auto",
        7 => "Aperture-priority AE",
        8 => "Shutter speed priority AE",
        9 => "Night Scene / Twilight",
        10 => "Hi-Speed Shutter",
        11 => "Twilight Portrait",
        12 => "Soft Snap/Portrait",
        13 => "Fireworks",
        14 => "Smile Shutter",
        15 => "Manual",
        18 => "High Sensitivity",
        19 => "Macro",
        20 => "Advanced Sports Shooting",
        29 => "Underwater",
        33 => "Food",
        34 => "Sweep Panorama",
        35 => "Handheld Night Shot",
        36 => "Anti Motion Blur",
        37 => "Pet",
        38 => "Backlight Correction HDR",
        39 => "Superior Auto",
        40 => "Background Defocus",
        41 => "Soft Skin",
        42 => "3D Image",
        65535 => "n/a",
    },
    exiv2: {
        0 => "Program AE",
        1 => "Portrait",
        2 => "Beach",
        3 => "Sports",
        4 => "Snow",
        5 => "Landscape",
        6 => "Auto",
        7 => "Aperture-priority AE",
        8 => "Shutter speed priority AE",
        9 => "Night Scene/Twilight",
        10 => "Hi-Speed Shutter",
        11 => "Twilight Portrait",
        12 => "Soft Snap/Portrait",
        13 => "Fireworks",
        14 => "Smile Shutter",
        15 => "Manual",
        35 => "Slow Shutter",
        36 => "Macro",
        37 => "Landscape",
        38 => "Sunset",
        39 => "Continuous Priority AE",
        40 => "Sweep Panorama",
        0xffff => "n/a",
    }
}

// AFMode (tag 0xB043): Sony.pm / sonymn_int.cpp
// ExifTool uses different decoding for older vs newer DSC models
// We use the general mapping that covers most cases
define_tag_decoder! {
    af_mode,
    exiftool: {
        0 => "Default",
        1 => "Multi",
        2 => "Center",
        3 => "Spot",
        4 => "Flexible Spot",
        6 => "Touch",
        10 => "Selective (for Miniature effect)",
        14 => "Tracking",
        15 => "Face Tracking",
        255 => "Manual",
        65535 => "n/a",
    },
    exiv2: {
        0 => "Default",
        1 => "Multi",
        2 => "Center",
        3 => "Spot",
        4 => "Flexible Spot",
        6 => "Touch",
        14 => "Tracking",
        15 => "Face Detected",
        0xffff => "n/a",
    }
}

// DynamicRangeOptimizer (tag 0xB025): Sony.pm / sonymn_int.cpp
define_tag_decoder! {
    dynamic_range_optimizer,
    both: {
        0 => "Off",
        1 => "Standard",
        2 => "Advanced Auto",
        3 => "Auto",
        8 => "Advanced Lv1",
        9 => "Advanced Lv2",
        10 => "Advanced Lv3",
        11 => "Advanced Lv4",
        12 => "Advanced Lv5",
        16 => "Lv1",
        17 => "Lv2",
        18 => "Lv3",
        19 => "Lv4",
        20 => "Lv5",
    }
}

// FocusMode (tag 0x201B): Sony.pm / sonymn_int.cpp sonyFocusMode2
define_tag_decoder! {
    focus_mode,
    both: {
        0 => "Manual",
        2 => "AF-S",
        3 => "AF-C",
        4 => "AF-A",
        6 => "DMF",
        7 => "AF-D",
    }
}

// FocusMode2 (tag 0xB042): Sony.pm for older DSC models
define_tag_decoder! {
    focus_mode2,
    both: {
        0 => "Manual",
        1 => "AF-S",
        2 => "AF-C",
        4 => "Permanent-AF",
        65535 => "n/a",
    }
}

// ImageStabilization (tag 0xB026): Sony.pm / sonymn_int.cpp
define_tag_decoder! {
    image_stabilization,
    exiftool: {
        0 => "Off",
        1 => "On",
        2 => "On (2)",
        3 => "On (Shooting)",
    },
    exiv2: {
        0 => "Off",
        1 => "On",
        0xffff => "n/a",
    }
}

// ZoneMatching (tag 0xB024): Sony.pm
define_tag_decoder! {
    zone_matching,
    type: u32,
    both: {
        0 => "ISO Setting Used",
        1 => "High Key",
        2 => "Low Key",
    }
}

// ColorMode (tag 0xB029): Sony.pm uses Minolta::sonyColorMode
define_tag_decoder! {
    color_mode,
    type: u32,
    both: {
        0 => "Standard",
        1 => "Vivid",
        2 => "Portrait",
        3 => "Landscape",
        4 => "Sunset",
        5 => "Night View/Portrait",
        6 => "B&W",
        7 => "Adobe RGB",
        12 => "Neutral",
        13 => "Clear",
        14 => "Deep",
        15 => "Light",
        16 => "Autumn Leaves",
        17 => "Sepia",
        18 => "FL",
        19 => "Vivid 2",
        20 => "IN",
        21 => "SH",
        22 => "FL2",
        23 => "FL3",
    }
}

/// Decode Sony FileFormat from 4 bytes (tag 0xB000)
/// Format: [major, minor, patch, 0] -> "ARW major.minor" or "ARW major.minor.patch"
pub fn decode_file_format(bytes: &[u8]) -> Option<String> {
    if bytes.len() < 4 {
        return None;
    }
    let key = format!("{} {} {} {}", bytes[0], bytes[1], bytes[2], bytes[3]);
    match key.as_str() {
        "0 0 0 2" => Some("JPEG".to_string()),
        "1 0 0 0" => Some("SR2".to_string()),
        "2 0 0 0" => Some("ARW 1.0".to_string()),
        "3 0 0 0" => Some("ARW 2.0".to_string()),
        "3 1 0 0" => Some("ARW 2.1".to_string()),
        "3 2 0 0" => Some("ARW 2.2".to_string()),
        "3 3 0 0" => Some("ARW 2.3".to_string()),
        "3 3 1 0" => Some("ARW 2.3.1".to_string()),
        "3 3 2 0" => Some("ARW 2.3.2".to_string()),
        "3 3 3 0" => Some("ARW 2.3.3".to_string()),
        "3 3 5 0" => Some("ARW 2.3.5".to_string()),
        _ => None,
    }
}

// ImageQuality (tag 0x0102): Sony.pm / sonymn_int.cpp
define_tag_decoder! {
    image_quality,
    both: {
        0 => "RAW",
        1 => "Super Fine",
        2 => "Fine",
        3 => "Standard",
        4 => "Economy",
        5 => "Extra Fine",
        6 => "RAW + JPEG/HEIF",
        7 => "Compressed RAW",
        8 => "Compressed RAW + JPEG",
        9 => "Light",
    }
}

// WhiteBalance (tag 0x0115): Sony.pm / sonymn_int.cpp
define_tag_decoder! {
    white_balance,
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
        0x80 => "Underwater",
    }
}

// LongExposureNoiseReduction (tag 0x2008): Sony.pm / sonymn_int.cpp
define_tag_decoder! {
    long_exposure_noise_reduction,
    type: u32,
    both: {
        0 => "Off",
        1 => "On (unused)",
        0x10001 => "On (dark subtracted)",
        0xffff0000 => "Off (65535)",
        0xffff0001 => "On (65535)",
        0xffffffff => "n/a",
    }
}

// HighISONoiseReduction (tag 0x2009): Sony.pm / sonymn_int.cpp
define_tag_decoder! {
    high_iso_noise_reduction,
    both: {
        0 => "Off",
        1 => "Low",
        2 => "Normal",
        3 => "High",
        256 => "Auto",
        65535 => "n/a",
    }
}

// SceneMode (tag 0xB023): Sony.pm / sonymn_int.cpp
define_tag_decoder! {
    scene_mode,
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
        18 => "Sweep Panorama",
        19 => "Handheld Night Shot",
        20 => "Anti Motion Blur",
        21 => "Cont. Priority AE",
        22 => "Auto+",
        23 => "3D Sweep Panorama",
        24 => "Superior Auto",
        25 => "High Sensitivity",
        26 => "Fireworks",
        27 => "Food",
        28 => "Pet",
        33 => "HDR",
        0xffff => "n/a",
    }
}

// Adjustment values (tags 0x2004/0x2005/0x2006): Contrast/Saturation/Sharpness
// ExifTool PrintConv: '$val > 0 ? "+$val" : $val' (shows raw number, + prefix for positive)
define_tag_decoder! {
    adjustment,
    type: i32,
    both: {
        -3 => "-3",
        -2 => "-2",
        -1 => "-1",
        0 => "0",
        1 => "+1",
        2 => "+2",
        3 => "+3",
    }
}

/// Decode ColorTemperature value (tag 0xB021) - ExifTool format
/// Returns None if value should be displayed as-is (actual temperature)
/// Note: exiv2 uses identical values - no separate version needed
pub fn decode_color_temperature_exiftool(value: u32) -> Option<&'static str> {
    match value {
        0 => Some("Auto"),
        _ => None, // Display actual temperature value
    }
}
// decode_color_temperature_exiv2 - same as exiftool, no separate function needed

// Teleconverter (tag 0x0105): Sony.pm / sonymn_int.cpp
define_tag_decoder! {
    teleconverter,
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

// PictureEffect (tag 0x200E): Sony.pm / sonymn_int.cpp
define_tag_decoder! {
    picture_effect,
    exiftool: {
        0 => "Off",
        1 => "Toy Camera",
        2 => "Pop Color",
        3 => "Posterization",
        4 => "Posterization B/W",
        5 => "Retro Photo",
        6 => "Soft High Key",
        7 => "Partial Color (red)",
        8 => "Partial Color (green)",
        9 => "Partial Color (blue)",
        10 => "Partial Color (yellow)",
        13 => "High Contrast Monochrome",
        16 => "Toy Camera (normal)",
        17 => "Toy Camera (cool)",
        18 => "Toy Camera (warm)",
        19 => "Toy Camera (green)",
        20 => "Toy Camera (magenta)",
        32 => "Soft Focus (low)",
        33 => "Soft Focus",
        34 => "Soft Focus (high)",
        36 => "Miniature (auto)",
        37 => "Miniature (top)",
        38 => "Miniature (middle horizontal)",
        39 => "Miniature (bottom)",
        40 => "Miniature (left)",
        41 => "Miniature (middle vertical)",
        42 => "Miniature (right)",
        48 => "HDR Painting (low)",
        49 => "HDR Painting",
        50 => "HDR Painting (high)",
        64 => "Rich-tone Monochrome",
        80 => "Watercolor",
        96 => "Illustration (low)",
        97 => "Illustration",
        98 => "Illustration (high)",
    },
    exiv2: {
        0 => "Off",
        1 => "Toy Camera",
        2 => "Pop Color",
        3 => "Posterization",
        4 => "Posterization B/W",
        5 => "Retro Photo",
        6 => "Soft High Key",
        7 => "Partial Color (red)",
        8 => "Partial Color (green)",
        9 => "Partial Color (blue)",
        10 => "Partial Color (yellow)",
        13 => "High Contrast Monochrome",
        16 => "Toy Camera (normal)",
        17 => "Toy Camera (cool)",
        18 => "Toy Camera (warm)",
        19 => "Toy Camera (green)",
        20 => "Toy Camera (magenta)",
        32 => "Soft Focus (low)",
        33 => "Soft Focus",
        34 => "Soft Focus (high)",
        48 => "Miniature (auto)",
        49 => "Miniature (top)",
        50 => "Miniature (middle horizontal)",
        51 => "Miniature (bottom)",
        52 => "Miniature (left)",
        53 => "Miniature (middle vertical)",
        54 => "Miniature (right)",
        64 => "HDR Painting (low)",
        65 => "HDR Painting",
        66 => "HDR Painting (high)",
        80 => "Rich-tone Monochrome",
        96 => "Watercolor",
        97 => "Watercolor 2",
        112 => "Illustration (low)",
        113 => "Illustration",
        114 => "Illustration (high)",
    }
}

// VignettingCorrection (tag 0x2011): Sony.pm / sonymn_int.cpp
define_tag_decoder! {
    vignetting_correction,
    type: u32,
    both: {
        0 => "Off",
        2 => "Auto",
        0xffffffff => "n/a",
    }
}

// DistortionCorrection (tag 0x2013): Sony.pm / sonymn_int.cpp
define_tag_decoder! {
    distortion_correction,
    type: u32,
    both: {
        0 => "Off",
        2 => "Auto",
        0xffffffff => "n/a",
    }
}

// ReleaseMode (tag 0xB049): Sony.pm / sonymn_int.cpp
define_tag_decoder! {
    release_mode,
    both: {
        0 => "Normal",
        2 => "Continuous",
        5 => "Exposure Bracketing",
        6 => "White Balance Bracketing",
        8 => "DRO Bracketing",
        65535 => "n/a",
    }
}

/// Decode MultiFrameNoiseReduction value (tag 0x200B) - ExifTool format
pub fn decode_multi_frame_noise_reduction_exiftool(value: u32) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        255 => "n/a",
        _ => "Unknown",
    }
}

/// Decode MultiFrameNoiseReduction value (tag 0x200B) - exiv2 format
/// Note: exiv2 uses 256 for n/a instead of 255
pub fn decode_multi_frame_noise_reduction_exiv2(value: u32) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        256 => "n/a",
        _ => "Unknown",
    }
}

// IntelligentAuto (tag 0xB052): Sony.pm / sonymn_int.cpp
define_tag_decoder! {
    intelligent_auto,
    both: {
        0 => "Off",
        1 => "On",
        2 => "Advanced",
    }
}

// HDR (tag 0x200A): Sony.pm / sonymn_int.cpp
define_tag_decoder! {
    hdr,
    both: {
        0x0 => "Off",
        0x01 => "Auto",
        0x10 => "1.0 EV",
        0x11 => "1.5 EV",
        0x12 => "2.0 EV",
        0x13 => "2.5 EV",
        0x14 => "3.0 EV",
        0x15 => "3.5 EV",
        0x16 => "4.0 EV",
        0x17 => "4.5 EV",
        0x18 => "5.0 EV",
        0x19 => "5.5 EV",
        0x1a => "6.0 EV",
    }
}

/// Decode HDR image status (second u16 of HDR tag 0x200A)
/// From Sony.pm line 999-1003
pub fn decode_hdr_image_status(value: u16) -> &'static str {
    match value {
        0 => "Uncorrected image",
        1 => "HDR image (good)",
        2 => "HDR image (fail 1)",
        3 => "HDR image (fail 2)",
        _ => "Unknown",
    }
}

/// Format complete HDR value (tag 0x200A) - stored as int32u, read as two int16u
/// Output format: "Off; Uncorrected image"
pub fn format_hdr_value(value: u32) -> String {
    let low = (value & 0xFFFF) as u16; // HDR level
    let high = ((value >> 16) & 0xFFFF) as u16; // Image status
    format!(
        "{}; {}",
        decode_hdr_exiftool(low),
        decode_hdr_image_status(high)
    )
}

/// Decode Quality value (tag 0x0102 and 0xB047) - ExifTool format
/// Note: This overlaps with decode_image_quality but with more values
pub fn decode_quality_exiftool(value: u32) -> &'static str {
    match value {
        0 => "RAW",
        1 => "Super Fine",
        2 => "Fine",
        3 => "Standard",
        4 => "Economy",
        5 => "Extra Fine",
        6 => "RAW + JPEG/HEIF",
        7 => "Compressed RAW",
        8 => "Compressed RAW + JPEG",
        9 => "Light",
        0xffffffff => "n/a",
        _ => "Unknown",
    }
}

/// Decode Quality value (tag 0xB047) - exiv2 format (sonyJPEGQuality)
/// Note: exiv2 uses different mapping for tag 0xB047 only
pub fn decode_quality_exiv2(value: u32) -> &'static str {
    match value {
        0 => "Standard",
        1 => "Fine",
        2 => "Extra Fine",
        0xffff => "n/a",
        _ => "Unknown",
    }
}

/// Decode JPEGQuality value (tag 0xB047) - ExifTool format
/// This is the same as decode_quality_exiv2, included for clarity
pub fn decode_jpeg_quality_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Standard",
        1 => "Fine",
        2 => "Extra Fine",
        65535 => "n/a",
        _ => "Unknown",
    }
}

/// Format FlashLevel or FlashExposureComp value as signed rational string
/// ExifTool formats these as "-2/3", "+1/3", "Normal", etc.
pub fn format_flash_comp(value: i16) -> String {
    match value {
        -32768 => "Low".to_string(),
        -9 => "-9/3".to_string(),
        -8 => "-8/3".to_string(),
        -7 => "-7/3".to_string(),
        -6 => "-6/3".to_string(),
        -5 => "-5/3".to_string(),
        -4 => "-4/3".to_string(),
        -3 => "-3/3".to_string(),
        -2 => "-2/3".to_string(),
        -1 => "-1/3".to_string(),
        0 => "Normal".to_string(),
        1 => "+1/3".to_string(),
        2 => "+2/3".to_string(),
        3 => "+3/3".to_string(),
        4 => "+4/3".to_string(),
        5 => "+5/3".to_string(),
        6 => "+6/3".to_string(),
        9 => "+9/3".to_string(),
        128 => "n/a".to_string(),
        32767 => "High".to_string(),
        _ => format!("{}", value),
    }
}

/// Decode a single BCD byte to its decimal value
/// e.g., 0x55 -> 55 (not 85)
fn decode_bcd_byte(byte: u8) -> u16 {
    let high = (byte >> 4) as u16;
    let low = (byte & 0x0f) as u16;
    high * 10 + low
}

/// Decode two BCD bytes to a focal length value
/// e.g., [0x00, 0x55] -> 55, [0x02, 0x00] -> 200
fn decode_bcd_focal(high_byte: u8, low_byte: u8) -> u16 {
    decode_bcd_byte(high_byte) * 100 + decode_bcd_byte(low_byte)
}

/// Format LensSpec byte array as a readable string matching ExifTool's format
/// LensSpec is 8 bytes in ExifTool format:
/// - byte 0: flags1 (lens mount type: DT=0x01, FE=0x02, E=0x03, etc.)
/// - bytes 1-2: short focal length (as hex digits forming decimal number)
/// - bytes 3-4: long focal length (as hex digits forming decimal number)
/// - byte 5: max aperture at short focal (divided by 10)
/// - byte 6: max aperture at long focal (divided by 10)
/// - byte 7: flags2 (lens features: SSM=0x01, SAM=0x02, ZA=0x04, G=0x08, etc.)
pub fn format_lens_spec(bytes: &[u8]) -> String {
    if bytes.len() < 8 {
        return format!("{:?}", bytes);
    }

    let flags1 = bytes[0];
    // Focal lengths are stored as BCD (Binary Coded Decimal)
    // Each byte's hex representation forms decimal digits
    // e.g., 0x00 0x55 -> "0055" -> 55mm (not 0x0055 = 85)
    let short_focal = decode_bcd_focal(bytes[1], bytes[2]);
    let long_focal = decode_bcd_focal(bytes[3], bytes[4]);
    // Apertures are also BCD and divided by 10
    let max_ap_short = decode_bcd_byte(bytes[5]) as f32 / 10.0;
    let max_ap_long = decode_bcd_byte(bytes[6]) as f32 / 10.0;
    let flags2 = bytes[7];

    // Check if all zeros (unknown lens)
    if short_focal == 0 && long_focal == 0 && bytes[5] == 0 && bytes[6] == 0 {
        return format!(
            "Unknown ({:02x} {} {} {} {} {:02x})",
            flags1, bytes[1], bytes[2], bytes[3], bytes[4], flags2
        );
    }

    // Build lens type prefix from flags1
    // ExifTool order: DT/FE/E comes before PZ (e.g., "E PZ" not "PZ E")
    let mut prefix = String::new();
    match flags1 & 0x03 {
        0x01 => prefix.push_str("DT "),
        0x02 => prefix.push_str("FE "),
        0x03 => prefix.push_str("E "),
        _ => {}
    }
    if flags1 & 0x40 != 0 {
        prefix.push_str("PZ ");
    }

    // Build focal length string
    let focal_str = if short_focal == long_focal || long_focal == 0 {
        format!("{}mm", short_focal)
    } else {
        format!("{}-{}mm", short_focal, long_focal)
    };

    // Build aperture string
    let aperture_str = if max_ap_short == max_ap_long || max_ap_long == 0.0 {
        if max_ap_short == max_ap_short.floor() {
            format!("F{}", max_ap_short as u32)
        } else {
            format!("F{}", max_ap_short)
        }
    } else {
        let short_str = if max_ap_short == max_ap_short.floor() {
            format!("{}", max_ap_short as u32)
        } else {
            format!("{}", max_ap_short)
        };
        let long_str = if max_ap_long == max_ap_long.floor() {
            format!("{}", max_ap_long as u32)
        } else {
            format!("{}", max_ap_long)
        };
        format!("F{}-{}", short_str, long_str)
    };

    // Build suffix from flags2
    let mut suffix = String::new();
    match flags2 & 0x0c {
        0x04 => suffix.push_str(" ZA"),
        0x08 => suffix.push_str(" G"),
        _ => {}
    }
    match flags2 & 0x03 {
        0x01 => suffix.push_str(" SSM"),
        0x02 => suffix.push_str(" SAM"),
        _ => {}
    }
    if flags1 & 0x80 != 0 {
        suffix.push_str(" OSS");
    }
    if flags1 & 0x20 != 0 {
        suffix.push_str(" LE");
    }

    format!("{}{} {}{}", prefix, focal_str, aperture_str, suffix)
}

/// Decode FlashAction value (tag 0x2017) - exiv2 format
pub fn decode_flash_action_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Did not fire",
        1 => "Flash fired",
        2 => "External flash fired",
        3 => "Wireless controlled flash fired",
        _ => "Unknown",
    }
}

/// Decode FlashAction value (tag 0x2017) - ExifTool format
pub fn decode_flash_action_exiftool(value: u32) -> &'static str {
    match value {
        0 => "Did not fire",
        1 => "Flash Fired",
        2 => "External Flash Fired",
        3 => "Wireless Controlled Flash Fired",
        _ => "Unknown",
    }
}

/// Decode AFTracking value (tag 0x2021) - exiv2 format
pub fn decode_af_tracking_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "Face tracking",
        2 => "Lock On AF",
        _ => "Unknown",
    }
}

/// Decode AFTracking value (tag 0x2021) - ExifTool format
pub fn decode_af_tracking_exiftool(value: u16) -> &'static str {
    decode_af_tracking_exiv2(value) // Same as exiv2
}

/// Decode MultiFrameNREffect value (tag 0x2023) - exiv2 format
pub fn decode_multi_frame_nr_effect_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Normal",
        1 => "High",
        _ => "Unknown",
    }
}

/// Decode MultiFrameNREffect value (tag 0x2023) - ExifTool format
/// Same as exiv2 format
pub fn decode_multi_frame_nr_effect_exiftool(value: u32) -> &'static str {
    match value {
        0 => "Normal",
        1 => "High",
        _ => "Unknown",
    }
}

/// Decode SequenceNumber value (tag 0xB04A) - ExifTool format
/// 0 = Single, other values are passed through as numbers
pub fn decode_sequence_number_exiftool(value: u16) -> String {
    match value {
        0 => "Single".to_string(),
        65535 => "n/a".to_string(),
        _ => value.to_string(),
    }
}

/// Decode ElectronicFrontCurtainShutter value (tag 0x201A) - ExifTool format
pub fn decode_efcs_exiftool(value: u32) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        _ => "Unknown",
    }
}

/// Decode RAWFileType value (tag 0x2029) - exiftool format
pub fn decode_raw_file_type_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Compressed RAW",
        1 => "Uncompressed RAW",
        2 => "Lossless Compressed RAW",
        3 => "Compressed RAW (HQ)",
        0xffff => "n/a",
        _ => "Unknown",
    }
}

/// Decode PrioritySetInAWB value (tag 0x202B) - exiftool format
pub fn decode_priority_set_in_awb_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Standard",
        1 => "Ambience",
        2 => "White",
        _ => "Unknown",
    }
}

/// Decode MeteringMode2 value (tag 0x202C) - exiftool format
pub fn decode_metering_mode2_exiftool(value: u16) -> &'static str {
    match value {
        0x100 => "Multi-segment",
        0x200 => "Center-weighted average",
        0x301 => "Spot (Standard)",
        0x302 => "Spot (Large)",
        0x400 => "Average",
        0x500 => "Highlight",
        _ => "Unknown",
    }
}

/// Decode JPEGHEIFSwitch value (tag 0x2039) - exiv2 format
pub fn decode_jpeg_heif_switch_exiv2(value: u16) -> &'static str {
    match value {
        0 => "JPEG",
        1 => "HEIF",
        0xffff => "n/a",
        _ => "Unknown",
    }
}

/// Decode FocusMode3 value (tag 0xB04E) - exiftool format
/// From Sony.pm: valid for DSC-HX9V generation and newer
pub fn decode_focus_mode3_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Manual",
        2 => "AF-S",
        3 => "AF-C",
        5 => "Semi-manual",
        6 => "DMF",
        _ => "Unknown",
    }
}

/// Decode FocusMode3 value (tag 0xB04E) - exiv2 format
pub fn decode_focus_mode3_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Manual",
        2 => "AF-S",
        3 => "AF-C",
        5 => "Semi-manual",
        6 => "DMF",
        _ => "Unknown",
    }
}

/// Decode SoftSkinEffect value (tag 0x200F) - exiv2 format
pub fn decode_soft_skin_effect_exiv2(value: u32) -> &'static str {
    match value {
        0 => "Off",
        1 => "Low",
        2 => "Mid",
        3 => "High",
        0xffffffff => "n/a",
        _ => "Unknown",
    }
}

/// Decode AutoPortraitFramed value (tag 0x2016) - exiv2 format
pub fn decode_auto_portrait_framed_exiv2(value: u16) -> &'static str {
    match value {
        0 => "No",
        1 => "Yes",
        _ => "Unknown",
    }
}

/// Decode AutoPortraitFramed value (tag 0x2016) - ExifTool format
/// Same values as exiv2
pub fn decode_auto_portrait_framed_exiftool(value: u16) -> &'static str {
    decode_auto_portrait_framed_exiv2(value)
}

/// Decode AFIlluminator value (tag 0xB044) - exiftool format
pub fn decode_af_illuminator_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "Auto",
        0xffff => "n/a",
        _ => "Unknown",
    }
}

/// Decode AFIlluminator value (tag 0xB044) - exiv2 format
pub fn decode_af_illuminator_exiv2(value: u16) -> &'static str {
    decode_af_illuminator_exiftool(value)
}

/// Decode Macro value (tag 0xB040) - exiftool format
pub fn decode_macro_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        2 => "Close Focus",
        0xffff => "n/a",
        _ => "Unknown",
    }
}

/// Decode Macro value (tag 0xB040) - exiv2 format
pub fn decode_macro_exiv2(value: u16) -> &'static str {
    decode_macro_exiftool(value)
}

/// Decode DynamicRangeOptimizer2 value (tag 0xB04F) - exiv2 format
pub fn decode_dynamic_range_optimizer2_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "Standard",
        2 => "Plus",
        _ => "Unknown",
    }
}

/// Decode HighISONoiseReduction2 value (tag 0xB050) - exiv2 format
pub fn decode_high_iso_noise_reduction2_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Normal",
        1 => "High",
        2 => "Low",
        3 => "Off",
        0xffff => "n/a",
        _ => "Unknown",
    }
}

/// Decode HighISONoiseReduction2 value (tag 0xB050) - ExifTool format
pub fn decode_high_iso_noise_reduction2_exiftool(value: u16) -> &'static str {
    // Same as exiv2 format for this tag
    decode_high_iso_noise_reduction2_exiv2(value)
}

/// Check if all values in a slice are zero
fn all_zeros_u8(vals: &[u8]) -> bool {
    vals.iter().all(|&v| v == 0)
}

/// Check if all values in a slice are zero
fn all_zeros_u16(vals: &[u16]) -> bool {
    vals.iter().all(|&v| v == 0)
}

/// Decode LateralChromaticAberration value (tag 0x2012) - exiv2 format
pub fn decode_lateral_chromatic_aberration_exiv2(value: u32) -> &'static str {
    match value {
        0 => "Off",
        2 => "Auto",
        0xffffffff => "n/a",
        _ => "Unknown",
    }
}

/// Decode LateralChromaticAberration value (tag 0x2012) - ExifTool format
/// Same as exiv2 format
pub fn decode_lateral_chromatic_aberration_exiftool(value: u32) -> &'static str {
    decode_lateral_chromatic_aberration_exiv2(value)
}

/// Decode AFAreaModeSetting value (tag 0x201C) - exiv2 format Set1
pub fn decode_af_area_mode_setting_set1_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Wide",
        4 => "Local",
        8 => "Zone",
        9 => "Spot",
        _ => "Unknown",
    }
}

/// Decode AFAreaModeSetting value (tag 0x201C) - ExifTool format
/// Uses NEX/ILCE mapping as default since most modern Sony cameras use this
pub fn decode_af_area_mode_setting_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Wide",
        1 => "Center",
        3 => "Flexible Spot",
        4 => "Flexible Spot (LA-EA4)",
        9 => "Center (LA-EA4)",
        11 => "Zone",
        12 => "Expanded Flexible Spot",
        _ => "Unknown",
    }
}

/// Decode AntiBlur value (tag 0xB04B) - ExifTool format
pub fn decode_anti_blur_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On (Continuous)",
        2 => "On (Shooting)",
        65535 => "n/a",
        _ => "Unknown",
    }
}

/// Decode WhiteBalance2 value (tag 0xB054) - exiv2 format
pub fn decode_white_balance2_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        4 => "Manual",
        5 => "Daylight",
        6 => "Cloudy",
        7 => "Cool White Fluorescent",
        8 => "Day White Fluorescent",
        9 => "Daylight Fluorescent",
        10 => "Incandescent2",
        11 => "Warm White Fluorescent",
        14 => "Incandescent",
        15 => "Flash",
        17 => "Underwater 1 (Blue Water)",
        18 => "Underwater 2 (Green Water)",
        19 => "Underwater Auto",
        _ => "Unknown",
    }
}

// ============================================================================
// Tag9050 subdirectory decoders (encrypted subdirectory at offset 0x9050)
// Reference: exiftool/lib/Image/ExifTool/Sony.pm Tag9050a/b/c/d tables
// ============================================================================

/// Decode FlashStatus value (Tag9050 offset 0x0031)
pub fn decode_flash_status_exiftool(value: u8) -> &'static str {
    match value {
        0 => "No Flash present",
        2 => "Flash Inhibited",
        64 => "Built-in Flash present",
        65 => "Built-in Flash Fired",
        66 => "Built-in Flash Inhibited",
        128 => "External Flash present",
        129 => "External Flash Fired",
        _ => "Unknown",
    }
}

/// Decode LensMount value (Tag9050 offset 0x0105)
pub fn decode_lens_mount_exiftool(value: u8) -> &'static str {
    match value {
        0 => "Unknown",
        1 => "A-mount",
        2 => "E-mount",
        _ => "Unknown",
    }
}

/// Decode LensFormat value (Tag9050 offset 0x0106)
pub fn decode_lens_format_exiftool(value: u8) -> &'static str {
    match value {
        0 => "Unknown",
        1 => "APS-C",
        2 => "Full-frame",
        _ => "Unknown",
    }
}

/// Decode ReleaseMode2 value (Tag9050 offset 0x003f or 0x0067)
pub fn decode_release_mode2_exiftool(value: u8) -> &'static str {
    match value {
        0 => "Normal",
        1 => "Continuous",
        2 => "Continuous - Exposure Bracketing",
        3 => "Continuous - White Balance Bracketing",
        5 => "Continuous - Burst",
        6 => "Single Frame - Capture During Movie",
        9 => "Continuous - HDR",
        12 => "Continuous - Speed/Advance Priority AE",
        18 => "Continuous - Soft High-key",
        19 => "Continuous - Soft Low-key",
        20 => "Continuous - Toy Camera",
        21 => "Continuous - Pop Color",
        23 => "Continuous - High Contrast Mono",
        27 => "Continuous - Auto HDR",
        28 => "Single Frame - Focus Stacking",
        29 => "Continuous - Focus Stacking",
        30 => "Continuous - Fast Burst",
        146 => "Single Frame - Movie Capture",
        _ => "Unknown",
    }
}

/// Format adjustment value like ExifTool: positive values get '+' prefix
/// PrintConv: '$val > 0 ? "+$val" : $val'
fn format_adjustment_value(val: i32) -> String {
    if val > 0 {
        format!("+{}", val)
    } else {
        val.to_string()
    }
}

// CameraSettings decode functions (for tag 0x0114)
// Based on ExifTool Sony.pm CameraSettings table

fn decode_drive_mode_cs_exiftool(value: u16) -> &'static str {
    match value & 0xff {
        0x01 => "Single Frame",
        0x02 => "Continuous High",
        0x12 => "Continuous Low",
        0x04 => "Self-timer 10 sec",
        0x05 => "Self-timer 2 sec, Mirror Lock-up",
        0x06 => "Single-frame Bracketing",
        0x07 => "Continuous Bracketing",
        0x18 => "White Balance Bracketing Low",
        0x28 => "White Balance Bracketing High",
        0x19 => "D-Range Optimizer Bracketing Low",
        0x29 => "D-Range Optimizer Bracketing High",
        0x0a => "Remote Commander",
        0x0b => "Mirror Lock-up",
        _ => "Unknown",
    }
}

fn decode_focus_mode_cs_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Manual",
        1 => "AF-S",
        2 => "AF-C",
        3 => "AF-A",
        4 => "DMF",
        _ => "Unknown",
    }
}

fn decode_af_area_mode_cs_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Wide",
        1 => "Local",
        2 => "Spot",
        _ => "Unknown",
    }
}

fn decode_local_af_area_point_cs_exiftool(value: u16) -> &'static str {
    match value {
        1 => "Center",
        2 => "Top",
        3 => "Upper-right",
        4 => "Right",
        5 => "Lower-right",
        6 => "Bottom",
        7 => "Lower-left",
        8 => "Left",
        9 => "Upper-left",
        10 => "Far Right",
        11 => "Far Left",
        _ => "Unknown",
    }
}

fn decode_metering_mode_cs_exiftool(value: u16) -> &'static str {
    match value {
        1 => "Multi-segment",
        2 => "Center-weighted Average",
        4 => "Spot",
        _ => "Unknown",
    }
}

fn decode_dro_mode_cs_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "Standard",
        2 => "Advanced Auto",
        3 => "Advanced Level",
        8 => "Advanced Lv1",
        9 => "Advanced Lv2",
        10 => "Advanced Lv3",
        11 => "Advanced Lv4",
        12 => "Advanced Lv5",
        _ => "Unknown",
    }
}

fn decode_creative_style_cs_exiftool(value: u16) -> &'static str {
    match value {
        1 => "Standard",
        2 => "Vivid",
        3 => "Portrait",
        4 => "Landscape",
        5 => "Sunset",
        6 => "Night View/Portrait",
        8 => "Black & White",
        9 => "Adobe RGB",
        11 => "Neutral",
        12 => "Clear",
        13 => "Deep",
        14 => "Light",
        15 => "Autumn Leaves",
        16 => "Sepia",
        _ => "Unknown",
    }
}

fn decode_flash_control_cs_exiftool(value: u16) -> &'static str {
    match value {
        0 => "ADI",
        1 => "Pre-flash TTL",
        2 => "Manual",
        _ => "Unknown",
    }
}

fn decode_priority_setup_cs_exiftool(value: u16) -> &'static str {
    match value {
        0 => "AF",
        1 => "Release",
        _ => "Unknown",
    }
}

fn decode_af_illuminator_cs_exiftool(value: u16) -> &'static str {
    // ExifTool: 0 => 'Auto', 1 => 'Off'
    match value {
        0 => "Auto",
        1 => "Off",
        _ => "Unknown",
    }
}

fn decode_on_off_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        _ => "Unknown",
    }
}

fn decode_af_with_shutter_exiftool(value: u16) -> &'static str {
    // ExifTool AFWithShutter: 0 => 'On', 1 => 'Off' (inverted from typical on/off)
    match value {
        0 => "On",
        1 => "Off",
        _ => "Unknown",
    }
}

fn decode_high_iso_nr_cs_exiftool(value: u16) -> &'static str {
    // ExifTool CameraSettings HighISONoiseReduction (different from tag 0x2009)
    match value {
        0 => "Normal",
        1 => "Low",
        2 => "High",
        3 => "Off",
        _ => "Unknown",
    }
}

/// Decode ISOSetting from CameraSettings
/// Formula: $val ? exp(($val/8-6)*log(2))*100 : $val
fn decode_iso_setting_cs(value: u16) -> String {
    if value == 0 {
        "Auto".to_string()
    } else {
        let iso = ((value as f64 / 8.0 - 6.0) * std::f64::consts::LN_2).exp() * 100.0;
        format!("{:.0}", iso)
    }
}

fn decode_image_style_cs_exiftool(value: u16) -> &'static str {
    match value {
        1 => "Standard",
        2 => "Vivid",
        3 => "Portrait",
        4 => "Landscape",
        5 => "Sunset",
        7 => "Night View/Portrait",
        8 => "Black & White",
        9 => "Adobe RGB",
        11 => "Neutral",
        129 => "StyleBox1",
        130 => "StyleBox2",
        131 => "StyleBox3",
        _ => "Unknown",
    }
}

fn decode_exposure_program_cs_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        1 => "Manual",
        2 => "Program AE",
        3 => "Aperture-priority AE",
        4 => "Shutter speed priority AE",
        8 => "Program Shift A",
        9 => "Program Shift S",
        16 => "Portrait",
        17 => "Sports",
        18 => "Sunset",
        19 => "Night Portrait",
        20 => "Landscape",
        21 => "Macro",
        35 => "Auto (No Flash)",
        _ => "Unknown",
    }
}

fn decode_rotation_cs_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Horizontal (normal)",
        1 => "Rotate 90 CW",
        2 => "Rotate 270 CW",
        _ => "Unknown",
    }
}

fn decode_sony_image_size_cs_exiftool(value: u16) -> &'static str {
    match value {
        1 => "Large",
        2 => "Medium",
        3 => "Small",
        _ => "Unknown",
    }
}

fn decode_aspect_ratio_cs_exiftool(value: u16) -> &'static str {
    match value {
        1 => "3:2",
        2 => "16:9",
        _ => "Unknown",
    }
}

fn decode_quality_cs_exiftool(value: u16) -> &'static str {
    match value {
        0 => "RAW",
        2 => "CRAW",
        34 => "RAW + JPEG",
        35 => "CRAW + JPEG",
        16 => "Extra Fine",
        32 => "Fine",
        48 => "Standard",
        _ => "Unknown",
    }
}

fn decode_exposure_level_increments_cs_exiftool(value: u16) -> &'static str {
    match value {
        33 => "1/3 EV",
        50 => "1/2 EV",
        _ => "Unknown",
    }
}

/// Helper to insert a CameraSettings tag with exiv2 mapping
fn insert_cs_tag(
    tags: &mut HashMap<u16, MakerNoteTag>,
    tag_id: u16,
    name: &'static str,
    value: ExifValue,
    raw_value: ExifValue,
) {
    let tag = if let Some((exiv2_group, exiv2_name)) =
        get_exiv2_sony_subfield(SONY_CAMERA_SETTINGS, name)
    {
        MakerNoteTag::with_exiv2(
            tag_id,
            Some(name),
            value,
            raw_value,
            exiv2_group,
            exiv2_name,
        )
    } else {
        MakerNoteTag::new(tag_id, Some(name), value)
    };
    tags.insert(tag_id, tag);
}

/// Read a big-endian u16 from CameraSettings data at the given word index
fn read_cs_u16(data: &[u8], word_index: usize) -> Option<u16> {
    let byte_offset = word_index * 2;
    if byte_offset + 2 <= data.len() {
        Some(u16::from_be_bytes([
            data[byte_offset],
            data[byte_offset + 1],
        ]))
    } else {
        None
    }
}

/// Read a big-endian i16 from CameraSettings data at the given word index
fn read_cs_i16(data: &[u8], word_index: usize) -> Option<i16> {
    read_cs_u16(data, word_index).map(|v| v as i16)
}

/// Parse CameraSettings binary data (tag 0x0114)
/// Extracts all supported fields from the binary blob
///
/// Supports:
/// - CameraSettings: 280 bytes (A200/A300/A350/A700) or 364 bytes (A850/A900)
/// - CameraSettings2: 332 bytes (A230/A290/A330/A380/A390)
///
/// CameraSettings3 (1536/2048 bytes) used by A450/A500/A550/A560/A580/A33/A35/A55/NEX
/// has a completely different format (int8u) and is not supported.
fn parse_camera_settings(data: &[u8], _endian: Endianness, tags: &mut HashMap<u16, MakerNoteTag>) {
    // CameraSettings is an array of int16u values (big-endian)
    // CameraSettings: 280 or 364 bytes (A200/A300/A350/A700/A850/A900)
    // CameraSettings2: 332 bytes (A230/A290/A330/A380/A390) - different offsets
    // CameraSettings3: 1536 or 2048 bytes (int8u format) - not supported

    let data_len = data.len();
    let is_cs1 = data_len == 280 || data_len == 364;
    let is_cs2 = data_len == 332;

    if !is_cs1 && !is_cs2 {
        // CameraSettings3 or unknown format - skip
        return;
    }

    // For CameraSettings (CS1): offsets are direct word indices
    // For CameraSettings2 (CS2): some offsets are shifted

    // DriveMode - offset 0x04 (CS1) or 0x04 (CS2)
    if let Some(raw) = read_cs_u16(data, 0x04) {
        let decoded = decode_drive_mode_cs_exiftool(raw);
        insert_cs_tag(
            tags,
            CS_DRIVE_MODE,
            "DriveMode",
            ExifValue::Ascii(decoded.to_string()),
            ExifValue::Short(vec![raw]),
        );
    }

    // WhiteBalanceFineTune - offset 0x06 (CS1) or 0x06 (CS2)
    if let Some(raw) = read_cs_i16(data, 0x06) {
        // ValueConv: $val > 128 ? $val - 256 : $val (but it's already signed)
        insert_cs_tag(
            tags,
            CS_WHITE_BALANCE_FINE_TUNE,
            "WhiteBalanceFineTune",
            ExifValue::Ascii(raw.to_string()),
            ExifValue::SShort(vec![raw]),
        );
    }

    // FocusMode - offset 0x10 (CS1) or 0x0f (CS2)
    let focus_mode_idx = if is_cs1 { 0x10 } else { 0x0f };
    if let Some(raw) = read_cs_u16(data, focus_mode_idx) {
        let decoded = decode_focus_mode_cs_exiftool(raw);
        insert_cs_tag(
            tags,
            CS_FOCUS_MODE,
            "FocusMode",
            ExifValue::Ascii(decoded.to_string()),
            ExifValue::Short(vec![raw]),
        );
    }

    // AFAreaMode - offset 0x11 (CS1) or 0x10 (CS2)
    let af_area_idx = if is_cs1 { 0x11 } else { 0x10 };
    if let Some(raw) = read_cs_u16(data, af_area_idx) {
        let decoded = decode_af_area_mode_cs_exiftool(raw);
        insert_cs_tag(
            tags,
            CS_AF_AREA_MODE,
            "AFAreaMode",
            ExifValue::Ascii(decoded.to_string()),
            ExifValue::Short(vec![raw]),
        );
    }

    // LocalAFAreaPoint - offset 0x12 (CS1) or 0x11 (CS2)
    let local_af_idx = if is_cs1 { 0x12 } else { 0x11 };
    if let Some(raw) = read_cs_u16(data, local_af_idx) {
        let decoded = decode_local_af_area_point_cs_exiftool(raw);
        insert_cs_tag(
            tags,
            CS_LOCAL_AF_AREA_POINT,
            "LocalAFAreaPoint",
            ExifValue::Ascii(decoded.to_string()),
            ExifValue::Short(vec![raw]),
        );
    }

    // MeteringMode - offset 0x15 (CS1) or 0x12 (CS2)
    let metering_idx = if is_cs1 { 0x15 } else { 0x12 };
    if let Some(raw) = read_cs_u16(data, metering_idx) {
        let decoded = decode_metering_mode_cs_exiftool(raw);
        insert_cs_tag(
            tags,
            CS_METERING_MODE,
            "MeteringMode",
            ExifValue::Ascii(decoded.to_string()),
            ExifValue::Short(vec![raw]),
        );
    }

    // ISOSetting - offset 0x16 (CS1) or 0x13 (CS2)
    let iso_idx = if is_cs1 { 0x16 } else { 0x13 };
    if let Some(raw) = read_cs_u16(data, iso_idx) {
        let decoded = decode_iso_setting_cs(raw);
        insert_cs_tag(
            tags,
            CS_ISO_SETTING,
            "ISOSetting",
            ExifValue::Ascii(decoded),
            ExifValue::Short(vec![raw]),
        );
    }

    // DynamicRangeOptimizerMode - offset 0x18 (CS1) or 0x16 (CS2)
    let dro_mode_idx = if is_cs1 { 0x18 } else { 0x16 };
    if let Some(raw) = read_cs_u16(data, dro_mode_idx) {
        let decoded = decode_dro_mode_cs_exiftool(raw);
        insert_cs_tag(
            tags,
            CS_DYNAMIC_RANGE_OPTIMIZER_MODE,
            "DynamicRangeOptimizerMode",
            ExifValue::Ascii(decoded.to_string()),
            ExifValue::Short(vec![raw]),
        );
    }

    // DynamicRangeOptimizerLevel - offset 0x19 (CS1) or 0x17 (CS2)
    let dro_level_idx = if is_cs1 { 0x19 } else { 0x17 };
    if let Some(raw) = read_cs_u16(data, dro_level_idx) {
        insert_cs_tag(
            tags,
            CS_DYNAMIC_RANGE_OPTIMIZER_LEVEL,
            "DynamicRangeOptimizerLevel",
            ExifValue::Ascii(raw.to_string()),
            ExifValue::Short(vec![raw]),
        );
    }

    // CreativeStyle - offset 0x1a (CS1) or 0x17 (CS2)
    let creative_idx = if is_cs1 { 0x1a } else { 0x17 };
    if let Some(raw) = read_cs_u16(data, creative_idx) {
        let decoded = decode_creative_style_cs_exiftool(raw);
        insert_cs_tag(
            tags,
            CS_CREATIVE_STYLE,
            "CreativeStyle",
            ExifValue::Ascii(decoded.to_string()),
            ExifValue::Short(vec![raw]),
        );
    }

    // Sharpness - offset 0x1c (CS1) or 0x19 (CS2)
    let sharpness_idx = if is_cs1 { 0x1c } else { 0x19 };
    if let Some(raw) = read_cs_u16(data, sharpness_idx) {
        let adjusted = (raw as i32) - 10;
        insert_cs_tag(
            tags,
            CS_SHARPNESS,
            "Sharpness",
            ExifValue::Ascii(format_adjustment_value(adjusted)),
            ExifValue::Short(vec![raw]),
        );
    }

    // Contrast - offset 0x1d (CS1) or 0x1a (CS2)
    let contrast_idx = if is_cs1 { 0x1d } else { 0x1a };
    if let Some(raw) = read_cs_u16(data, contrast_idx) {
        let adjusted = (raw as i32) - 10;
        insert_cs_tag(
            tags,
            CS_CONTRAST,
            "Contrast",
            ExifValue::Ascii(format_adjustment_value(adjusted)),
            ExifValue::Short(vec![raw]),
        );
    }

    // Saturation - offset 0x1e (CS1) or 0x1b (CS2)
    let saturation_idx = if is_cs1 { 0x1e } else { 0x1b };
    if let Some(raw) = read_cs_u16(data, saturation_idx) {
        let adjusted = (raw as i32) - 10;
        insert_cs_tag(
            tags,
            CS_SATURATION,
            "Saturation",
            ExifValue::Ascii(format_adjustment_value(adjusted)),
            ExifValue::Short(vec![raw]),
        );
    }

    // ZoneMatchingValue - offset 0x1f (CS1) only
    if is_cs1 {
        if let Some(raw) = read_cs_u16(data, 0x1f) {
            let adjusted = (raw as i32) - 10;
            insert_cs_tag(
                tags,
                CS_ZONE_MATCHING_VALUE,
                "ZoneMatchingValue",
                ExifValue::Ascii(format_adjustment_value(adjusted)),
                ExifValue::Short(vec![raw]),
            );
        }
    }

    // Brightness - offset 0x22 (CS1) only
    if is_cs1 {
        if let Some(raw) = read_cs_u16(data, 0x22) {
            let adjusted = (raw as i32) - 10;
            insert_cs_tag(
                tags,
                CS_BRIGHTNESS,
                "Brightness",
                ExifValue::Ascii(format_adjustment_value(adjusted)),
                ExifValue::Short(vec![raw]),
            );
        }
    }

    // FlashControl - offset 0x23 (CS1) or 0x1f (CS2)
    let flash_ctrl_idx = if is_cs1 { 0x23 } else { 0x1f };
    if let Some(raw) = read_cs_u16(data, flash_ctrl_idx) {
        let decoded = decode_flash_control_cs_exiftool(raw);
        insert_cs_tag(
            tags,
            CS_FLASH_CONTROL,
            "FlashControl",
            ExifValue::Ascii(decoded.to_string()),
            ExifValue::Short(vec![raw]),
        );
    }

    // PrioritySetupShutterRelease - offset 0x28 (CS1) only
    if is_cs1 {
        if let Some(raw) = read_cs_u16(data, 0x28) {
            let decoded = decode_priority_setup_cs_exiftool(raw);
            insert_cs_tag(
                tags,
                CS_PRIORITY_SETUP_SHUTTER_RELEASE,
                "PrioritySetupShutterRelease",
                ExifValue::Ascii(decoded.to_string()),
                ExifValue::Short(vec![raw]),
            );
        }
    }

    // AFIlluminator - offset 0x29 (CS1) only
    if is_cs1 {
        if let Some(raw) = read_cs_u16(data, 0x29) {
            let decoded = decode_af_illuminator_cs_exiftool(raw);
            insert_cs_tag(
                tags,
                CS_AF_ILLUMINATOR,
                "AFIlluminator",
                ExifValue::Ascii(decoded.to_string()),
                ExifValue::Short(vec![raw]),
            );
        }
    }

    // AFWithShutter - offset 0x2a (CS1) only
    if is_cs1 {
        if let Some(raw) = read_cs_u16(data, 0x2a) {
            let decoded = decode_af_with_shutter_exiftool(raw);
            insert_cs_tag(
                tags,
                CS_AF_WITH_SHUTTER,
                "AFWithShutter",
                ExifValue::Ascii(decoded.to_string()),
                ExifValue::Short(vec![raw]),
            );
        }
    }

    // LongExposureNoiseReduction - offset 0x2b (CS1) only
    if is_cs1 {
        if let Some(raw) = read_cs_u16(data, 0x2b) {
            let decoded = decode_on_off_exiftool(raw);
            insert_cs_tag(
                tags,
                CS_LONG_EXPOSURE_NR,
                "LongExposureNoiseReduction",
                ExifValue::Ascii(decoded.to_string()),
                ExifValue::Short(vec![raw]),
            );
        }
    }

    // HighISONoiseReduction - offset 0x2c (CS1) only
    if is_cs1 {
        if let Some(raw) = read_cs_u16(data, 0x2c) {
            let decoded = decode_high_iso_nr_cs_exiftool(raw);
            insert_cs_tag(
                tags,
                CS_HIGH_ISO_NR,
                "HighISONoiseReduction",
                ExifValue::Ascii(decoded.to_string()),
                ExifValue::Short(vec![raw]),
            );
        }
    }

    // ImageStyle - offset 0x2d (CS1) only
    if is_cs1 {
        if let Some(raw) = read_cs_u16(data, 0x2d) {
            let decoded = decode_image_style_cs_exiftool(raw);
            insert_cs_tag(
                tags,
                CS_IMAGE_STYLE,
                "ImageStyle",
                ExifValue::Ascii(decoded.to_string()),
                ExifValue::Short(vec![raw]),
            );
        }
    }

    // ExposureProgram - offset 0x3c (CS1 and CS2)
    if let Some(raw) = read_cs_u16(data, 0x3c) {
        let decoded = decode_exposure_program_cs_exiftool(raw);
        insert_cs_tag(
            tags,
            CS_EXPOSURE_PROGRAM,
            "ExposureProgram",
            ExifValue::Ascii(decoded.to_string()),
            ExifValue::Short(vec![raw]),
        );
    }

    // ImageStabilization - offset 0x3d (CS1 and CS2)
    if let Some(raw) = read_cs_u16(data, 0x3d) {
        let decoded = decode_on_off_exiftool(raw);
        insert_cs_tag(
            tags,
            CS_IMAGE_STABILIZATION,
            "ImageStabilization",
            ExifValue::Ascii(decoded.to_string()),
            ExifValue::Short(vec![raw]),
        );
    }

    // Rotation - offset 0x3f (CS1 and CS2)
    if let Some(raw) = read_cs_u16(data, 0x3f) {
        let decoded = decode_rotation_cs_exiftool(raw);
        insert_cs_tag(
            tags,
            CS_ROTATION,
            "Rotation",
            ExifValue::Ascii(decoded.to_string()),
            ExifValue::Short(vec![raw]),
        );
    }

    // SonyImageSize - offset 0x54 (CS1 and CS2)
    if let Some(raw) = read_cs_u16(data, 0x54) {
        let decoded = decode_sony_image_size_cs_exiftool(raw);
        insert_cs_tag(
            tags,
            CS_SONY_IMAGE_SIZE,
            "SonyImageSize",
            ExifValue::Ascii(decoded.to_string()),
            ExifValue::Short(vec![raw]),
        );
    }

    // AspectRatio - offset 0x55 (CS1 and CS2)
    if let Some(raw) = read_cs_u16(data, 0x55) {
        let decoded = decode_aspect_ratio_cs_exiftool(raw);
        insert_cs_tag(
            tags,
            CS_ASPECT_RATIO,
            "AspectRatio",
            ExifValue::Ascii(decoded.to_string()),
            ExifValue::Short(vec![raw]),
        );
    }

    // Quality - offset 0x56 (CS1 and CS2)
    if let Some(raw) = read_cs_u16(data, 0x56) {
        let decoded = decode_quality_cs_exiftool(raw);
        insert_cs_tag(
            tags,
            CS_QUALITY,
            "Quality",
            ExifValue::Ascii(decoded.to_string()),
            ExifValue::Short(vec![raw]),
        );
    }

    // ExposureLevelIncrements - offset 0x58 (CS1 and CS2)
    if let Some(raw) = read_cs_u16(data, 0x58) {
        let decoded = decode_exposure_level_increments_cs_exiftool(raw);
        insert_cs_tag(
            tags,
            CS_EXPOSURE_LEVEL_INCREMENTS,
            "ExposureLevelIncrements",
            ExifValue::Ascii(decoded.to_string()),
            ExifValue::Short(vec![raw]),
        );
    }
}

/// Parse Tag9050 encrypted subdirectory
/// Returns parsed tags with synthetic tag IDs in the 0x9050xxxx range
fn parse_tag9050(data: &[u8], endian: Endianness, tags: &mut HashMap<u16, MakerNoteTag>) {
    if data.is_empty() {
        return;
    }

    // Decipher the data
    let decrypted = decipher_sony_data(data);

    // Tag9050 fields (offsets from ExifTool Sony.pm Tag9050a)
    // Note: Some offsets differ by model, we use common ones

    // 0x0031: FlashStatus (1 byte)
    if decrypted.len() > 0x31 {
        let val = decrypted[0x31];
        let decoded = decode_flash_status_exiftool(val);
        // Use a synthetic tag ID for Tag9050 fields
        let synth_id = 0x9031_u16;
        tags.insert(
            synth_id,
            MakerNoteTag::new(
                synth_id,
                Some("FlashStatus"),
                ExifValue::Ascii(decoded.to_string()),
            ),
        );
    }

    // 0x0032: ShutterCount (4 bytes, masked with 0x00ffffff)
    if decrypted.len() > 0x35 {
        let count = read_u32(&decrypted[0x32..], endian) & 0x00ffffff;
        if count > 0 && count < 0x00ffffff {
            let synth_id = 0x9032_u16;
            tags.insert(
                synth_id,
                MakerNoteTag::new(synth_id, Some("ShutterCount"), ExifValue::Long(vec![count])),
            );
        }
    }

    // 0x003a: SonyExposureTime (2 bytes) - ValueConv: 2^(16 - val/256)
    if decrypted.len() > 0x3b {
        let raw = read_u16(&decrypted[0x3a..], endian);
        if raw > 0 {
            let exp_time = 2.0_f64.powf(16.0 - (raw as f64) / 256.0);
            let formatted = if exp_time >= 1.0 {
                // ExifTool outputs with one decimal place for values >= 1
                let rounded = (exp_time * 10.0).round() / 10.0;
                if rounded == rounded.floor() {
                    format!("{}", rounded as u32)
                } else {
                    format!("{:.1}", rounded)
                }
            } else {
                format!("1/{}", (1.0 / exp_time).round() as u32)
            };
            let synth_id = 0x903A_u16;
            tags.insert(
                synth_id,
                MakerNoteTag::new(
                    synth_id,
                    Some("SonyExposureTime"),
                    ExifValue::Ascii(formatted),
                ),
            );
        }
    }

    // 0x003c: SonyFNumber (2 bytes) - ValueConv: 2^((val/256 - 16) / 2)
    if decrypted.len() > 0x3d {
        let raw = read_u16(&decrypted[0x3c..], endian);
        if raw > 0 {
            let fnum = 2.0_f64.powf(((raw as f64) / 256.0 - 16.0) / 2.0);
            let formatted = format!("{:.1}", fnum);
            let synth_id = 0x903C_u16;
            tags.insert(
                synth_id,
                MakerNoteTag::new(synth_id, Some("SonyFNumber"), ExifValue::Ascii(formatted)),
            );
        }
    }

    // 0x003f: ReleaseMode2 (1 byte)
    if decrypted.len() > 0x3f {
        let val = decrypted[0x3f];
        let decoded = decode_release_mode2_exiftool(val);
        if decoded != "Unknown" {
            let synth_id = 0x903F_u16;
            tags.insert(
                synth_id,
                MakerNoteTag::new(
                    synth_id,
                    Some("ReleaseMode2"),
                    ExifValue::Ascii(decoded.to_string()),
                ),
            );
        }
    }

    // 0x0105: LensMount (1 byte)
    if decrypted.len() > 0x105 {
        let val = decrypted[0x105];
        if val > 0 {
            let decoded = decode_lens_mount_exiftool(val);
            let synth_id = 0x9105_u16;
            tags.insert(
                synth_id,
                MakerNoteTag::new(
                    synth_id,
                    Some("LensMount"),
                    ExifValue::Ascii(decoded.to_string()),
                ),
            );
        }
    }

    // 0x0106: LensFormat (1 byte)
    if decrypted.len() > 0x106 {
        let val = decrypted[0x106];
        if val > 0 {
            let decoded = decode_lens_format_exiftool(val);
            let synth_id = 0x9106_u16;
            tags.insert(
                synth_id,
                MakerNoteTag::new(
                    synth_id,
                    Some("LensFormat"),
                    ExifValue::Ascii(decoded.to_string()),
                ),
            );
        }
    }

    // 0x0107: LensType2 (2 bytes) - for E-mount lenses
    if decrypted.len() > 0x108 {
        let lens_type = read_u16(&decrypted[0x107..], endian);
        if lens_type > 0 && lens_type != 0xffff {
            // Try to look up lens name
            if let Some(name) = get_sony_lens_name(lens_type as u32) {
                let synth_id = 0x9107_u16;
                tags.insert(
                    synth_id,
                    MakerNoteTag::new(
                        synth_id,
                        Some("LensType2"),
                        ExifValue::Ascii(name.to_string()),
                    ),
                );
            }
        }
    }

    // 0x0109: LensType (2 bytes) - for A-mount lenses
    if decrypted.len() > 0x10a {
        let lens_type = read_u16(&decrypted[0x109..], endian);
        if lens_type > 0 && lens_type != 0xffff && (lens_type & 0xff00) != 0x8000 {
            // Try to look up lens name
            if let Some(name) = get_sony_lens_name(lens_type as u32) {
                let synth_id = 0x9109_u16;
                tags.insert(
                    synth_id,
                    MakerNoteTag::new(
                        synth_id,
                        Some("LensType"),
                        ExifValue::Ascii(name.to_string()),
                    ),
                );
            }
        }
    }

    // 0x0000: SonyMaxAperture (1 byte) - ValueConv: 2^(($val/8 - 1.06) / 2)
    // Only for SLT/ILCA models (not NEX/ILCE)
    if !decrypted.is_empty() {
        let val = decrypted[0] as f64;
        if val > 0.0 && val < 200.0 {
            let aperture = 2.0_f64.powf((val / 8.0 - 1.06) / 2.0);
            let formatted = format!("{:.1}", aperture);
            let synth_id = 0x9000_u16;
            tags.insert(
                synth_id,
                MakerNoteTag::new(
                    synth_id,
                    Some("SonyMaxAperture"),
                    ExifValue::Ascii(formatted),
                ),
            );
        }
    }

    // 0x0001: SonyMinAperture (1 byte) - ValueConv: 2^(($val/8 - 1.06) / 2)
    if decrypted.len() > 1 {
        let val = decrypted[1] as f64;
        if val > 0.0 && val < 200.0 {
            let aperture = 2.0_f64.powf((val / 8.0 - 1.06) / 2.0);
            let formatted = format!("{:.0}", aperture);
            let synth_id = 0x9001_u16;
            tags.insert(
                synth_id,
                MakerNoteTag::new(
                    synth_id,
                    Some("SonyMinAperture"),
                    ExifValue::Ascii(formatted),
                ),
            );
        }
    }

    // 0x007c or 0x00f0: InternalSerialNumber (4-5 bytes as hex)
    // For ILCE/NEX: 0x007c (4 bytes), for SLT/ILCA: 0x00f0 (5 bytes)
    // Try 0x00f0 first (SLT/ILCA - 5 bytes)
    if decrypted.len() > 0xf4 {
        let serial_bytes = &decrypted[0xf0..0xf5];
        let serial: String = serial_bytes.iter().map(|b| format!("{:02x}", b)).collect();
        if serial != "0000000000" && serial != "ffffffffff" && !serial.starts_with("0000") {
            let synth_id = 0x90F0_u16;
            tags.insert(
                synth_id,
                MakerNoteTag::new(
                    synth_id,
                    Some("InternalSerialNumber"),
                    ExifValue::Ascii(serial),
                ),
            );
        }
    }
    // Try 0x007c (ILCE/NEX - 4 bytes) if we didn't get one at 0xf0
    if !tags.contains_key(&0x90F0) && decrypted.len() > 0x7f {
        let serial_bytes = &decrypted[0x7c..0x80];
        let serial: String = serial_bytes.iter().map(|b| format!("{:02x}", b)).collect();
        if serial != "00000000" && serial != "ffffffff" {
            let synth_id = 0x907C_u16;
            tags.insert(
                synth_id,
                MakerNoteTag::new(
                    synth_id,
                    Some("InternalSerialNumber"),
                    ExifValue::Ascii(serial),
                ),
            );
        }
    }

    // 0x0020: Shutter (6 bytes as int16u[3])
    if decrypted.len() > 0x25 {
        let v1 = read_u16(&decrypted[0x20..], endian);
        let v2 = read_u16(&decrypted[0x22..], endian);
        let v3 = read_u16(&decrypted[0x24..], endian);
        if v1 == 0 && v2 == 0 && v3 == 0 {
            let synth_id = 0x9020_u16;
            tags.insert(
                synth_id,
                MakerNoteTag::new(
                    synth_id,
                    Some("Shutter"),
                    ExifValue::Ascii("Silent / Electronic (0 0 0)".to_string()),
                ),
            );
        } else {
            let synth_id = 0x9020_u16;
            tags.insert(
                synth_id,
                MakerNoteTag::new(
                    synth_id,
                    Some("Shutter"),
                    ExifValue::Ascii(format!("Mechanical ({} {} {})", v1, v2, v3)),
                ),
            );
        }
    }
}

/// Parse ShotInfo binary structure (tag 0x3000)
/// Contains SonyImageWidth/Height and other shot-specific info
fn parse_shot_info(data: &[u8], endian: Endianness, tags: &mut HashMap<u16, MakerNoteTag>) {
    if data.len() < 0x1e {
        return;
    }

    // ShotInfo structure offsets (from ExifTool Sony.pm):
    // 0x1a: SonyImageHeight (int16u)
    // 0x1c: SonyImageWidth (int16u)

    // SonyImageHeight at offset 0x1a
    let height = read_u16(&data[0x1a..], endian);
    if height > 0 && height < 10000 {
        let synth_id = 0x301A_u16;
        let value = ExifValue::Short(vec![height]);
        let tag = if let Some((exiv2_group, exiv2_name)) =
            get_exiv2_sony_subfield(SONY_SHOT_INFO, "SonyImageHeight")
        {
            MakerNoteTag::with_exiv2(
                synth_id,
                Some("SonyImageHeight"),
                value.clone(),
                value,
                exiv2_group,
                exiv2_name,
            )
        } else {
            MakerNoteTag::new(synth_id, Some("SonyImageHeight"), value)
        };
        tags.insert(synth_id, tag);
    }

    // SonyImageWidth at offset 0x1c
    let width = read_u16(&data[0x1c..], endian);
    if width > 0 && width < 15000 {
        let synth_id = 0x301C_u16;
        let value = ExifValue::Short(vec![width]);
        let tag = if let Some((exiv2_group, exiv2_name)) =
            get_exiv2_sony_subfield(SONY_SHOT_INFO, "SonyImageWidth")
        {
            MakerNoteTag::with_exiv2(
                synth_id,
                Some("SonyImageWidth"),
                value.clone(),
                value,
                exiv2_group,
                exiv2_name,
            )
        } else {
            MakerNoteTag::new(synth_id, Some("SonyImageWidth"), value)
        };
        tags.insert(synth_id, tag);
    }

    // FaceInfoOffset at 0x02 (int16u)
    if data.len() > 0x04 {
        let offset = read_u16(&data[0x02..], endian);
        if offset > 0 {
            let synth_id = 0x3002_u16;
            let value = ExifValue::Short(vec![offset]);
            let tag = if let Some((exiv2_group, exiv2_name)) =
                get_exiv2_sony_subfield(SONY_SHOT_INFO, "FaceInfoOffset")
            {
                MakerNoteTag::with_exiv2(
                    synth_id,
                    Some("FaceInfoOffset"),
                    value.clone(),
                    value,
                    exiv2_group,
                    exiv2_name,
                )
            } else {
                MakerNoteTag::new(synth_id, Some("FaceInfoOffset"), value)
            };
            tags.insert(synth_id, tag);
        }
    }

    // SonyDateTime at 0x06 (string[20])
    if data.len() > 0x1a {
        let date_str: String = data[0x06..0x1a]
            .iter()
            .take_while(|&&b| b != 0)
            .map(|&b| b as char)
            .collect();
        if !date_str.is_empty() && date_str.starts_with("20") {
            let synth_id = 0x3006_u16;
            let value = ExifValue::Ascii(date_str);
            let tag = if let Some((exiv2_group, exiv2_name)) =
                get_exiv2_sony_subfield(SONY_SHOT_INFO, "SonyDateTime")
            {
                MakerNoteTag::with_exiv2(
                    synth_id,
                    Some("SonyDateTime"),
                    value.clone(),
                    value,
                    exiv2_group,
                    exiv2_name,
                )
            } else {
                MakerNoteTag::new(synth_id, Some("SonyDateTime"), value)
            };
            tags.insert(synth_id, tag);
        }
    }
}

/// Parse MRWInfo binary structure (tag 0x7250)
/// Contains MinoltaRaw RIF (Requested Image Format) data for A100 and older cameras
/// RIF structure has Saturation at offset 1, Contrast at offset 2, Sharpness at offset 3 (int8s)
fn parse_mrw_info(data: &[u8], tags: &mut HashMap<u16, MakerNoteTag>) {
    // MRW data starts with "\0MRM" header (4 bytes), followed by MRW segments
    // We need to find the RIF segment which starts with "\0RIF"
    // The MRWInfo tag in ARW files contains the raw MRW header data

    if data.len() < 8 {
        return;
    }

    // Look for RIF segment in the MRW data
    // MRW format: each segment is [4-byte tag][4-byte length][data...]
    // Tag "\0PRD" = Picture Raw Dimensions
    // Tag "\0WBG" = White Balance Gains
    // Tag "\0RIF" = Requested Image Format (contains Saturation/Contrast/Sharpness)

    let mut offset = 0;

    // Skip initial MRM header if present
    if data.len() > 4 && &data[0..4] == b"\0MRM" {
        // Read offset to first segment (stored at offset 4 as big-endian u32)
        if data.len() > 8 {
            let mrw_offset = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as usize;
            offset = 8; // Skip past the 8-byte MRM header
                        // The mrw_offset points to the RAW data, segments are between header and RAW data
            let _ = mrw_offset; // We scan through segments anyway
        }
    }

    // Scan for segments
    while offset + 8 <= data.len() {
        let segment_tag = &data[offset..offset + 4];
        let segment_len = if offset + 8 <= data.len() {
            u32::from_be_bytes([
                data[offset + 4],
                data[offset + 5],
                data[offset + 6],
                data[offset + 7],
            ]) as usize
        } else {
            break;
        };

        if segment_tag == b"\0RIF" {
            // Found RIF segment!
            let rif_data_offset = offset + 8;
            if rif_data_offset + 4 <= data.len() && segment_len >= 4 {
                // RIF structure:
                // Offset 0: int8u - unknown
                // Offset 1: int8s - Saturation
                // Offset 2: int8s - Contrast
                // Offset 3: int8s - Sharpness

                let saturation = data[rif_data_offset + 1] as i8;
                let contrast = data[rif_data_offset + 2] as i8;
                let sharpness = data[rif_data_offset + 3] as i8;

                // Insert Saturation
                tags.insert(
                    RIF_SATURATION,
                    MakerNoteTag::new(
                        RIF_SATURATION,
                        Some("Saturation"),
                        ExifValue::Ascii(saturation.to_string()),
                    ),
                );

                // Insert Contrast
                tags.insert(
                    RIF_CONTRAST,
                    MakerNoteTag::new(
                        RIF_CONTRAST,
                        Some("Contrast"),
                        ExifValue::Ascii(contrast.to_string()),
                    ),
                );

                // Insert Sharpness
                tags.insert(
                    RIF_SHARPNESS,
                    MakerNoteTag::new(
                        RIF_SHARPNESS,
                        Some("Sharpness"),
                        ExifValue::Ascii(sharpness.to_string()),
                    ),
                );
            }
            break;
        }

        // Move to next segment
        offset += 8 + segment_len;

        // Safety check to avoid infinite loops
        if segment_len == 0 {
            break;
        }
    }
}

/// Parse a single IFD entry from Sony maker notes
fn parse_ifd_entry(
    data: &[u8],
    entry_offset: usize,
    endian: Endianness,
    tiff_data: Option<&[u8]>,
    tiff_offset: usize,
) -> Option<(u16, ExifValue)> {
    if entry_offset + 12 > data.len() {
        return None;
    }

    let tag_id = read_u16(&data[entry_offset..], endian);
    let tag_type = read_u16(&data[entry_offset + 2..], endian);
    let count = read_u32(&data[entry_offset + 4..], endian) as usize;
    let value_offset_bytes = &data[entry_offset + 8..entry_offset + 12];

    // Calculate the size of the data
    let type_size = match tag_type {
        1 | 2 | 6 | 7 => 1, // BYTE, ASCII, SBYTE, UNDEFINED
        3 | 8 => 2,         // SHORT, SSHORT
        4 | 9 | 11 => 4,    // LONG, SLONG, FLOAT
        5 | 10 | 12 => 8,   // RATIONAL, SRATIONAL, DOUBLE
        _ => return None,
    };

    let total_size = count * type_size;

    // Get the actual data location
    // Sony maker notes use offsets relative to the TIFF header (like Canon)
    let value_data: &[u8] = if total_size <= 4 {
        value_offset_bytes
    } else {
        let offset = read_u32(value_offset_bytes, endian) as usize;

        // Try to use TIFF data first (Sony uses TIFF-relative offsets)
        if let Some(tiff) = tiff_data {
            let abs_offset = tiff_offset + offset;
            if abs_offset + total_size <= tiff.len() {
                &tiff[abs_offset..abs_offset + total_size]
            } else {
                // Fall back to MakerNote-relative offset
                if offset + total_size <= data.len() {
                    &data[offset..offset + total_size]
                } else {
                    return None;
                }
            }
        } else {
            // No TIFF data, try MakerNote-relative offset
            if offset + total_size <= data.len() {
                &data[offset..offset + total_size]
            } else {
                return None;
            }
        }
    };

    // Parse based on type
    let value = match tag_type {
        1 | 6 => {
            // BYTE or SBYTE
            let bytes = value_data[..count.min(value_data.len())].to_vec();

            // Apply value decoders for single-byte tags
            if bytes.len() == 1 {
                let v = bytes[0] as u16;
                let decoded = match tag_id {
                    SONY_FOCUS_MODE => {
                        // Skip if Unknown - let other FocusMode tags take precedence
                        let decoded = decode_focus_mode_exiftool(v);
                        if decoded == "Unknown" {
                            None
                        } else {
                            Some(decoded.to_string())
                        }
                    }
                    SONY_AF_AREA_MODE_SETTING => {
                        Some(decode_af_area_mode_setting_exiftool(v).to_string())
                    }
                    SONY_AF_TRACKING => Some(decode_af_tracking_exiftool(v).to_string()),
                    SONY_AF_POINT_SELECTED => {
                        // For DSC/NEX/ILCE cameras without LA-EA adapters, value 0 = "n/a"
                        // For SLT cameras with LA-EA, value 0 = "Auto"
                        // We use "n/a" as default since most cameras output this
                        if v == 0 {
                            Some("n/a".to_string())
                        } else {
                            None // Leave as raw value for non-zero
                        }
                    }
                    _ => None,
                };

                if let Some(s) = decoded {
                    ExifValue::Ascii(s)
                } else if tag_id == SONY_FOCUS_MODE {
                    // Skip FocusMode entirely when Unknown to avoid overwriting valid values
                    return None;
                } else {
                    ExifValue::Byte(bytes)
                }
            } else if tag_id == SONY_LENS_SPEC && bytes.len() >= 8 {
                // Format LensSpec byte array
                ExifValue::Ascii(format_lens_spec(&bytes))
            } else if tag_id == SONY_FILE_FORMAT && bytes.len() >= 4 {
                // Decode Sony FileFormat (ARW version)
                if let Some(fmt) = decode_file_format(&bytes) {
                    ExifValue::Ascii(fmt)
                } else {
                    ExifValue::Byte(bytes)
                }
            } else {
                ExifValue::Byte(bytes)
            }
        }
        2 => {
            // ASCII
            let s = value_data[..count.min(value_data.len())]
                .iter()
                .take_while(|&&b| b != 0)
                .map(|&b| b as char)
                .collect::<String>();
            // Transform CreativeStyle ASCII values to match ExifTool naming
            if tag_id == SONY_CREATIVE_STYLE {
                let transformed = match s.as_str() {
                    "Autumnleaves" => "Autumn Leaves".to_string(),
                    "BW" => "B&W".to_string(),
                    _ => s,
                };
                ExifValue::Ascii(transformed)
            } else {
                ExifValue::Ascii(s)
            }
        }
        3 | 8 => {
            // SHORT or SSHORT
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                if i * 2 + 2 <= value_data.len() {
                    values.push(read_u16(&value_data[i * 2..], endian));
                }
            }

            // Apply value decoders for single-value tags
            if values.len() == 1 {
                let v = values[0];

                // Handle signed values (SSHORT, tag_type == 8)
                if tag_type == 8 && tag_id == SONY_FLASH_LEVEL {
                    // FlashLevel is signed int16
                    let signed_v = v as i16;
                    return Some((tag_id, ExifValue::Ascii(format_flash_comp(signed_v))));
                }

                let decoded = match tag_id {
                    SONY_IMAGE_QUALITY => Some(decode_image_quality_exiftool(v).to_string()),
                    SONY_WHITE_BALANCE => Some(decode_white_balance_exiftool(v).to_string()),
                    SONY_CREATIVE_STYLE => Some(decode_creative_style_exiftool(v).to_string()),
                    SONY_EXPOSURE_MODE => {
                        // Check for n/a value (65535)
                        if v == 65535 {
                            Some("n/a".to_string())
                        } else {
                            Some(decode_exposure_mode_exiftool(v).to_string())
                        }
                    }
                    SONY_AF_MODE => {
                        // Check for n/a value (65535)
                        if v == 65535 {
                            Some("n/a".to_string())
                        } else {
                            Some(decode_af_mode_exiftool(v).to_string())
                        }
                    }
                    SONY_DYNAMIC_RANGE_OPTIMIZER => {
                        Some(decode_dynamic_range_optimizer_exiftool(v).to_string())
                    }
                    SONY_FOCUS_MODE => {
                        // Skip if Unknown - let other FocusMode tags take precedence
                        let decoded = decode_focus_mode_exiftool(v);
                        if decoded == "Unknown" {
                            None
                        } else {
                            Some(decoded.to_string())
                        }
                    }
                    SONY_IMAGE_STABILIZATION => {
                        Some(decode_image_stabilization_exiftool(v).to_string())
                    }
                    SONY_HIGH_ISO_NOISE_REDUCTION => {
                        Some(decode_high_iso_noise_reduction_exiftool(v).to_string())
                    }
                    SONY_SCENE_MODE => Some(decode_scene_mode_exiftool(v).to_string()),
                    SONY_SONY_MODEL_ID => get_sony_model_name(v).map(|s| s.to_string()),
                    SONY_PICTURE_EFFECT => Some(decode_picture_effect_exiftool(v).to_string()),
                    SONY_RELEASE_MODE => Some(decode_release_mode_exiftool(v).to_string()),
                    SONY_INTELLIGENT_AUTO => Some(decode_intelligent_auto_exiftool(v).to_string()),
                    SONY_QUALITY_2 => {
                        // Tag 0xB047 is JPEGQuality, not general Quality
                        if v == 65535 {
                            Some("n/a".to_string())
                        } else {
                            Some(decode_jpeg_quality_exiftool(v).to_string())
                        }
                    }
                    SONY_AUTO_PORTRAIT_FRAMED => {
                        Some(decode_auto_portrait_framed_exiftool(v).to_string())
                    }
                    SONY_FOCUS_MODE_2 => {
                        // FocusMode2 (0xB042) - check for n/a value
                        // Skip if n/a (65535) or Unknown - let other FocusMode tags take precedence
                        if v == 65535 {
                            None // Skip instead of outputting "n/a"
                        } else {
                            let decoded = decode_focus_mode2_exiftool(v);
                            if decoded == "Unknown" {
                                None
                            } else {
                                Some(decoded.to_string())
                            }
                        }
                    }
                    SONY_FOCUS_MODE_3 => {
                        // FocusMode3 (0xB04E) - valid for DSC-HX9V generation and newer
                        // Skip if Unknown - let other FocusMode tags take precedence
                        let decoded = decode_focus_mode3_exiftool(v);
                        if decoded == "Unknown" {
                            None
                        } else {
                            Some(decoded.to_string())
                        }
                    }
                    SONY_AF_AREA_MODE_SETTING => {
                        Some(decode_af_area_mode_setting_exiftool(v).to_string())
                    }
                    SONY_AF_TRACKING => Some(decode_af_tracking_exiftool(v).to_string()),
                    SONY_ANTI_BLUR => {
                        if v == 65535 {
                            Some("n/a".to_string())
                        } else {
                            Some(decode_anti_blur_exiftool(v).to_string())
                        }
                    }
                    SONY_SEQUENCE_NUMBER => Some(decode_sequence_number_exiftool(v)),
                    SONY_MACRO => Some(decode_macro_exiftool(v).to_string()),
                    SONY_AF_ILLUMINATOR => Some(decode_af_illuminator_exiftool(v).to_string()),
                    _ => None,
                };

                if let Some(s) = decoded {
                    ExifValue::Ascii(s)
                } else if matches!(
                    tag_id,
                    SONY_FOCUS_MODE | SONY_FOCUS_MODE_2 | SONY_FOCUS_MODE_3
                ) {
                    // Skip FocusMode tags entirely when Unknown to avoid overwriting valid values
                    return None;
                } else {
                    ExifValue::Short(values)
                }
            } else if values.len() == 2 && tag_id == SONY_HDR {
                // HDR is stored as two u16 values, decode the first one
                ExifValue::Ascii(decode_hdr_exiftool(values[0]).to_string())
            } else {
                ExifValue::Short(values)
            }
        }
        4 | 9 => {
            // LONG or SLONG
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                if i * 4 + 4 <= value_data.len() {
                    values.push(read_u32(&value_data[i * 4..], endian));
                }
            }

            // Apply value decoders for single-value LONG tags
            if values.len() == 1 {
                let v = values[0];
                // First check tags that need full u32 range
                if tag_id == SONY_LONG_EXPOSURE_NOISE_REDUCTION {
                    ExifValue::Ascii(decode_long_exposure_noise_reduction_exiftool(v).to_string())
                } else if tag_id == SONY_LENS_ID {
                    // LensID uses u32 for lookup
                    if let Some(name) = get_sony_lens_name(v) {
                        ExifValue::Ascii(name.to_string())
                    } else {
                        ExifValue::Long(values)
                    }
                } else if tag_id == SONY_COLOR_TEMPERATURE {
                    // ColorTemperature: 0 = "Auto", otherwise show value
                    if let Some(s) = decode_color_temperature_exiftool(v) {
                        ExifValue::Ascii(s.to_string())
                    } else {
                        ExifValue::Long(values)
                    }
                } else if tag_id == SONY_WHITE_BALANCE {
                    // WhiteBalance is int32u in some cameras
                    if v <= u16::MAX as u32 {
                        ExifValue::Ascii(decode_white_balance_exiftool(v as u16).to_string())
                    } else {
                        ExifValue::Long(values)
                    }
                } else if tag_id == SONY_CONTRAST
                    || tag_id == SONY_SATURATION
                    || tag_id == SONY_SHARPNESS
                {
                    // Contrast/Saturation/Sharpness as signed value
                    let signed_v = v as i32;
                    ExifValue::Ascii(decode_adjustment_exiftool(signed_v).to_string())
                } else if tag_id == SONY_TELECONVERTER {
                    // Teleconverter uses full u32 range
                    ExifValue::Ascii(decode_teleconverter_exiftool(v).to_string())
                } else if tag_id == SONY_VIGNETTING_CORRECTION {
                    ExifValue::Ascii(decode_vignetting_correction_exiftool(v).to_string())
                } else if tag_id == SONY_DISTORTION_CORRECTION {
                    ExifValue::Ascii(decode_distortion_correction_exiftool(v).to_string())
                } else if tag_id == SONY_MULTI_FRAME_NOISE_REDUCTION {
                    ExifValue::Ascii(decode_multi_frame_noise_reduction_exiftool(v).to_string())
                } else if tag_id == SONY_IMAGE_QUALITY {
                    ExifValue::Ascii(decode_quality_exiftool(v).to_string())
                } else if tag_id == SONY_ZONE_MATCHING {
                    ExifValue::Ascii(decode_zone_matching_exiftool(v).to_string())
                } else if tag_id == SONY_COLOR_MODE {
                    ExifValue::Ascii(decode_color_mode_exiftool(v).to_string())
                } else if tag_id == SONY_FLASH_ACTION {
                    ExifValue::Ascii(decode_flash_action_exiftool(v).to_string())
                } else if tag_id == SONY_HDR {
                    // HDR is stored as int32u, but read as two int16u values
                    // Format: "HDRLevel; ImageStatus"
                    ExifValue::Ascii(format_hdr_value(v))
                } else if tag_id == SONY_MULTI_FRAME_NR_EFFECT {
                    // MultiFrameNREffect: 0=Normal, 1=High
                    ExifValue::Ascii(decode_multi_frame_nr_effect_exiftool(v).to_string())
                } else if tag_id == SONY_EFCS {
                    // ElectronicFrontCurtainShutter: 0=Off, 1=On
                    ExifValue::Ascii(decode_efcs_exiftool(v).to_string())
                } else if tag_id == SONY_LATERAL_CHROMATIC_ABERRATION {
                    // LateralChromaticAberration: 0=Off, 2=Auto
                    ExifValue::Ascii(decode_lateral_chromatic_aberration_exiftool(v).to_string())
                } else if v <= u16::MAX as u32 {
                    // Then check tags that fit in u16
                    let v16 = v as u16;
                    let decoded = match tag_id {
                        SONY_DYNAMIC_RANGE_OPTIMIZER => {
                            Some(decode_dynamic_range_optimizer_exiftool(v16).to_string())
                        }
                        SONY_IMAGE_STABILIZATION => {
                            Some(decode_image_stabilization_exiftool(v16).to_string())
                        }
                        SONY_SCENE_MODE => Some(decode_scene_mode_exiftool(v16).to_string()),
                        SONY_SONY_MODEL_ID => get_sony_model_name(v16).map(|s| s.to_string()),
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
            } else {
                ExifValue::Long(values)
            }
        }
        5 => {
            // RATIONAL (unsigned)
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                if i * 8 + 8 <= value_data.len() {
                    let num = read_u32(&value_data[i * 8..], endian);
                    let den = read_u32(&value_data[i * 8 + 4..], endian);
                    values.push((num, den));
                }
            }
            ExifValue::Rational(values)
        }
        10 => {
            // SRATIONAL (signed)
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                if i * 8 + 8 <= value_data.len() {
                    let num = read_i32(&value_data[i * 8..], endian);
                    let den = read_i32(&value_data[i * 8 + 4..], endian);
                    values.push((num, den));
                }
            }
            ExifValue::SRational(values)
        }
        7 => {
            // UNDEFINED
            ExifValue::Undefined(value_data[..total_size.min(value_data.len())].to_vec())
        }
        _ => ExifValue::Undefined(vec![]),
    };

    Some((tag_id, value))
}

/// Parse Sony maker notes
pub fn parse_sony_maker_notes(
    data: &[u8],
    endian: Endianness,
    tiff_data: Option<&[u8]>,
    tiff_offset: usize,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    // Sony maker notes can start with "SONY DSC " or be in standard TIFF IFD format
    let mut offset = 0;

    if data.len() >= 12 && data.starts_with(b"SONY DSC ") {
        offset = 12;
    }

    if offset >= data.len() || data.len() < offset + 2 {
        return Ok(tags);
    }

    // Read number of entries
    let num_entries = read_u16(&data[offset..], endian);
    offset += 2;

    // Parse IFD entries
    for i in 0..num_entries as usize {
        let entry_offset = offset + i * 12;
        if entry_offset + 12 > data.len() {
            break;
        }

        if let Some((tag_id, value)) =
            parse_ifd_entry(data, entry_offset, endian, tiff_data, tiff_offset)
        {
            // Decode certain tag values to human-readable strings for ExifTool compatibility
            let decoded_value = match tag_id {
                SONY_SOFT_SKIN_EFFECT => {
                    if let ExifValue::Long(ref v) = value {
                        if !v.is_empty() {
                            ExifValue::Ascii(decode_soft_skin_effect_exiv2(v[0]).to_string())
                        } else {
                            value
                        }
                    } else {
                        value
                    }
                }
                SONY_QUALITY_2 => {
                    // Tag 0xB047 is JPEGQuality (different from tag 0x0102 ImageQuality)
                    if let ExifValue::Long(ref v) = value {
                        if !v.is_empty() {
                            let val = v[0];
                            if val == 65535 {
                                ExifValue::Ascii("n/a".to_string())
                            } else if val <= u16::MAX as u32 {
                                ExifValue::Ascii(
                                    decode_jpeg_quality_exiftool(val as u16).to_string(),
                                )
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    } else {
                        value
                    }
                }
                SONY_IMAGE_QUALITY => {
                    if let ExifValue::Long(ref v) = value {
                        if !v.is_empty() {
                            ExifValue::Ascii(decode_quality_exiftool(v[0]).to_string())
                        } else {
                            value
                        }
                    } else {
                        value
                    }
                }
                SONY_HIGH_ISO_NOISE_REDUCTION_2 => {
                    // Tag 0xB050: decode noise reduction level
                    if let ExifValue::Short(ref v) = value {
                        if !v.is_empty() {
                            ExifValue::Ascii(
                                decode_high_iso_noise_reduction2_exiftool(v[0]).to_string(),
                            )
                        } else {
                            value
                        }
                    } else {
                        value
                    }
                }
                SONY_AF_POINTS_USED | SONY_FOCAL_PLANE_AF_POINTS_USED => {
                    // AFPointsUsed/FocalPlaneAFPointsUsed: all zeros => "(none)"
                    match &value {
                        ExifValue::Byte(vals) if all_zeros_u8(vals) => {
                            ExifValue::Ascii("(none)".to_string())
                        }
                        ExifValue::Short(vals) if all_zeros_u16(vals) => {
                            ExifValue::Ascii("(none)".to_string())
                        }
                        _ => value,
                    }
                }
                SONY_VARIABLE_LOW_PASS_FILTER => {
                    // VariableLowPassFilter: int16u[2], "0 0" or "65535 65535" => "n/a"
                    if let ExifValue::Short(vals) = &value {
                        if vals.len() >= 2 {
                            if (vals[0] == 0 && vals[1] == 0)
                                || (vals[0] == 65535 && vals[1] == 65535)
                            {
                                ExifValue::Ascii("n/a".to_string())
                            } else if vals[0] == 1 && vals[1] == 0 {
                                ExifValue::Ascii("Off".to_string())
                            } else if vals[0] == 1 && vals[1] == 1 {
                                ExifValue::Ascii("Standard".to_string())
                            } else if vals[0] == 1 && vals[1] == 2 {
                                ExifValue::Ascii("High".to_string())
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    } else {
                        value
                    }
                }
                SONY_RAW_FILE_TYPE => {
                    if let ExifValue::Short(ref v) = value {
                        if !v.is_empty() {
                            ExifValue::Ascii(decode_raw_file_type_exiftool(v[0]).to_string())
                        } else {
                            value
                        }
                    } else {
                        value
                    }
                }
                SONY_PRIORITY_SET_IN_AWB => {
                    if let ExifValue::Byte(ref v) = value {
                        if !v.is_empty() {
                            ExifValue::Ascii(
                                decode_priority_set_in_awb_exiftool(v[0] as u16).to_string(),
                            )
                        } else {
                            value
                        }
                    } else {
                        value
                    }
                }
                SONY_METERING_MODE_2 => {
                    if let ExifValue::Short(ref v) = value {
                        if !v.is_empty() {
                            ExifValue::Ascii(decode_metering_mode2_exiftool(v[0]).to_string())
                        } else {
                            value
                        }
                    } else {
                        value
                    }
                }
                _ => value,
            };

            // Handle encrypted subdirectories and CameraSettings
            if tag_id == SONY_TAG_9050 {
                // Parse Tag9050 encrypted subdirectory
                if let ExifValue::Undefined(ref raw_data) = decoded_value {
                    parse_tag9050(raw_data, endian, &mut tags);
                }
                // Don't insert the raw encrypted blob
                continue;
            }

            // Handle CameraSettings binary blob (tag 0x0114)
            // Used by A200, A300, A350, A700, A850, A900
            if tag_id == SONY_CAMERA_SETTINGS {
                if let ExifValue::Undefined(ref raw_data) = decoded_value {
                    parse_camera_settings(raw_data, endian, &mut tags);
                } else if let ExifValue::Byte(ref raw_data) = decoded_value {
                    parse_camera_settings(raw_data, endian, &mut tags);
                }
                // Don't insert the raw binary blob
                continue;
            }

            // Handle ShotInfo binary blob (tag 0x3000)
            // Contains SonyImageWidth/Height and other info
            if tag_id == SONY_SHOT_INFO {
                if let ExifValue::Undefined(ref raw_data) = decoded_value {
                    parse_shot_info(raw_data, endian, &mut tags);
                } else if let ExifValue::Byte(ref raw_data) = decoded_value {
                    parse_shot_info(raw_data, endian, &mut tags);
                }
                // Don't insert the raw binary blob
                continue;
            }

            // Handle MRWInfo binary blob (tag 0x7250)
            // Contains MinoltaRaw RIF structure with Saturation/Contrast/Sharpness for A100
            if tag_id == SONY_MRW_INFO {
                if let ExifValue::Undefined(ref raw_data) = decoded_value {
                    parse_mrw_info(raw_data, &mut tags);
                } else if let ExifValue::Byte(ref raw_data) = decoded_value {
                    parse_mrw_info(raw_data, &mut tags);
                }
                // Don't insert the raw binary blob
                continue;
            }

            // Handle MinoltaMakerNote subdirectory (tag 0xB028)
            // Used by Sony A100 - contains CameraInfoA100 with AFMode etc
            if tag_id == SONY_MINOLTA_MAKER_NOTE {
                if let ExifValue::Long(ref v) = decoded_value {
                    if !v.is_empty() {
                        // The value is an offset pointer to the MinoltaMakerNote IFD
                        let minolta_offset = v[0] as usize;
                        if let Some(tiff) = tiff_data {
                            let abs_offset = tiff_offset + minolta_offset;
                            if abs_offset < tiff.len() {
                                // Parse the MinoltaMakerNote starting at this offset
                                let minolta_data = &tiff[abs_offset..];
                                if let Ok(minolta_tags) = minolta::parse_minolta_maker_notes(
                                    minolta_data,
                                    endian,
                                    Some(tiff),
                                    tiff_offset,
                                ) {
                                    // Merge Minolta MakerNote tags for A100
                                    // - WhiteBalance (0x0115): should override Sony MakerNote WhiteBalance
                                    // - CameraInfoA100 sub-tags (0x2000-0x2FFF): AFMode, AFPoint, AFAreaMode
                                    // - CameraSettingsA100 sub-tags (0x3000-0x3FFF): FocusMode (tag 0x0c)
                                    for (minolta_tag_id, minolta_tag) in minolta_tags {
                                        // Include Minolta WhiteBalance - it overrides Sony MakerNote value
                                        if minolta_tag_id == minolta::MINOLTA_WHITE_BALANCE {
                                            tags.insert(SONY_WHITE_BALANCE, minolta_tag);
                                        }
                                        // Include CameraInfoA100 sub-tags (0x2000-0x2FFF)
                                        else if (0x2000..0x3000).contains(&minolta_tag_id) {
                                            let composite_id = 0x4000 + (minolta_tag_id - 0x2000);
                                            tags.insert(composite_id, minolta_tag);
                                        }
                                        // Include CameraSettingsA100 sub-tags (0x3000-0x3FFF)
                                        else if (0x3000..0x4000).contains(&minolta_tag_id) {
                                            let composite_id = 0x5000 + (minolta_tag_id - 0x3000);
                                            tags.insert(composite_id, minolta_tag);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // Don't insert the raw offset pointer
                continue;
            }

            tags.insert(
                tag_id,
                MakerNoteTag::new(tag_id, get_sony_tag_name(tag_id), decoded_value),
            );
        }
    }

    // Post-processing: Fix AFPointSelected for SLT/ILCA models
    // SLT and ILCA models map value 0 to "Auto", while NEX/ILCE/DSC map to "n/a"
    // Check if we have a SonyModelID that indicates SLT/ILCA model
    if let Some(model_tag) = tags.get(&SONY_SONY_MODEL_ID) {
        if let ExifValue::Ascii(model_name) = &model_tag.value {
            // SLT-*, HV, and ILCA-* models use "Auto" for AFPointSelected value 0
            let is_slt_or_ilca = model_name.starts_with("SLT-")
                || model_name.starts_with("ILCA-")
                || model_name == "HV";

            if is_slt_or_ilca {
                if let Some(af_tag) = tags.get_mut(&SONY_AF_POINT_SELECTED) {
                    if let ExifValue::Ascii(s) = &af_tag.value {
                        if s == "n/a" {
                            af_tag.value = ExifValue::Ascii("Auto".to_string());
                        }
                    }
                }
            }
        }
    }

    Ok(tags)
}
