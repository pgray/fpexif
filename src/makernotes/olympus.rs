// makernotes/olympus.rs - Olympus maker notes parsing

use crate::data_types::{Endianness, ExifValue};
use crate::define_tag_decoder;
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use std::collections::HashMap;

// Olympus MakerNote main IFD tag IDs
pub const OLYMPUS_THUMBNAIL_IMAGE: u16 = 0x0100;
pub const OLYMPUS_SPECIAL_MODE: u16 = 0x0200;
pub const OLYMPUS_JPEG_QUALITY: u16 = 0x0201;
pub const OLYMPUS_MACRO: u16 = 0x0202;
pub const OLYMPUS_BW_MODE: u16 = 0x0203;
pub const OLYMPUS_DIGITAL_ZOOM: u16 = 0x0204;
pub const OLYMPUS_FOCAL_PLANE_DIAGONAL: u16 = 0x0205;
pub const OLYMPUS_LENS_DISTORTION_PARAMS: u16 = 0x0206;
pub const OLYMPUS_CAMERA_TYPE: u16 = 0x0207;
pub const OLYMPUS_TEXT_INFO: u16 = 0x0208;
pub const OLYMPUS_CAMERA_ID: u16 = 0x0209;
pub const OLYMPUS_EPSON_IMAGE_WIDTH: u16 = 0x020B;
pub const OLYMPUS_EPSON_IMAGE_HEIGHT: u16 = 0x020C;
pub const OLYMPUS_EPSON_SOFTWARE: u16 = 0x020D;
pub const OLYMPUS_PREVIEW_IMAGE: u16 = 0x0280;
pub const OLYMPUS_PRE_CAPTURE_FRAMES: u16 = 0x0300;
pub const OLYMPUS_WHITE_BOARD: u16 = 0x0301;
pub const OLYMPUS_ONE_TOUCH_WB: u16 = 0x0302;
pub const OLYMPUS_WHITE_BALANCE_BRACKET: u16 = 0x0303;
pub const OLYMPUS_WHITE_BALANCE_BIAS: u16 = 0x0304;
pub const OLYMPUS_SCENE_MODE: u16 = 0x0403;
pub const OLYMPUS_SERIAL_NUMBER: u16 = 0x0404;
pub const OLYMPUS_FIRMWARE: u16 = 0x0405;
pub const OLYMPUS_DATA_DUMP: u16 = 0x0E00;
pub const OLYMPUS_EQUIPMENT_IFD: u16 = 0x2010;
pub const OLYMPUS_CAMERA_SETTINGS_IFD: u16 = 0x2020;
pub const OLYMPUS_RAW_DEVELOPMENT_IFD: u16 = 0x2030;
pub const OLYMPUS_RAW_DEVELOPMENT2_IFD: u16 = 0x2031;
pub const OLYMPUS_IMAGE_PROCESSING_IFD: u16 = 0x2040;
pub const OLYMPUS_FOCUS_INFO_IFD: u16 = 0x2050;
pub const OLYMPUS_RAW_INFO_IFD: u16 = 0x3000;

// Equipment IFD tags (0x2010)
pub const EQUIP_VERSION: u16 = 0x0000;
pub const EQUIP_CAMERA_TYPE: u16 = 0x0100;
pub const EQUIP_SERIAL_NUMBER: u16 = 0x0101;
pub const EQUIP_INTERNAL_SERIAL_NUMBER: u16 = 0x0102;
pub const EQUIP_FOCAL_PLANE_DIAGONAL: u16 = 0x0103;
pub const EQUIP_BODY_FIRMWARE_VERSION: u16 = 0x0104;
pub const EQUIP_LENS_TYPE: u16 = 0x0201;
pub const EQUIP_LENS_SERIAL_NUMBER: u16 = 0x0202;
pub const EQUIP_LENS_MODEL: u16 = 0x0203;
pub const EQUIP_LENS_FIRMWARE_VERSION: u16 = 0x0204;
pub const EQUIP_MAX_APERTURE_AT_MIN_FOCAL: u16 = 0x0205;
pub const EQUIP_MAX_APERTURE_AT_MAX_FOCAL: u16 = 0x0206;
pub const EQUIP_MIN_FOCAL_LENGTH: u16 = 0x0207;
pub const EQUIP_MAX_FOCAL_LENGTH: u16 = 0x0208;
pub const EQUIP_MAX_APERTURE: u16 = 0x020A;
pub const EQUIP_LENS_PROPERTIES: u16 = 0x020B;
pub const EQUIP_EXTENDER: u16 = 0x0301;
pub const EQUIP_EXTENDER_SERIAL_NUMBER: u16 = 0x0302;
pub const EQUIP_EXTENDER_MODEL: u16 = 0x0303;
pub const EQUIP_EXTENDER_FIRMWARE_VERSION: u16 = 0x0304;
pub const EQUIP_FLASH_TYPE: u16 = 0x1000;
pub const EQUIP_FLASH_MODEL: u16 = 0x1001;
pub const EQUIP_FLASH_FIRMWARE_VERSION: u16 = 0x1002;
pub const EQUIP_FLASH_SERIAL_NUMBER: u16 = 0x1003;

// Camera Settings IFD tags (0x2020)
pub const CS_VERSION: u16 = 0x0000;
pub const CS_PREVIEW_IMAGE_VALID: u16 = 0x0100;
pub const CS_PREVIEW_IMAGE_START: u16 = 0x0101;
pub const CS_PREVIEW_IMAGE_LENGTH: u16 = 0x0102;
pub const CS_EXPOSURE_MODE: u16 = 0x0200;
pub const CS_AE_LOCK: u16 = 0x0201;
pub const CS_METERING_MODE: u16 = 0x0202;
pub const CS_EXPOSURE_SHIFT: u16 = 0x0203;
pub const CS_MACRO_MODE: u16 = 0x0300;
pub const CS_FOCUS_MODE: u16 = 0x0301;
pub const CS_FOCUS_PROCESS: u16 = 0x0302;
pub const CS_AF_SEARCH: u16 = 0x0303;
pub const CS_AF_AREAS: u16 = 0x0304;
pub const CS_AF_POINT_SELECTED: u16 = 0x0305;
pub const CS_AF_FINE_TUNE: u16 = 0x0306;
pub const CS_FLASH_MODE: u16 = 0x0400;
pub const CS_FLASH_EXPOSURE_COMP: u16 = 0x0401;
pub const CS_FLASH_REMOTE_CONTROL: u16 = 0x0403;
pub const CS_FLASH_CONTROL_MODE: u16 = 0x0404;
pub const CS_WHITE_BALANCE: u16 = 0x0500;
pub const CS_WHITE_BALANCE_TEMPERATURE: u16 = 0x0501;
pub const CS_WHITE_BALANCE_BRACKET: u16 = 0x0502;
pub const CS_CUSTOM_SATURATION: u16 = 0x0503;
pub const CS_MODIFIED_SATURATION: u16 = 0x0504;
pub const CS_CONTRAST_SETTING: u16 = 0x0505;
pub const CS_SHARPNESS_SETTING: u16 = 0x0506;
pub const CS_COLOR_SPACE: u16 = 0x0507;
pub const CS_SCENE_MODE: u16 = 0x0509;
pub const CS_NOISE_REDUCTION: u16 = 0x050A;
pub const CS_DISTORTION_CORRECTION: u16 = 0x050B;
pub const CS_SHADING_COMPENSATION: u16 = 0x050C;
pub const CS_COMPRESSION_FACTOR: u16 = 0x050D;
pub const CS_GRADATION: u16 = 0x050F;
pub const CS_PICTURE_MODE: u16 = 0x0520;
pub const CS_PICTURE_MODE_SATURATION: u16 = 0x0521;
pub const CS_PICTURE_MODE_CONTRAST: u16 = 0x0523;
pub const CS_PICTURE_MODE_SHARPNESS: u16 = 0x0524;
pub const CS_PICTURE_MODE_BW_FILTER: u16 = 0x0525;
pub const CS_PICTURE_MODE_TONE: u16 = 0x0526;
pub const CS_NOISE_FILTER: u16 = 0x0527;
pub const CS_PICTURE_MODE_HUE: u16 = 0x0522;
pub const CS_ART_FILTER: u16 = 0x0529;
pub const CS_MAGIC_FILTER: u16 = 0x052C;
pub const CS_DRIVE_MODE: u16 = 0x0600;
pub const CS_PANORAMA_MODE: u16 = 0x0601;
pub const CS_IMAGE_QUALITY2: u16 = 0x0603;
pub const CS_IMAGE_STABILIZATION: u16 = 0x0604;

// Raw Development IFD tags (0x2030)
pub const RD_VERSION: u16 = 0x0000;
pub const RD_EXPOSURE_BIAS_VALUE: u16 = 0x0100;
pub const RD_WHITE_BALANCE_VALUE: u16 = 0x0101;
pub const RD_WB_FINE_ADJUSTMENT: u16 = 0x0102;
pub const RD_GRAY_POINT: u16 = 0x0103;
pub const RD_SATURATION_EMPHASIS: u16 = 0x0104;
pub const RD_MEMORY_COLOR_EMPHASIS: u16 = 0x0105;
pub const RD_CONTRAST_VALUE: u16 = 0x0106;
pub const RD_SHARPNESS_VALUE: u16 = 0x0107;
pub const RD_COLOR_SPACE: u16 = 0x0108;
pub const RD_ENGINE: u16 = 0x0109;
pub const RD_NOISE_REDUCTION: u16 = 0x010A;
pub const RD_EDIT_STATUS: u16 = 0x010B;
pub const RD_SETTINGS: u16 = 0x010C;

// Image Processing IFD tags (0x2040)
pub const IP_VERSION: u16 = 0x0000;
pub const IP_WB_RB_LEVELS: u16 = 0x0100;
pub const IP_WB_RB_LEVELS_3000K: u16 = 0x0102;
pub const IP_WB_RB_LEVELS_3300K: u16 = 0x0103;
pub const IP_WB_RB_LEVELS_3600K: u16 = 0x0104;
pub const IP_WB_RB_LEVELS_3900K: u16 = 0x0105;
pub const IP_WB_RB_LEVELS_4000K: u16 = 0x0106;
pub const IP_WB_RB_LEVELS_4300K: u16 = 0x0107;
pub const IP_WB_RB_LEVELS_4500K: u16 = 0x0108;
pub const IP_WB_RB_LEVELS_4800K: u16 = 0x0109;
pub const IP_WB_RB_LEVELS_5300K: u16 = 0x010A;
pub const IP_WB_RB_LEVELS_6000K: u16 = 0x010B;
pub const IP_WB_RB_LEVELS_6600K: u16 = 0x010C;
pub const IP_WB_RB_LEVELS_7500K: u16 = 0x010D;
pub const IP_COLOR_MATRIX: u16 = 0x0200;
pub const IP_BLACK_LEVEL: u16 = 0x0600;
pub const IP_GAIN_BASE: u16 = 0x0610;
pub const IP_VALID_BITS: u16 = 0x0611;
pub const IP_CROP_LEFT: u16 = 0x0612;
pub const IP_CROP_TOP: u16 = 0x0613;
pub const IP_CROP_WIDTH: u16 = 0x0614;
pub const IP_CROP_HEIGHT: u16 = 0x0615;
pub const IP_NOISE_REDUCTION2: u16 = 0x1010;
pub const IP_DISTORTION_CORRECTION2: u16 = 0x1011;
pub const IP_SHADING_COMPENSATION2: u16 = 0x1012;
pub const IP_MULTIPLE_EXPOSURE_MODE: u16 = 0x101C;
pub const IP_ASPECT_RATIO: u16 = 0x1112;
pub const IP_FACES_DETECTED: u16 = 0x1200;
pub const IP_FACE_DETECT_AREA: u16 = 0x1201;

// Focus Info IFD tags (0x2050)
pub const FI_VERSION: u16 = 0x0000;
pub const FI_AUTO_FOCUS: u16 = 0x0209;
pub const FI_SCENE_DETECT: u16 = 0x0210;
pub const FI_ZOOM_STEP_COUNT: u16 = 0x0300;
pub const FI_FOCUS_STEP_COUNT: u16 = 0x0301;
pub const FI_FOCUS_STEP_INFINITY: u16 = 0x0303;
pub const FI_FOCUS_STEP_NEAR: u16 = 0x0304;
pub const FI_FOCUS_DISTANCE: u16 = 0x0305;
pub const FI_AF_POINT: u16 = 0x0308;
pub const FI_EXTERNAL_FLASH: u16 = 0x1201;
pub const FI_EXTERNAL_FLASH_BOUNCE: u16 = 0x1203;
pub const FI_EXTERNAL_FLASH_ZOOM: u16 = 0x1204;
pub const FI_INTERNAL_FLASH: u16 = 0x1208;
pub const FI_MANUAL_FLASH: u16 = 0x1209;
pub const FI_MACRO_LED: u16 = 0x120A;
pub const FI_SENSOR_TEMPERATURE: u16 = 0x1500;
pub const FI_IMAGE_STABILIZATION: u16 = 0x1600;

/// Get the name of an Olympus MakerNote tag
pub fn get_olympus_tag_name(tag_id: u16, ifd_type: OlympusIfdType) -> Option<&'static str> {
    match ifd_type {
        OlympusIfdType::Main => get_main_tag_name(tag_id),
        OlympusIfdType::Equipment => get_equipment_tag_name(tag_id),
        OlympusIfdType::CameraSettings => get_camera_settings_tag_name(tag_id),
        OlympusIfdType::RawDevelopment => get_raw_development_tag_name(tag_id),
        OlympusIfdType::ImageProcessing => get_image_processing_tag_name(tag_id),
        OlympusIfdType::FocusInfo => get_focus_info_tag_name(tag_id),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OlympusIfdType {
    Main,
    Equipment,
    CameraSettings,
    RawDevelopment,
    ImageProcessing,
    FocusInfo,
}

fn get_main_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        OLYMPUS_THUMBNAIL_IMAGE => Some("ThumbnailImage"),
        OLYMPUS_SPECIAL_MODE => Some("SpecialMode"),
        OLYMPUS_JPEG_QUALITY => Some("JpegQuality"),
        OLYMPUS_MACRO => Some("Macro"),
        OLYMPUS_BW_MODE => Some("BWMode"),
        OLYMPUS_DIGITAL_ZOOM => Some("DigitalZoom"),
        OLYMPUS_FOCAL_PLANE_DIAGONAL => Some("FocalPlaneDiagonal"),
        OLYMPUS_CAMERA_TYPE => Some("CameraType"),
        OLYMPUS_CAMERA_ID => Some("CameraID"),
        OLYMPUS_PREVIEW_IMAGE => Some("PreviewImage"),
        OLYMPUS_SERIAL_NUMBER => Some("SerialNumber"),
        OLYMPUS_FIRMWARE => Some("Firmware"),
        OLYMPUS_EQUIPMENT_IFD => Some("EquipmentIFD"),
        OLYMPUS_CAMERA_SETTINGS_IFD => Some("CameraSettingsIFD"),
        OLYMPUS_RAW_DEVELOPMENT_IFD => Some("RawDevelopmentIFD"),
        OLYMPUS_RAW_DEVELOPMENT2_IFD => Some("RawDevelopment2IFD"),
        OLYMPUS_IMAGE_PROCESSING_IFD => Some("ImageProcessingIFD"),
        OLYMPUS_FOCUS_INFO_IFD => Some("FocusInfoIFD"),
        OLYMPUS_RAW_INFO_IFD => Some("RawInfoIFD"),
        _ => None,
    }
}

fn get_equipment_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        EQUIP_VERSION => Some("EquipmentVersion"),
        EQUIP_CAMERA_TYPE => Some("CameraType2"),
        EQUIP_SERIAL_NUMBER => Some("SerialNumber"),
        EQUIP_INTERNAL_SERIAL_NUMBER => Some("InternalSerialNumber"),
        EQUIP_FOCAL_PLANE_DIAGONAL => Some("FocalPlaneDiagonal"),
        EQUIP_BODY_FIRMWARE_VERSION => Some("BodyFirmwareVersion"),
        EQUIP_LENS_TYPE => Some("LensType"),
        EQUIP_LENS_SERIAL_NUMBER => Some("LensSerialNumber"),
        EQUIP_LENS_MODEL => Some("LensModel"),
        EQUIP_LENS_FIRMWARE_VERSION => Some("LensFirmwareVersion"),
        EQUIP_MAX_APERTURE_AT_MIN_FOCAL => Some("MaxApertureAtMinFocal"),
        EQUIP_MAX_APERTURE_AT_MAX_FOCAL => Some("MaxApertureAtMaxFocal"),
        EQUIP_MIN_FOCAL_LENGTH => Some("MinFocalLength"),
        EQUIP_MAX_FOCAL_LENGTH => Some("MaxFocalLength"),
        EQUIP_MAX_APERTURE => Some("MaxAperture"),
        EQUIP_LENS_PROPERTIES => Some("LensProperties"),
        EQUIP_EXTENDER => Some("Extender"),
        EQUIP_EXTENDER_SERIAL_NUMBER => Some("ExtenderSerialNumber"),
        EQUIP_EXTENDER_MODEL => Some("ExtenderModel"),
        EQUIP_EXTENDER_FIRMWARE_VERSION => Some("ExtenderFirmwareVersion"),
        EQUIP_FLASH_TYPE => Some("FlashType"),
        EQUIP_FLASH_MODEL => Some("FlashModel"),
        EQUIP_FLASH_FIRMWARE_VERSION => Some("FlashFirmwareVersion"),
        EQUIP_FLASH_SERIAL_NUMBER => Some("FlashSerialNumber"),
        _ => None,
    }
}

fn get_camera_settings_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        CS_VERSION => Some("CameraSettingsVersion"),
        CS_PREVIEW_IMAGE_VALID => Some("PreviewImageValid"),
        CS_PREVIEW_IMAGE_START => Some("PreviewImageStart"),
        CS_PREVIEW_IMAGE_LENGTH => Some("PreviewImageLength"),
        CS_EXPOSURE_MODE => Some("ExposureMode"),
        CS_AE_LOCK => Some("AELock"),
        CS_METERING_MODE => Some("MeteringMode"),
        CS_EXPOSURE_SHIFT => Some("ExposureShift"),
        CS_MACRO_MODE => Some("MacroMode"),
        CS_FOCUS_MODE => Some("FocusMode"),
        CS_FOCUS_PROCESS => Some("FocusProcess"),
        CS_AF_SEARCH => Some("AFSearch"),
        CS_AF_AREAS => Some("AFAreas"),
        CS_AF_POINT_SELECTED => Some("AFPointSelected"),
        CS_AF_FINE_TUNE => Some("AFFineTune"),
        CS_FLASH_MODE => Some("FlashMode"),
        CS_FLASH_EXPOSURE_COMP => Some("FlashExposureComp"),
        CS_FLASH_REMOTE_CONTROL => Some("FlashRemoteControl"),
        CS_FLASH_CONTROL_MODE => Some("FlashControlMode"),
        CS_WHITE_BALANCE => Some("WhiteBalance2"),
        CS_WHITE_BALANCE_TEMPERATURE => Some("WhiteBalanceTemperature"),
        CS_WHITE_BALANCE_BRACKET => Some("WhiteBalanceBracket"),
        CS_CUSTOM_SATURATION => Some("CustomSaturation"),
        CS_MODIFIED_SATURATION => Some("ModifiedSaturation"),
        CS_CONTRAST_SETTING => Some("ContrastSetting"),
        CS_SHARPNESS_SETTING => Some("SharpnessSetting"),
        CS_COLOR_SPACE => Some("ColorSpace"),
        CS_SCENE_MODE => Some("SceneMode"),
        CS_NOISE_REDUCTION => Some("NoiseReduction"),
        CS_DISTORTION_CORRECTION => Some("DistortionCorrection"),
        CS_SHADING_COMPENSATION => Some("ShadingCompensation"),
        CS_GRADATION => Some("Gradation"),
        CS_PICTURE_MODE => Some("PictureMode"),
        CS_PICTURE_MODE_SATURATION => Some("PictureModeSaturation"),
        CS_PICTURE_MODE_HUE => Some("PictureModeHue"),
        CS_PICTURE_MODE_CONTRAST => Some("PictureModeContrast"),
        CS_PICTURE_MODE_SHARPNESS => Some("PictureModeSharpness"),
        CS_PICTURE_MODE_BW_FILTER => Some("PictureModeBWFilter"),
        CS_PICTURE_MODE_TONE => Some("PictureModeTone"),
        CS_NOISE_FILTER => Some("NoiseFilter"),
        CS_ART_FILTER => Some("ArtFilter"),
        CS_MAGIC_FILTER => Some("MagicFilter"),
        CS_DRIVE_MODE => Some("DriveMode"),
        CS_PANORAMA_MODE => Some("PanoramaMode"),
        CS_IMAGE_QUALITY2 => Some("ImageQuality2"),
        CS_IMAGE_STABILIZATION => Some("ImageStabilization"),
        _ => None,
    }
}

fn get_raw_development_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        RD_VERSION => Some("RawDevVersion"),
        RD_EXPOSURE_BIAS_VALUE => Some("RawDevExposureBiasValue"),
        RD_WHITE_BALANCE_VALUE => Some("RawDevWhiteBalanceValue"),
        RD_WB_FINE_ADJUSTMENT => Some("RawDevWBFineAdjustment"),
        RD_GRAY_POINT => Some("RawDevGrayPoint"),
        RD_SATURATION_EMPHASIS => Some("RawDevSaturationEmphasis"),
        RD_MEMORY_COLOR_EMPHASIS => Some("RawDevMemoryColorEmphasis"),
        RD_CONTRAST_VALUE => Some("RawDevContrastValue"),
        RD_SHARPNESS_VALUE => Some("RawDevSharpnessValue"),
        RD_COLOR_SPACE => Some("RawDevColorSpace"),
        RD_ENGINE => Some("RawDevEngine"),
        RD_NOISE_REDUCTION => Some("RawDevNoiseReduction"),
        RD_EDIT_STATUS => Some("RawDevEditStatus"),
        RD_SETTINGS => Some("RawDevSettings"),
        _ => None,
    }
}

fn get_image_processing_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        IP_VERSION => Some("ImageProcessingVersion"),
        IP_WB_RB_LEVELS => Some("WB_RBLevels"),
        IP_WB_RB_LEVELS_3000K => Some("WB_RBLevels3000K"),
        IP_WB_RB_LEVELS_3300K => Some("WB_RBLevels3300K"),
        IP_WB_RB_LEVELS_3600K => Some("WB_RBLevels3600K"),
        IP_WB_RB_LEVELS_3900K => Some("WB_RBLevels3900K"),
        IP_WB_RB_LEVELS_4000K => Some("WB_RBLevels4000K"),
        IP_WB_RB_LEVELS_4300K => Some("WB_RBLevels4300K"),
        IP_WB_RB_LEVELS_4500K => Some("WB_RBLevels4500K"),
        IP_WB_RB_LEVELS_4800K => Some("WB_RBLevels4800K"),
        IP_WB_RB_LEVELS_5300K => Some("WB_RBLevels5300K"),
        IP_WB_RB_LEVELS_6000K => Some("WB_RBLevels6000K"),
        IP_WB_RB_LEVELS_6600K => Some("WB_RBLevels6600K"),
        IP_WB_RB_LEVELS_7500K => Some("WB_RBLevels7500K"),
        IP_COLOR_MATRIX => Some("ColorMatrix"),
        IP_BLACK_LEVEL => Some("BlackLevel2"),
        IP_GAIN_BASE => Some("GainBase"),
        IP_VALID_BITS => Some("ValidBits"),
        IP_CROP_LEFT => Some("CropLeft"),
        IP_CROP_TOP => Some("CropTop"),
        IP_CROP_WIDTH => Some("CropWidth"),
        IP_CROP_HEIGHT => Some("CropHeight"),
        IP_NOISE_REDUCTION2 => Some("NoiseReduction2"),
        IP_DISTORTION_CORRECTION2 => Some("DistortionCorrection2"),
        IP_SHADING_COMPENSATION2 => Some("ShadingCompensation2"),
        IP_MULTIPLE_EXPOSURE_MODE => Some("MultipleExposureMode"),
        IP_ASPECT_RATIO => Some("AspectRatio"),
        IP_FACES_DETECTED => Some("FacesDetected"),
        IP_FACE_DETECT_AREA => Some("FaceDetectArea"),
        _ => None,
    }
}

fn get_focus_info_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        FI_VERSION => Some("FocusInfoVersion"),
        FI_AUTO_FOCUS => Some("AutoFocus"),
        FI_SCENE_DETECT => Some("SceneDetect"),
        FI_ZOOM_STEP_COUNT => Some("ZoomStepCount"),
        FI_FOCUS_STEP_COUNT => Some("FocusStepCount"),
        FI_FOCUS_STEP_INFINITY => Some("FocusStepInfinity"),
        FI_FOCUS_STEP_NEAR => Some("FocusStepNear"),
        FI_FOCUS_DISTANCE => Some("FocusDistance"),
        FI_AF_POINT => Some("AFPoint"),
        FI_EXTERNAL_FLASH => Some("ExternalFlash"),
        FI_EXTERNAL_FLASH_BOUNCE => Some("ExternalFlashBounce"),
        FI_EXTERNAL_FLASH_ZOOM => Some("ExternalFlashZoom"),
        FI_INTERNAL_FLASH => Some("InternalFlash"),
        FI_MANUAL_FLASH => Some("ManualFlash"),
        FI_MACRO_LED => Some("MacroLED"),
        FI_SENSOR_TEMPERATURE => Some("SensorTemperature"),
        FI_IMAGE_STABILIZATION => Some("ImageStabilization"),
        _ => None,
    }
}

// =============================================================================
// Decode Functions - ExifTool and exiv2 compatible value decoders
// =============================================================================

// ExposureMode (CS 0x0200): Olympus.pm / olympusmn_int.cpp
define_tag_decoder! {
    exposure_mode,
    both: {
        1 => "Manual",
        2 => "Program",
        3 => "Aperture-priority AE",
        4 => "Shutter speed priority AE",
        5 => "Program-shift",
    }
}

// MeteringMode (CS 0x0202): Olympus.pm / olympusmn_int.cpp
define_tag_decoder! {
    metering_mode,
    both: {
        2 => "Center-weighted average",
        3 => "Spot",
        5 => "ESP",
        261 => "Pattern+AF",
        515 => "Spot+Highlight control",
        1027 => "Spot+Shadow control",
    }
}

// ExifMeteringMode (tag 0x9207) with Olympus-specific names
// Olympus uses "ESP" instead of "Multi-segment" for pattern metering
define_tag_decoder! {
    exif_metering_mode,
    both: {
        0 => "Unknown",
        1 => "Average",
        2 => "Center-weighted average",
        3 => "Spot",
        4 => "Multi-spot",
        5 => "ESP",
        6 => "Partial",
        255 => "Other",
    }
}

// MacroMode (CS 0x0300 and Main 0x0202): Olympus.pm / olympusmn_int.cpp
define_tag_decoder! {
    macro_mode,
    both: {
        0 => "Off",
        1 => "On",
        2 => "Super Macro",
    }
}

// FocusMode (CS 0x0301): Olympus.pm / olympusmn_int.cpp
define_tag_decoder! {
    focus_mode,
    exiftool: {
        0 => "Single AF",
        1 => "Sequential shooting AF",
        2 => "Continuous AF",
        3 => "Multi AF",
        4 => "Face Detect",
        10 => "MF",
    },
    exiv2: {
        0 => "Single AF",
        1 => "Sequential shooting AF",
        2 => "Continuous AF",
        3 => "Multi AF",
        10 => "MF",
    }
}

// FocusProcess (CS 0x0302): Olympus.pm / olympusmn_int.cpp
define_tag_decoder! {
    focus_process,
    both: {
        0 => "AF Not Used",
        1 => "AF Used",
    }
}

// AFSearch (CS 0x0303): Olympus.pm / olympusmn_int.cpp
define_tag_decoder! {
    af_search,
    both: {
        0 => "Not Ready",
        1 => "Ready",
    }
}

/// Decode SpecialMode value (0x0200) - 3 values: shooting mode, sequence, panorama direction
/// Format: "Mode, Sequence: N, Panorama: Direction"
pub fn decode_special_mode(values: &[u32]) -> String {
    if values.len() < 3 {
        return format!("{:?}", values);
    }

    let mode = match values[0] {
        0 => "Normal",
        1 => "Unknown (1)",
        2 => "Fast",
        3 => "Panorama",
        n => {
            return format!(
                "Unknown ({}), Sequence: {}, Panorama: Unknown",
                n, values[1]
            )
        }
    };

    let panorama = match values[2] {
        0 => "(none)",
        1 => "Left to Right",
        2 => "Right to Left",
        3 => "Bottom to Top",
        4 => "Top to Bottom",
        n => {
            return format!(
                "{}, Sequence: {}, Panorama: Unknown ({})",
                mode, values[1], n
            )
        }
    };

    format!("{}, Sequence: {}, Panorama: {}", mode, values[1], panorama)
}

/// Decode FlashMode value (CS 0x0400) - ExifTool format (bitmask)
/// Note: exiv2 uses identical values - no separate version needed
pub fn decode_flash_mode_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        2 => "Fill-in",
        3 => "On, Fill-in",
        4 => "Red-eye",
        8 => "Slow-sync",
        16 => "Forced On",
        32 => "2nd Curtain",
        _ => {
            // Bitmask combinations - return simple description
            if value & 1 != 0 {
                "On"
            } else {
                "Unknown"
            }
        }
    }
}
// decode_flash_mode_exiv2 - same as exiftool, no separate function needed

// WhiteBalance (CS 0x0500): Olympus.pm / olympusmn_int.cpp
define_tag_decoder! {
    white_balance,
    both: {
        0 => "Auto",
        1 => "Auto (Keep Warm Color Off)",
        16 => "7500K (Fine Weather with Shade)",
        17 => "6000K (Cloudy)",
        18 => "5300K (Fine Weather)",
        20 => "3000K (Tungsten light)",
        21 => "3600K (Tungsten light-like)",
        22 => "Auto Setup",
        23 => "5500K (Flash)",
        33 => "6600K (Daylight fluorescent)",
        34 => "4500K (Neutral white fluorescent)",
        35 => "4000K (Cool white fluorescent)",
        36 => "White Fluorescent",
        48 => "3600K (Tungsten light-like)",
        67 => "Underwater",
        256 => "One Touch WB 1",
        257 => "One Touch WB 2",
        258 => "One Touch WB 3",
        259 => "One Touch WB 4",
        512 => "Custom WB 1",
        513 => "Custom WB 2",
        514 => "Custom WB 3",
        515 => "Custom WB 4",
    }
}

// ColorSpace (CS 0x0507): Olympus.pm / olympusmn_int.cpp
define_tag_decoder! {
    color_space,
    both: {
        0 => "sRGB",
        1 => "Adobe RGB",
        2 => "Pro Photo RGB",
    }
}

/// Decode SceneMode value (CS 0x0509) - ExifTool format
pub fn decode_scene_mode_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Standard",
        6 => "Auto",
        7 => "Sport",
        8 => "Portrait",
        9 => "Landscape+Portrait",
        10 => "Landscape",
        11 => "Night Scene",
        12 => "Self Portrait",
        13 => "Panorama",
        14 => "2 in 1",
        15 => "Movie",
        16 => "Landscape+Portrait",
        17 => "Night+Portrait",
        18 => "Indoor",
        19 => "Fireworks",
        20 => "Sunset",
        21 => "Beauty Skin",
        22 => "Macro",
        23 => "Super Macro",
        24 => "Food",
        25 => "Documents",
        26 => "Museum",
        27 => "Shoot & Select",
        28 => "Beach & Snow",
        29 => "Self Portrait+Timer",
        30 => "Candle",
        31 => "Available Light",
        32 => "Behind Glass",
        33 => "My Mode",
        34 => "Pet",
        35 => "Underwater Wide1",
        36 => "Underwater Macro",
        37 => "Shoot & Select1",
        38 => "Shoot & Select2",
        39 => "High Key",
        40 => "Digital Image Stabilization",
        41 => "Auction",
        42 => "Beach",
        43 => "Snow",
        44 => "Underwater Wide2",
        45 => "Low Key",
        46 => "Children",
        47 => "Vivid",
        48 => "Nature Macro",
        49 => "Underwater Snapshot",
        50 => "Shooting Guide",
        54 => "Face Portrait",
        57 => "Bulb",
        59 => "Smile Shot",
        60 => "Quick Shutter",
        63 => "Slow Shutter",
        64 => "Bird Watching",
        65 => "Multiple Exposure",
        66 => "e-Portrait",
        67 => "Soft Background Shot",
        142 => "Hand-held Starlight",
        154 => "HDR",
        197 => "Panning",
        203 => "Light Trails",
        204 => "Backlight HDR",
        205 => "Silent",
        206 => "Multi Focus Shot",
        _ => "Unknown",
    }
}

/// Decode SceneMode value (CS 0x0509) - exiv2 format
/// Note: exiv2 has fewer scene modes than ExifTool
pub fn decode_scene_mode_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Standard",
        6 => "Auto",
        7 => "Sport",
        8 => "Portrait",
        9 => "Landscape+Portrait",
        10 => "Landscape",
        11 => "Night Scene",
        12 => "Self Portrait",
        13 => "Panorama",
        14 => "2 in 1",
        15 => "Movie",
        16 => "Landscape+Portrait",
        17 => "Night+Portrait",
        18 => "Indoor",
        19 => "Fireworks",
        20 => "Sunset",
        22 => "Macro",
        23 => "Super Macro",
        24 => "Food",
        25 => "Documents",
        26 => "Museum",
        27 => "Shoot & Select",
        28 => "Beach & Snow",
        29 => "Self Protrait+Timer",
        30 => "Candle",
        31 => "Available Light",
        32 => "Behind Glass",
        33 => "My Mode",
        34 => "Pet",
        35 => "Underwater Wide1",
        36 => "Underwater Macro",
        37 => "Shoot & Select1",
        38 => "Shoot & Select2",
        39 => "High Key",
        40 => "Digital Image Stabilization",
        41 => "Auction",
        42 => "Beach",
        43 => "Snow",
        44 => "Underwater Wide2",
        45 => "Low Key",
        46 => "Children",
        47 => "Vivid",
        48 => "Nature Macro",
        49 => "Underwater Snapshot",
        50 => "Shooting Guide",
        _ => "Unknown",
    }
}

/// Decode NoiseReduction value (CS 0x050A) - ExifTool format (bitmask)
/// Note: exiv2 uses identical values - no separate version needed
pub fn decode_noise_reduction_exiftool(value: u16) -> &'static str {
    match value {
        0 => "(none)",
        1 => "Noise Reduction",
        2 => "Noise Filter",
        3 => "Noise Reduction, Noise Filter",
        4 => "Noise Filter (ISO Boost)",
        8 => "Auto",
        _ => {
            if value & 8 != 0 {
                "Auto"
            } else if value != 0 {
                "On"
            } else {
                "Unknown"
            }
        }
    }
}
// decode_noise_reduction_exiv2 - same as exiftool, no separate function needed

/// Decode Extender value from make/model bytes
/// Returns ExifValue::Ascii with extender name or the original value
fn decode_extender(make: u16, model: u16) -> ExifValue {
    // ExifTool uses hex format "make model" to look up extender name
    // Common values from Olympus.pm PrintConv:
    // '0 00' => 'None'
    // '1 10' => 'Olympus Zuiko Digital EC-14 1.4x Teleconverter'
    // '1 20' => 'Olympus EX-25 Extension Tube'
    // '1 30' => 'Olympus Zuiko Digital EC-20 2x Teleconverter'
    let name = match (make, model) {
        (0, 0) => "None",
        (1, 0x10) => "Olympus Zuiko Digital EC-14 1.4x Teleconverter",
        (1, 0x20) => "Olympus EX-25 Extension Tube",
        (1, 0x30) => "Olympus Zuiko Digital EC-20 2x Teleconverter",
        _ => return ExifValue::Ascii(format!("{} {:02x}", make, model)),
    };
    ExifValue::Ascii(name.to_string())
}

/// Decode Gradation value (CS 0x050F) - 4 int16s values
/// First 3 values decode to gradation type, 4th value is mode
fn decode_gradation(vals: &[i16]) -> String {
    if vals.len() < 3 {
        return vals
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ");
    }

    // Decode first 3 values as gradation type
    let gradation_type = match (vals[0], vals[1], vals[2]) {
        (0, 0, 0) => "n/a",
        (-1, -1, 1) => "Low Key",
        (0, -1, 1) => "Normal",
        (1, -1, 1) => "High Key",
        _ => {
            return vals
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        }
    };

    // Decode 4th value as mode (if present)
    if vals.len() >= 4 {
        let mode = match vals[3] {
            0 => "User-Selected",
            1 => "Auto-Override",
            _ => return format!("{}; {}", gradation_type, vals[3]),
        };
        format!("{}; {}", gradation_type, mode)
    } else {
        gradation_type.to_string()
    }
}

// DistortionCorrection (CS 0x050B): Olympus.pm / olympusmn_int.cpp
define_tag_decoder! {
    distortion_correction,
    both: {
        0 => "Off",
        1 => "On",
    }
}

// ShadingCompensation (CS 0x050C): Olympus.pm / olympusmn_int.cpp
define_tag_decoder! {
    shading_compensation,
    both: {
        0 => "Off",
        1 => "On",
    }
}

// PictureMode (CS 0x0520): Olympus.pm / olympusmn_int.cpp
define_tag_decoder! {
    picture_mode,
    exiftool: {
        1 => "Vivid",
        2 => "Natural",
        3 => "Muted",
        4 => "Portrait",
        5 => "i-Enhance",
        6 => "e-Portrait",
        7 => "Color Creator",
        8 => "Underwater",
        9 => "Color Profile 1",
        10 => "Color Profile 2",
        11 => "Color Profile 3",
        12 => "Monochrome Profile 1",
        13 => "Monochrome Profile 2",
        14 => "Monochrome Profile 3",
        17 => "Art Mode",
        18 => "Monochrome Profile 4",
        256 => "Monotone",
        512 => "Sepia",
    },
    exiv2: {
        1 => "Vivid",
        2 => "Natural",
        3 => "Muted",
        4 => "Portrait",
        5 => "i-Enhance",
        6 => "e-Portrait",
        7 => "Color Creator",
        9 => "Color Profile 1",
        10 => "Color Profile 2",
        11 => "Color Profile 3",
        12 => "Monochrome Profile 1",
        13 => "Monochrome Profile 2",
        14 => "Monochrome Profile 3",
        256 => "Monotone",
        512 => "Sepia",
    }
}

// ImageQuality (CS 0x0603): Olympus.pm / olympusmn_int.cpp
define_tag_decoder! {
    image_quality,
    exiftool: {
        1 => "SQ",
        2 => "HQ",
        3 => "SHQ",
        4 => "RAW",
        5 => "SQ (5)",
    },
    exiv2: {
        1 => "SQ",
        2 => "HQ",
        3 => "SHQ",
        4 => "RAW",
    }
}

// JpegQuality (Main 0x0201): Olympus.pm / olympusmn_int.cpp
define_tag_decoder! {
    jpeg_quality,
    exiftool: {
        1 => "SQ",
        2 => "HQ",
        3 => "SHQ",
        4 => "RAW",
        5 => "Medium-Fine",
        6 => "Small-Fine",
        33 => "Uncompressed",
    },
    exiv2: {
        1 => "Standard Quality (SQ)",
        2 => "High Quality (HQ)",
        3 => "Super High Quality (SHQ)",
        6 => "Raw",
    }
}

// AELock (CS 0x0201): Olympus.pm / olympusmn_int.cpp
define_tag_decoder! {
    ae_lock,
    both: {
        0 => "Off",
        1 => "On",
    }
}

// FlashRemoteControl (CS 0x0403): Olympus.pm / olympusmn_int.cpp
define_tag_decoder! {
    flash_remote_control,
    both: {
        0x0 => "Off",
        0x1 => "Channel 1, Low",
        0x2 => "Channel 2, Low",
        0x3 => "Channel 3, Low",
        0x4 => "Channel 4, Low",
        0x9 => "Channel 1, Mid",
        0xa => "Channel 2, Mid",
        0xb => "Channel 3, Mid",
        0xc => "Channel 4, Mid",
        0x11 => "Channel 1, High",
        0x12 => "Channel 2, High",
        0x13 => "Channel 3, High",
        0x14 => "Channel 4, High",
    }
}

// FlashControlMode (CS 0x0404): Olympus.pm / olympusmn_int.cpp
define_tag_decoder! {
    flash_control_mode,
    both: {
        0 => "Off",
        3 => "TTL",
        4 => "Auto",
        5 => "Manual",
    }
}

// ModifiedSaturation (CS 0x0504): Olympus.pm / olympusmn_int.cpp
define_tag_decoder! {
    modified_saturation,
    both: {
        0 => "Off",
        1 => "CM1 (Red Enhance)",
        2 => "CM2 (Green Enhance)",
        3 => "CM3 (Blue Enhance)",
        4 => "CM4 (Skin Tones)",
    }
}

// ArtFilter (CS 0x0529): Olympus.pm / olympusmn_int.cpp
define_tag_decoder! {
    art_filter,
    both: {
        0 => "Off",
        1 => "Soft Focus",
        2 => "Pop Art",
        3 => "Pale & Light Color",
        4 => "Light Tone",
        5 => "Pin Hole",
        6 => "Grainy Film",
        9 => "Diorama",
        10 => "Cross Process",
        12 => "Fish Eye",
        13 => "Drawing",
        14 => "Gentle Sepia",
        15 => "Pale & Light Color II",
        16 => "Pop Art II",
        17 => "Pin Hole II",
        18 => "Pin Hole III",
        19 => "Grainy Film II",
        20 => "Dramatic Tone",
        21 => "Punk",
        22 => "Soft Focus 2",
        23 => "Sparkle",
        24 => "Watercolor",
        25 => "Key Line",
        26 => "Key Line II",
        27 => "Miniature",
        28 => "Reflection",
        29 => "Fragmented",
        31 => "Cross Process II",
        32 => "Dramatic Tone II",
        33 => "Watercolor I",
        34 => "Watercolor II",
        35 => "Diorama II",
        36 => "Vintage",
        37 => "Vintage II",
        38 => "Vintage III",
        39 => "Partial Color",
        40 => "Partial Color II",
        41 => "Partial Color III",
    }
}

// ImageStabilization (CS 0x0604): Olympus.pm / olympusmn_int.cpp
define_tag_decoder! {
    image_stabilization,
    both: {
        0 => "Off",
        1 => "On, Mode 1",
        2 => "On, Mode 2",
        3 => "On, Mode 3",
        4 => "On, Mode 4",
    }
}

// MultipleExposureMode (IP 0x101C): Olympus.pm / olympusmn_int.cpp
define_tag_decoder! {
    multiple_exposure_mode,
    both: {
        0 => "Off",
        2 => "On (2 frames)",
        3 => "On (3 frames)",
    }
}

// AspectRatio (IP 0x1112): Olympus.pm / olympusmn_int.cpp
define_tag_decoder! {
    aspect_ratio,
    both: {
        1 => "4:3",
        2 => "3:2",
        3 => "16:9",
        4 => "6:6",
        5 => "5:4",
        6 => "7:6",
        7 => "6:5",
        8 => "7:5",
        9 => "3:4",
    }
}

// ExternalFlashBounce (FI 0x1203): Olympus.pm / olympusmn_int.cpp
define_tag_decoder! {
    external_flash_bounce,
    both: {
        0 => "Bounce or Off",
        1 => "Direct",
    }
}

// RawDevColorSpace (RD 0x0108): Olympus.pm
define_tag_decoder! {
    raw_dev_color_space,
    both: {
        0 => "sRGB",
        1 => "Adobe RGB",
        2 => "Pro Photo RGB",
    }
}

// RawDevEngine (RD 0x0109): Olympus.pm
define_tag_decoder! {
    raw_dev_engine,
    both: {
        0 => "High Speed",
        1 => "High Function",
        2 => "Advanced High Speed",
        3 => "Advanced High Function",
    }
}

// RawDevNoiseReduction (RD 0x010A): Olympus.pm
define_tag_decoder! {
    raw_dev_noise_reduction,
    both: {
        0 => "(none)",
        1 => "Noise Reduction",
        2 => "Noise Filter",
        3 => "Noise Reduction, Noise Filter",
        4 => "Noise Filter (ISO Boost)",
    }
}

// RawDevEditStatus (RD 0x010B): Olympus.pm
define_tag_decoder! {
    raw_dev_edit_status,
    both: {
        0 => "Original",
        1 => "Edited (Landscape)",
        6 => "Edited (Portrait)",
        8 => "Edited (Portrait)",
    }
}

// PanoramaMode (CS 0x0601): Olympus.pm
define_tag_decoder! {
    panorama_mode,
    both: {
        0 => "Off",
        1 => "On",
    }
}

// =============================================================================
// IFD Parsing Functions
// =============================================================================

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

/// Read i16 with given endianness
fn read_i16(data: &[u8], endian: Endianness) -> i16 {
    match endian {
        Endianness::Little => i16::from_le_bytes([data[0], data[1]]),
        Endianness::Big => i16::from_be_bytes([data[0], data[1]]),
    }
}

/// Read i32 with given endianness
fn read_i32(data: &[u8], endian: Endianness) -> i32 {
    match endian {
        Endianness::Little => i32::from_le_bytes([data[0], data[1], data[2], data[3]]),
        Endianness::Big => i32::from_be_bytes([data[0], data[1], data[2], data[3]]),
    }
}

/// Parse a single IFD entry and return the tag value
///
/// # Arguments
/// * `data` - The IFD data containing entries
/// * `entry_offset` - Offset to this entry within data
/// * `base_offset` - Base offset for value offsets
/// * `endian` - Byte order
/// * `value_data` - Optional alternate data source for value offsets (for old format)
/// * `value_base_offset` - Base offset when using value_data
fn parse_ifd_entry(
    data: &[u8],
    entry_offset: usize,
    base_offset: usize,
    endian: Endianness,
    value_data: Option<&[u8]>,
    value_base_offset: usize,
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
        1 | 2 | 6 | 7 => 1,   // BYTE, ASCII, SBYTE, UNDEFINED
        3 | 8 => 2,           // SHORT, SSHORT
        4 | 9 | 11 | 13 => 4, // LONG, SLONG, FLOAT, IFD
        5 | 10 | 12 => 8,     // RATIONAL, SRATIONAL, DOUBLE
        _ => return None,
    };

    let total_size = count * type_size;

    // Get the actual data location
    // For values > 4 bytes, use value_data (TIFF data) if provided, else use data
    let value_bytes: &[u8] = if total_size <= 4 {
        value_offset_bytes
    } else {
        let raw_offset = read_u32(value_offset_bytes, endian) as usize;

        // Use alternate data source if provided (for old Olympus format)
        let (src_data, src_base) = if let Some(vd) = value_data {
            (vd, value_base_offset)
        } else {
            (data, base_offset)
        };

        let offset = raw_offset + src_base;
        if tag_id == 0x2010 || tag_id == 0x2020 {
            eprintln!(
                "DEBUG: tag {:04x}: raw_offset={}, src_base={}, offset={}, src_data.len={}, total_size={}",
                tag_id, raw_offset, src_base, offset, src_data.len(), total_size
            );
        }
        if offset + total_size > src_data.len() {
            return None;
        }
        &src_data[offset..offset + total_size]
    };

    // Parse based on type
    let value = match tag_type {
        1 => {
            // BYTE
            ExifValue::Byte(value_bytes[..count].to_vec())
        }
        2 => {
            // ASCII
            let s = value_bytes[..count]
                .iter()
                .take_while(|&&b| b != 0)
                .map(|&b| b as char)
                .collect::<String>();
            ExifValue::Ascii(s)
        }
        3 => {
            // SHORT
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                if i * 2 + 2 <= value_bytes.len() {
                    values.push(read_u16(&value_bytes[i * 2..], endian));
                }
            }
            ExifValue::Short(values)
        }
        4 => {
            // LONG
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                if i * 4 + 4 <= value_bytes.len() {
                    values.push(read_u32(&value_bytes[i * 4..], endian));
                }
            }
            ExifValue::Long(values)
        }
        5 => {
            // RATIONAL
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                if i * 8 + 8 <= value_bytes.len() {
                    let num = read_u32(&value_bytes[i * 8..], endian);
                    let den = read_u32(&value_bytes[i * 8 + 4..], endian);
                    values.push((num, den));
                }
            }
            ExifValue::Rational(values)
        }
        6 => {
            // SBYTE
            ExifValue::SByte(value_bytes[..count].iter().map(|&b| b as i8).collect())
        }
        7 => {
            // UNDEFINED
            ExifValue::Undefined(value_bytes[..count.min(value_bytes.len())].to_vec())
        }
        8 => {
            // SSHORT
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                if i * 2 + 2 <= value_bytes.len() {
                    values.push(read_i16(&value_bytes[i * 2..], endian));
                }
            }
            ExifValue::SShort(values)
        }
        9 => {
            // SLONG
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                if i * 4 + 4 <= value_bytes.len() {
                    values.push(read_i32(&value_bytes[i * 4..], endian));
                }
            }
            ExifValue::SLong(values)
        }
        10 => {
            // SRATIONAL
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                if i * 8 + 8 <= value_bytes.len() {
                    let num = read_i32(&value_bytes[i * 8..], endian);
                    let den = read_i32(&value_bytes[i * 8 + 4..], endian);
                    values.push((num, den));
                }
            }
            ExifValue::SRational(values)
        }
        13 => {
            // IFD pointer - treat like LONG (offset to sub-IFD)
            let mut values = Vec::with_capacity(count);
            if count == 1 {
                let value = read_u32(value_offset_bytes, endian);
                values.push(value);
            } else {
                let offset = read_u32(value_offset_bytes, endian) as usize + base_offset;
                if offset + count * 4 <= data.len() {
                    for i in 0..count {
                        let value = read_u32(&data[offset + i * 4..], endian);
                        values.push(value);
                    }
                }
            }
            ExifValue::Long(values)
        }
        _ => return None,
    };

    Some((tag_id, value))
}

/// Parse an embedded sub-IFD (old Olympus format where sub-IFD data is stored directly)
fn parse_embedded_sub_ifd(
    sub_data: &[u8],
    endian: Endianness,
    ifd_type: OlympusIfdType,
    tags: &mut HashMap<u16, MakerNoteTag>,
    prefix: &str,
) {
    // Embedded sub-IFDs have self-contained data, no need for TIFF data
    parse_olympus_ifd(sub_data, 0, 0, endian, ifd_type, tags, prefix, None, 0);
}

/// Parse an Olympus IFD and return tags
///
/// # Arguments
/// * `data` - The IFD data (maker note data)
/// * `ifd_offset` - Offset to IFD entry count within data
/// * `base_offset` - Base offset for IFD entry offsets
/// * `endian` - Byte order
/// * `ifd_type` - Type of IFD being parsed
/// * `tags` - Output HashMap for parsed tags
/// * `prefix` - Tag name prefix
/// * `value_data` - Optional alternate data source for value offsets (for old format)
/// * `value_base_offset` - Base offset when using value_data
#[allow(clippy::too_many_arguments)]
fn parse_olympus_ifd(
    data: &[u8],
    ifd_offset: usize,
    base_offset: usize,
    endian: Endianness,
    ifd_type: OlympusIfdType,
    tags: &mut HashMap<u16, MakerNoteTag>,
    prefix: &str,
    value_data: Option<&[u8]>,
    value_base_offset: usize,
) {
    if ifd_offset + 2 > data.len() {
        return;
    }

    let num_entries = read_u16(&data[ifd_offset..], endian) as usize;

    // Sanity check - relax by a few bytes to handle slightly short embedded IFDs
    if num_entries > 500 || ifd_offset + 2 + num_entries * 12 > data.len() + 4 {
        return;
    }

    // Calculate how many entries we can actually read
    let available_bytes = data.len().saturating_sub(ifd_offset + 2);
    let max_entries = available_bytes / 12;
    let actual_entries = num_entries.min(max_entries);

    for i in 0..actual_entries {
        let entry_offset = ifd_offset + 2 + i * 12;

        if let Some((tag_id, value)) = parse_ifd_entry(
            data,
            entry_offset,
            base_offset,
            endian,
            value_data,
            value_base_offset,
        ) {
            let tag_name = get_olympus_tag_name(tag_id, ifd_type);

            // Create a unique tag ID for storage (combine prefix with tag ID)
            let storage_id = match ifd_type {
                OlympusIfdType::Main => tag_id,
                OlympusIfdType::Equipment => 0x2010 + tag_id,
                OlympusIfdType::CameraSettings => 0x2020 + tag_id,
                OlympusIfdType::RawDevelopment => 0x2030 + tag_id,
                OlympusIfdType::ImageProcessing => 0x2040 + tag_id,
                OlympusIfdType::FocusInfo => 0x2050 + tag_id,
            };

            // Skip thumbnail/preview data to save memory
            if tag_id == OLYMPUS_THUMBNAIL_IMAGE || tag_id == OLYMPUS_PREVIEW_IMAGE {
                continue;
            }

            // Convert specific tags to decoded values
            let value = match ifd_type {
                OlympusIfdType::Main => match tag_id {
                    OLYMPUS_CAMERA_ID => {
                        // CameraID is actually an ASCII string with null terminator
                        if let ExifValue::Undefined(bytes) = &value {
                            let s: String = bytes
                                .iter()
                                .take_while(|&&b| b != 0)
                                .map(|&b| b as char)
                                .collect();
                            ExifValue::Ascii(s.trim().to_string())
                        } else {
                            value
                        }
                    }
                    OLYMPUS_SPECIAL_MODE => {
                        // SpecialMode is 3 uint32 values that need to be formatted
                        if let ExifValue::Long(vals) = &value {
                            ExifValue::Ascii(decode_special_mode(vals))
                        } else {
                            value
                        }
                    }
                    OLYMPUS_FOCAL_PLANE_DIAGONAL => {
                        // FocalPlaneDiagonal: add " mm" suffix
                        match &value {
                            ExifValue::Rational(vals) if !vals.is_empty() => {
                                let (num, denom) = vals[0];
                                if denom != 0 {
                                    let v = num as f64 / denom as f64;
                                    // Format without trailing zeros (ExifTool style)
                                    let formatted = format!("{:.2}", v);
                                    let formatted =
                                        formatted.trim_end_matches('0').trim_end_matches('.');
                                    ExifValue::Ascii(format!("{} mm", formatted))
                                } else {
                                    value
                                }
                            }
                            _ => value,
                        }
                    }
                    _ => value,
                },
                OlympusIfdType::Equipment => match tag_id {
                    EQUIP_VERSION => {
                        // Version is 4 bytes of ASCII that should be displayed as string
                        if let ExifValue::Undefined(bytes) = &value {
                            let s: String = bytes
                                .iter()
                                .take_while(|&&b| b != 0)
                                .map(|&b| b as char)
                                .collect();
                            ExifValue::Ascii(s)
                        } else {
                            value
                        }
                    }
                    EQUIP_EXTENDER => {
                        // Extender: 6 int8u values - decode as "None" if all zeros
                        // Can be stored as Byte (u8) or Short (u16) depending on parser
                        match &value {
                            ExifValue::Byte(vals) => {
                                if vals.len() >= 6 && vals.iter().all(|&v| v == 0) {
                                    ExifValue::Ascii("None".to_string())
                                } else if vals.len() >= 3 {
                                    decode_extender(vals[0] as u16, vals[2] as u16)
                                } else {
                                    value
                                }
                            }
                            ExifValue::Short(vals) => {
                                if vals.len() >= 6 && vals.iter().all(|&v| v == 0) {
                                    ExifValue::Ascii("None".to_string())
                                } else if vals.len() >= 3 {
                                    decode_extender(vals[0], vals[2])
                                } else {
                                    value
                                }
                            }
                            _ => value,
                        }
                    }
                    EQUIP_FOCAL_PLANE_DIAGONAL => {
                        // FocalPlaneDiagonal: add " mm" suffix
                        match &value {
                            ExifValue::Rational(vals) if !vals.is_empty() => {
                                let (num, denom) = vals[0];
                                if denom != 0 {
                                    let v = num as f64 / denom as f64;
                                    // Format without trailing zeros (ExifTool style)
                                    let formatted = format!("{:.2}", v);
                                    let formatted =
                                        formatted.trim_end_matches('0').trim_end_matches('.');
                                    ExifValue::Ascii(format!("{} mm", formatted))
                                } else {
                                    value
                                }
                            }
                            _ => value,
                        }
                    }
                    _ => value,
                },
                OlympusIfdType::CameraSettings => match tag_id {
                    CS_VERSION => {
                        // Version is 4 bytes of ASCII that should be displayed as string
                        if let ExifValue::Undefined(bytes) = &value {
                            let s: String = bytes
                                .iter()
                                .take_while(|&&b| b != 0)
                                .map(|&b| b as char)
                                .collect();
                            ExifValue::Ascii(s)
                        } else {
                            value
                        }
                    }
                    CS_EXPOSURE_MODE => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(decode_exposure_mode_exiftool(vals[0]).to_string())
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_METERING_MODE => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(decode_metering_mode_exiftool(vals[0]).to_string())
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_MACRO_MODE => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(decode_macro_mode_exiftool(vals[0]).to_string())
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_FOCUS_MODE => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(decode_focus_mode_exiftool(vals[0]).to_string())
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_FOCUS_PROCESS => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(decode_focus_process_exiftool(vals[0]).to_string())
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_AF_SEARCH => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(decode_af_search_exiftool(vals[0]).to_string())
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_FLASH_MODE => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(decode_flash_mode_exiftool(vals[0]).to_string())
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_WHITE_BALANCE => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(decode_white_balance_exiftool(vals[0]).to_string())
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_COLOR_SPACE => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(decode_color_space_exiftool(vals[0]).to_string())
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_SCENE_MODE => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(decode_scene_mode_exiftool(vals[0]).to_string())
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_NOISE_REDUCTION => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_noise_reduction_exiftool(vals[0]).to_string(),
                                )
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_AE_LOCK => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(decode_ae_lock_exiftool(vals[0]).to_string())
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_GRADATION => {
                        // Gradation: 3-4 int16s values
                        if let ExifValue::SShort(vals) = &value {
                            ExifValue::Ascii(decode_gradation(vals))
                        } else if let ExifValue::Short(vals) = &value {
                            // Convert u16 to i16 for decode
                            let signed: Vec<i16> = vals.iter().map(|&v| v as i16).collect();
                            ExifValue::Ascii(decode_gradation(&signed))
                        } else {
                            value
                        }
                    }
                    CS_FLASH_REMOTE_CONTROL => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_flash_remote_control_exiftool(vals[0]).to_string(),
                                )
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_FLASH_CONTROL_MODE => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_flash_control_mode_exiftool(vals[0]).to_string(),
                                )
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_MODIFIED_SATURATION => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_modified_saturation_exiftool(vals[0]).to_string(),
                                )
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_PANORAMA_MODE => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(decode_panorama_mode_exiftool(vals[0]).to_string())
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_IMAGE_STABILIZATION => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_image_stabilization_exiftool(vals[0]).to_string(),
                                )
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    _ => value,
                },
                OlympusIfdType::RawDevelopment => match tag_id {
                    RD_VERSION => {
                        // Version is 4 bytes of ASCII that should be displayed as string
                        if let ExifValue::Undefined(bytes) = &value {
                            let s: String = bytes
                                .iter()
                                .take_while(|&&b| b != 0)
                                .map(|&b| b as char)
                                .collect();
                            ExifValue::Ascii(s)
                        } else {
                            value
                        }
                    }
                    RD_COLOR_SPACE => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_raw_dev_color_space_exiftool(vals[0]).to_string(),
                                )
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    RD_ENGINE => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_raw_dev_engine_exiftool(vals[0]).to_string(),
                                )
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    RD_NOISE_REDUCTION => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_raw_dev_noise_reduction_exiftool(vals[0]).to_string(),
                                )
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    RD_EDIT_STATUS => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_raw_dev_edit_status_exiftool(vals[0]).to_string(),
                                )
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    _ => value,
                },
                OlympusIfdType::ImageProcessing => match tag_id {
                    IP_VERSION => {
                        // Version is 4 bytes of ASCII that should be displayed as string
                        if let ExifValue::Undefined(bytes) = &value {
                            let s: String = bytes
                                .iter()
                                .take_while(|&&b| b != 0)
                                .map(|&b| b as char)
                                .collect();
                            ExifValue::Ascii(s)
                        } else {
                            value
                        }
                    }
                    IP_MULTIPLE_EXPOSURE_MODE => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_multiple_exposure_mode_exiftool(vals[0]).to_string(),
                                )
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    IP_ASPECT_RATIO => {
                        if let ExifValue::Byte(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_aspect_ratio_exiftool(vals[0] as u16).to_string(),
                                )
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    _ => value,
                },
                OlympusIfdType::FocusInfo => match tag_id {
                    FI_VERSION => {
                        // Version is 4 bytes of ASCII that should be displayed as string
                        if let ExifValue::Undefined(bytes) = &value {
                            let s: String = bytes
                                .iter()
                                .take_while(|&&b| b != 0)
                                .map(|&b| b as char)
                                .collect();
                            ExifValue::Ascii(s)
                        } else {
                            value
                        }
                    }
                    FI_FOCUS_DISTANCE => {
                        // FocusDistance: stored as rational64u but ExifTool reads as int32u[2]
                        // ExifTool: Format => 'int32u', Count => 2
                        // ValueConv => 'my ($a,$b) = split " ",$val; return 0 if $a == 0xffffffff; return $a / 1000;'
                        // PrintConv => '$val ? "$val m" : "inf"'
                        // The conversion only uses the first value (numerator in mm) / 1000 for meters
                        match &value {
                            ExifValue::Rational(vals) if !vals.is_empty() => {
                                // Rational stores (numerator, denominator)
                                // ExifTool just uses numerator / 1000, ignoring denominator
                                let (num, _) = vals[0];
                                if num == 0xFFFFFFFF {
                                    ExifValue::Ascii("inf".to_string())
                                } else {
                                    let v = num as f64 / 1000.0;
                                    if v == 0.0 {
                                        ExifValue::Ascii("inf".to_string())
                                    } else {
                                        // Format without trailing zeros (ExifTool style)
                                        let formatted = format!("{:.2}", v);
                                        let formatted =
                                            formatted.trim_end_matches('0').trim_end_matches('.');
                                        ExifValue::Ascii(format!("{} m", formatted))
                                    }
                                }
                            }
                            ExifValue::Long(vals) if !vals.is_empty() => {
                                // Format: int32u[2] - just use first value / 1000
                                let num = vals[0];
                                if num == 0xFFFFFFFF {
                                    ExifValue::Ascii("inf".to_string())
                                } else {
                                    let v = num as f64 / 1000.0;
                                    if v == 0.0 {
                                        ExifValue::Ascii("inf".to_string())
                                    } else {
                                        // Format without trailing zeros (ExifTool style)
                                        let formatted = format!("{:.2}", v);
                                        let formatted =
                                            formatted.trim_end_matches('0').trim_end_matches('.');
                                        ExifValue::Ascii(format!("{} m", formatted))
                                    }
                                }
                            }
                            _ => value,
                        }
                    }
                    _ => value,
                },
            };

            tags.insert(
                storage_id,
                MakerNoteTag {
                    tag_id,
                    tag_name,
                    value,
                },
            );

            // Handle sub-IFDs
            if ifd_type == OlympusIfdType::Main {
                match tag_id {
                    OLYMPUS_EQUIPMENT_IFD => {
                        // Extract sub-IFD info before borrowing tags mutably
                        let sub_ifd_info = tags.get(&storage_id).and_then(|tag| match &tag.value {
                            ExifValue::Long(offsets) if !offsets.is_empty() => {
                                Some((Some(offsets[0] as usize + base_offset), None))
                            }
                            ExifValue::Undefined(sub_data) if sub_data.len() > 2 => {
                                Some((None, Some(sub_data.clone())))
                            }
                            _ => None,
                        });
                        if let Some((sub_offset_opt, sub_data_opt)) = sub_ifd_info {
                            if let Some(sub_offset) = sub_offset_opt {
                                parse_olympus_ifd(
                                    data,
                                    sub_offset,
                                    base_offset,
                                    endian,
                                    OlympusIfdType::Equipment,
                                    tags,
                                    &format!("{}Equipment.", prefix),
                                    value_data,
                                    value_base_offset,
                                );
                            } else if let Some(sub_data) = sub_data_opt {
                                parse_embedded_sub_ifd(
                                    &sub_data,
                                    endian,
                                    OlympusIfdType::Equipment,
                                    tags,
                                    &format!("{}Equipment.", prefix),
                                );
                            }
                        }
                    }
                    OLYMPUS_CAMERA_SETTINGS_IFD => {
                        let sub_ifd_info = tags.get(&storage_id).and_then(|tag| match &tag.value {
                            ExifValue::Long(offsets) if !offsets.is_empty() => {
                                Some((Some(offsets[0] as usize + base_offset), None))
                            }
                            ExifValue::Undefined(sub_data) if sub_data.len() > 2 => {
                                Some((None, Some(sub_data.clone())))
                            }
                            _ => None,
                        });
                        if let Some((sub_offset_opt, sub_data_opt)) = sub_ifd_info {
                            if let Some(sub_offset) = sub_offset_opt {
                                parse_olympus_ifd(
                                    data,
                                    sub_offset,
                                    base_offset,
                                    endian,
                                    OlympusIfdType::CameraSettings,
                                    tags,
                                    &format!("{}CameraSettings.", prefix),
                                    value_data,
                                    value_base_offset,
                                );
                            } else if let Some(sub_data) = sub_data_opt {
                                parse_embedded_sub_ifd(
                                    &sub_data,
                                    endian,
                                    OlympusIfdType::CameraSettings,
                                    tags,
                                    &format!("{}CameraSettings.", prefix),
                                );
                            }
                        }
                    }
                    OLYMPUS_RAW_DEVELOPMENT_IFD | OLYMPUS_RAW_DEVELOPMENT2_IFD => {
                        if let ExifValue::Long(offsets) =
                            tags.get(&storage_id).map(|t| &t.value).unwrap()
                        {
                            if !offsets.is_empty() {
                                let sub_offset = offsets[0] as usize + base_offset;
                                parse_olympus_ifd(
                                    data,
                                    sub_offset,
                                    base_offset,
                                    endian,
                                    OlympusIfdType::RawDevelopment,
                                    tags,
                                    &format!("{}RawDev.", prefix),
                                    value_data,
                                    value_base_offset,
                                );
                            }
                        }
                    }
                    OLYMPUS_IMAGE_PROCESSING_IFD => {
                        if let ExifValue::Long(offsets) =
                            tags.get(&storage_id).map(|t| &t.value).unwrap()
                        {
                            if !offsets.is_empty() {
                                let sub_offset = offsets[0] as usize + base_offset;
                                parse_olympus_ifd(
                                    data,
                                    sub_offset,
                                    base_offset,
                                    endian,
                                    OlympusIfdType::ImageProcessing,
                                    tags,
                                    &format!("{}ImageProc.", prefix),
                                    value_data,
                                    value_base_offset,
                                );
                            }
                        }
                    }
                    OLYMPUS_FOCUS_INFO_IFD => {
                        if let ExifValue::Long(offsets) =
                            tags.get(&storage_id).map(|t| &t.value).unwrap()
                        {
                            if !offsets.is_empty() {
                                let sub_offset = offsets[0] as usize + base_offset;
                                parse_olympus_ifd(
                                    data,
                                    sub_offset,
                                    base_offset,
                                    endian,
                                    OlympusIfdType::FocusInfo,
                                    tags,
                                    &format!("{}FocusInfo.", prefix),
                                    value_data,
                                    value_base_offset,
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Parse Olympus maker notes
pub fn parse_olympus_maker_notes(
    data: &[u8],
    _endian: Endianness,
    tiff_data: Option<&[u8]>,
    tiff_offset: usize,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    // Olympus maker notes have two formats:
    // 1. New format: "OLYMPUS\0II\x03\0" or "OLYMPUS\0MM\x00\x03" (12 byte header)
    //    - Offsets are relative to maker note start
    // 2. Old format: "OLYMP\0" (6 byte header) - used by older cameras like E-1, E-300
    //    - Offsets are relative to TIFF header (need tiff_data to resolve)
    if data.len() < 8 {
        return Ok(tags);
    }

    // Determine format and endianness
    let (mn_endian, ifd_offset, is_old_format) = if data.starts_with(b"OLYMPUS\0") {
        // New format: 8-byte header + 2-byte endian + 2-byte version
        if data.len() < 14 {
            return Ok(tags);
        }
        let endian = if data[8] == b'I' && data[9] == b'I' {
            Endianness::Little
        } else if data[8] == b'M' && data[9] == b'M' {
            Endianness::Big
        } else {
            return Ok(tags);
        };
        (endian, 12, false)
    } else if data.starts_with(b"OLYMP\0") {
        // Old format: 6-byte header + 2-byte version, IFD at byte 8
        // Always little-endian based on observed files
        (Endianness::Little, 8, true)
    } else {
        return Ok(tags);
    };

    // For old format, offsets in IFD entries are relative to TIFF header.
    // We pass tiff_data for resolving those offsets.
    let value_data = if is_old_format { tiff_data } else { None };
    let value_base_offset = if is_old_format { tiff_offset } else { 0 };

    // Parse the main IFD from maker note data
    parse_olympus_ifd(
        data,
        ifd_offset,
        0, // base_offset for reading IFD entries from maker note
        mn_endian,
        OlympusIfdType::Main,
        &mut tags,
        "Olympus.",
        value_data,
        value_base_offset,
    );

    Ok(tags)
}
