// makernotes/olympus.rs - Olympus maker notes parsing

use crate::data_types::{Endianness, ExifValue};
use crate::define_tag_decoder;
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use std::collections::HashMap;

// exiv2 group names for Olympus sub-IFDs
// Note: exiv2 uses "Olympus" for older cameras (C/E-series before E-1) and "Olympus2" for newer
pub const EXIV2_GROUP_OLYMPUS: &str = "Olympus";
pub const EXIV2_GROUP_OLYMPUS2: &str = "Olympus2";
pub const EXIV2_GROUP_OLYMPUS_CS: &str = "OlympusCs";
pub const EXIV2_GROUP_OLYMPUS_EQ: &str = "OlympusEq";
pub const EXIV2_GROUP_OLYMPUS_RD: &str = "OlympusRd";
pub const EXIV2_GROUP_OLYMPUS_RD2: &str = "OlympusRd2";
pub const EXIV2_GROUP_OLYMPUS_IP: &str = "OlympusIp";
pub const EXIV2_GROUP_OLYMPUS_FI: &str = "OlympusFi";
pub const EXIV2_GROUP_OLYMPUS_RI: &str = "OlympusRi";

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
pub const OLYMPUS_RED_BALANCE: u16 = 0x1017;
pub const OLYMPUS_BLUE_BALANCE: u16 = 0x1018;
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
pub const FI_EXTERNAL_FLASH_GUIDE_NUMBER: u16 = 0x1203;
pub const FI_EXTERNAL_FLASH_BOUNCE: u16 = 0x1204;
pub const FI_EXTERNAL_FLASH_ZOOM: u16 = 0x1205;
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

/// Get the exiv2 group name for a given Olympus IFD type
/// Note: Uses "Olympus2" for main IFD since most test cameras are newer models
pub fn get_exiv2_group_for_ifd(ifd_type: OlympusIfdType) -> &'static str {
    match ifd_type {
        OlympusIfdType::Main => EXIV2_GROUP_OLYMPUS2,
        OlympusIfdType::Equipment => EXIV2_GROUP_OLYMPUS_EQ,
        OlympusIfdType::CameraSettings => EXIV2_GROUP_OLYMPUS_CS,
        OlympusIfdType::RawDevelopment => EXIV2_GROUP_OLYMPUS_RD,
        OlympusIfdType::ImageProcessing => EXIV2_GROUP_OLYMPUS_IP,
        OlympusIfdType::FocusInfo => EXIV2_GROUP_OLYMPUS_FI,
    }
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
        OLYMPUS_SCENE_MODE => Some("SceneMode"),
        OLYMPUS_SERIAL_NUMBER => Some("SerialNumber"),
        OLYMPUS_FIRMWARE => Some("Firmware"),
        OLYMPUS_RED_BALANCE => Some("RedBalance"),
        OLYMPUS_BLUE_BALANCE => Some("BlueBalance"),
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
        FI_EXTERNAL_FLASH_GUIDE_NUMBER => Some("ExternalFlashGuideNumber"),
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

// FocusMode second element bitmask decoder (CS 0x0301)
// Bits: 0=S-AF, 2=C-AF, 4=MF, 5=Face Detect, 6=Imager AF, 7=Live View Magnification Frame, 8=AF sensor, 9=Starry Sky AF
fn decode_focus_mode_bitmask(value: u16) -> String {
    if value == 0 {
        return "(none)".to_string();
    }
    let mut parts = Vec::new();
    if value & 0x0001 != 0 {
        parts.push("S-AF");
    }
    if value & 0x0004 != 0 {
        parts.push("C-AF");
    }
    if value & 0x0010 != 0 {
        parts.push("MF");
    }
    if value & 0x0020 != 0 {
        parts.push("Face Detect");
    }
    if value & 0x0040 != 0 {
        parts.push("Imager AF");
    }
    if value & 0x0080 != 0 {
        parts.push("Live View Magnification Frame");
    }
    if value & 0x0100 != 0 {
        parts.push("AF sensor");
    }
    if value & 0x0200 != 0 {
        parts.push("Starry Sky AF");
    }
    if parts.is_empty() {
        format!("Unknown ({})", value)
    } else {
        parts.join(", ")
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
            // When mode is unknown, panorama value should also show the raw value
            return format!(
                "Unknown ({}), Sequence: {}, Panorama: Unknown ({})",
                n, values[1], values[2]
            );
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
            );
        }
    };

    format!("{}, Sequence: {}, Panorama: {}", mode, values[1], panorama)
}

/// Decode FlashMode value (CS/FI 0x0400) - ExifTool format (bitmask)
/// BITMASK: bit0=On, bit1=Fill-in, bit2=Red-eye, bit3=Slow-sync, bit4=Forced On, bit5=2nd Curtain
/// Note: exiv2 uses identical values - no separate version needed
pub fn decode_flash_mode_exiftool(value: u16) -> String {
    if value == 0 {
        return "Off".to_string();
    }
    let mut parts: Vec<&str> = Vec::new();
    if value & 0x01 != 0 {
        parts.push("On");
    }
    if value & 0x02 != 0 {
        parts.push("Fill-in");
    }
    if value & 0x04 != 0 {
        parts.push("Red-eye");
    }
    if value & 0x08 != 0 {
        parts.push("Slow-sync");
    }
    if value & 0x10 != 0 {
        parts.push("Forced On");
    }
    if value & 0x20 != 0 {
        parts.push("2nd Curtain");
    }
    if parts.is_empty() {
        format!("Unknown ({})", value)
    } else {
        parts.join(", ")
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

/// Decode Main IFD SceneMode value (0x0403) - ExifTool format
/// Note: This is different from CS SceneMode (0x0509) - main IFD uses 0=Normal, 1=Standard
pub fn decode_main_scene_mode_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Normal",
        1 => "Standard",
        2 => "Auto",
        3 => "Intelligent Auto",
        4 => "Portrait",
        5 => "Landscape+Portrait",
        6 => "Landscape",
        7 => "Night Scene",
        8 => "Night+Portrait",
        9 => "Sport",
        10 => "Self Portrait",
        11 => "Indoor",
        12 => "Beach & Snow",
        13 => "Beach",
        14 => "Snow",
        15 => "Self Portrait+Self Timer",
        16 => "Sunset",
        17 => "Cuisine",
        18 => "Documents",
        19 => "Candle",
        20 => "Fireworks",
        21 => "Available Light",
        22 => "Vivid",
        23 => "Underwater Wide1",
        24 => "Underwater Macro",
        25 => "Museum",
        26 => "Behind Glass",
        27 => "Auction",
        28 => "Shoot & Select1",
        29 => "Shoot & Select2",
        30 => "Underwater Wide2",
        31 => "Smile Shot",
        32 => "Quick Shutter",
        43 => "Hand-held Starlight",
        100 => "Panorama",
        203 => "HDR",
        _ => "Unknown",
    }
}

/// Decode NoiseReduction value (CS 0x050A) - ExifTool format (bitmask)
/// Note: exiv2 uses identical values - no separate version needed
/// BITMASK: bit0=Noise Reduction, bit1=Noise Filter, bit2=Noise Filter (ISO Boost), bit3=Auto
pub fn decode_noise_reduction_exiftool(value: u16) -> String {
    if value == 0 {
        return "(none)".to_string();
    }

    let mut parts = Vec::new();
    if value & 0x01 != 0 {
        parts.push("Noise Reduction");
    }
    if value & 0x02 != 0 {
        parts.push("Noise Filter");
    }
    if value & 0x04 != 0 {
        parts.push("Noise Filter (ISO Boost)");
    }
    if value & 0x08 != 0 {
        parts.push("Auto");
    }

    if parts.is_empty() {
        format!("Unknown ({})", value)
    } else {
        parts.join(", ")
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
                .join(" ");
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

// NoiseFilter (CS 0x0527): Olympus.pm - decoded from 3-element int16s array
/// Decode NoiseFilter from 3-value array
fn decode_noise_filter(vals: &[i16]) -> Option<&'static str> {
    if vals.len() < 3 {
        return None;
    }
    // The array is (setting, min, max) where setting determines the filter
    // '0 0 0' => 'n/a'
    // '-2 -2 1' => 'Off'
    // '-1 -2 1' => 'Low'
    // '0 -2 1' => 'Standard'
    // '1 -2 1' => 'High'
    match (vals[0], vals[1], vals[2]) {
        (0, 0, 0) => Some("n/a"),
        (-2, -2, 1) => Some("Off"),
        (-1, -2, 1) => Some("Low"),
        (0, -2, 1) => Some("Standard"),
        (1, -2, 1) => Some("High"),
        _ => None,
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

// ImageQuality2 (CS 0x0603): Olympus.pm
define_tag_decoder! {
    image_quality2,
    both: {
        1 => "SQ",
        2 => "HQ",
        3 => "SHQ",
        4 => "RAW",
        5 => "SQ (5)",
    }
}

// ImageStabilization (CS 0x0604): Olympus.pm (int32u)
define_tag_decoder! {
    image_stabilization,
    type: u32,
    both: {
        0 => "Off",
        1 => "On, S-IS1 (All Direction Shake IS)",
        2 => "On, S-IS2 (Vertical Shake IS)",
        3 => "On, S-IS3 (Horizontal Shake IS)",
        4 => "On, S-IS Auto",
    }
}

/// Decode FocusInfo ImageStabilization (FI 0x1600): binary blob
/// From Olympus.pm: if first 4 bytes are null => Off
/// Otherwise: On + bit 0x01 of byte 44 determines Mode 1 vs Mode 2
pub fn decode_fi_image_stabilization(data: &[u8]) -> String {
    if data.len() < 4 {
        return "Unknown".to_string();
    }
    // If first 4 bytes are all zeros, stabilization is off
    if data[0..4] == [0, 0, 0, 0] {
        return "Off".to_string();
    }
    // Otherwise, it's on - check byte 44 to determine mode
    if data.len() > 44 {
        let mode = if data[44] & 0x01 != 0 {
            "Mode 1"
        } else {
            "Mode 2"
        };
        format!("On, {}", mode)
    } else {
        // Not enough data to determine mode
        "On".to_string()
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

// PictureModeTone (CS 0x0526): Olympus.pm
define_tag_decoder! {
    picture_mode_tone,
    both: {
        0 => "n/a",
        1 => "Neutral",
        2 => "Sepia",
        3 => "Blue",
        4 => "Purple",
        5 => "Green",
    }
}

/// Olympus lens type lookup - converts 6-byte lens type to lens name
/// Format: bytes [make_hi, make_lo, model, subtype, ?, ?]
/// Key format in ExifTool: "make model subtype" in hex (e.g., "0 13 10")
pub fn get_olympus_lens_name(bytes: &[u8]) -> Option<&'static str> {
    if bytes.len() < 4 {
        return None;
    }
    // Make is in byte[1], model in byte[2], subtype in byte[3]
    let make = bytes[1];
    let model = bytes[2];
    let subtype = bytes[3];

    // Lookup table based on ExifTool's %olympusLensTypes
    match (make, model, subtype) {
        (0, 0, 0) => Some("None"),
        // Olympus 4/3 lenses (subtype 0x00 or 0x01)
        (0, 0x01, 0x00) => Some("Olympus Zuiko Digital ED 50mm F2.0 Macro"),
        (0, 0x01, 0x01) => Some("Olympus Zuiko Digital 40-150mm F3.5-4.5"),
        (0, 0x02, 0x00) => Some("Olympus Zuiko Digital ED 150mm F2.0"),
        (0, 0x03, 0x00) => Some("Olympus Zuiko Digital ED 300mm F2.8"),
        (0, 0x05, 0x00) => Some("Olympus Zuiko Digital 14-54mm F2.8-3.5"),
        (0, 0x05, 0x01) => Some("Olympus Zuiko Digital Pro ED 90-250mm F2.8"),
        (0, 0x06, 0x00) => Some("Olympus Zuiko Digital ED 50-200mm F2.8-3.5"),
        (0, 0x06, 0x01) => Some("Olympus Zuiko Digital ED 8mm F3.5 Fisheye"),
        (0, 0x07, 0x00) => Some("Olympus Zuiko Digital 11-22mm F2.8-3.5"),
        (0, 0x07, 0x01) => Some("Olympus Zuiko Digital 18-180mm F3.5-6.3"),
        (0, 0x08, 0x01) => Some("Olympus Zuiko Digital 70-300mm F4.0-5.6"),
        (0, 0x10, 0x01) => Some("Kenko Tokina Reflex 300mm F6.3 MF Macro"),
        (0, 0x15, 0x00) => Some("Olympus Zuiko Digital ED 7-14mm F4.0"),
        (0, 0x17, 0x00) => Some("Olympus Zuiko Digital Pro ED 35-100mm F2.0"),
        (0, 0x18, 0x00) => Some("Olympus Zuiko Digital 14-45mm F3.5-5.6"),
        (0, 0x20, 0x00) => Some("Olympus Zuiko Digital 35mm F3.5 Macro"),
        (0, 0x22, 0x00) => Some("Olympus Zuiko Digital 17.5-45mm F3.5-5.6"),
        (0, 0x23, 0x00) => Some("Olympus Zuiko Digital ED 14-42mm F3.5-5.6"),
        (0, 0x24, 0x00) => Some("Olympus Zuiko Digital ED 40-150mm F4.0-5.6"),
        (0, 0x30, 0x00) => Some("Olympus Zuiko Digital ED 50-200mm F2.8-3.5 SWD"),
        (0, 0x31, 0x00) => Some("Olympus Zuiko Digital ED 12-60mm F2.8-4.0 SWD"),
        (0, 0x32, 0x00) => Some("Olympus Zuiko Digital ED 14-35mm F2.0 SWD"),
        (0, 0x33, 0x00) => Some("Olympus Zuiko Digital 25mm F2.8"),
        (0, 0x34, 0x00) => Some("Olympus Zuiko Digital ED 9-18mm F4.0-5.6"),
        (0, 0x35, 0x00) => Some("Olympus Zuiko Digital 14-54mm F2.8-3.5 II"),
        // Olympus Micro 4/3 lenses (subtype 0x10)
        // ExifTool uses hex strings like '0 10 10' meaning make=0, model=0x10, subtype=0x10
        (0, 0x01, 0x10) => Some("Olympus M.Zuiko Digital ED 14-42mm F3.5-5.6"),
        (0, 0x02, 0x10) => Some("Olympus M.Zuiko Digital 17mm F2.8 Pancake"),
        (0, 0x03, 0x10) => Some("Olympus M.Zuiko Digital ED 14-150mm F4.0-5.6 [II]"),
        (0, 0x04, 0x10) => Some("Olympus M.Zuiko Digital ED 9-18mm F4.0-5.6"),
        (0, 0x05, 0x10) => Some("Olympus M.Zuiko Digital ED 14-42mm F3.5-5.6 L"),
        (0, 0x06, 0x10) => Some("Olympus M.Zuiko Digital ED 40-150mm F4.0-5.6"),
        (0, 0x07, 0x10) => Some("Olympus M.Zuiko Digital ED 12mm F2.0"),
        (0, 0x08, 0x10) => Some("Olympus M.Zuiko Digital ED 75-300mm F4.8-6.7"),
        (0, 0x09, 0x10) => Some("Olympus M.Zuiko Digital 14-42mm F3.5-5.6 II"),
        (0, 0x10, 0x10) => Some("Olympus M.Zuiko Digital ED 12-50mm F3.5-6.3 EZ"),
        (0, 0x11, 0x10) => Some("Olympus M.Zuiko Digital 45mm F1.8"),
        (0, 0x12, 0x10) => Some("Olympus M.Zuiko Digital ED 60mm F2.8 Macro"),
        (0, 0x13, 0x10) => Some("Olympus M.Zuiko Digital 14-42mm F3.5-5.6 II R"),
        (0, 0x14, 0x10) => Some("Olympus M.Zuiko Digital ED 40-150mm F4.0-5.6 R"),
        (0, 0x15, 0x10) => Some("Olympus M.Zuiko Digital ED 75mm F1.8"),
        (0, 0x16, 0x10) => Some("Olympus M.Zuiko Digital 17mm F1.8"),
        (0, 0x18, 0x10) => Some("Olympus M.Zuiko Digital ED 75-300mm F4.8-6.7 II"),
        (0, 0x19, 0x10) => Some("Olympus M.Zuiko Digital ED 12-40mm F2.8 Pro"),
        (0, 0x20, 0x10) => Some("Olympus M.Zuiko Digital ED 40-150mm F2.8 Pro"),
        (0, 0x21, 0x10) => Some("Olympus M.Zuiko Digital ED 14-42mm F3.5-5.6 EZ"),
        (0, 0x22, 0x10) => Some("Olympus M.Zuiko Digital 25mm F1.8"),
        (0, 0x23, 0x10) => Some("Olympus M.Zuiko Digital ED 7-14mm F2.8 Pro"),
        (0, 0x24, 0x10) => Some("Olympus M.Zuiko Digital ED 300mm F4.0 IS Pro"),
        (0, 0x25, 0x10) => Some("Olympus M.Zuiko Digital ED 8mm F1.8 Fisheye Pro"),
        (0, 0x26, 0x10) => Some("Olympus M.Zuiko Digital ED 12-100mm F4.0 IS Pro"),
        (0, 0x27, 0x10) => Some("Olympus M.Zuiko Digital ED 30mm F3.5 Macro"),
        (0, 0x28, 0x10) => Some("Olympus M.Zuiko Digital ED 25mm F1.2 Pro"),
        (0, 0x29, 0x10) => Some("Olympus M.Zuiko Digital ED 17mm F1.2 Pro"),
        (0, 0x30, 0x10) => Some("Olympus M.Zuiko Digital ED 45mm F1.2 Pro"),
        (0, 0x32, 0x10) => Some("Olympus M.Zuiko Digital ED 12-200mm F3.5-6.3"),
        (0, 0x33, 0x10) => Some("Olympus M.Zuiko Digital 150-400mm F4.5 TC1.25x IS Pro"),
        (0, 0x34, 0x10) => Some("Olympus M.Zuiko Digital ED 12-45mm F4.0 Pro"),
        (0, 0x35, 0x10) => Some("Olympus M.Zuiko 100-400mm F5.0-6.3"),
        (0, 0x36, 0x10) => Some("Olympus M.Zuiko Digital ED 8-25mm F4 Pro"),
        (0, 0x37, 0x10) => Some("Olympus M.Zuiko Digital ED 40-150mm F4.0 Pro"),
        (0, 0x38, 0x10) => Some("Olympus M.Zuiko Digital ED 20mm F1.4 Pro"),
        (0, 0x39, 0x10) => Some("Olympus M.Zuiko Digital ED 90mm F3.5 Macro IS Pro"),
        (0, 0x40, 0x10) => Some("Olympus M.Zuiko Digital ED 150-600mm F5.0-6.3"),
        (0, 0x41, 0x10) => Some("OM System M.Zuiko Digital ED 50-200mm F2.8 IS Pro"),
        // Sigma lenses for 4/3 (subtype 0x00)
        (1, 0x01, 0x00) => Some("Sigma 18-50mm F3.5-5.6 DC"),
        (1, 0x02, 0x00) => Some("Sigma 55-200mm F4.0-5.6 DC"),
        (1, 0x03, 0x00) => Some("Sigma 18-125mm F3.5-5.6 DC"),
        (1, 0x04, 0x00) => Some("Sigma 18-125mm F3.5-5.6 DC"),
        (1, 0x05, 0x00) => Some("Sigma 30mm F1.4 EX DC HSM"),
        (1, 0x06, 0x00) => Some("Sigma APO 50-500mm F4.0-6.3 EX DG HSM"),
        (1, 0x07, 0x00) => Some("Sigma Macro 105mm F2.8 EX DG"),
        (1, 0x08, 0x00) => Some("Sigma APO Macro 150mm F2.8 EX DG HSM"),
        // Sigma lenses for Micro 4/3 (subtype 0x10)
        (1, 0x01, 0x10) => Some("Sigma 30mm F2.8 EX DN"),
        (1, 0x02, 0x10) => Some("Sigma 19mm F2.8 EX DN"),
        (1, 0x03, 0x10) => Some("Sigma 30mm F2.8 DN | A"),
        (1, 0x04, 0x10) => Some("Sigma 19mm F2.8 DN | A"),
        (1, 0x05, 0x10) => Some("Sigma 60mm F2.8 DN | A"),
        (1, 0x06, 0x10) => Some("Sigma 30mm F1.4 DC DN | C"),
        (1, 0x07, 0x10) => Some("Sigma 16mm F1.4 DC DN | C (017)"),
        _ => None,
    }
}

// NoiseReduction2 (IP 0x100F): Olympus.pm - BITMASK
/// Decode NoiseReduction2 bitmask value
pub fn decode_noise_reduction2_exiftool(value: u16) -> String {
    if value == 0 {
        return "(none)".to_string();
    }
    let mut parts = Vec::new();
    if value & 0x01 != 0 {
        parts.push("Noise Reduction");
    }
    if value & 0x02 != 0 {
        parts.push("Noise Filter");
    }
    if value & 0x04 != 0 {
        parts.push("Noise Filter (ISO Boost)");
    }
    if parts.is_empty() {
        format!("Unknown ({})", value)
    } else {
        parts.join(", ")
    }
}

// AFPoint (FI 0x0308): Olympus.pm - model-dependent decoding

/// Decode AFPoint for E-3/E-5/E-30 (11-point AF with split encoding)
/// Lower 5 bits = AF point, Upper bits = target selection mode
fn decode_af_point_e3_e5_e30(value: u16) -> String {
    let point = value & 0x1f;
    let mode = value & 0xffe0;

    let point_str = match point {
        0x00 => "(none)",
        0x01 => "Top-left (horizontal)",
        0x02 => "Top-center (horizontal)",
        0x03 => "Top-right (horizontal)",
        0x04 => "Left (horizontal)",
        0x05 => "Mid-left (horizontal)",
        0x06 => "Center (horizontal)",
        0x07 => "Mid-right (horizontal)",
        0x08 => "Right (horizontal)",
        0x09 => "Bottom-left (horizontal)",
        0x0a => "Bottom-center (horizontal)",
        0x0b => "Bottom-right (horizontal)",
        0x0c => "Top-left (vertical)",
        0x0d => "Top-center (vertical)",
        0x0e => "Top-right (vertical)",
        0x0f => "Left (vertical)",
        0x10 => "Mid-left (vertical)",
        0x11 => "Center (vertical)",
        0x12 => "Mid-right (vertical)",
        0x13 => "Right (vertical)",
        0x14 => "Bottom-left (vertical)",
        0x15 => "Bottom-center (vertical)",
        0x16 => "Bottom-right (vertical)",
        0x1f => "n/a",
        _ => return format!("Unknown (0x{:x})", value),
    };

    let mode_str = match mode {
        0x00 => "Single Target",
        0x40 => "All Target",
        0x80 => "Dynamic Single Target",
        0xe0 => "n/a",
        _ => return format!("{}; Unknown (0x{:x})", point_str, mode),
    };

    format!("{}; {}", point_str, mode_str)
}

/// Decode AFPoint for E-520/E-600/E-620 (7-point AF with split encoding)
fn decode_af_point_7point(value: u16) -> String {
    let point = value & 0x1f;
    let mode = value & 0xffe0;

    let point_str: String = match point {
        0x00 => "(none)".to_string(),
        0x02 => "Top-center (horizontal)".to_string(),
        0x04 => "Right (horizontal)".to_string(),
        0x05 => "Mid-right (horizontal)".to_string(),
        0x06 => "Center (horizontal)".to_string(),
        0x07 => "Mid-left (horizontal)".to_string(),
        0x08 => "Left (horizontal)".to_string(),
        0x0a => "Bottom-center (horizontal)".to_string(),
        0x0c => "Top-center (vertical)".to_string(),
        0x0f => "Right (vertical)".to_string(),
        0x10 => "Mid-right (vertical)".to_string(),
        0x11 => "Center (vertical)".to_string(),
        0x12 => "Mid-left (vertical)".to_string(),
        0x13 => "Left (vertical)".to_string(),
        0x15 => "Bottom-center (vertical)".to_string(),
        _ => format!("Unknown (0x{:x})", value),
    };

    let mode_str = match mode {
        0x00 => "Single Target",
        0x40 => "All Target",
        _ => return format!("{}; Unknown (0x{:x})", point_str, mode),
    };

    format!("{}; {}", point_str, mode_str)
}

/// Decode AFPoint based on camera model
pub fn decode_af_point_by_model(value: u16, model: Option<&str>) -> String {
    if let Some(m) = model {
        // E-M* and OM-* models: don't decode (ExifTool returns raw value)
        if m.starts_with("E-M") || m.starts_with("OM-") {
            return value.to_string();
        }

        // E-3, E-5, E-30: 11-point AF with split encoding
        // Match exact model names (from decode_camera_type): "E-3", "E-5", "E-30"
        if m == "E-3" || m == "E-5" || m == "E-30" {
            return decode_af_point_e3_e5_e30(value);
        }

        // E-520, E-600, E-620: 7-point AF
        if m == "E-520" || m == "E-600" || m == "E-620" {
            return decode_af_point_7point(value);
        }
    }

    // Default for other models (E-P1, E-510, etc.): basic 4-point AF
    let decoded = decode_af_point_exiftool(value);
    if decoded == "Unknown" {
        // Show raw value for unknown AF points
        format!("Unknown ({})", value)
    } else {
        decoded.to_string()
    }
}

// Simple AFPoint decoder for basic models (E-P1, E-510, etc.)
define_tag_decoder! {
    af_point,
    both: {
        0 => "Left (or n/a)",
        1 => "Center (horizontal)",
        2 => "Right",
        3 => "Center (vertical)",
        255 => "None",
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

// ManualFlash (FI 0x1209): Olympus.pm - int16u[2] (on/off, strength)
/// Decode ManualFlash from 2-value array
fn decode_manual_flash(vals: &[u16]) -> String {
    if vals.is_empty() || vals[0] == 0 {
        return "Off".to_string();
    }
    // vals[0] is on/off, vals[1] is strength
    if vals.len() < 2 {
        return "On".to_string();
    }
    let strength = vals[1];
    let strength_str = if strength == 1 {
        "Full".to_string()
    } else {
        format!("1/{}", strength)
    };
    format!("On ({} strength)", strength_str)
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

// BWMode (Main 0x0203): Olympus.pm
define_tag_decoder! {
    bw_mode,
    both: {
        0 => "Off",
        1 => "On",
    }
}

// AFFineTune (CS 0x0306): Olympus.pm
define_tag_decoder! {
    af_fine_tune,
    both: {
        0 => "Off",
        1 => "On",
    }
}

// InternalFlash (FI 0x1208): Olympus.pm
define_tag_decoder! {
    internal_flash,
    both: {
        0 => "Off",
        1 => "On",
    }
}

// MacroLED (FI 0x120A): Olympus.pm
define_tag_decoder! {
    macro_led,
    both: {
        0 => "Off",
        1 => "On",
    }
}

// ExternalFlash (FI 0x1201): Olympus.pm (first value is off/on)
define_tag_decoder! {
    external_flash,
    both: {
        0 => "Off",
        1 => "On",
    }
}

// FlashType (Equip 0x1000): Olympus.pm
define_tag_decoder! {
    flash_type,
    both: {
        0 => "None",
        2 => "Simple E-System",
        3 => "E-System",
    }
}

/// Decode DriveMode (CS 0x0600): Olympus.pm
/// Format: 2, 3, 5 or 6 int16u values
/// 1. Mode, 2. Shot number, 3. Mode bits, 5. Shutter mode, 6. Shooting mode
fn decode_drive_mode(values: &[u16]) -> String {
    if values.is_empty() {
        return String::new();
    }

    let mode = values[0];
    let shot_num = if values.len() > 1 && values[1] > 0 {
        format!(", Shot {}", values[1])
    } else {
        String::new()
    };

    // Shutter mode (5th value, index 4)
    let shutter = if values.len() >= 5 {
        match values[4] {
            0 => "; Mechanical shutter",
            2 => "; Anti-shock",
            4 => "", // Electronic shutter (default, don't show)
            _ => "",
        }
    } else {
        ""
    };

    // Check 6th value (index 5) for newer models' shooting modes
    if values.len() >= 6 && values[5] != 0 {
        let mode_str = match values[5] {
            0x01 => "Single Shot",
            0x02 => "Sequential L",
            0x03 => "Sequential H",
            0x07 => "Sequential",
            0x11 => "Single Shot",
            0x12 => "Sequential L",
            0x13 => "Sequential H",
            0x14 => "Self-Timer 12 sec",
            0x15 => "Self-Timer 2 sec",
            0x16 => "Custom Self-Timer",
            0x17 => "Sequential",
            0x21 => "Single Shot",
            0x22 => "Sequential L",
            0x23 => "Sequential H",
            0x24 => "Self-Timer 2 sec",
            0x25 => "Self-Timer 12 sec",
            0x26 => "Custom Self-Timer",
            0x27 => "Sequential",
            0x28 => "Sequential SH1",
            0x29 => "Sequential SH2",
            0x30 => "HighRes Shot",
            0x41 => "ProCap H",
            0x42 => "ProCap L",
            0x43 => "ProCap",
            0x48 => "ProCap SH1",
            0x49 => "ProCap SH2",
            _ => return format!("Unknown ({})", values[5]),
        };
        return format!("{}{}{}", mode_str, shot_num, shutter);
    }

    // Mode 5 with mode bits (3rd value) = bracketing type
    if mode == 5 && values.len() >= 3 {
        let mode_bits = values[2];
        let mut bracket_types = Vec::new();
        if mode_bits & 0x01 != 0 {
            bracket_types.push("AE");
        }
        if mode_bits & 0x02 != 0 {
            bracket_types.push("WB");
        }
        if mode_bits & 0x04 != 0 {
            bracket_types.push("FL");
        }
        if mode_bits & 0x08 != 0 {
            bracket_types.push("MF");
        }
        if mode_bits & 0x10 != 0 {
            bracket_types.push("ISO");
        }
        if mode_bits & 0x20 != 0 {
            bracket_types.push("AE Auto");
        }
        if mode_bits & 0x40 != 0 {
            bracket_types.push("Focus");
        }
        let bracket_str = if bracket_types.is_empty() {
            "Bracketing".to_string()
        } else {
            format!("{} Bracketing", bracket_types.join("+"))
        };
        return format!("{}{}{}", bracket_str, shot_num, shutter);
    }

    // Basic mode decoding
    let mode_str = match mode {
        0 => "Single Shot",
        1 => "Continuous Shooting",
        2 => "Exposure Bracketing",
        3 => "White Balance Bracketing",
        4 => "Exposure+WB Bracketing",
        5 => "Bracketing", // Fallback if no mode bits
        _ => return format!("Unknown ({})", mode),
    };

    format!("{}{}{}", mode_str, shot_num, shutter)
}

/// Format array values as "value (min min_val, max max_val)"
/// ExifTool uses this format for settings like CustomSaturation, ContrastSetting, etc.
fn format_min_max_array(values: &[i16]) -> String {
    if values.len() >= 3 {
        format!("{} (min {}, max {})", values[0], values[1], values[2])
    } else if !values.is_empty() {
        values
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        String::new()
    }
}

/// Format CustomSaturation for E-1 model (CS-style labels)
/// E-1 uses: a-=b; c-=b; return "CS$a (min CS0, max CS$c)"
fn format_custom_saturation_e1(values: &[i16]) -> String {
    if values.len() >= 3 {
        let a = values[0] - values[1];
        let c = values[2] - values[1];
        format!("CS{} (min CS0, max CS{})", a, c)
    } else if !values.is_empty() {
        values
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        String::new()
    }
}

/// Format array values from unsigned shorts
fn format_min_max_array_u16(values: &[u16]) -> String {
    if values.len() >= 3 {
        // Convert to signed for proper display
        let v0 = values[0] as i16;
        let v1 = values[1] as i16;
        let v2 = values[2] as i16;
        format!("{} (min {}, max {})", v0, v1, v2)
    } else if !values.is_empty() {
        values
            .iter()
            .map(|v| (*v as i16).to_string())
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        String::new()
    }
}

/// Decode CameraType codes to model names (from ExifTool Olympus.pm %olympusCameraTypes)
fn decode_camera_type(val: &str) -> Option<&'static str> {
    Some(match val {
        "D4028" => "X-2,C-50Z",
        "D4029" => "E-20,E-20N,E-20P",
        "D4034" => "C720UZ",
        "D4040" => "E-1",
        "D4041" => "E-300",
        "D4083" => "C2Z,D520Z,C220Z",
        "D4106" => "u20D,S400D,u400D",
        "D4120" => "X-1",
        "D4122" => "u10D,S300D,u300D",
        "D4125" => "AZ-1",
        "D4141" => "C150,D390",
        "D4193" => "C-5000Z",
        "D4194" => "X-3,C-60Z",
        "D4199" => "u30D,S410D,u410D",
        "D4205" => "X450,D535Z,C370Z",
        "D4210" => "C160,D395",
        "D4211" => "C725UZ",
        "D4213" => "FerrariMODEL2003",
        "D4216" => "u15D",
        "D4217" => "u25D",
        "D4220" => "u-miniD,Stylus V",
        "D4221" => "u40D,S500,uD500",
        "D4231" => "FerrariMODEL2004",
        "D4240" => "X500,D590Z,C470Z",
        "D4244" => "uD800,S800",
        "D4256" => "u720SW,S720SW",
        "D4261" => "X600,D630,FE5500",
        "D4262" => "uD600,S600",
        "D4301" => "u810/S810",
        "D4302" => "u710,S710",
        "D4303" => "u700,S700",
        "D4304" => "FE100,X710",
        "D4305" => "FE110,X705",
        "D4310" => "FE-130,X-720",
        "D4311" => "FE-140,X-725",
        "D4312" => "FE150,X730",
        "D4313" => "FE160,X735",
        "D4314" => "u740,S740",
        "D4315" => "u750,S750",
        "D4316" => "u730/S730",
        "D4317" => "FE115,X715",
        "D4321" => "SP550UZ",
        "D4322" => "SP510UZ",
        "D4324" => "FE170,X760",
        "D4326" => "FE200",
        "D4327" => "FE190/X750",
        "D4328" => "u760,S760",
        "D4330" => "FE180/X745",
        "D4331" => "u1000/S1000",
        "D4332" => "u770SW,S770SW",
        "D4333" => "FE240/X795",
        "D4334" => "FE210,X775",
        "D4336" => "FE230/X790",
        "D4337" => "FE220,X785",
        "D4338" => "u725SW,S725SW",
        "D4339" => "FE250/X800",
        "D4341" => "u780,S780",
        "D4343" => "u790SW,S790SW",
        "D4344" => "u1020,S1020",
        "D4346" => "FE15,X10",
        "D4348" => "FE280,X820,C520",
        "D4349" => "FE300,X830",
        "D4350" => "u820,S820",
        "D4351" => "u1200,S1200",
        "D4352" => "FE270,X815,C510",
        "D4353" => "u795SW,S795SW",
        "D4354" => "u1030SW,S1030SW",
        "D4355" => "SP560UZ",
        "D4356" => "u1010,S1010",
        "D4357" => "u830,S830",
        "D4359" => "u840,S840",
        "D4360" => "FE350WIDE,X865",
        "D4361" => "u850SW,S850SW",
        "D4362" => "FE340,X855,C560",
        "D4363" => "FE320,X835,C540",
        "D4364" => "SP570UZ",
        "D4366" => "FE330,X845,C550",
        "D4368" => "FE310,X840,C530",
        "D4370" => "u1050SW,S1050SW",
        "D4371" => "u1060,S1060",
        "D4372" => "FE370,X880,C575",
        "D4374" => "SP565UZ",
        "D4377" => "u1040,S1040",
        "D4378" => "FE360,X875,C570",
        "D4379" => "FE20,X15,C25",
        "D4380" => "uT6000,ST6000",
        "D4381" => "uT8000,ST8000",
        "D4382" => "u9000,S9000",
        "D4384" => "SP590UZ",
        "D4385" => "FE3010,X895",
        "D4386" => "FE3000,X890",
        "D4387" => "FE35,X30",
        "D4388" => "u550WP,S550WP",
        "D4390" => "FE5000,X905",
        "D4391" => "u5000",
        "D4392" => "u7000,S7000",
        "D4396" => "FE5010,X915",
        "D4397" => "FE25,X20",
        "D4398" => "FE45,X40",
        "D4401" => "XZ-1",
        "D4402" => "uT6010,ST6010",
        "D4406" => "u7010,S7010 / u7020,S7020",
        "D4407" => "FE4010,X930",
        "D4408" => "X560WP",
        "D4409" => "FE26,X21",
        "D4410" => "FE4000,X920,X925",
        "D4411" => "FE46,X41,X42",
        "D4412" => "FE5020,X935",
        "D4413" => "uTough-3000",
        "D4414" => "StylusTough-6020",
        "D4415" => "StylusTough-8010",
        "D4417" => "u5010,S5010",
        "D4418" => "u7040,S7040",
        "D4419" => "u9010,S9010",
        "D4423" => "FE4040",
        "D4424" => "FE47,X43",
        "D4426" => "FE4030,X950",
        "D4428" => "FE5030,X965,X960",
        "D4430" => "u7030,S7030",
        "D4432" => "SP600UZ",
        "D4434" => "SP800UZ",
        "D4439" => "FE4020,X940",
        "D4442" => "FE5035",
        "D4448" => "FE4050,X970",
        "D4450" => "FE5050,X985",
        "D4454" => "u-7050",
        "D4464" => "T10,X27",
        "D4470" => "FE5040,X980",
        "D4472" => "TG-310",
        "D4474" => "TG-610",
        "D4476" => "TG-810",
        "D4478" => "VG145,VG140,D715",
        "D4479" => "VG130,D710",
        "D4480" => "VG120,D705",
        "D4482" => "VR310,D720",
        "D4484" => "VR320,D725",
        "D4486" => "VR330,D730",
        "D4488" => "VG110,D700",
        "D4490" => "SP-610UZ",
        "D4492" => "SZ-10",
        "D4494" => "SZ-20",
        "D4496" => "SZ-30MR",
        "D4498" => "SP-810UZ",
        "D4500" => "SZ-11",
        "D4504" => "TG-615",
        "D4508" => "TG-620",
        "D4510" => "TG-820",
        "D4512" => "TG-1",
        "D4516" => "SH-21",
        "D4519" => "SZ-14",
        "D4520" => "SZ-31MR",
        "D4521" => "SH-25MR",
        "D4523" => "SP-720UZ",
        "D4529" => "VG170",
        "D4530" => "VH210",
        "D4531" => "XZ-2",
        "D4535" => "SP-620UZ",
        "D4536" => "TG-320",
        "D4537" => "VR340,D750",
        "D4538" => "VG160,X990,D745",
        "D4541" => "SZ-12",
        "D4545" => "VH410",
        "D4546" => "XZ-10",
        "D4547" => "TG-2",
        "D4548" => "TG-830",
        "D4549" => "TG-630",
        "D4550" => "SH-50",
        "D4553" => "SZ-16,DZ-105",
        "D4562" => "SP-820UZ",
        "D4566" => "SZ-15",
        "D4572" => "STYLUS1",
        "D4574" => "TG-3",
        "D4575" => "TG-850",
        "D4579" => "SP-100EE",
        "D4580" => "SH-60",
        "D4581" => "SH-1",
        "D4582" => "TG-835",
        "D4585" => "SH-2 / SH-3",
        "D4586" => "TG-4",
        "D4587" => "TG-860",
        "D4590" => "TG-TRACKER",
        "D4591" => "TG-870",
        "D4593" => "TG-5",
        "D4603" => "TG-6",
        "D4605" => "TG-7",
        "D4809" => "C2500L",
        "D4842" => "E-10",
        "D4856" => "C-1",
        "D4857" => "C-1Z,D-150Z",
        "DCHC" => "D500L",
        "DCHT" => "D600L / D620L",
        "K0055" => "AIR-A01",
        "S0003" => "E-330",
        "S0004" => "E-500",
        "S0009" => "E-400",
        "S0010" => "E-510",
        "S0011" => "E-3",
        "S0013" => "E-410",
        "S0016" => "E-420",
        "S0017" => "E-30",
        "S0018" => "E-520",
        "S0019" => "E-P1",
        "S0023" => "E-620",
        "S0026" => "E-P2",
        "S0027" => "E-PL1",
        "S0029" => "E-450",
        "S0030" => "E-600",
        "S0032" => "E-P3",
        "S0033" => "E-5",
        "S0034" => "E-PL2",
        "S0036" => "E-M5",
        "S0038" => "E-PL3",
        "S0039" => "E-PM1",
        "S0040" => "E-PL1s",
        "S0042" => "E-PL5",
        "S0043" => "E-PM2",
        "S0044" => "E-P5",
        "S0045" => "E-PL6",
        "S0046" => "E-PL7",
        "S0047" => "E-M1",
        "S0051" => "E-M10",
        "S0052" => "E-M5MarkII",
        "S0059" => "E-M10MarkII",
        "S0061" => "PEN-F",
        "S0065" => "E-PL8",
        "S0067" => "E-M1MarkII",
        "S0068" => "E-M10MarkIII",
        "S0076" => "E-PL9",
        "S0080" => "E-M1X",
        "S0085" => "E-PL10",
        "S0088" => "E-M10MarkIV",
        "S0089" => "E-M5MarkIII",
        "S0092" => "E-M1MarkIII",
        "S0093" => "E-P7",
        "S0094" => "E-M10MarkIIIS",
        "S0095" => "OM-1",
        "S0101" => "OM-5",
        "S0121" => "OM-1MarkII",
        "S0123" => "OM-3",
        "S0130" => "OM-5MarkII",
        "SR45" => "D220",
        "SR55" => "D320L",
        "SR83" => "D340L",
        "SR85" => "C830L,D340R",
        "SR852" => "C860L,D360L",
        "SR872" => "C900Z,D400Z",
        "SR874" => "C960Z,D460Z",
        "SR951" => "C2000Z",
        "SR952" => "C21",
        "SR953" => "C21T.commu",
        "SR954" => "C2020Z",
        "SR955" => "C990Z,D490Z",
        "SR956" => "C211Z",
        "SR959" => "C990ZS,D490Z",
        "SR95A" => "C2100UZ",
        "SR971" => "C100,D370",
        "SR973" => "C2,D230",
        "SX151" => "E100RS",
        "SX351" => "C3000Z / C3030Z",
        "SX354" => "C3040Z",
        "SX355" => "C2040Z",
        "SX357" => "C700UZ",
        "SX358" => "C200Z,D510Z",
        "SX374" => "C3100Z,C3020Z",
        "SX552" => "C4040Z",
        "SX553" => "C40Z,D40Z",
        "SX556" => "C730UZ",
        "SX558" => "C5050Z",
        "SX571" => "C120,D380",
        "SX574" => "C300Z,D550Z",
        "SX575" => "C4100Z,C4000Z",
        "SX751" => "X200,D560Z,C350Z",
        "SX752" => "X300,D565Z,C450Z",
        "SX753" => "C750UZ",
        "SX754" => "C740UZ",
        "SX755" => "C755UZ",
        "SX756" => "C5060WZ",
        "SX757" => "C8080WZ",
        "SX758" => "X350,D575Z,C360Z",
        "SX759" => "X400,D580Z,C460Z",
        "SX75A" => "AZ-2ZOOM",
        "SX75B" => "D595Z,C500Z",
        "SX75C" => "X550,D545Z,C480Z",
        "SX75D" => "IR-300",
        "SX75F" => "C55Z,C5500Z",
        "SX75G" => "C170,D425",
        "SX75J" => "C180,D435",
        "SX771" => "C760UZ",
        "SX772" => "C770UZ",
        "SX773" => "C745UZ",
        "SX774" => "X250,D560Z,C350Z",
        "SX775" => "X100,D540Z,C310Z",
        "SX776" => "C460ZdelSol",
        "SX777" => "C765UZ",
        "SX77A" => "D555Z,C315Z",
        "SX851" => "C7070WZ",
        "SX852" => "C70Z,C7000Z",
        "SX853" => "SP500UZ",
        "SX854" => "SP310",
        "SX855" => "SP350",
        "SX873" => "SP320",
        "SX875" => "FE180/X745",
        "SX876" => "FE190/X750",
        _ => return None, // Return None for unknown types
    })
}

/// Decode FlashModel (from ExifTool Olympus.pm Equipment tag 0x1001)
fn decode_flash_model(val: u16) -> &'static str {
    match val {
        0 => "None",
        1 => "FL-20",
        2 => "FL-50",
        3 => "RF-11",
        4 => "TF-22",
        5 => "FL-36",
        6 => "FL-50R",
        7 => "FL-36R",
        9 => "FL-14",
        11 => "FL-600R",
        13 => "FL-LM3",
        15 => "FL-900R",
        _ => "Unknown",
    }
}

/// Format firmware version from u32
/// ExifTool: '$val=sprintf("%x",$val);$val=~s/(.{3})$/\.$1/;$val'
/// e.g., 4100 (0x1004) -> "1.004", 4357 (0x1105) -> "1.105"
/// For short hex strings (< 4 chars): just use hex without decimal point
fn format_firmware_version(val: u32) -> String {
    let hex = format!("{:x}", val);
    if hex.len() >= 4 {
        let (major, minor) = hex.split_at(hex.len() - 3);
        format!("{}.{}", major, minor)
    } else {
        // Short hex string - no decimal point (e.g., 121 -> "79")
        hex
    }
}

/// Decode PictureMode (from ExifTool Olympus.pm CameraSettings tag 0x0520)
fn decode_picture_mode(val: u16) -> &'static str {
    match val {
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
        _ => "Unknown",
    }
}

/// Decode PictureMode array (1 or 2 values)
fn decode_picture_mode_array(values: &[u16]) -> String {
    if values.is_empty() {
        return String::new();
    }
    let mode = decode_picture_mode(values[0]);
    if mode == "Unknown" {
        return format!("Unknown ({})", values[0]);
    }
    if values.len() >= 2 && values[1] != 0 {
        format!("{}; {}", mode, values[1])
    } else {
        mode.to_string()
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
                tag_id,
                raw_offset,
                src_base,
                offset,
                src_data.len(),
                total_size
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
///
/// For most embedded IFDs, value offsets are within the embedded data itself.
/// However, some (like FocusInfo in old format) have value offsets relative to TIFF header,
/// so we accept optional value_data and value_base_offset for external value resolution.
#[allow(clippy::too_many_arguments)]
fn parse_embedded_sub_ifd(
    sub_data: &[u8],
    endian: Endianness,
    ifd_type: OlympusIfdType,
    tags: &mut HashMap<u16, MakerNoteTag>,
    prefix: &str,
    camera_model: &mut Option<String>,
    preview_offset_adjust: usize,
    value_data: Option<&[u8]>,
    value_base_offset: usize,
) {
    // Parse embedded sub-IFD, optionally using external data for value resolution
    parse_olympus_ifd(
        sub_data,
        0,
        0,
        endian,
        ifd_type,
        tags,
        prefix,
        value_data,
        value_base_offset,
        camera_model,
        preview_offset_adjust,
    );
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
/// * `preview_offset_adjust` - Offset to add to PreviewImageStart for file-absolute offset
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
    camera_model: &mut Option<String>,
    preview_offset_adjust: usize,
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

            // Skip thumbnail/preview data to save memory (only in Main IFD)
            if ifd_type == OlympusIfdType::Main
                && (tag_id == OLYMPUS_THUMBNAIL_IMAGE || tag_id == OLYMPUS_PREVIEW_IMAGE)
            {
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
                        // ExifTool outputs raw rational value with trailing zeros trimmed
                        match &value {
                            ExifValue::Rational(vals) if !vals.is_empty() => {
                                let (num, denom) = vals[0];
                                if denom != 0 {
                                    let v = num as f64 / denom as f64;
                                    // Format with 3 decimals, trim trailing zeros
                                    let formatted = format!("{:.3}", v);
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
                    OLYMPUS_MACRO => {
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
                    OLYMPUS_BW_MODE => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(decode_bw_mode_exiftool(vals[0]).to_string())
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    OLYMPUS_CAMERA_TYPE => {
                        // CameraType: decode internal code to model name
                        if let ExifValue::Ascii(s) = &value {
                            let trimmed = s.trim();
                            if let Some(decoded) = decode_camera_type(trimmed) {
                                // Store model for AFPoint model-specific decoding
                                *camera_model = Some(decoded.to_string());
                                ExifValue::Ascii(decoded.to_string())
                            } else {
                                value // Keep original if not in lookup table
                            }
                        } else {
                            value
                        }
                    }
                    OLYMPUS_RED_BALANCE | OLYMPUS_BLUE_BALANCE => {
                        // RedBalance/BlueBalance: int16u[2], first value / 256
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                let v = vals[0] as f64 / 256.0;
                                ExifValue::Ascii(format!("{}", v))
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    OLYMPUS_SCENE_MODE => {
                        // Main IFD SceneMode (0x0403) - different mapping from CS SceneMode
                        // 0=Normal (not Standard like CS)
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_main_scene_mode_exiftool(vals[0]).to_string(),
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
                        // ExifTool outputs raw rational value with trailing zeros trimmed
                        match &value {
                            ExifValue::Rational(vals) if !vals.is_empty() => {
                                let (num, denom) = vals[0];
                                if denom != 0 {
                                    let v = num as f64 / denom as f64;
                                    // Format with 3 decimals, trim trailing zeros
                                    let formatted = format!("{:.3}", v);
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
                    EQUIP_FLASH_TYPE => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(decode_flash_type_exiftool(vals[0]).to_string())
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    EQUIP_FLASH_MODEL => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(decode_flash_model(vals[0]).to_string())
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    EQUIP_FLASH_FIRMWARE_VERSION => {
                        // FlashFirmwareVersion: convert u32 to hex version string
                        // e.g., 4101 decimal = 0x1005 -> "1.005"
                        if let ExifValue::Long(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(format_firmware_version(vals[0]))
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    EQUIP_LENS_FIRMWARE_VERSION => {
                        // LensFirmwareVersion: convert u32 to hex version string
                        if let ExifValue::Long(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(format_firmware_version(vals[0]))
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    EQUIP_BODY_FIRMWARE_VERSION => {
                        // BodyFirmwareVersion: same format as LensFirmwareVersion
                        if let ExifValue::Long(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(format_firmware_version(vals[0]))
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    EQUIP_MAX_APERTURE
                    | EQUIP_MAX_APERTURE_AT_MIN_FOCAL
                    | EQUIP_MAX_APERTURE_AT_MAX_FOCAL => {
                        // MaxAperture tags: int16u APEX value in 1/256 units
                        // Formula: sqrt(2)**(val/256) = 2**(val/512)
                        // Output: formatted as %.1f
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() && vals[0] != 0 {
                                let apex = vals[0] as f64 / 256.0;
                                let aperture = 2.0_f64.powf(apex / 2.0);
                                ExifValue::Ascii(format!("{:.1}", aperture))
                            } else {
                                ExifValue::Ascii("0".to_string())
                            }
                        } else {
                            value
                        }
                    }
                    EQUIP_LENS_PROPERTIES => {
                        // LensProperties: int16u displayed as hex "0xNNNN"
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(format!("0x{:x}", vals[0]))
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    EQUIP_LENS_TYPE => {
                        // LensType: 6 int8u values - lookup lens name
                        match &value {
                            ExifValue::Byte(vals) if vals.len() >= 4 => {
                                if let Some(name) = get_olympus_lens_name(vals) {
                                    ExifValue::Ascii(name.to_string())
                                } else {
                                    value
                                }
                            }
                            ExifValue::Short(vals) if vals.len() >= 4 => {
                                // Convert shorts to bytes for lookup
                                let bytes: Vec<u8> = vals.iter().map(|&v| v as u8).collect();
                                if let Some(name) = get_olympus_lens_name(&bytes) {
                                    ExifValue::Ascii(name.to_string())
                                } else {
                                    value
                                }
                            }
                            _ => value,
                        }
                    }
                    EQUIP_CAMERA_TYPE => {
                        // CameraType2: decode internal code and store model for AFPoint
                        if let ExifValue::Ascii(s) = &value {
                            let trimmed = s.trim();
                            if let Some(decoded) = decode_camera_type(trimmed) {
                                // Store model for AFPoint model-specific decoding
                                *camera_model = Some(decoded.to_string());
                                ExifValue::Ascii(decoded.to_string())
                            } else {
                                // Use raw value as model if not in lookup table
                                *camera_model = Some(trimmed.to_string());
                                value
                            }
                        } else {
                            value
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
                        // FocusMode: int16u[1 or 2], first decoded, second as bitmask
                        // Format: "Single AF; S-AF, Imager AF"
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                let first = decode_focus_mode_exiftool(vals[0]);
                                let result = if vals.len() > 1 {
                                    let bitmask = decode_focus_mode_bitmask(vals[1]);
                                    format!("{}; {}", first, bitmask)
                                } else {
                                    first.to_string()
                                };
                                ExifValue::Ascii(result)
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_FOCUS_PROCESS => {
                        // FocusProcess: int16u[1 or 2], first decoded, second as number
                        // Format: "AF Used; 64"
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                let first = decode_focus_process_exiftool(vals[0]);
                                let result = if vals.len() > 1 {
                                    format!("{}; {}", first, vals[1])
                                } else {
                                    first.to_string()
                                };
                                ExifValue::Ascii(result)
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
                    CS_AF_AREAS => {
                        // AFAreas: int32u[64], decode to coordinate format
                        // Each non-zero int32u encodes 4 bytes: (x1, y1, x2, y2) in big-endian
                        // Special values have names: 0x36794285=Left, 0x79798585=Center, 0xBD79C985=Right
                        if let ExifValue::Long(vals) = &value {
                            if vals.iter().all(|&v| v == 0) {
                                ExifValue::Ascii("none".to_string())
                            } else {
                                let mut parts: Vec<String> = Vec::new();
                                for &v in vals {
                                    if v == 0 {
                                        continue;
                                    }
                                    let x1 = ((v >> 24) & 0xFF) as u8;
                                    let y1 = ((v >> 16) & 0xFF) as u8;
                                    let x2 = ((v >> 8) & 0xFF) as u8;
                                    let y2 = (v & 0xFF) as u8;
                                    let coords = format!("({},{})-({},{})", x1, y1, x2, y2);
                                    // Check for special named AF points
                                    let name = match v {
                                        0x36794285 => Some("Left"),
                                        0x79798585 => Some("Center"),
                                        0xBD79C985 => Some("Right"),
                                        _ => None,
                                    };
                                    if let Some(n) = name {
                                        parts.push(format!("{} {}", n, coords));
                                    } else {
                                        parts.push(coords);
                                    }
                                }
                                if parts.is_empty() {
                                    ExifValue::Ascii("none".to_string())
                                } else {
                                    ExifValue::Ascii(parts.join(", "))
                                }
                            }
                        } else {
                            value
                        }
                    }
                    CS_AF_POINT_SELECTED => {
                        // AFPointSelected: rational64s[5], format as "(X1%,Y1%) (X2%,Y2%)"
                        // First check if all values contain "undef" (all undefined)
                        let val_str = format!("{}", value);
                        // Count number of undef entries
                        let undef_count = val_str.matches("undef").count();
                        if undef_count >= 5 {
                            // All values are undefined
                            ExifValue::Ascii("n/a".to_string())
                        } else if let ExifValue::SRational(vals) = &value {
                            // rational64s[5]: [?, x1, y1, x2, y2]
                            // Convert to percentages and format (ExifTool uses floor/truncation)
                            if vals.len() >= 5 {
                                // Check if all zeros (both points at origin = n/a)
                                let all_zero = vals[1..5]
                                    .iter()
                                    .all(|(n, d)| *d == 0 || (*n as f64 / *d as f64).abs() < 0.005);
                                if all_zero {
                                    ExifValue::Ascii("n/a".to_string())
                                } else {
                                    let x1 = (vals[1].0 as f64 / vals[1].1 as f64 * 100.0).floor()
                                        as i32;
                                    let y1 = (vals[2].0 as f64 / vals[2].1 as f64 * 100.0).floor()
                                        as i32;
                                    let x2 = (vals[3].0 as f64 / vals[3].1 as f64 * 100.0).floor()
                                        as i32;
                                    let y2 = (vals[4].0 as f64 / vals[4].1 as f64 * 100.0).floor()
                                        as i32;
                                    ExifValue::Ascii(format!("({}%,{}%) ({}%,{}%)", x1, y1, x2, y2))
                                }
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
                                ExifValue::Ascii(decode_flash_mode_exiftool(vals[0]))
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
                    CS_WHITE_BALANCE_TEMPERATURE => {
                        // WhiteBalanceTemperature: 0 = "Auto", otherwise show value
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() && vals[0] == 0 {
                                ExifValue::Ascii("Auto".to_string())
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
                    CS_CUSTOM_SATURATION => {
                        // CustomSaturation: 3 values - current, min, max
                        // E-1 uses CS-style labels, other models use numeric
                        // Match E-1 but not E-10, E-100, E-1 Mark II, etc.
                        let is_e1 = camera_model.as_ref().is_some_and(|m| {
                            m == "E-1"
                                || m.starts_with("E-1 ")
                                || m.ends_with(" E-1")
                                || m.contains(" E-1 ")
                        });
                        if let ExifValue::SShort(vals) = &value {
                            if is_e1 {
                                ExifValue::Ascii(format_custom_saturation_e1(vals))
                            } else {
                                ExifValue::Ascii(format_min_max_array(vals))
                            }
                        } else if let ExifValue::Short(vals) = &value {
                            if is_e1 {
                                let signed: Vec<i16> = vals.iter().map(|&v| v as i16).collect();
                                ExifValue::Ascii(format_custom_saturation_e1(&signed))
                            } else {
                                ExifValue::Ascii(format_min_max_array_u16(vals))
                            }
                        } else {
                            value
                        }
                    }
                    CS_CONTRAST_SETTING => {
                        if let ExifValue::SShort(vals) = &value {
                            ExifValue::Ascii(format_min_max_array(vals))
                        } else if let ExifValue::Short(vals) = &value {
                            ExifValue::Ascii(format_min_max_array_u16(vals))
                        } else {
                            value
                        }
                    }
                    CS_SHARPNESS_SETTING => {
                        if let ExifValue::SShort(vals) = &value {
                            ExifValue::Ascii(format_min_max_array(vals))
                        } else if let ExifValue::Short(vals) = &value {
                            ExifValue::Ascii(format_min_max_array_u16(vals))
                        } else {
                            value
                        }
                    }
                    CS_NOISE_FILTER => {
                        // NoiseFilter is int16s[3]: (setting, min, max)
                        // The array maps to: n/a, Off, Low, Standard, High
                        if let ExifValue::SShort(vals) = &value {
                            if let Some(decoded) = decode_noise_filter(vals) {
                                ExifValue::Ascii(decoded.to_string())
                            } else {
                                value
                            }
                        } else if let ExifValue::Short(vals) = &value {
                            // Handle unsigned shorts by converting to signed
                            let signed: Vec<i16> = vals.iter().map(|&v| v as i16).collect();
                            if let Some(decoded) = decode_noise_filter(&signed) {
                                ExifValue::Ascii(decoded.to_string())
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_PICTURE_MODE_SATURATION => {
                        if let ExifValue::SShort(vals) = &value {
                            ExifValue::Ascii(format_min_max_array(vals))
                        } else if let ExifValue::Short(vals) = &value {
                            ExifValue::Ascii(format_min_max_array_u16(vals))
                        } else {
                            value
                        }
                    }
                    CS_PICTURE_MODE_CONTRAST => {
                        if let ExifValue::SShort(vals) = &value {
                            ExifValue::Ascii(format_min_max_array(vals))
                        } else if let ExifValue::Short(vals) = &value {
                            ExifValue::Ascii(format_min_max_array_u16(vals))
                        } else {
                            value
                        }
                    }
                    CS_PICTURE_MODE_SHARPNESS => {
                        if let ExifValue::SShort(vals) = &value {
                            ExifValue::Ascii(format_min_max_array(vals))
                        } else if let ExifValue::Short(vals) = &value {
                            ExifValue::Ascii(format_min_max_array_u16(vals))
                        } else {
                            value
                        }
                    }
                    CS_PICTURE_MODE_BW_FILTER => {
                        // PictureModeBWFilter: stored as int16s
                        let decode_bw_filter = |v: i16| -> &'static str {
                            match v {
                                0 => "n/a",
                                1 => "Neutral",
                                2 => "Yellow",
                                3 => "Orange",
                                4 => "Red",
                                5 => "Green",
                                _ => "",
                            }
                        };
                        match &value {
                            ExifValue::SShort(vals) if !vals.is_empty() => {
                                let decoded = decode_bw_filter(vals[0]);
                                if !decoded.is_empty() {
                                    ExifValue::Ascii(decoded.to_string())
                                } else {
                                    value
                                }
                            }
                            ExifValue::Short(vals) if !vals.is_empty() => {
                                let decoded = decode_bw_filter(vals[0] as i16);
                                if !decoded.is_empty() {
                                    ExifValue::Ascii(decoded.to_string())
                                } else {
                                    value
                                }
                            }
                            _ => value,
                        }
                    }
                    CS_PICTURE_MODE_TONE => {
                        // PictureModeTone: decode using picture_mode_tone lookup
                        if let ExifValue::SShort(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_picture_mode_tone_exiftool(vals[0] as u16).to_string(),
                                )
                            } else {
                                value
                            }
                        } else if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_picture_mode_tone_exiftool(vals[0]).to_string(),
                                )
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_SCENE_MODE => {
                        // Skip CS SceneMode if Main IFD SceneMode (0x0403) already exists
                        // Main IFD SceneMode has different value mapping (0=Normal vs 0=Standard)
                        // and ExifTool outputs Main IFD SceneMode when both are present
                        if tags.contains_key(&OLYMPUS_SCENE_MODE) {
                            continue;
                        }
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
                                ExifValue::Ascii(decode_noise_reduction_exiftool(vals[0]))
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
                        // FlashControlMode: int16u[3 or 4], first decoded, rest as numbers
                        // Format: "Off; 0; 0; 0"
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                let first = decode_flash_control_mode_exiftool(vals[0]);
                                let rest: Vec<String> =
                                    vals.iter().skip(1).map(|v| v.to_string()).collect();
                                let result = if rest.is_empty() {
                                    first.to_string()
                                } else {
                                    format!("{}; {}", first, rest.join("; "))
                                };
                                ExifValue::Ascii(result)
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
                    CS_IMAGE_QUALITY2 => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_image_quality2_exiftool(vals[0]).to_string(),
                                )
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_IMAGE_STABILIZATION => {
                        // ImageStabilization is int32u
                        if let ExifValue::Long(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_image_stabilization_exiftool(vals[0]).to_string(),
                                )
                            } else {
                                value
                            }
                        } else if let ExifValue::Short(vals) = &value {
                            // Fallback for older cameras that might store as Short
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_image_stabilization_exiftool(vals[0] as u32).to_string(),
                                )
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_DISTORTION_CORRECTION => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_distortion_correction_exiftool(vals[0]).to_string(),
                                )
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_SHADING_COMPENSATION => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_shading_compensation_exiftool(vals[0]).to_string(),
                                )
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_AF_FINE_TUNE => {
                        // AFFineTune can be stored as int8u or int16u
                        match &value {
                            ExifValue::Short(vals) if !vals.is_empty() => {
                                ExifValue::Ascii(decode_af_fine_tune_exiftool(vals[0]).to_string())
                            }
                            ExifValue::Byte(vals) if !vals.is_empty() => ExifValue::Ascii(
                                decode_af_fine_tune_exiftool(vals[0] as u16).to_string(),
                            ),
                            _ => value,
                        }
                    }
                    CS_DRIVE_MODE => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(decode_drive_mode(vals))
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_PICTURE_MODE => {
                        // PictureMode: 1 or 2 int16u values
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(decode_picture_mode_array(vals))
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_ART_FILTER | CS_MAGIC_FILTER => {
                        // ArtFilter/MagicFilter: int16u[4], first element decoded, rest as numbers
                        // Format: "Off; 0; 0; 0"
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                let first = decode_art_filter_exiftool(vals[0]);
                                let rest: Vec<String> =
                                    vals.iter().skip(1).map(|v| v.to_string()).collect();
                                let result = if rest.is_empty() {
                                    first.to_string()
                                } else {
                                    format!("{}; {}", first, rest.join("; "))
                                };
                                ExifValue::Ascii(result)
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    CS_PREVIEW_IMAGE_VALID => {
                        // PreviewImageValid: int32u, 0 = No, 1 = Yes
                        match &value {
                            ExifValue::Long(vals) if !vals.is_empty() => ExifValue::Ascii(
                                if vals[0] == 0 { "No" } else { "Yes" }.to_string(),
                            ),
                            ExifValue::Short(vals) if !vals.is_empty() => ExifValue::Ascii(
                                if vals[0] == 0 { "No" } else { "Yes" }.to_string(),
                            ),
                            _ => value,
                        }
                    }
                    CS_PREVIEW_IMAGE_START => {
                        // PreviewImageStart: int32u - convert to file-absolute offset
                        // ExifTool outputs file-absolute offset, so add the MakerNote base offset
                        match &value {
                            ExifValue::Long(vals) if !vals.is_empty() => {
                                let adjusted = vals[0] as usize + preview_offset_adjust;
                                ExifValue::Long(vec![adjusted as u32])
                            }
                            _ => value,
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
                    RD_SETTINGS => {
                        // RawDevSettings: 0 = (none), otherwise bitmask
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() && vals[0] == 0 {
                                ExifValue::Ascii("(none)".to_string())
                            } else {
                                value
                            }
                        } else if let ExifValue::Long(vals) = &value {
                            if !vals.is_empty() && vals[0] == 0 {
                                ExifValue::Ascii("(none)".to_string())
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
                        // MultipleExposureMode: int16u[2], first decoded, second as number
                        // Format: "Off; 1"
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                let first = decode_multiple_exposure_mode_exiftool(vals[0]);
                                let result = if vals.len() > 1 {
                                    format!("{}; {}", first, vals[1])
                                } else {
                                    first.to_string()
                                };
                                ExifValue::Ascii(result)
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    IP_ASPECT_RATIO => {
                        // AspectRatio: 2 bytes - first is aspect type, second indicates RAW/non-RAW
                        // ExifTool: '2 1' => "3:2 (RAW)", '2 2' => "3:2", '1 4' => "1:1"
                        // Second byte: 1 = RAW, same as first = non-RAW, 4 with first=1 = special "1:1"
                        if let ExifValue::Byte(vals) = &value {
                            if vals.len() >= 2 {
                                // Special case: '1 4' = "1:1"
                                if vals[0] == 1 && vals[1] == 4 {
                                    ExifValue::Ascii("1:1".to_string())
                                } else {
                                    let aspect: &str = match vals[0] {
                                        1 => "4:3",
                                        2 => "3:2",
                                        3 => "16:9",
                                        4 => "6:6",
                                        5 => "5:4",
                                        6 => "7:6",
                                        7 => "6:5",
                                        8 => "7:5",
                                        9 => "3:4",
                                        _ => "",
                                    };
                                    if aspect.is_empty() {
                                        // Unknown aspect ratio, output raw values
                                        ExifValue::Ascii(format!("{} {}", vals[0], vals[1]))
                                    } else {
                                        // Second byte: 1 = RAW (when first != 1), otherwise just output aspect
                                        let suffix = if vals[1] == 1 && vals[0] != 1 {
                                            " (RAW)"
                                        } else {
                                            ""
                                        };
                                        ExifValue::Ascii(format!("{}{}", aspect, suffix))
                                    }
                                }
                            } else if !vals.is_empty() {
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
                    IP_DISTORTION_CORRECTION2 => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_distortion_correction_exiftool(vals[0]).to_string(),
                                )
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    IP_SHADING_COMPENSATION2 => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_shading_compensation_exiftool(vals[0]).to_string(),
                                )
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    IP_NOISE_REDUCTION2 => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(decode_noise_reduction2_exiftool(vals[0]))
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    IP_COLOR_MATRIX => {
                        // ColorMatrix: int16s[9] - values should be interpreted as signed
                        if let ExifValue::Short(vals) = &value {
                            let signed: Vec<i16> = vals.iter().map(|&v| v as i16).collect();
                            let formatted: Vec<String> =
                                signed.iter().map(|v| v.to_string()).collect();
                            ExifValue::Ascii(formatted.join(" "))
                        } else if let ExifValue::SShort(vals) = &value {
                            let formatted: Vec<String> =
                                vals.iter().map(|v| v.to_string()).collect();
                            ExifValue::Ascii(formatted.join(" "))
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
                                        // Format with 3 decimal places, trim trailing zeros (ExifTool style)
                                        let formatted = format!("{:.3}", v);
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
                                        // Format with 3 decimal places, trim trailing zeros (ExifTool style)
                                        let formatted = format!("{:.3}", v);
                                        let formatted =
                                            formatted.trim_end_matches('0').trim_end_matches('.');
                                        ExifValue::Ascii(format!("{} m", formatted))
                                    }
                                }
                            }
                            _ => value,
                        }
                    }
                    FI_INTERNAL_FLASH => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_internal_flash_exiftool(vals[0]).to_string(),
                                )
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    FI_MACRO_LED => {
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(decode_macro_led_exiftool(vals[0]).to_string())
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    FI_AF_POINT => {
                        // Model-specific AFPoint decoding:
                        // - E-3/E-5/E-30: 11-point AF with split encoding (lower 5 bits = point, upper = mode)
                        // - E-520/E-600/E-620: 7-point AF
                        // - E-M*/OM-*: don't decode (raw value)
                        // - Others: basic 4-point decoding
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                let decoded =
                                    decode_af_point_by_model(vals[0], camera_model.as_deref());
                                ExifValue::Ascii(decoded)
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    FI_EXTERNAL_FLASH => {
                        // ExternalFlash is int16u[2], first value is On/Off
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                ExifValue::Ascii(
                                    decode_external_flash_exiftool(vals[0]).to_string(),
                                )
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    FI_EXTERNAL_FLASH_BOUNCE => {
                        // ExternalFlashBounce: int16u, 0 => "Bounce or Off", 1 => "Direct"
                        if let ExifValue::Short(vals) = &value {
                            if !vals.is_empty() {
                                match vals[0] {
                                    0 => ExifValue::Ascii("Bounce or Off".to_string()),
                                    1 => ExifValue::Ascii("Direct".to_string()),
                                    _ => value,
                                }
                            } else {
                                value
                            }
                        } else {
                            value
                        }
                    }
                    FI_MANUAL_FLASH => {
                        // ManualFlash is int16u[2]: (on/off, strength)
                        // "0 0" => "Off", "1 1" => "Full", "1 2" => "1/2", etc.
                        if let ExifValue::Short(vals) = &value {
                            ExifValue::Ascii(decode_manual_flash(vals))
                        } else {
                            value
                        }
                    }
                    FI_SENSOR_TEMPERATURE => {
                        // SensorTemperature: int16s
                        // For E-1/E-M* or count > 1: raw value with " C" suffix
                        // For others: apply formula 84 - 3 * val / 26, format as "%.1f C"
                        // Check if model is E-1 or E-M5 specifically (not E-M1, E-M10, etc.)
                        let is_e1_or_em5 = camera_model
                            .as_ref()
                            .is_some_and(|m| m == "E-1" || m == "E-M5");
                        match &value {
                            ExifValue::SShort(vals) if vals.len() == 1 && !is_e1_or_em5 => {
                                // Single value on non-E-M camera - apply temperature formula
                                let raw = vals[0] as f64;
                                let temp = 84.0 - 3.0 * raw / 26.0;
                                ExifValue::Ascii(format!("{:.1} C", temp))
                            }
                            ExifValue::SShort(vals) if vals.len() == 1 => {
                                // E-M or E-1 camera - output raw value
                                ExifValue::Ascii(format!("{} C", vals[0]))
                            }
                            ExifValue::SShort(vals) => {
                                // Multiple values - raw format with " 0 0" stripped
                                let vals_str: Vec<String> =
                                    vals.iter().map(|v| v.to_string()).collect();
                                let mut result = vals_str.join(" ");
                                if result.ends_with(" 0 0") {
                                    result = result[..result.len() - 4].to_string();
                                }
                                ExifValue::Ascii(format!("{} C", result))
                            }
                            ExifValue::Short(vals) if vals.len() == 1 && !is_e1_or_em5 => {
                                // Single value on non-E-M camera - apply temperature formula
                                let raw = vals[0] as i16 as f64;
                                let temp = 84.0 - 3.0 * raw / 26.0;
                                ExifValue::Ascii(format!("{:.1} C", temp))
                            }
                            ExifValue::Short(vals) if vals.len() == 1 => {
                                // E-M or E-1 camera - output raw value
                                ExifValue::Ascii(format!("{} C", vals[0] as i16))
                            }
                            ExifValue::Short(vals) => {
                                let vals_str: Vec<String> =
                                    vals.iter().map(|v| (*v as i16).to_string()).collect();
                                let mut result = vals_str.join(" ");
                                if result.ends_with(" 0 0") {
                                    result = result[..result.len() - 4].to_string();
                                }
                                ExifValue::Ascii(format!("{} C", result))
                            }
                            _ => value,
                        }
                    }
                    FI_IMAGE_STABILIZATION => {
                        // FocusInfo ImageStabilization (0x1600): binary blob
                        // First 4 bytes all zero = Off, otherwise On + mode from byte 44
                        if let ExifValue::Undefined(bytes) = &value {
                            ExifValue::Ascii(decode_fi_image_stabilization(bytes))
                        } else {
                            value
                        }
                    }
                    _ => value,
                },
            };

            // Create tag with exiv2 group for proper exiv2-format output
            let exiv2_group = get_exiv2_group_for_ifd(ifd_type);
            // For exiv2, use the raw tag name without prefix
            let exiv2_name = tag_name.map(|n| {
                // Strip any prefix (e.g., "Equipment." or "CameraSettings.")
                n.rfind('.').map(|pos| &n[pos + 1..]).unwrap_or(n)
            });
            let tag = if let Some(name) = exiv2_name {
                MakerNoteTag::with_exiv2(tag_id, tag_name, value.clone(), value, exiv2_group, name)
            } else {
                MakerNoteTag::new(tag_id, tag_name, value)
            };
            tags.insert(storage_id, tag);

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
                                    camera_model,
                                    preview_offset_adjust,
                                );
                            } else if let Some(sub_data) = sub_data_opt {
                                parse_embedded_sub_ifd(
                                    &sub_data,
                                    endian,
                                    OlympusIfdType::Equipment,
                                    tags,
                                    &format!("{}Equipment.", prefix),
                                    camera_model,
                                    preview_offset_adjust,
                                    value_data,
                                    value_base_offset,
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
                                    camera_model,
                                    preview_offset_adjust,
                                );
                            } else if let Some(sub_data) = sub_data_opt {
                                parse_embedded_sub_ifd(
                                    &sub_data,
                                    endian,
                                    OlympusIfdType::CameraSettings,
                                    tags,
                                    &format!("{}CameraSettings.", prefix),
                                    camera_model,
                                    preview_offset_adjust,
                                    value_data,
                                    value_base_offset,
                                );
                            }
                        }
                    }
                    OLYMPUS_RAW_DEVELOPMENT_IFD | OLYMPUS_RAW_DEVELOPMENT2_IFD => {
                        if let ExifValue::Long(offsets) =
                            tags.get(&storage_id).map(|t| &t.value).unwrap()
                            && !offsets.is_empty()
                        {
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
                                camera_model,
                                preview_offset_adjust,
                            );
                        }
                    }
                    OLYMPUS_IMAGE_PROCESSING_IFD => {
                        if let ExifValue::Long(offsets) =
                            tags.get(&storage_id).map(|t| &t.value).unwrap()
                            && !offsets.is_empty()
                        {
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
                                camera_model,
                                preview_offset_adjust,
                            );
                        }
                    }
                    OLYMPUS_FOCUS_INFO_IFD => {
                        // Handle both offset-based sub-IFD and embedded IFD data
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
                                    OlympusIfdType::FocusInfo,
                                    tags,
                                    &format!("{}FocusInfo.", prefix),
                                    value_data,
                                    value_base_offset,
                                    camera_model,
                                    preview_offset_adjust,
                                );
                            } else if let Some(sub_data) = sub_data_opt {
                                parse_embedded_sub_ifd(
                                    &sub_data,
                                    endian,
                                    OlympusIfdType::FocusInfo,
                                    tags,
                                    &format!("{}FocusInfo.", prefix),
                                    camera_model,
                                    preview_offset_adjust,
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
    tiff_endian: Endianness,
    tiff_data: Option<&[u8]>,
    tiff_offset: usize,
    makernote_file_offset: Option<usize>,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    // Olympus maker notes have three formats:
    // 1. New format: "OLYMPUS\0II\x03\0" or "OLYMPUS\0MM\x00\x03" (12 byte header)
    //    - Offsets are relative to maker note start
    // 2. OM Digital format: "OM SYSTEM\0\0\0II\x04\0" (16 byte header)
    //    - Used by OM Digital Solutions cameras (OM-1, OM-3, OM-5, etc.)
    //    - Offsets are relative to maker note start
    // 3. Old format: "OLYMP\0" (6 byte header) - used by older cameras like E-1, E-300
    //    - Offsets are relative to TIFF header (need tiff_data to resolve)
    if data.len() < 8 {
        return Ok(tags);
    }

    // Determine format and endianness
    let (mn_endian, ifd_offset, is_old_format) = if data.starts_with(b"OM SYSTEM\0") {
        // OM Digital format: "OM SYSTEM\0\0\0" (12 bytes) + 2-byte endian + 2-byte version
        // IFD starts at offset 16
        if data.len() < 18 {
            return Ok(tags);
        }
        let endian = if data[12] == b'I' && data[13] == b'I' {
            Endianness::Little
        } else if data[12] == b'M' && data[13] == b'M' {
            Endianness::Big
        } else {
            return Ok(tags);
        };
        (endian, 16, false)
    } else if data.starts_with(b"OLYMPUS\0") {
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
        // Inherit endianness from TIFF header (E20 uses big-endian, most others little-endian)
        (tiff_endian, 8, true)
    } else {
        return Ok(tags);
    };

    // For old format, offsets in IFD entries are relative to TIFF header.
    // We pass tiff_data for resolving those offsets.
    let value_data = if is_old_format { tiff_data } else { None };
    let value_base_offset = if is_old_format { tiff_offset } else { 0 };

    // Calculate offset adjustment for PreviewImageStart to get TIFF-relative offsets
    // ExifTool outputs offsets relative to TIFF header start
    // For new format: MakerNote-relative -> TIFF-relative = makernote_offset_in_TIFF + raw_value
    //   where makernote_offset_in_TIFF = makernote_file_offset - tiff_offset
    // For old format: values are already TIFF-relative, no adjustment needed
    let preview_offset_adjust = if is_old_format {
        0 // Old format values are already TIFF-relative
    } else {
        // New format: convert from MakerNote-relative to TIFF-relative
        makernote_file_offset
            .map(|mn_off| mn_off.saturating_sub(tiff_offset))
            .unwrap_or(0)
    };

    // Track camera model for model-specific AFPoint decoding
    // E-3/E-5/E-30 use 11-point AF, E-520/E-600/E-620 use 7-point AF, E-M/OM- don't decode
    let mut camera_model: Option<String> = None;

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
        &mut camera_model,
        preview_offset_adjust,
    );

    Ok(tags)
}
