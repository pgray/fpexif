// makernotes/nikon.rs - Nikon maker notes parsing

use crate::data_types::{Endianness, ExifValue};
use crate::define_tag_decoder;
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Cursor;

// exiv2 group names for Nikon sub-IFDs
pub const EXIV2_GROUP_NIKON3: &str = "Nikon3";
pub const EXIV2_GROUP_NIKON_PC: &str = "NikonPc";
pub const EXIV2_GROUP_NIKON_LD: &str = "NikonLd3"; // Most common lens data version
pub const EXIV2_GROUP_NIKON_VR: &str = "NikonVr";
pub const EXIV2_GROUP_NIKON_WT: &str = "NikonWt";
pub const EXIV2_GROUP_NIKON_II: &str = "NikonIi";
pub const EXIV2_GROUP_NIKON_AF: &str = "NikonAf";
pub const EXIV2_GROUP_NIKON_AF2: &str = "NikonAf2";
pub const EXIV2_GROUP_NIKON_FI: &str = "NikonFi";
pub const EXIV2_GROUP_NIKON_FL: &str = "NikonFl3"; // Most common flash version
pub const EXIV2_GROUP_NIKON_ME: &str = "NikonMe";

/// Map Nikon sub-IFD field names to exiv2 group and tag name
/// Returns (exiv2_group, exiv2_name) for the given field name and parent tag
pub fn get_exiv2_nikon_subfield(
    parent_tag: u16,
    field_name: &str,
) -> Option<(&'static str, &'static str)> {
    match parent_tag {
        NIKON_PICTURE_CONTROL_DATA => match field_name {
            "PictureControlVersion" => Some((EXIV2_GROUP_NIKON_PC, "Version")),
            "PictureControlName" => Some((EXIV2_GROUP_NIKON_PC, "Name")),
            "PictureControlBase" => Some((EXIV2_GROUP_NIKON_PC, "Base")),
            "PictureControlAdjust" => Some((EXIV2_GROUP_NIKON_PC, "Adjust")),
            "PictureControlQuickAdjust" => Some((EXIV2_GROUP_NIKON_PC, "QuickAdjust")),
            "Sharpness" => Some((EXIV2_GROUP_NIKON_PC, "Sharpness")),
            "Contrast" => Some((EXIV2_GROUP_NIKON_PC, "Contrast")),
            "Brightness" => Some((EXIV2_GROUP_NIKON_PC, "Brightness")),
            "Saturation" => Some((EXIV2_GROUP_NIKON_PC, "Saturation")),
            "HueAdjustment" => Some((EXIV2_GROUP_NIKON_PC, "HueAdjustment")),
            "FilterEffect" => Some((EXIV2_GROUP_NIKON_PC, "FilterEffect")),
            "ToningEffect" => Some((EXIV2_GROUP_NIKON_PC, "ToningEffect")),
            "ToningSaturation" => Some((EXIV2_GROUP_NIKON_PC, "ToningSaturation")),
            _ => None,
        },
        NIKON_LENS_DATA => match field_name {
            "LensDataVersion" => Some((EXIV2_GROUP_NIKON_LD, "Version")),
            "LensIDNumber" => Some((EXIV2_GROUP_NIKON_LD, "LensIDNumber")),
            "LensFStops" => Some((EXIV2_GROUP_NIKON_LD, "LensFStops")),
            "MinFocalLength" => Some((EXIV2_GROUP_NIKON_LD, "MinFocalLength")),
            "MaxFocalLength" => Some((EXIV2_GROUP_NIKON_LD, "MaxFocalLength")),
            "MaxApertureAtMinFocal" => Some((EXIV2_GROUP_NIKON_LD, "MaxApertureAtMinFocal")),
            "MaxApertureAtMaxFocal" => Some((EXIV2_GROUP_NIKON_LD, "MaxApertureAtMaxFocal")),
            "MCUVersion" => Some((EXIV2_GROUP_NIKON_LD, "MCUVersion")),
            "EffectiveMaxAperture" => Some((EXIV2_GROUP_NIKON_LD, "EffectiveMaxAperture")),
            "FocalLength" => Some((EXIV2_GROUP_NIKON_LD, "FocalLength")),
            "FocusDistance" => Some((EXIV2_GROUP_NIKON_LD, "FocusDistance")),
            "AFAperture" => Some((EXIV2_GROUP_NIKON_LD, "AFAperture")),
            _ => None,
        },
        NIKON_VR_INFO => match field_name {
            "VRInfoVersion" => Some((EXIV2_GROUP_NIKON_VR, "Version")),
            "VibrationReduction" => Some((EXIV2_GROUP_NIKON_VR, "VibrationReduction")),
            "VRMode" => Some((EXIV2_GROUP_NIKON_VR, "VRMode")),
            _ => None,
        },
        NIKON_WORLD_TIME => match field_name {
            "Timezone" => Some((EXIV2_GROUP_NIKON_WT, "Timezone")),
            "DaylightSavings" => Some((EXIV2_GROUP_NIKON_WT, "DaylightSavings")),
            "DateDisplayFormat" => Some((EXIV2_GROUP_NIKON_WT, "DateDisplayFormat")),
            _ => None,
        },
        NIKON_ISO_INFO => match field_name {
            "ISO" => Some((EXIV2_GROUP_NIKON_II, "ISO")),
            "ISOExpansion" => Some((EXIV2_GROUP_NIKON_II, "ISOExpansion")),
            "ISO2" => Some((EXIV2_GROUP_NIKON_II, "ISO2")),
            "ISOExpansion2" => Some((EXIV2_GROUP_NIKON_II, "ISOExpansion2")),
            _ => None,
        },
        NIKON_AF_INFO => match field_name {
            "AFAreaMode" => Some((EXIV2_GROUP_NIKON_AF, "AFAreaMode")),
            "AFPoint" => Some((EXIV2_GROUP_NIKON_AF, "AFPoint")),
            "AFPointsInFocus" => Some((EXIV2_GROUP_NIKON_AF, "AFPointsInFocus")),
            _ => None,
        },
        NIKON_AF_INFO_2 => match field_name {
            "Version" | "AFInfo2Version" => Some((EXIV2_GROUP_NIKON_AF2, "Version")),
            "ContrastDetectAF" => Some((EXIV2_GROUP_NIKON_AF2, "ContrastDetectAF")),
            "AFAreaMode" => Some((EXIV2_GROUP_NIKON_AF2, "AFAreaMode")),
            "PhaseDetectAF" => Some((EXIV2_GROUP_NIKON_AF2, "PhaseDetectAF")),
            "PrimaryAFPoint" => Some((EXIV2_GROUP_NIKON_AF2, "PrimaryAFPoint")),
            "AFPointsUsed" => Some((EXIV2_GROUP_NIKON_AF2, "AFPointsUsed")),
            _ => None,
        },
        NIKON_FLASH_INFO | NIKON_FLASH_INFO_2 => match field_name {
            "FlashInfoVersion" => Some((EXIV2_GROUP_NIKON_FL, "Version")),
            "FlashSource" => Some((EXIV2_GROUP_NIKON_FL, "FlashSource")),
            "ExternalFlashFirmware" => Some((EXIV2_GROUP_NIKON_FL, "ExternalFlashFirmware")),
            "ExternalFlashFlags" => Some((EXIV2_GROUP_NIKON_FL, "ExternalFlashFlags")),
            "FlashCommanderMode" => Some((EXIV2_GROUP_NIKON_FL, "FlashCommanderMode")),
            "FlashControlMode" => Some((EXIV2_GROUP_NIKON_FL, "FlashControlMode")),
            "FlashGNDistance" => Some((EXIV2_GROUP_NIKON_FL, "FlashGNDistance")),
            "FlashColorFilter" => Some((EXIV2_GROUP_NIKON_FL, "FlashColorFilter")),
            _ => None,
        },
        NIKON_FILE_INFO => match field_name {
            "FileInfoVersion" => Some((EXIV2_GROUP_NIKON_FI, "Version")),
            "DirectoryNumber" => Some((EXIV2_GROUP_NIKON_FI, "DirectoryNumber")),
            "FileNumber" => Some((EXIV2_GROUP_NIKON_FI, "FileNumber")),
            _ => None,
        },
        NIKON_MULTI_EXPOSURE => match field_name {
            "MultiExposureVersion" => Some((EXIV2_GROUP_NIKON_ME, "Version")),
            "MultiExposureMode" => Some((EXIV2_GROUP_NIKON_ME, "MultiExposureMode")),
            "MultiExposureShots" => Some((EXIV2_GROUP_NIKON_ME, "MultiExposureShots")),
            "MultiExposureAutoGain" => Some((EXIV2_GROUP_NIKON_ME, "MultiExposureAutoGain")),
            _ => None,
        },
        _ => None,
    }
}

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
pub const NIKON_EXTERNAL_FLASH_EXPOSURE_COMP: u16 = 0x0017;
pub const NIKON_FLASH_EXPOSURE_BRACKET_VALUE: u16 = 0x0018;
pub const NIKON_EXPOSURE_BRACKET_VALUE: u16 = 0x0019;
pub const NIKON_IMAGE_PROCESSING: u16 = 0x001A;
pub const NIKON_CROP_HI_SPEED: u16 = 0x001B;
pub const NIKON_EXPOSURE_TUNING: u16 = 0x001C;
pub const NIKON_SERIAL_NUMBER: u16 = 0x001D;
pub const NIKON_COLOR_SPACE: u16 = 0x001E;
pub const NIKON_VR_INFO: u16 = 0x001F;
pub const NIKON_IMAGE_AUTHENTICATION: u16 = 0x0020;
pub const NIKON_FACE_DETECT: u16 = 0x0021;
pub const NIKON_ACTIVE_D_LIGHTING: u16 = 0x0022;
pub const NIKON_HIGH_ISO_NOISE_REDUCTION: u16 = 0x00B1;
pub const NIKON_PICTURE_CONTROL_DATA: u16 = 0x0023;
pub const NIKON_PICTURE_CONTROL_DATA_2: u16 = 0x00BD; // Coolpix (P6000, P7000, etc.)
pub const NIKON_WORLD_TIME: u16 = 0x0024;
pub const NIKON_ISO_INFO: u16 = 0x0025;
pub const NIKON_VIGNETTE_CONTROL: u16 = 0x002A;
pub const NIKON_DISTORTION_CONTROL: u16 = 0x002B;
pub const NIKON_AUXILIARY_LENS: u16 = 0x0082;
pub const NIKON_LENS_TYPE: u16 = 0x0083;
pub const NIKON_LENS: u16 = 0x0084;
pub const NIKON_MANUAL_FOCUS_DISTANCE: u16 = 0x0085;
pub const NIKON_DIGITAL_ZOOM: u16 = 0x0086;
pub const NIKON_FLASH_MODE: u16 = 0x0087;
pub const NIKON_AF_INFO: u16 = 0x0088;
pub const NIKON_SHOT_INFO: u16 = 0x0091;
pub const NIKON_HUE_ADJUSTMENT: u16 = 0x0092;
pub const NIKON_NEF_COMPRESSION: u16 = 0x0093;
pub const NIKON_SATURATION: u16 = 0x0094;
pub const NIKON_NOISE_REDUCTION: u16 = 0x0095;
pub const NIKON_COLOR_BALANCE: u16 = 0x0097;
pub const NIKON_LENS_DATA: u16 = 0x0098;
pub const NIKON_RAW_IMAGE_CENTER: u16 = 0x0099;
pub const NIKON_FLASH_INFO: u16 = 0x00A8;
pub const NIKON_DATE_STAMP_MODE: u16 = 0x009D;
pub const NIKON_RETOUCH_HISTORY: u16 = 0x009E;
pub const NIKON_SERIAL_NUMBER_2: u16 = 0x00A0;
pub const NIKON_IMAGE_DATA_SIZE: u16 = 0x00A2;
pub const NIKON_IMAGE_COUNT: u16 = 0x00A5;
pub const NIKON_DELETED_IMAGE_COUNT: u16 = 0x00A6;
pub const NIKON_SHUTTER_COUNT: u16 = 0x00A7;
pub const NIKON_IMAGE_OPTIMIZATION: u16 = 0x00A9;
pub const NIKON_SATURATION_2: u16 = 0x00AA;
pub const NIKON_VARI_PROGRAM: u16 = 0x00AB;
pub const NIKON_IMAGE_STABILIZATION: u16 = 0x00AC;
pub const NIKON_AF_RESPONSE: u16 = 0x00AD;
pub const NIKON_SILENT_PHOTOGRAPHY: u16 = 0x00BF;
pub const NIKON_LIGHT_SOURCE: u16 = 0x0090;
pub const NIKON_SHOOTING_MODE: u16 = 0x0089;
pub const NIKON_AUTO_BRACKET_RELEASE: u16 = 0x008A;
pub const NIKON_LENS_F_STOPS: u16 = 0x008B;
pub const NIKON_CONTRAST_CURVE: u16 = 0x008C;
pub const NIKON_COLOR_HUE: u16 = 0x008D;
pub const NIKON_SCENE_MODE: u16 = 0x008F;
pub const NIKON_IMAGE_ADJUSTMENT: u16 = 0x0080;
pub const NIKON_TONE_COMP: u16 = 0x0081;
pub const NIKON_SENSOR_PIXEL_SIZE: u16 = 0x009A;
pub const NIKON_UNKNOWN_TAG_9B: u16 = 0x009B;
pub const NIKON_SCENE_ASSIST: u16 = 0x009C;
pub const NIKON_TONING_EFFECT: u16 = 0x00B3;
pub const NIKON_BAROMETER_INFO: u16 = 0x00C3;
pub const NIKON_PRINT_IM: u16 = 0x0E00;
pub const NIKON_CAPTURE_DATA: u16 = 0x0E01;
pub const NIKON_CAPTURE_VERSION: u16 = 0x0E09;
pub const NIKON_CAPTURE_OFFSETS: u16 = 0x0E0E;
pub const NIKON_SCAN_IFD: u16 = 0x0E10;
pub const NIKON_ICC_PROFILE: u16 = 0x0E1D;
pub const NIKON_CAPTURE_OUTPUT: u16 = 0x0E1E;
pub const NIKON_MULTI_EXPOSURE: u16 = 0x00B0; // Also HDRInfo for some cameras
pub const NIKON_LOCATION_INFO: u16 = 0x00B5;
pub const NIKON_BLACK_LEVEL: u16 = 0x003D;
pub const NIKON_POWER_UP_TIME_2: u16 = 0x00B6;
pub const NIKON_AF_INFO_2: u16 = 0x00B7;
pub const NIKON_FILE_INFO: u16 = 0x00B8;
pub const NIKON_AF_TUNE: u16 = 0x00B9;
pub const NIKON_RETOUCH_INFO: u16 = 0x00BB;
pub const NIKON_PICTURE_CONTROL_VERSION: u16 = 0x00BC;
pub const NIKON_SILENT_PHOTO: u16 = 0x00BD;
pub const NIKON_SHUTTER_MODE: u16 = 0x0034;
pub const NIKON_HDR_INFO: u16 = 0x0035;
pub const NIKON_MECHANICAL_SHUTTER_COUNT: u16 = 0x0037;

// PictureControl sub-IFD tags (indices in tag 0x0023)
pub const PC_VERSION: u16 = 0x0000;
pub const PC_NAME: u16 = 0x0004;
pub const PC_BASE: u16 = 0x0018;
pub const PC_ADJUST: u16 = 0x0030;
pub const PC_QUICK_ADJUST: u16 = 0x0031;
pub const PC_SHARPNESS: u16 = 0x0032;
pub const PC_CLARITY: u16 = 0x0033;
pub const PC_CONTRAST: u16 = 0x0034;
pub const PC_BRIGHTNESS: u16 = 0x0035;
pub const PC_SATURATION: u16 = 0x0036;
pub const PC_HUE_ADJUSTMENT: u16 = 0x0037;
pub const PC_FILTER_EFFECT: u16 = 0x0038;
pub const PC_TONING_EFFECT: u16 = 0x0039;
pub const PC_TONING_SATURATION: u16 = 0x003A;

// WorldTime sub-IFD tags (indices in tag 0x0024)
pub const WT_TIMEZONE: u16 = 0x0000;
pub const WT_DAYLIGHT_SAVINGS: u16 = 0x0002;
pub const WT_DATE_DISPLAY_FORMAT: u16 = 0x0003;

// ISOInfo sub-IFD tags (indices in tag 0x0025)
pub const II_ISO: u16 = 0x0000;
pub const II_ISO_EXPANSION: u16 = 0x0004;
pub const II_ISO2: u16 = 0x0006;
pub const II_ISO_EXPANSION2: u16 = 0x000A;

// VRInfo sub-IFD tags (indices in tag 0x001F)
pub const VR_VERSION: u16 = 0x0000;
pub const VR_VIBRATION_REDUCTION: u16 = 0x0004;
pub const VR_TYPE: u16 = 0x0008;

// AFInfo sub-IFD tags (indices in tag 0x0088)
pub const AF_AREA_MODE: u16 = 0x0000;
pub const AF_POINT: u16 = 0x0001;
pub const AF_POINTS_IN_FOCUS: u16 = 0x0002;

// ShotInfo sub-IFD tags (varies by version, common ones)
pub const SI_VERSION: u16 = 0x0000;
pub const SI_FIRMWARE_VERSION: u16 = 0x0004;
pub const SI_SHOOTING_MODE: u16 = 0x000A;
pub const SI_AUTO_FOCUS_MODE: u16 = 0x000B;
pub const SI_SHUTTER_COUNT: u16 = 0x000C;
pub const SI_AUTO_FLASH_MODE: u16 = 0x000D;
pub const SI_AUTO_FLASH_COMP: u16 = 0x000E;
pub const SI_AUTO_FOCUS_POINT: u16 = 0x000F;
pub const SI_FLASH_TYPE_2: u16 = 0x0010;
pub const SI_AUTO_BRACKET_MODE: u16 = 0x0011;
pub const SI_AUTO_BRACKET_VALUE: u16 = 0x0012;
pub const SI_ISO_AUTO: u16 = 0x0014;
pub const SI_AUTO_AREA_ZOOM: u16 = 0x0015;
pub const SI_PHOTO_INFO_DISPLAY: u16 = 0x0016;
pub const SI_IMAGE_STABILIZATION: u16 = 0x0017;
pub const SI_AF_AREA_MODE_2: u16 = 0x0018;
pub const SI_VR_MODE: u16 = 0x0019;
pub const SI_ACTIVE_D_LIGHTING_2: u16 = 0x001A;
pub const SI_HIGH_ISO_NR: u16 = 0x001B;
pub const SI_VIGNETTE_CONTROL: u16 = 0x001C;
pub const SI_AUTO_DISTORTION: u16 = 0x001D;
pub const SI_DIFFRACTION_COMP: u16 = 0x001E;
pub const SI_FLASH_FOCAL_LENGTH: u16 = 0x001F;
pub const SI_FACE_DETECT: u16 = 0x0020;
pub const SI_ROLL_ANGLE: u16 = 0x0021;
pub const SI_PITCH_ANGLE: u16 = 0x0022;
pub const SI_YAW_ANGLE: u16 = 0x0023;
pub const SI_FLASH_SHUTTER_SPEED: u16 = 0x0024;
pub const SI_ILLUMINATION: u16 = 0x0025;

// LensData sub-IFD tags (common indices)
pub const LD_VERSION: u16 = 0x0000;
pub const LD_EXIT_PUPIL_POSITION: u16 = 0x0004;
pub const LD_AF_APERTURE: u16 = 0x0005;
pub const LD_FOCUS_POSITION: u16 = 0x0008;
pub const LD_FOCUS_DISTANCE: u16 = 0x0009;
pub const LD_FOCAL_LENGTH: u16 = 0x000A;
pub const LD_LENS_ID_NUMBER: u16 = 0x000B;
pub const LD_LENS_F_STOPS: u16 = 0x000C;
pub const LD_MIN_FOCAL_LENGTH: u16 = 0x000D;
pub const LD_MAX_FOCAL_LENGTH: u16 = 0x000E;
pub const LD_MAX_APERTURE_AT_MIN_FOCAL: u16 = 0x000F;
pub const LD_MAX_APERTURE_AT_MAX_FOCAL: u16 = 0x0010;
pub const LD_MCU_VERSION: u16 = 0x0011;
pub const LD_EFFECTIVE_MAX_APERTURE: u16 = 0x0012;

// ColorBalance sub-IFD tags (indices in tag 0x0097)
pub const CB_VERSION: u16 = 0x0000;
pub const CB_WB_RB_LEVELS: u16 = 0x0004;
pub const CB_WB_GR_LEVEL: u16 = 0x0008;
pub const CB_WB_GB_LEVEL: u16 = 0x000A;

// FlashInfo sub-IFD tags (common indices)
pub const FI_VERSION: u16 = 0x0000;
pub const FI_FLASH_SOURCE: u16 = 0x0004;
pub const FI_EXTERNAL_FLASH_FIRMWARE: u16 = 0x0005;
pub const FI_EXTERNAL_FLASH_FLAGS: u16 = 0x0008;
pub const FI_FLASH_COMMANDER_MODE: u16 = 0x0009;
pub const FI_FLASH_CONTROL_MODE: u16 = 0x000A;
pub const FI_FLASH_OUTPUT: u16 = 0x000B;
pub const FI_FLASH_COMP_BUILTIN: u16 = 0x000C;
pub const FI_FLASH_COMP_EXTERNAL: u16 = 0x000D;
pub const FI_FLASH_COMP_OTHER: u16 = 0x000E;
pub const FI_FLASH_GUID_NUMBER: u16 = 0x000F;
pub const FI_FLASH_GROUP_A_CONTROL_MODE: u16 = 0x0010;
pub const FI_FLASH_GROUP_B_CONTROL_MODE: u16 = 0x0011;
pub const FI_FLASH_GROUP_C_CONTROL_MODE: u16 = 0x0012;
pub const FI_FLASH_GROUP_A_OUTPUT: u16 = 0x0013;
pub const FI_FLASH_GROUP_B_OUTPUT: u16 = 0x0014;
pub const FI_FLASH_GROUP_C_OUTPUT: u16 = 0x0015;
pub const FI_FLASH_GROUP_A_COMP: u16 = 0x0016;
pub const FI_FLASH_GROUP_B_COMP: u16 = 0x0017;
pub const FI_FLASH_GROUP_C_COMP: u16 = 0x0018;
pub const FI_EXTERNAL_FLASH_FLAGS_2: u16 = 0x0019;
pub const FI_FLASH_COLOR_FILTER: u16 = 0x001A;

// MultiExposure sub-IFD tags (indices in tag 0x00B1)
pub const ME_VERSION: u16 = 0x0000;
pub const ME_MULTI_EXPOSURE_MODE: u16 = 0x0004;
pub const ME_MULTI_EXPOSURE_SHOTS: u16 = 0x0005;
pub const ME_MULTI_EXPOSURE_AUTO_GAIN: u16 = 0x0006;

// HDRInfo sub-IFD tags (indices in tag 0x00B0)
pub const HDR_VERSION: u16 = 0x0000;
pub const HDR_HDR: u16 = 0x0004;
pub const HDR_LEVEL: u16 = 0x0005;
pub const HDR_SMOOTHING: u16 = 0x0006;
pub const HDR_LEVEL_2: u16 = 0x0007;

// AFInfo2 sub-IFD tags (indices in tag 0x00B7)
pub const AF2_VERSION: u16 = 0x0000;
pub const AF2_CONTRAST_DETECT_AF: u16 = 0x0004;
pub const AF2_AF_AREA_MODE: u16 = 0x0005;
pub const AF2_PHASE_DETECT_AF: u16 = 0x0006;
pub const AF2_PRIMARY_AF_POINT: u16 = 0x0007;
pub const AF2_AF_POINTS_USED: u16 = 0x0008;
pub const AF2_AF_IMAGE_WIDTH: u16 = 0x0010;
pub const AF2_AF_IMAGE_HEIGHT: u16 = 0x0012;
pub const AF2_AF_AREA_X_POSITION: u16 = 0x0014;
pub const AF2_AF_AREA_Y_POSITION: u16 = 0x0016;
pub const AF2_AF_AREA_WIDTH: u16 = 0x0018;
pub const AF2_AF_AREA_HEIGHT: u16 = 0x001A;
pub const AF2_CONTRAST_DETECT_AF_IN_FOCUS: u16 = 0x001C;

// FileInfo sub-IFD tags (indices in tag 0x00B8)
pub const FILE_VERSION: u16 = 0x0000;
pub const FILE_DIRECTORY_NUMBER: u16 = 0x0002;
pub const FILE_FILE_NUMBER: u16 = 0x0004;

// AFTune sub-IFD tags (indices in tag 0x00B9)
pub const AFT_AF_FINE_TUNE: u16 = 0x0000;
pub const AFT_AF_FINE_TUNE_INDEX: u16 = 0x0001;
pub const AFT_AF_FINE_TUNE_ADJ: u16 = 0x0002;

// LocationInfo sub-IFD tags (indices in tag 0x00B5)
pub const LOC_VERSION: u16 = 0x0000;
pub const LOC_LATITUDE: u16 = 0x0001;
pub const LOC_LONGITUDE: u16 = 0x0002;
pub const LOC_ALTITUDE: u16 = 0x0003;
pub const LOC_SPEED: u16 = 0x0004;
pub const LOC_GPS_DATE_TIME: u16 = 0x0005;

// RetouchInfo sub-IFD tags (indices in tag 0x00BB)
pub const RET_VERSION: u16 = 0x0000;
pub const RET_RETOUCH_HISTORY: u16 = 0x0004;

// Additional main IFD tags
pub const NIKON_SENSOR_TYPE: u16 = 0x00A1;
pub const NIKON_AF_POINT_SELECT: u16 = 0x00A3;
pub const NIKON_FLASH_DATA: u16 = 0x00A4;
pub const NIKON_SUBJECT_DIST_RANGE: u16 = 0x00B2;
pub const NIKON_DIGITAL_VARI_PROGRAM: u16 = 0x00B4;
pub const NIKON_GPS_INFO: u16 = 0x00AE;
pub const NIKON_DUST_REFERENCE_DATA: u16 = 0x00AF;

// More ShotInfo tags for completeness
pub const SI_DIGITAL_VARI_PROGRAM_2: u16 = 0x0030;
pub const SI_PRIMARY_AF_POINT: u16 = 0x0031;
pub const SI_AF_POINT_ACTIVE: u16 = 0x0032;
pub const SI_AF_PRIMARY_IN_FOCUS: u16 = 0x0033;
pub const SI_CONTRAST_DETECT_AF: u16 = 0x0034;
pub const SI_AF_AREA_MODE_3: u16 = 0x0035;
pub const SI_LENS_TYPE_2: u16 = 0x0036;
pub const SI_TIME_ZONE: u16 = 0x0037;

// Extended LensData tags
pub const LD_LENS_TYPE: u16 = 0x0013;
pub const LD_MIN_FOCUS_DISTANCE: u16 = 0x0014;
pub const LD_AF_DRIVE: u16 = 0x0015;
pub const LD_LENS_SERIAL: u16 = 0x0016;

// Extended ColorBalance tags
pub const CB_WB_RGGB_LEVELS: u16 = 0x000C;
pub const CB_WB_GRBG_LEVELS: u16 = 0x0010;

// Extended FlashInfo tags
pub const FI_FLASH_READY: u16 = 0x001B;
pub const FI_FLASH_SYNC: u16 = 0x001C;
pub const FI_FLASH_CURTAIN: u16 = 0x001D;
pub const FI_REPEATING_FLASH_COUNT: u16 = 0x001E;
pub const FI_REPEATING_FLASH_RATE: u16 = 0x001F;
pub const FI_REPEATING_FLASH_OUTPUT: u16 = 0x0020;

// Extended HDRInfo tags
pub const HDR_D_LIGHTING_HQ: u16 = 0x0008;
pub const HDR_NEF_LINKED: u16 = 0x0009;

// Extended AFInfo2 tags
pub const AF2_FOCUS_PRIORITY: u16 = 0x001E;
pub const AF2_AF_TRACKING: u16 = 0x0020;
pub const AF2_3D_TRACKING_WATCH_AREA: u16 = 0x0022;
pub const AF2_AF_ACTIVATION: u16 = 0x0024;

// Extended FileInfo tags
pub const FILE_FILE_TYPE: u16 = 0x0006;
pub const FILE_SEQUENCE_NUMBER: u16 = 0x0008;

// Extended LocationInfo tags
pub const LOC_DIRECTION: u16 = 0x0006;
pub const LOC_DATE: u16 = 0x0007;
pub const LOC_TIME: u16 = 0x0008;
pub const LOC_MAP_DATUM: u16 = 0x0009;

// More extended main IFD tags
pub const NIKON_NEF_BIT_DEPTH: u16 = 0x0E22;
pub const NIKON_EXTRA_INFO: u16 = 0x00A0;
pub const NIKON_ORIENTATION: u16 = 0x0036;
pub const NIKON_CAPTURE_OFFSET: u16 = 0x0E02;
pub const NIKON_CAPTURE_INFO: u16 = 0x0E03;
pub const NIKON_CAPTURE_EDIT_VERSIONS: u16 = 0x0E04;
pub const NIKON_CAPTURE_NX_VERSION: u16 = 0x0E05;
pub const NIKON_CAPTURE_COLOR: u16 = 0x0E06;
pub const NIKON_CAPTURE_TONE: u16 = 0x0E07;
pub const NIKON_CAPTURE_SHARPENER: u16 = 0x0E08;
pub const NIKON_CAPTURE_NX_COLOR_MODE: u16 = 0x0E0A;
pub const NIKON_CAPTURE_NX_OUTPUT: u16 = 0x0E0B;
pub const NIKON_CAPTURE_NX_TONE_CURVE: u16 = 0x0E0C;
pub const NIKON_CAPTURE_NX_DEE: u16 = 0x0E0D;
pub const NIKON_SCAN_INDEX: u16 = 0x0E11;
pub const NIKON_SCAN_SERIAL_INFO: u16 = 0x0E12;
pub const NIKON_SCAN_PREVIEW: u16 = 0x0E13;
pub const NIKON_SCAN_NEGATIVE_SIZE: u16 = 0x0E14;
pub const NIKON_SCAN_EXPOSURE_INFO: u16 = 0x0E15;

// More ShotInfo extended tags
pub const SI_AF_STATUS: u16 = 0x0040;
pub const SI_AF_FINE_TUNE_2: u16 = 0x0041;
pub const SI_HAZE_CONTROL: u16 = 0x0042;
pub const SI_PICTURE_CONTROL_ADJUST: u16 = 0x0043;
pub const SI_CLARITY: u16 = 0x0044;
pub const SI_MID_RANGE_SHARPNESS: u16 = 0x0045;
pub const SI_SECONDARY_SLOT_FUNCTION: u16 = 0x0046;
pub const SI_ELECTRONIC_FRONT_CURTAIN: u16 = 0x0047;
pub const SI_BRACKETING_PROGRAM: u16 = 0x0048;
pub const SI_EXPOSURE_PROGRAM_2: u16 = 0x0049;
pub const SI_TIMER_RECORDING: u16 = 0x004A;
pub const SI_SILENT_SHOOTING: u16 = 0x004B;
pub const SI_FLICKER_REDUCTION: u16 = 0x004C;
pub const SI_NEGATIVE_DIGITIZER: u16 = 0x004D;
pub const SI_SKIN_SOFTENING: u16 = 0x004E;
pub const SI_PORTRAIT_IMPRESSION_BALANCE: u16 = 0x004F;

// Extended PictureControl tags
pub const PC_SHARPENING: u16 = 0x003B;
pub const PC_CLARITY_2: u16 = 0x003C;
pub const PC_MID_RANGE_SHARPNESS: u16 = 0x003D;

// Extended LensData (for newer lenses)
pub const LD_LENS_ID: u16 = 0x0020;
pub const LD_FOCUS_POSITION_2: u16 = 0x0021;
pub const LD_APERTURE: u16 = 0x0022;
pub const LD_VR_MODE: u16 = 0x0023;

// Additional ColorBalance/WhiteBalance tags
pub const CB_WB_PRESET: u16 = 0x0014;
pub const CB_WB_FINE_TUNE: u16 = 0x0015;
pub const CB_WB_RGB_LEVELS: u16 = 0x0016;

// Preview sub-IFD tags (in tag 0x0011)
pub const PRV_IFD_VERSION: u16 = 0x0000;
pub const PRV_COMPRESSION: u16 = 0x0103;
pub const PRV_IMAGE_WIDTH: u16 = 0x0100;
pub const PRV_IMAGE_HEIGHT: u16 = 0x0101;
pub const PRV_BITS_PER_SAMPLE: u16 = 0x0102;
pub const PRV_STRIP_OFFSETS: u16 = 0x0111;
pub const PRV_SAMPLES_PER_PIXEL: u16 = 0x0115;
pub const PRV_ROWS_PER_STRIP: u16 = 0x0116;
pub const PRV_STRIP_BYTE_COUNTS: u16 = 0x0117;
pub const PRV_JPEG_INTERCHANGE_FORMAT: u16 = 0x0201;
pub const PRV_JPEG_INTERCHANGE_FORMAT_LENGTH: u16 = 0x0202;
pub const PRV_YCB_CR_POSITIONING: u16 = 0x0213;

// Additional main IFD tags (0x00xx range)
pub const NIKON_AE_BRACKET_COMP: u16 = 0x0026;
pub const NIKON_EXPOSURE_SEQUENCE_NUMBER: u16 = 0x0027;
pub const NIKON_COLOR_BALANCE_A2: u16 = 0x0028;
pub const NIKON_SILHOUETTE: u16 = 0x0029;
pub const NIKON_FOCUS_SHIFT: u16 = 0x002C;
pub const NIKON_POWER_UP_TIME: u16 = 0x002D;
pub const NIKON_AF_INFO_3: u16 = 0x002E;
pub const NIKON_FLASH_INFO_2: u16 = 0x002F;
pub const NIKON_COLOR_TEMP_AUTO: u16 = 0x0035;
pub const NIKON_ACTIVE_D_LIGHTING_2: u16 = 0x0039;

// Additional ShotInfo extended tags (0x005x-0x00Ax)
pub const SI_AF_AREA_MODE_4: u16 = 0x0050;
pub const SI_VIBRATION_REDUCTION_2: u16 = 0x0051;
pub const SI_CONTRAST: u16 = 0x0052;
pub const SI_SATURATION: u16 = 0x0053;
pub const SI_HUE: u16 = 0x0054;
pub const SI_SHARPNESS: u16 = 0x0055;
pub const SI_BRIGHTNESS: u16 = 0x0056;
pub const SI_NOISE_REDUCTION: u16 = 0x0057;
pub const SI_ISO_SENSITIVITY: u16 = 0x0058;
pub const SI_EXPOSURE_COMP: u16 = 0x0059;
pub const SI_WB_FINE_TUNE_VALUE: u16 = 0x005A;
pub const SI_WB_PRESET: u16 = 0x005B;
pub const SI_COLOR_SPACE: u16 = 0x005C;
pub const SI_AF_LOCK_MODE: u16 = 0x005D;
pub const SI_FLASH_SYNC_SPEED: u16 = 0x005E;
pub const SI_AE_LOCK_MODE: u16 = 0x005F;

// Additional PictureControl extended tags
pub const PC_VERSION_2: u16 = 0x003E;
pub const PC_ORIGINAL: u16 = 0x003F;
pub const PC_MODIFIED: u16 = 0x0040;
pub const PC_A_VALUE: u16 = 0x0041;
pub const PC_B_VALUE: u16 = 0x0042;

// Additional LensData extended tags
pub const LD_AF_STATUS: u16 = 0x0024;
pub const LD_AF_TRACKING_STATUS: u16 = 0x0025;
pub const LD_AF_RESULT: u16 = 0x0026;
pub const LD_FOCUS_RESULT: u16 = 0x0027;
pub const LD_DRIVE_MOTOR: u16 = 0x0028;
pub const LD_VR_DATA: u16 = 0x0029;

// Additional FlashInfo extended tags
pub const FI_HIGH_SPEED_SYNC: u16 = 0x0021;
pub const FI_FLASH_EXPOSURE_LOCK: u16 = 0x0022;
pub const FI_FLASH_EXPOSURE_DELTA: u16 = 0x0023;
pub const FI_BUILT_IN_FLASH_VALUE: u16 = 0x0024;
pub const FI_EXTERNAL_FLASH_VALUE: u16 = 0x0025;
pub const FI_FLASH_ISO: u16 = 0x0026;

// Additional AFInfo2 extended tags
pub const AF2_AF_FINE_TUNE_ADJ: u16 = 0x0026;
pub const AF2_AF_FINE_TUNE_INDEX_2: u16 = 0x0028;
pub const AF2_FOCUS_MODE: u16 = 0x002A;
pub const AF2_FOCUS_STATUS: u16 = 0x002C;
pub const AF2_AF_MICRO_ADJ_MODE: u16 = 0x002E;
pub const AF2_AF_MICRO_ADJ_VALUE: u16 = 0x0030;

// Additional MultiExposure tags
pub const ME_OVERLAY_MODE: u16 = 0x0007;
pub const ME_AUTO_GAIN_CONTROL: u16 = 0x0008;
pub const ME_OVERLAY_SHOT: u16 = 0x0009;

// Additional HDRInfo tags
pub const HDR_EXPOSURE_DIFF: u16 = 0x000A;
pub const HDR_SHOT_INFO: u16 = 0x000B;
pub const HDR_EXTRA_SHOTS: u16 = 0x000C;

// Additional AFTune tags
pub const AFT_SAVED_VALUE: u16 = 0x0003;
pub const AFT_AF_FINE_TUNE_DISABLE: u16 = 0x0004;

// Additional ShotInfo D5/D500/D850 series tags
pub const SI_EXP_DELAY_MODE: u16 = 0x0060;
pub const SI_SHUTTER_DELAY: u16 = 0x0061;
pub const SI_EXPOSURE_TIME: u16 = 0x0062;
pub const SI_FOCUS_DISTANCE_2: u16 = 0x0063;
pub const SI_AF_POINT_SELECTION: u16 = 0x0064;
pub const SI_HIGHLIGHT_PROTECTION: u16 = 0x0065;
pub const SI_FOCUS_TRACKING_LOCK_ON: u16 = 0x0066;
pub const SI_AUTO_ISO_CENTER_PRIORITY: u16 = 0x0067;

// Capture NX/NX2 extended tags
pub const NIKON_NX_TONE_COMP: u16 = 0x0E0F;
pub const NIKON_NX_COLOR_BALANCE: u16 = 0x0E17;
pub const NIKON_NX_NOISE_REDUCTION: u16 = 0x0E18;
pub const NIKON_NX_ACTIVE_D_LIGHTING: u16 = 0x0E19;
pub const NIKON_NX_CAPTURE_LENS: u16 = 0x0E1A;
pub const NIKON_NX_CAPTURE_CAMERA: u16 = 0x0E1B;
pub const NIKON_NX_CAPTURE_SERIAL: u16 = 0x0E1C;

// Additional Z-series mirrorless tags
pub const NIKON_SUBJECT_DETECTION: u16 = 0x00C0;
pub const NIKON_FOCUS_PEAKING: u16 = 0x00C1;
pub const NIKON_AF_C_PRIORITY: u16 = 0x00C2;
pub const NIKON_VIEWFINDER_WARNING: u16 = 0x00C4;
pub const NIKON_EYE_DETECTION: u16 = 0x00C5;

/// Format a rational value like ExifTool does:
/// - For whole numbers: "1" instead of "1.000000"
/// - For fractions: show enough precision (up to 8 decimal places, no trailing zeros)
fn format_rational_like_exiftool(n: u32, d: u32) -> String {
    if d == 0 {
        return n.to_string();
    }
    let value = n as f64 / d as f64;
    // Check if it's a whole number
    if (value.fract().abs() < 1e-10) || (value - value.round()).abs() < 1e-10 {
        return format!("{}", value.round() as i64);
    }
    // Format with 9 decimal places and strip trailing zeros
    let formatted = format!("{:.9}", value);
    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
    trimmed.to_string()
}

/// Get the name of a Nikon MakerNote tag
pub fn get_nikon_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        NIKON_VERSION => Some("MakerNoteVersion"),
        NIKON_ISO_SETTING => Some("ISO"),
        NIKON_COLOR_MODE => Some("ColorMode"),
        NIKON_QUALITY => Some("Quality"),
        NIKON_WHITE_BALANCE => Some("WhiteBalance"),
        NIKON_WB_RB_LEVELS => Some("WB_RBLevels"),
        NIKON_SHARPNESS => Some("Sharpness"),
        NIKON_FOCUS_MODE => Some("FocusMode"),
        NIKON_FLASH_SETTING => Some("FlashSetting"),
        NIKON_FLASH_TYPE => Some("FlashType"),
        NIKON_SERIAL_NUMBER => Some("SerialNumber"),
        NIKON_COLOR_SPACE => Some("ColorSpace"),
        NIKON_VR_INFO => Some("VRInfo"),
        NIKON_BLACK_LEVEL => Some("BlackLevel"),
        NIKON_ACTIVE_D_LIGHTING => Some("ActiveD-Lighting"),
        NIKON_PICTURE_CONTROL_DATA => Some("PictureControlData"),
        NIKON_PICTURE_CONTROL_DATA_2 => Some("PictureControlData"),
        NIKON_VIGNETTE_CONTROL => Some("VignetteControl"),
        NIKON_DISTORTION_CONTROL => Some("DistortionControl"),
        NIKON_AF_TUNE => Some("AFTune"),
        NIKON_POWER_UP_TIME_2 => Some("PowerUpTime"),
        NIKON_AUXILIARY_LENS => Some("AuxiliaryLens"),
        NIKON_LENS_TYPE => Some("LensType"),
        NIKON_LENS => Some("Lens"),
        NIKON_MANUAL_FOCUS_DISTANCE => Some("ManualFocusDistance"),
        NIKON_DIGITAL_ZOOM => Some("DigitalZoom"),
        NIKON_FLASH_MODE => Some("FlashMode"),
        NIKON_AF_INFO => Some("AFInfo"),
        NIKON_SHOT_INFO => Some("ShotInfo"),
        NIKON_HUE_ADJUSTMENT => Some("HueAdjustment"),
        NIKON_NEF_COMPRESSION => Some("NEFCompression"),
        NIKON_SATURATION => Some("SaturationAdj"),
        NIKON_NOISE_REDUCTION => Some("NoiseReduction"),
        NIKON_COLOR_BALANCE => Some("ColorBalance"),
        NIKON_LENS_DATA => Some("LensData"),
        NIKON_RAW_IMAGE_CENTER => Some("RawImageCenter"),
        NIKON_SENSOR_PIXEL_SIZE => Some("SensorPixelSize"),
        NIKON_FLASH_INFO => Some("FlashInfo"),
        NIKON_HIGH_ISO_NOISE_REDUCTION => Some("HighISONoiseReduction"),
        NIKON_DATE_STAMP_MODE => Some("DateStampMode"),
        NIKON_RETOUCH_HISTORY => Some("RetouchHistory"),
        NIKON_LIGHT_SOURCE => Some("LightSource"),
        NIKON_SHOOTING_MODE => Some("ShootingMode"),
        NIKON_AUTO_BRACKET_RELEASE => Some("AutoBracketRelease"),
        NIKON_LENS_F_STOPS => Some("LensFStops"),
        NIKON_CONTRAST_CURVE => Some("ContrastCurve"),
        NIKON_COLOR_HUE => Some("ColorHue"),
        NIKON_SCENE_MODE => Some("SceneMode"),
        NIKON_IMAGE_ADJUSTMENT => Some("ImageAdjustment"),
        NIKON_TONE_COMP => Some("ToneComp"),
        NIKON_SERIAL_NUMBER_2 => Some("SerialNumber"),
        NIKON_IMAGE_DATA_SIZE => Some("ImageDataSize"),
        NIKON_IMAGE_COUNT => Some("ImageCount"),
        NIKON_DELETED_IMAGE_COUNT => Some("DeletedImageCount"),
        NIKON_SHUTTER_COUNT => Some("ShutterCount"),
        NIKON_IMAGE_OPTIMIZATION => Some("ImageOptimization"),
        NIKON_SATURATION_2 => Some("Saturation"),
        NIKON_VARI_PROGRAM => Some("VariProgram"),
        NIKON_IMAGE_STABILIZATION => Some("ImageStabilization"),
        NIKON_AF_RESPONSE => Some("AFResponse"),
        NIKON_SILENT_PHOTOGRAPHY => Some("SilentPhotography"),
        NIKON_WHITE_BALANCE_FINE => Some("WhiteBalanceFineTune"),
        NIKON_PROGRAM_SHIFT => Some("ProgramShift"),
        NIKON_EXPOSURE_DIFFERENCE => Some("ExposureDifference"),
        NIKON_ISO_SELECTION => Some("ISOSelection"),
        NIKON_FLASH_EXPOSURE_COMP => Some("FlashExposureComp"),
        NIKON_ISO_SETTING_2 => Some("ISOSetting"),
        NIKON_COLOR_BALANCE_A => Some("NRWData"),
        NIKON_IMAGE_BOUNDARY => Some("ImageBoundary"),
        NIKON_EXTERNAL_FLASH_EXPOSURE_COMP => Some("ExternalFlashExposureComp"),
        NIKON_FLASH_EXPOSURE_BRACKET_VALUE => Some("FlashExposureBracketValue"),
        NIKON_EXPOSURE_BRACKET_VALUE => Some("ExposureBracketValue"),
        NIKON_CROP_HI_SPEED => Some("CropHiSpeed"),
        NIKON_EXPOSURE_TUNING => Some("ExposureTuning"),
        NIKON_IMAGE_AUTHENTICATION => Some("ImageAuthentication"),
        NIKON_FACE_DETECT => Some("FaceDetect"),
        NIKON_WORLD_TIME => Some("WorldTime"),
        NIKON_ISO_INFO => Some("ISOInfo"),
        NIKON_SHUTTER_MODE => Some("ShutterMode"),
        NIKON_HDR_INFO => Some("HDRInfo"),
        NIKON_MECHANICAL_SHUTTER_COUNT => Some("MechanicalShutterCount"),
        NIKON_NEF_BIT_DEPTH => Some("NEFBitDepth"),
        NIKON_IMAGE_PROCESSING => Some("ImageProcessing"),
        NIKON_MULTI_EXPOSURE => Some("MultiExposure"),
        NIKON_AF_INFO_2 => Some("AFInfo2"),
        NIKON_FILE_INFO => Some("FileInfo"),
        NIKON_FLASH_INFO_2 => Some("FlashInfo"),
        NIKON_SCENE_ASSIST => Some("SceneAssist"),
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
        "76 58 50 50 14 14 7A 02" => Some("AF Nikkor 50mm f/1.8D"),
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
        "A0 54 50 50 0C 0C A2 06" => Some("AF-S Nikkor 50mm f/1.4G"),
        "A1 40 18 37 2C 34 BB 06" => Some("AF-S DX Nikkor 10-24mm f/3.5-4.5G ED"),
        "A2 48 5C 80 24 24 BC 0E" => Some("AF-S Nikkor 70-200mm f/2.8G ED VR II"),
        "A3 3C 29 44 30 30 BD 0E" => Some("AF-S Nikkor 16-35mm f/4G ED VR"),
        "A4 54 37 37 0C 0C BE 06" => Some("AF-S Nikkor 24mm f/1.4G ED"),
        "A5 40 3C 8E 2C 3C BF 0E" => Some("AF-S Nikkor 28-300mm f/3.5-5.6G ED VR"),
        "A6 48 8E 8E 24 24 C0 0E" => Some("AF-S Nikkor 300mm f/2.8G ED VR II"),
        "A8 48 8E 8E 30 30 C3 4E" => Some("AF-S Nikkor 300mm f/4E PF ED VR"),
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
        // Additional DX lenses not in above list
        "7F 40 2D 5C 2C 34 84 06" => Some("AF-S DX Zoom-Nikkor 18-70mm f/3.5-4.5G IF-ED"),
        "7D 48 2B 53 24 24 82 06" => Some("AF-S DX Zoom-Nikkor 17-55mm f/2.8G IF-ED"),
        "8B 40 2D 80 2C 3C 8D 0E" => Some("AF-S DX VR Zoom-Nikkor 18-200mm f/3.5-5.6G IF-ED"),
        "8B 40 2D 80 2C 3C FD 0E" => Some("AF-S DX VR Zoom-Nikkor 18-200mm f/3.5-5.6G IF-ED [II]"),
        "A0 40 2D 74 2C 3C BB 0E" => Some("AF-S DX Nikkor 18-140mm f/3.5-5.6G ED VR"),
        "8D 44 5C 8E 34 3C 8F 0E" => Some("AF-S VR Zoom-Nikkor 70-300mm f/4.5-5.6G IF-ED"),
        "93 48 37 5C 24 24 95 06" => Some("AF-S Zoom-Nikkor 24-70mm f/2.8G ED"),
        "9A 40 2D 53 2C 3C 9C 0E" => Some("AF-S DX VR Zoom-Nikkor 18-55mm f/3.5-5.6G"),
        "94 40 2D 53 2C 3C 96 06" => Some("AF-S DX Zoom-Nikkor 18-55mm f/3.5-5.6G ED II"),
        "9E 40 2D 6A 2C 3C A0 0E" => Some("AF-S DX VR Zoom-Nikkor 18-105mm f/3.5-5.6G ED"),
        "A2 40 2D 53 2C 3C BD 0E" => Some("AF-S DX Nikkor 18-55mm f/3.5-5.6G VR II"),
        "B0 4C 50 50 14 14 B2 06" => Some("AF-S Nikkor 50mm f/1.8G"),
        "B4 40 37 62 2C 34 B6 0E" => Some("AF-S Zoom-Nikkor 24-85mm f/3.5-4.5G IF-ED VR"),
        // Additional Nikon lenses
        "69 48 5C 8E 30 3C 6F 06" => Some("AF Zoom-Nikkor 70-300mm f/4-5.6G"),
        "9F 58 44 44 14 14 A1 06" => Some("AF-S DX Nikkor 35mm f/1.8G"),
        "92 48 24 37 24 24 94 06" => Some("AF-S Zoom-Nikkor 14-24mm f/2.8G ED"),
        "A0 40 2D 53 2C 3C CA 8E" => Some("AF-P DX Nikkor 18-55mm f/3.5-5.6G"),
        "A5 54 6A 6A 0C 0C D0 46" => Some("AF-S Nikkor 105mm f/1.4E ED"),
        "A9 4C 31 31 14 14 C4 06" => Some("AF-S Nikkor 20mm f/1.8G ED"),
        "99 40 29 62 2C 3C 9B 0E" => Some("AF-S DX VR Zoom-Nikkor 16-85mm f/3.5-5.6G ED"),
        "8F 40 2D 72 2C 3C 91 06" => Some("AF-S DX Zoom-Nikkor 18-135mm f/3.5-5.6G IF-ED"),
        "8A 54 6A 6A 24 24 8C 0E" => Some("AF-S VR Micro-Nikkor 105mm f/2.8G IF-ED"),
        "7A 3C 1F 37 30 30 7E 06" => Some("AF-S DX Zoom-Nikkor 12-24mm f/4G IF-ED"),
        // Third-party lenses
        "26 40 2D 50 2C 3C 1C 06" => Some("Sigma 18-50mm F3.5-5.6 DC"),
        "26 40 2D 70 2B 3C 1C 06" => Some("Sigma 18-125mm F3.5-5.6 DC"),
        "A1 41 19 31 2C 2C 4B 06" => Some("Sigma 10-20mm F3.5 EX DC HSM"),
        "91 54 44 44 0C 0C 4B 06" => Some("Sigma 35mm F1.4 DG HSM | A"),
        "87 2C 2D 8E 2C 40 4B 0E" => Some("Sigma 18-300mm F3.5-6.3 DC Macro HSM"),
        "92 35 2D 88 2C 40 4B 0E" => Some("Sigma 18-250mm F3.5-6.3 DC Macro OS HSM"),
        "FA 54 3C 5E 24 24 DF 06" => {
            Some("Tamron SP AF 28-75mm f/2.8 XR Di LD Aspherical (IF) Macro (A09NII)")
        }
        "FE 48 37 5C 24 24 DF 0E" => Some("Tamron SP 24-70mm f/2.8 Di VC USD (A007)"),
        "E8 4C 44 44 14 14 DF 0E" => Some("Tamron SP 35mm f/1.8 Di VC USD (F012)"),
        "00 40 2D 88 2C 40 62 06" => {
            Some("Tamron AF 18-250mm f/3.5-6.3 Di II LD Aspherical (IF) Macro (A18)")
        }
        "8F 48 2B 50 24 24 4B 0E" => Some("Sigma 17-50mm F2.8 EX DC OS HSM"),
        "7A 48 1C 30 24 24 7E 06" => Some("Tokina AT-X 11-20 F2.8 PRO DX (AF 11-20mm f/2.8)"),
        // Manual/no CPU lenses
        "00 00 00 00 00 00 00 01" => Some("Manual Lens No CPU"),
        // Z-mount lenses (Nikon Z)
        "01 00 00 00 00 00 00 00" => Some("Nikon Z Lens"),
        _ => None,
    }
}

/// Decode Nikon ASCII tag values to human-readable strings
fn decode_nikon_ascii_value(tag_id: u16, value: &str) -> String {
    match tag_id {
        NIKON_QUALITY => decode_quality_exiftool(value).to_string(),
        NIKON_WHITE_BALANCE => {
            let trimmed = value.trim();
            match trimmed {
                "AUTO" => "Auto".to_string(),
                "AUTO0" => "Auto0".to_string(),
                "AUTO1" => "Auto1".to_string(),
                "AUTO2" => "Auto2".to_string(),
                "SUNNY" => "Sunny".to_string(),
                "DIRECT SUNLIGHT" => "Direct Sunlight".to_string(),
                "SHADE" => "Shade".to_string(),
                "CLOUDY" => "Cloudy".to_string(),
                "TUNGSTEN" | "INCANDESCENT" => "Incandescent".to_string(),
                "FLUORESCENT" => "Fluorescent".to_string(),
                "FLASH" => "Flash".to_string(),
                "PRESET" => "Preset".to_string(),
                _ if trimmed.starts_with("PRESET") => {
                    // Handle PRESET0, PRESET1, etc. -> Preset0, Preset1, etc.
                    format!("Preset{}", &trimmed[6..])
                }
                _ => trimmed.to_string(),
            }
        }
        NIKON_FOCUS_MODE => decode_focus_mode_exiftool(value).to_string(),
        NIKON_FLASH_SETTING => decode_flash_setting_exiftool(value).to_string(),
        NIKON_SHARPNESS => decode_sharpening_exiftool(value).to_string(),
        NIKON_TONE_COMP => decode_tone_comp_exiftool(value).to_string(),
        NIKON_COLOR_MODE => decode_color_mode_exiftool(value),
        NIKON_FLASH_TYPE => decode_flash_type_exiftool(value),
        NIKON_NOISE_REDUCTION => decode_noise_reduction_exiftool(value),
        NIKON_IMAGE_PROCESSING => decode_image_processing_exiftool(value),
        NIKON_LIGHT_SOURCE => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                String::new() // ExifTool outputs empty string for empty LightSource
            } else {
                match trimmed {
                    "NATURAL" => "Natural".to_string(),
                    "SPEEDLIGHT" => "Speedlight".to_string(),
                    "COLORED" => "Colored".to_string(),
                    "MIXED" => "Mixed".to_string(),
                    _ => trimmed.to_string(),
                }
            }
        }
        NIKON_COLOR_HUE => {
            // Transform MODE1 -> Mode1, MODE3a -> Mode3a, etc.
            let trimmed = value.trim();
            if let Some(suffix) = trimmed.strip_prefix("MODE") {
                format!("Mode{}", suffix)
            } else {
                trimmed.to_string()
            }
        }
        NIKON_ISO_SELECTION => decode_iso_selection_exiftool(value).to_string(),
        NIKON_IMAGE_STABILIZATION => decode_image_stabilization_exiftool(value).to_string(),
        NIKON_AUXILIARY_LENS => {
            // Transform OFF -> Off
            let trimmed = value.trim();
            match trimmed {
                "OFF" => "Off".to_string(),
                "TC-14E" | "TC-14E II" | "TC-17E II" | "TC-20E" | "TC-20E II" => {
                    trimmed.to_string()
                }
                _ => trimmed.to_string(),
            }
        }
        NIKON_IMAGE_OPTIMIZATION => {
            // Transform NORMAL -> Normal, etc.
            let trimmed = value.trim();
            match trimmed {
                "NORMAL" => "Normal".to_string(),
                "VIVID" => "Vivid".to_string(),
                "SHARPER" => "Sharper".to_string(),
                "SOFTER" => "Softer".to_string(),
                "DIRECT PRINT" => "Direct Print".to_string(),
                "PORTRAIT" => "Portrait".to_string(),
                "LANDSCAPE" => "Landscape".to_string(),
                "CUSTOM" => "Custom".to_string(),
                "B & W" => "B & W".to_string(),
                _ => trimmed.to_string(),
            }
        }
        NIKON_VARI_PROGRAM => {
            // Transform AUTO -> Auto, SPORT -> Sport, etc.
            let trimmed = value.trim();
            match trimmed {
                "AUTO" => "Auto".to_string(),
                "AUTO(FLASH OFF)" => "Auto(Flash Off)".to_string(),
                "SCENE AUTO" => "Scene Auto".to_string(),
                "SPORT" => "Sport".to_string(),
                "PORTRAIT" => "Portrait".to_string(),
                "LANDSCAPE" => "Landscape".to_string(),
                "CHILD" => "Child".to_string(),
                "NIGHT PORTRAIT" => "Night Portrait".to_string(),
                "PARTY/INDOOR" => "Party/Indoor".to_string(),
                "BEACH/SNOW" => "Beach/Snow".to_string(),
                "SUNSET" => "Sunset".to_string(),
                "DUSK/DAWN" => "Dusk/Dawn".to_string(),
                "PET PORTRAIT" => "Pet Portrait".to_string(),
                "CANDLELIGHT" => "Candlelight".to_string(),
                "BLOSSOM" => "Blossom".to_string(),
                "AUTUMN COLORS" => "Autumn Colors".to_string(),
                "FOOD" => "Food".to_string(),
                "SILHOUETTE" => "Silhouette".to_string(),
                "HIGH KEY" => "High Key".to_string(),
                "LOW KEY" => "Low Key".to_string(),
                "" => "".to_string(),
                _ => trimmed.to_string(),
            }
        }
        NIKON_SCENE_MODE => {
            // Transform AUTO -> Auto, etc.
            let trimmed = value.trim();
            match trimmed {
                "AUTO" => "Auto".to_string(),
                "SCENE AUTO" => "Scene Auto".to_string(),
                "PORTRAIT" => "Portrait".to_string(),
                "LANDSCAPE" => "Landscape".to_string(),
                "SPORT" => "Sport".to_string(),
                "CLOSE UP" | "CLOSEUP" => "Close Up".to_string(),
                "NIGHT PORTRAIT" => "Night Portrait".to_string(),
                "PARTY/INDOOR" => "Party/Indoor".to_string(),
                "BEACH/SNOW" => "Beach/Snow".to_string(),
                "SUNSET" => "Sunset".to_string(),
                "DUSK/DAWN" => "Dusk/Dawn".to_string(),
                "PET PORTRAIT" => "Pet Portrait".to_string(),
                "CANDLELIGHT" => "Candlelight".to_string(),
                "FOOD" => "Food".to_string(),
                "" => "".to_string(),
                _ => trimmed.to_string(),
            }
        }
        NIKON_SERIAL_NUMBER | NIKON_SERIAL_NUMBER_2 => {
            // Transform "NO= " -> "No= " to match ExifTool
            let trimmed = value.trim();
            if let Some(rest) = trimmed.strip_prefix("NO= ") {
                format!("No= {}", rest)
            } else {
                trimmed.to_string()
            }
        }
        NIKON_AF_RESPONSE => {
            // Transform "STANDARD" -> "Standard", "FOCUS" -> "Focus", etc.
            let trimmed = value.trim();
            match trimmed {
                "STANDARD" => "Standard".to_string(),
                "FOCUS" => "Focus".to_string(),
                "LOCK" => "Lock".to_string(),
                "RELEASE" => "Release".to_string(),
                _ => trimmed.to_string(),
            }
        }
        _ => value.trim().to_string(),
    }
}

// Active D-Lighting (tag 0x0022): Nikon.pm / nikonmn_int.cpp nikonActiveDLighting[]
define_tag_decoder! {
    active_d_lighting,
    exiftool: {
        0 => "Off",
        1 => "Low",
        3 => "Normal",
        5 => "High",
        7 => "Extra High",
        8 => "Extra High 1",
        9 => "Extra High 2",
        65535 => "Auto",
    },
    exiv2: {
        0 => "Off",
        1 => "Low",
        3 => "Normal",
        5 => "High",
        7 => "Extra High",
        65535 => "Auto",
    }
}

// JPG Compression: Nikon.pm
define_tag_decoder! {
    jpg_compression,
    both: {
        1 => "Size Priority",
        3 => "Optimal Quality",
    }
}

// VRType: Nikon.pm VRInfo subdirectory
define_tag_decoder! {
    vr_type,
    type: u8,
    both: {
        2 => "In-body",
        3 => "In-body + Lens",
    }
}

// DaylightSavings: Nikon.pm WorldTime / nikonmn_int.cpp nikonYesNo[]
define_tag_decoder! {
    daylight_savings,
    type: u8,
    both: {
        0 => "No",
        1 => "Yes",
    }
}

/// Decode Image Stabilization ASCII value - ExifTool format
/// Tag 0x00ac is ASCII string type, this decodes common string values
/// From Nikon.pm - tag 0x00ac is just stored as string, but some cameras
/// store numeric-like strings that can be decoded
pub fn decode_image_stabilization_exiftool(value: &str) -> &'static str {
    match value.trim() {
        "0" | "OFF" => "Off",
        "1" | "ON" => "On",
        "2" | "ON (2)" => "On (2)",
        "3" | "ON (3)" => "On (3)",
        "4" | "ON (4)" => "On (4)",
        "VR-ON" => "VR-On",
        "VR-OFF" => "VR-Off",
        "VR ON" => "VR On",
        "VR OFF" => "VR Off",
        "VR-On" => "VR-On",
        "VR-Off" => "VR-Off",
        _ => "Unknown",
    }
}

/// Decode Image Stabilization ASCII value - exiv2 format
/// exiv2 treats this as plain ASCII string, using ExifTool interpretations
pub fn decode_image_stabilization_exiv2(value: &str) -> &'static str {
    decode_image_stabilization_exiftool(value)
}

// ColorSpace (tag 0x001E): Nikon.pm / nikonmn_int.cpp nikonColorSpace[]
define_tag_decoder! {
    color_space,
    exiftool: {
        1 => "sRGB",
        2 => "Adobe RGB",
        4 => "BT.2100",
    },
    exiv2: {
        1 => "sRGB",
        2 => "Adobe RGB",
    }
}

// VignetteControl (tag 0x002A): Nikon.pm / nikonmn_int.cpp
define_tag_decoder! {
    vignette_control,
    both: {
        0 => "Off",
        1 => "Low",
        3 => "Normal",
        5 => "High",
    }
}

// HighISONoiseReduction (tag 0x00B1): Nikon.pm / nikonmn_int.cpp nikonHighISONoiseReduction[]
define_tag_decoder! {
    high_iso_noise_reduction,
    exiftool: {
        0 => "Off",
        1 => "Minimal",
        2 => "Low",
        3 => "Medium Low",
        4 => "Normal",
        5 => "Medium High",
        6 => "High",
    },
    exiv2: {
        0 => "Off",
        1 => "Minimal",
        2 => "Low",
        4 => "Normal",
        6 => "High",
    }
}

// DateStampMode (tag 0x009D): Nikon.pm
define_tag_decoder! {
    date_stamp_mode,
    both: {
        0 => "Off",
        1 => "Date & Time",
        2 => "Date",
        3 => "Date Counter",
    }
}

// NEFCompression (tag 0x0093): Nikon.pm
define_tag_decoder! {
    nef_compression,
    both: {
        1 => "Lossy (type 1)",
        2 => "Uncompressed",
        3 => "Lossless",
        4 => "Lossy (type 2)",
        5 => "Striped packed 12 bits",
        6 => "Uncompressed (reduced to 12 bit)",
        7 => "Unpacked 12 bits",
        8 => "Small",
        9 => "Packed 12 bits",
        10 => "Packed 14 bits",
        13 => "High Efficiency",
        14 => "High Efficiency*",
    }
}

// RetouchHistory (tag 0x009E): Nikon.pm
define_tag_decoder! {
    retouch_history,
    both: {
        0 => "None",
        3 => "B & W",
        4 => "Sepia",
        5 => "Trim",
        6 => "Small Picture",
        7 => "D-Lighting",
        8 => "Red Eye",
        9 => "Cyanotype",
        10 => "Sky Light",
        11 => "Warm Tone",
        12 => "Color Custom",
        13 => "Image Overlay",
    }
}

/// Decode packed rational value (4 bytes: a, b, c, _)
/// For signed values: result = a * (b / c) where a, b, c are signed chars
/// Used for ProgramShift, ExposureDifference, FlashExposureComp
pub fn decode_packed_rational_signed(data: &[u8]) -> Option<f64> {
    if data.len() < 4 {
        return None;
    }
    let a = data[0] as i8 as f64;
    let b = data[1] as i8 as f64;
    let c = data[2] as i8 as f64;
    if c == 0.0 {
        Some(0.0)
    } else {
        Some(a * (b / c))
    }
}

/// Decode packed rational value (4 bytes: a, b, c, _)
/// For unsigned values: result = a * (b / c) where a, b, c are unsigned chars
/// Used for LensFStops
pub fn decode_packed_rational_unsigned(data: &[u8]) -> Option<f64> {
    if data.len() < 4 {
        return None;
    }
    let a = data[0] as f64;
    let b = data[1] as f64;
    let c = data[2] as f64;
    if c == 0.0 {
        Some(0.0)
    } else {
        Some(a * (b / c))
    }
}

// FlashMode (tag 0x0087): Nikon.pm
// Note: For unknown values, ExifTool returns "Unknown (X)" format, so we use a custom function
// instead of the macro to preserve the numeric value in the output
pub fn decode_flash_mode_exiftool(value: u8) -> String {
    match value {
        0 => "Did Not Fire".to_string(),
        1 => "Fired, Manual".to_string(),
        3 => "Not Ready".to_string(),
        7 => "Fired, External".to_string(),
        8 => "Fired, Commander Mode".to_string(),
        9 => "Fired, TTL Mode".to_string(),
        18 => "LED Light".to_string(),
        _ => format!("Unknown ({})", value),
    }
}

pub fn decode_flash_mode_exiv2(value: u8) -> String {
    decode_flash_mode_exiftool(value) // Same mapping
}

/// Decode Shooting Mode bitmask (tag 0x0089) - ExifTool format
/// This is a bitmask with special handling for single-frame mode
pub fn decode_shooting_mode_exiftool(value: u16) -> String {
    // Check for single-frame mode (no bits 0,1,2,7 set)
    if value & 0x87 == 0 {
        if value == 0 {
            return "Single-Frame".to_string();
        }
        // Has other bits but not continuous mode bits
        let mut result = "Single-Frame".to_string();
        let decoded_bits = decode_shooting_mode_bits(value, false);
        if !decoded_bits.is_empty() {
            result.push_str(", ");
            result.push_str(&decoded_bits);
        }
        return result;
    }

    decode_shooting_mode_bits(value, false)
}

/// Decode Shooting Mode bitmask (tag 0x0089) - exiv2 format
/// exiv2 decodes all bits without special single-frame handling
pub fn decode_shooting_mode_exiv2(value: u16) -> String {
    decode_shooting_mode_bits(value, true)
}

/// Helper to decode shooting mode bits
fn decode_shooting_mode_bits(value: u16, _is_exiv2: bool) -> String {
    let mut modes: Vec<String> = Vec::new();

    // Known bit mappings from ExifTool Nikon.pm
    let known_bits: [(u16, &str); 10] = [
        (0, "Continuous"),
        (1, "Delay"),
        (2, "PC Control"),
        (3, "Self-timer"),
        (4, "Exposure Bracketing"),
        (5, "Auto ISO"), // Note: For D70, this means "Unused LE-NR Slowdown"
        (6, "White-Balance Bracketing"),
        (7, "IR Control"),
        (8, "D-Lighting Bracketing"),
        (11, "Pre-capture"), // Z9 pre-release burst
    ];

    // Track which bits we've handled
    let mut handled_mask: u16 = 0;

    for (bit, name) in known_bits {
        let mask = 1u16 << bit;
        handled_mask |= mask;
        if value & mask != 0 {
            modes.push(name.to_string());
        }
    }

    // Output unknown bits as "[n]" format (matching ExifTool's DecodeBits)
    for bit in 0..16 {
        let mask = 1u16 << bit;
        if (handled_mask & mask) == 0 && (value & mask) != 0 {
            modes.push(format!("[{}]", bit));
        }
    }

    // Note: "Single-Frame" prefix logic is handled by decode_shooting_mode_exiftool
    modes.join(", ")
}

// AutoBracketRelease (tag 0x008A): Nikon.pm / nikonmn_int.cpp
define_tag_decoder! {
    auto_bracket_release,
    exiftool: {
        0 => "None",
        1 => "Auto Release",
        2 => "Manual Release",
    },
    exiv2: {
        0 => "None",
        1 => "Auto release",
        2 => "Manual release",
    }
}

// ShutterMode (tag 0x0034): Nikon.pm
define_tag_decoder! {
    shutter_mode,
    both: {
        0 => "Mechanical",
        16 => "Electronic",
        33 => "Unknown (33)",
        48 => "Electronic Front Curtain",
        64 => "Electronic (Movie)",
        80 => "Auto (Mechanical)",
        81 => "Auto (Electronic Front Curtain)",
        96 => "Electronic (High Speed)",
    }
}

/// Decode Quality value (tag 0x0004) - ExifTool format
/// From Nikon.pm - tag 0x0004 is ASCII string type
/// Note: ExifTool stores these as-is, we preserve the original for unrecognized values
pub fn decode_quality_exiftool(value: &str) -> String {
    let val = value.trim().to_uppercase();
    let result = match val.as_str() {
        "RAW" => "RAW",
        "FINE" => "Fine",
        "NORMAL" => "Normal",
        "BASIC" => "Basic",
        "RAW+FINE" | "RAW + FINE" => "RAW + Fine",
        "RAW+NORMAL" | "RAW + NORMAL" => "RAW + Normal",
        "RAW+BASIC" | "RAW + BASIC" => "RAW + Basic",
        "NRW" => "NRW",                       // Nikon RAW format for Coolpix
        _ => return value.trim().to_string(), // Return original value for unknown
    };
    result.to_string()
}

/// Decode Quality value (tag 0x0004) - exiv2 format
/// exiv2 treats this as plain ASCII string, using ExifTool interpretations
pub fn decode_quality_exiv2(value: &str) -> String {
    decode_quality_exiftool(value)
}

/// Decode FocusMode value (tag 0x0007) - ExifTool format
/// From Nikon.pm - tag 0x0007 is ASCII string type
pub fn decode_focus_mode_exiftool(value: &str) -> String {
    let val = value.trim();
    let result = match val {
        "AF-S" | "AF-S  " => "AF-S",
        "AF-C" | "AF-C  " => "AF-C",
        "AF-A" | "AF-A  " => "AF-A",
        "MF" | "MANUAL" | "Manual" => "Manual",
        _ => return val.to_string(), // Return original value
    };
    result.to_string()
}

/// Decode FocusMode value (tag 0x0007) - exiv2 format
/// From nikonmn_int.cpp print0x0007
pub fn decode_focus_mode_exiv2(value: &str) -> String {
    let val = value.trim();
    match val {
        "AF-C  " => "Continuous autofocus".to_string(),
        "AF-S  " => "Single autofocus".to_string(),
        "AF-A  " => "Automatic".to_string(),
        _ => decode_focus_mode_exiftool(value),
    }
}

/// Decode FlashSetting/FlashSyncMode value (tag 0x0008) - ExifTool format
/// From Nikon.pm - values: "Normal", "Slow", "Rear", "Rear Slow", "RED-EYE", "RED-EYE SLOW"
pub fn decode_flash_setting_exiftool(value: &str) -> String {
    let val = value.trim().to_uppercase();
    let result = match val.as_str() {
        "NORMAL" => "Normal",
        "SLOW" => "Slow",
        "REAR" => "Rear",
        "REAR SLOW" => "Rear Slow",
        "RED-EYE" | "REDEYE" => "Red-Eye",
        "RED-EYE SLOW" | "REDEYE SLOW" => "Red-Eye Slow",
        "SLOW REAR" => "Slow Rear",
        "" => "",                             // Keep empty as empty to match ExifTool
        _ => return value.trim().to_string(), // Return original value
    };
    result.to_string()
}

/// Decode FlashSetting value (tag 0x0008) - exiv2 format
/// exiv2 treats this as plain ASCII string, using ExifTool interpretations
pub fn decode_flash_setting_exiv2(value: &str) -> String {
    decode_flash_setting_exiftool(value)
}

/// Decode Sharpening/Sharpness value (tag 0x0006) - ExifTool format
/// From Nikon.pm - tag 0x0006 is ASCII string type
pub fn decode_sharpening_exiftool(value: &str) -> String {
    let val = value.trim().to_uppercase();
    let result = match val.as_str() {
        "AUTO" => "Auto",
        "NORMAL" => "Normal",
        "LOW" | "SOFT" => "Soft",
        "MED.L" | "MEDIUM LOW" | "MEDIUMLOW" => "Medium Low",
        "MED.H" | "MEDIUM HIGH" | "MEDIUMHIGH" => "Medium High",
        "HIGH" => "High",
        "HARD" => "Hard",
        "NONE" => "None",
        _ => return value.trim().to_string(), // Return original value
    };
    result.to_string()
}

/// Decode Sharpening value (tag 0x0006) - exiv2 format
/// exiv2 treats this as plain ASCII string, using ExifTool interpretations
pub fn decode_sharpening_exiv2(value: &str) -> String {
    decode_sharpening_exiftool(value)
}

/// Decode ToneComp value (tag 0x0081) - ExifTool format
/// From Nikon.pm - tag 0x0081 is ASCII string type
/// Values: Auto, Normal, Low, Med.L, Med.H, High, Custom
pub fn decode_tone_comp_exiftool(value: &str) -> String {
    let val = value.trim().to_uppercase();
    let result = match val.as_str() {
        "AUTO" => "Auto",
        "NORMAL" => "Normal",
        "LOW" => "Low",
        "MED.L" => "Med.L",
        "MED.H" => "Med.H",
        "HIGH" => "High",
        "CUSTOM" => "Custom",
        "CS" => "Custom",
        _ => return value.trim().to_string(), // Return original value
    };
    result.to_string()
}

/// Decode ToneComp value (tag 0x0081) - exiv2 format
pub fn decode_tone_comp_exiv2(value: &str) -> String {
    decode_tone_comp_exiftool(value)
}

/// Decode Saturation value (tag 0x0094) - ExifTool format
/// From Nikon.pm - tag 0x0094 is signed short, no PrintConv (just raw value)
/// Note: ExifTool displays this as a raw signed integer value
pub fn decode_saturation_exiftool(value: i16) -> String {
    format!("{:+}", value)
}

/// Decode Saturation value (tag 0x0094) - exiv2 format
/// From nikonmn_int.cpp - tag 0x0094 is signed short, no special decoding
pub fn decode_saturation_exiv2(value: i16) -> String {
    decode_saturation_exiftool(value)
}

/// Decode ISOSelection value (tag 0x000F) - ExifTool format
/// From Nikon.pm - tag 0x000F is ASCII string type
pub fn decode_iso_selection_exiftool(value: &str) -> String {
    let val = value.trim().to_uppercase();
    let result = match val.as_str() {
        "AUTO" => "Auto",
        "MANUAL" | "MAN" => "Manual",
        _ => return value.trim().to_string(), // Return original value
    };
    result.to_string()
}

/// Decode ISOSelection value (tag 0x000F) - exiv2 format
/// exiv2 treats this as plain ASCII string, using ExifTool interpretations
pub fn decode_iso_selection_exiv2(value: &str) -> String {
    decode_iso_selection_exiftool(value)
}

// VRMode: Nikon.pm vRModeZ9 / nikonmn_int.cpp VRInfo
define_tag_decoder! {
    vr_mode,
    exiftool: {
        0 => "Off",
        1 => "Normal",
        2 => "Sport",
        3 => "Active",
    },
    exiv2: {
        0 => "Off",
        1 => "Normal",
        2 => "Sport",
    }
}

// AutoDistortionControl: Nikon.pm
define_tag_decoder! {
    auto_distortion_control,
    both: {
        0 => "Off",
        1 => "On",
        2 => "Off",
    }
}

// HDRMode: Nikon.pm multipleExposureModeZ9
define_tag_decoder! {
    hdr_mode,
    both: {
        0 => "Off",
        1 => "On",
        2 => "On (series)",
    }
}

// HDRLevel: Nikon.pm hdrLevelZ8
define_tag_decoder! {
    hdr_level,
    both: {
        0 => "Auto",
        1 => "Extra Low",
        2 => "Low",
        3 => "Normal",
        4 => "High",
        5 => "Extra High",
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

/// Decode ISOExpansion value to string
fn decode_iso_expansion_exiftool(value: u16) -> &'static str {
    match value {
        0x000 => "Off",
        0x101 => "Hi 0.3",
        0x102 => "Hi 0.5",
        0x103 => "Hi 0.7",
        0x104 => "Hi 1.0",
        0x105 => "Hi 1.3",
        0x106 => "Hi 1.5",
        0x107 => "Hi 1.7",
        0x108 => "Hi 2.0",
        0x109 => "Hi 2.3",
        0x10a => "Hi 2.5",
        0x10b => "Hi 2.7",
        0x10c => "Hi 3.0",
        0x10d => "Hi 3.3",
        0x10e => "Hi 3.5",
        0x201 => "Lo 0.3",
        0x202 => "Lo 0.5",
        0x203 => "Lo 0.7",
        0x204 => "Lo 1.0",
        _ => "Unknown",
    }
}

/// Parse ISOInfo tag data (tag 0x0025)
/// BigEndian byte order (forced)
fn parse_iso_info(data: &[u8]) -> Vec<(String, String)> {
    let mut tags = Vec::new();

    if data.len() < 14 {
        return tags;
    }

    // Offset 0x00: ISO (int8u)
    // Note: ISOInfo ISO is always just a number, the Hi/Lo prefix only comes from tag 0x0002
    let iso_raw = data[0];
    if iso_raw != 0 {
        let iso = (100.0 * ((iso_raw as f64 / 12.0 - 5.0) * 2.0_f64.ln()).exp()).round() as u32;
        tags.push(("ISO".to_string(), iso.to_string()));
    }

    // Offset 0x04: ISOExpansion (int16u, BigEndian)
    let iso_expansion = u16::from_be_bytes([data[4], data[5]]);
    // Always output ISOExpansion - 0 = "Off"
    tags.push((
        "ISOExpansion".to_string(),
        decode_iso_expansion_exiftool(iso_expansion).to_string(),
    ));

    // Offset 0x06: ISO2 (int8u)
    let iso2_raw = data[6];
    if iso2_raw != 0 {
        let iso2 = (100.0 * ((iso2_raw as f64 / 12.0 - 5.0) * 2.0_f64.ln()).exp()).round() as u32;
        tags.push(("ISO2".to_string(), iso2.to_string()));
    }

    // Offset 0x0A: ISOExpansion2 (int16u, BigEndian)
    let iso_expansion2 = u16::from_be_bytes([data[10], data[11]]);
    // Always output ISOExpansion2 - 0 = "Off"
    tags.push((
        "ISOExpansion2".to_string(),
        decode_iso_expansion_exiftool(iso_expansion2).to_string(),
    ));

    tags
}

/// Check if model is a Nikon Z-series camera
/// Based on ExifTool's %infoZSeries condition
fn is_nikon_z_series(model: Option<&str>) -> bool {
    match model {
        Some(m) => {
            let upper = m.to_uppercase();
            // Pattern: "NIKON Z " followed by model number (with space)
            // or "NIKON Z" directly followed by model (no space, Oct 2023+)
            upper.contains("NIKON Z 30")
                || upper.contains("NIKON Z 5")
                || upper.contains("NIKON Z 50")
                || upper.contains("NIKON Z 6")
                || upper.contains("NIKON Z 7")
                || upper.contains("NIKON Z 8")
                || upper.contains("NIKON Z 9")
                || upper.contains("NIKON Z F")
                || upper.contains("NIKON Z FC")
                || upper.contains("NIKON Z5_2")
                || upper.contains("NIKON Z50_2")
                || upper.contains("NIKON Z6_2")
                || upper.contains("NIKON Z6_3")
                || upper.contains("NIKON Z7_2")
        }
        None => false,
    }
}

/// Parse VRInfo tag data (tag 0x001F)
fn parse_vr_info(data: &[u8], _endian: Endianness, model: Option<&str>) -> Vec<(String, String)> {
    let mut tags = Vec::new();

    if data.len() < 8 {
        return tags;
    }

    // Offset 0x00: VRInfoVersion (undef[4])
    let version = String::from_utf8_lossy(&data[0..4]).to_string();
    tags.push(("VRInfoVersion".to_string(), version));

    // Offset 0x04: VibrationReduction (int8u)
    let vr = data[4];
    let vr_str = match vr {
        0 => "n/a",
        1 => "On",
        2 => "Off",
        _ => "Unknown",
    };
    tags.push(("VibrationReduction".to_string(), vr_str.to_string()));

    // Offset 0x06: VRMode (int8u)
    // Z-series cameras use different mapping than older cameras
    let vr_mode = data[6];
    let is_z_series = is_nikon_z_series(model);
    let vr_mode_str = if is_z_series {
        // Z-series mapping (vRModeZ9 in ExifTool)
        match vr_mode {
            0 => "Off",
            1 => "Normal",
            2 => "Sport",
            _ => "Unknown",
        }
    } else {
        // Non-Z-series mapping
        match vr_mode {
            0 => "Normal",
            1 => "On (1)",
            2 => "Active",
            3 => "Sport",
            _ => "Unknown",
        }
    };
    tags.push(("VRMode".to_string(), vr_mode_str.to_string()));

    tags
}

/// Parse DistortInfo tag data (tag 0x002B)
fn parse_distort_info(data: &[u8]) -> Vec<(String, String)> {
    let mut tags = Vec::new();

    if data.len() < 5 {
        return tags;
    }

    // Offset 0x00: DistortionVersion (undef[4])
    let version = String::from_utf8_lossy(&data[0..4]).to_string();
    tags.push(("DistortionVersion".to_string(), version));

    // Offset 0x04: AutoDistortionControl (int8u)
    let adc = data[4];
    let adc_str = match adc {
        0 => "Off",
        1 => "On",
        2 => "On (underwater)",
        _ => "Unknown",
    };
    tags.push(("AutoDistortionControl".to_string(), adc_str.to_string()));

    tags
}

/// Parse AFTune tag data (tag 0x00B9)
fn parse_af_tune(data: &[u8]) -> Vec<(String, String)> {
    let mut tags = Vec::new();

    if data.is_empty() {
        return tags;
    }

    // Offset 0x00: AFFineTune (int8u)
    let af_fine_tune = data[0];
    let af_str = match af_fine_tune {
        0 => "Off",
        1 => "On (1)",
        2 => "On (2)",
        3 => "On (Zoom)",
        _ => "Unknown",
    };
    tags.push(("AFFineTune".to_string(), af_str.to_string()));

    // Offset 0x01: AFFineTuneIndex (int8u)
    if data.len() > 1 {
        let index = data[1];
        let index_str = if index == 255 {
            "n/a".to_string()
        } else {
            index.to_string()
        };
        tags.push(("AFFineTuneIndex".to_string(), index_str));
    }

    // Offset 0x02: AFFineTuneAdj (int8s)
    if data.len() > 2 {
        let adj = data[2] as i8;
        let adj_str = if adj > 0 {
            format!("+{}", adj)
        } else {
            adj.to_string()
        };
        tags.push(("AFFineTuneAdj".to_string(), adj_str));
    }

    // Offset 0x03: AFFineTuneAdjTele (int8s)
    if data.len() > 3 {
        let adj_tele = data[3] as i8;
        let adj_str = if adj_tele > 0 {
            format!("+{}", adj_tele)
        } else {
            adj_tele.to_string()
        };
        tags.push(("AFFineTuneAdjTele".to_string(), adj_str));
    }

    tags
}

/// Parse PictureControlData tag data (tag 0x0023)
/// Version-dependent structure
fn parse_picture_control(data: &[u8]) -> Vec<(String, String)> {
    let mut tags = Vec::new();

    if data.len() < 4 {
        return tags;
    }

    // Offset 0x00: Version (undef[4])
    let version = String::from_utf8_lossy(&data[0..4]).to_string();
    tags.push(("PictureControlVersion".to_string(), version.clone()));

    // Version 0100 structure (D300)
    if version == "0100" && data.len() >= 58 {
        // Offset 0x04: PictureControlName (string[20])
        let name_bytes = &data[4..24];
        let name = String::from_utf8_lossy(name_bytes)
            .trim_end_matches('\0')
            .to_string();
        if !name.is_empty() {
            let formatted_name = format_picture_control_string(&name);
            tags.push(("PictureControlName".to_string(), formatted_name));
        }

        // Offset 0x18: PictureControlBase (string[20])
        let base_bytes = &data[24..44];
        let base = String::from_utf8_lossy(base_bytes)
            .trim_end_matches('\0')
            .to_string();
        if !base.is_empty() {
            let formatted_base = format_picture_control_string(&base);
            tags.push(("PictureControlBase".to_string(), formatted_base));
        }

        // Offset 0x30: PictureControlAdjust (int8u)
        if data.len() > 0x30 {
            let adjust = data[0x30];
            let adjust_str = match adjust {
                0 => "Default Settings",
                1 => "Quick Adjust",
                2 => "Full Control",
                _ => "Unknown",
            };
            tags.push(("PictureControlAdjust".to_string(), adjust_str.to_string()));
        }

        // Offset 0x31: PictureControlQuickAdjust (int8u)
        if data.len() > 0x31 {
            let quick = data[0x31];
            if quick == 0xff {
                tags.push(("PictureControlQuickAdjust".to_string(), "n/a".to_string()));
            } else {
                let value = quick.wrapping_sub(128) as i8;
                if value == 0 {
                    tags.push((
                        "PictureControlQuickAdjust".to_string(),
                        "Normal".to_string(),
                    ));
                } else {
                    tags.push(("PictureControlQuickAdjust".to_string(), value.to_string()));
                }
            }
        }

        // Offset 0x32: Sharpness (int8u)
        if data.len() > 0x32 {
            let sharp = data[0x32];
            let value = sharp.wrapping_sub(128) as i8;
            if value == 0 {
                tags.push(("Sharpness".to_string(), "Normal".to_string()));
            } else {
                tags.push(("Sharpness".to_string(), value.to_string()));
            }
        }

        // Offset 0x33: Contrast (int8u)
        if data.len() > 0x33 {
            let contrast = data[0x33];
            let value = contrast.wrapping_sub(128) as i8;
            if value == 0 {
                tags.push(("Contrast".to_string(), "Normal".to_string()));
            } else {
                tags.push(("Contrast".to_string(), value.to_string()));
            }
        }

        // Offset 0x34: Brightness (int8u)
        if data.len() > 0x34 {
            let bright = data[0x34];
            if bright == 0xff {
                tags.push(("Brightness".to_string(), "n/a".to_string()));
            } else {
                let value = bright.wrapping_sub(128) as i8;
                if value == 0 {
                    tags.push(("Brightness".to_string(), "Normal".to_string()));
                } else {
                    tags.push(("Brightness".to_string(), value.to_string()));
                }
            }
        }

        // Offset 0x35: Saturation (int8u)
        if data.len() > 0x35 {
            let sat = data[0x35];
            if sat != 0xff {
                let value = sat.wrapping_sub(128) as i8;
                if value == 0 {
                    tags.push(("Saturation".to_string(), "Normal".to_string()));
                } else {
                    tags.push(("Saturation".to_string(), value.to_string()));
                }
            }
        }

        // Offset 0x36: HueAdjustment (int8u)
        if data.len() > 0x36 {
            let hue = data[0x36];
            if hue != 0xff {
                let value = hue.wrapping_sub(128) as i8;
                if value == 0 {
                    tags.push(("HueAdjustment".to_string(), "None".to_string()));
                } else {
                    tags.push(("HueAdjustment".to_string(), value.to_string()));
                }
            } else {
                tags.push(("HueAdjustment".to_string(), "n/a".to_string()));
            }
        }

        // Offset 0x37: FilterEffect (int8u) - for Monochrome only
        if data.len() > 0x37 {
            let filter = data[0x37];
            let filter_str = match filter {
                0x80 => "Off",
                0x81 => "Yellow",
                0x82 => "Orange",
                0x83 => "Red",
                0x84 => "Green",
                0xff => "n/a",
                _ => "Unknown",
            };
            tags.push(("FilterEffect".to_string(), filter_str.to_string()));
        }

        // Offset 0x38: ToningEffect (int8u) - for Monochrome only
        if data.len() > 0x38 {
            let toning = data[0x38];
            let toning_str = match toning {
                0x80 => "B&W",
                0x81 => "Sepia",
                0x82 => "Cyanotype",
                0x83 => "Red",
                0x84 => "Yellow",
                0x85 => "Green",
                0x86 => "Blue-green",
                0x87 => "Blue",
                0x88 => "Purple-blue",
                0x89 => "Red-purple",
                0xff => "n/a",
                _ => "Unknown",
            };
            tags.push(("ToningEffect".to_string(), toning_str.to_string()));
        }

        // Offset 0x39: ToningSaturation (int8u)
        if data.len() > 0x39 {
            let toning_sat = data[0x39];
            if toning_sat != 0xff {
                let value = toning_sat.wrapping_sub(128) as i8;
                tags.push(("ToningSaturation".to_string(), value.to_string()));
            } else {
                tags.push(("ToningSaturation".to_string(), "n/a".to_string()));
            }
        }
    } else if version.starts_with("02") && data.len() >= 68 {
        // Version 02xx structure (PictureControl2 - D4, D800, D5xxx, etc.)
        // Name and Base at same offsets as 0100
        parse_picture_control_name_base(&mut tags, data, 4, 24);

        // Offset 48: PictureControlAdjust
        if data.len() > 48 {
            let adjust_str = match data[48] {
                0 => "Default Settings",
                1 => "Quick Adjust",
                2 => "Full Control",
                _ => "Unknown",
            };
            tags.push(("PictureControlAdjust".to_string(), adjust_str.to_string()));
        }

        // Offset 49: PictureControlQuickAdjust (uses "Normal" for zero value)
        parse_pc_value(&mut tags, data, 49, "PictureControlQuickAdjust", false);

        // Offset 51: Sharpness (uses "None" for zero, divisor 4, format %.2f)
        parse_pc_value_div(&mut tags, data, 51, "Sharpness", 4.0);

        // Offset 53: Clarity (uses "None" for zero, divisor 4, format %.2f)
        parse_pc_value_div(&mut tags, data, 53, "Clarity", 4.0);

        // Offset 55: Contrast
        parse_pc_value(&mut tags, data, 55, "Contrast", false);

        // Offset 57: Brightness
        parse_pc_value(&mut tags, data, 57, "Brightness", false);

        // Offset 59: Saturation
        parse_pc_value(&mut tags, data, 59, "Saturation", false);

        // Offset 61: Hue
        parse_pc_value(&mut tags, data, 61, "Hue", true);

        // Offset 63: FilterEffect
        if data.len() > 63 {
            let filter_str = match data[63] {
                0x80 => "Off",
                0x81 => "Yellow",
                0x82 => "Orange",
                0x83 => "Red",
                0x84 => "Green",
                0xff => "n/a",
                _ => "Unknown",
            };
            tags.push(("FilterEffect".to_string(), filter_str.to_string()));
        }

        // Offset 64: ToningEffect
        if data.len() > 64 {
            let toning_str = decode_toning_effect(data[64]);
            tags.push(("ToningEffect".to_string(), toning_str.to_string()));
        }

        // Offset 65: ToningSaturation
        parse_pc_value(&mut tags, data, 65, "ToningSaturation", true);
    } else if version.starts_with("03") && data.len() >= 74 {
        // Version 03xx structure (PictureControl3 - Z cameras, etc.)
        // Name at 8, Base at 28 (different from 01xx/02xx!)
        parse_picture_control_name_base(&mut tags, data, 8, 28);

        // Offset 54: PictureControlAdjust
        if data.len() > 54 {
            let adjust_str = match data[54] {
                0 => "Default Settings",
                1 => "Quick Adjust",
                2 => "Full Control",
                _ => "Unknown",
            };
            tags.push(("PictureControlAdjust".to_string(), adjust_str.to_string()));
        }

        // Offset 55: PictureControlQuickAdjust (uses "Normal" for zero value)
        parse_pc_value(&mut tags, data, 55, "PictureControlQuickAdjust", false);

        // Offset 57: Sharpness (uses "None" for zero, divisor 4, format %.2f)
        parse_pc_value_div(&mut tags, data, 57, "Sharpness", 4.0);

        // Offset 59: MidRangeSharpness (uses "None" for zero, divisor 4, format %.2f)
        parse_pc_value_div(&mut tags, data, 59, "MidRangeSharpness", 4.0);

        // Offset 61: Clarity (uses "None" for zero, divisor 4, format %.2f)
        parse_pc_value_div(&mut tags, data, 61, "Clarity", 4.0);

        // Offset 63: Contrast
        parse_pc_value(&mut tags, data, 63, "Contrast", false);

        // Offset 65: Brightness
        parse_pc_value(&mut tags, data, 65, "Brightness", false);

        // Offset 67: Saturation
        parse_pc_value(&mut tags, data, 67, "Saturation", false);

        // Offset 69: Hue
        parse_pc_value(&mut tags, data, 69, "Hue", true);

        // Offset 71: FilterEffect
        if data.len() > 71 {
            let filter_str = match data[71] {
                0x80 => "Off",
                0x81 => "Yellow",
                0x82 => "Orange",
                0x83 => "Red",
                0x84 => "Green",
                0xff => "n/a",
                _ => "Unknown",
            };
            tags.push(("FilterEffect".to_string(), filter_str.to_string()));
        }

        // Offset 73: ToningEffect
        if data.len() > 73 {
            let toning_str = decode_toning_effect(data[73]);
            tags.push(("ToningEffect".to_string(), toning_str.to_string()));
        }
    }

    tags
}

/// Helper to parse PictureControlName and PictureControlBase
fn parse_picture_control_name_base(
    tags: &mut Vec<(String, String)>,
    data: &[u8],
    name_offset: usize,
    base_offset: usize,
) {
    // PictureControlName (string[20])
    if data.len() >= name_offset + 20 {
        let name = String::from_utf8_lossy(&data[name_offset..name_offset + 20])
            .trim_end_matches('\0')
            .to_string();
        if !name.is_empty() {
            tags.push((
                "PictureControlName".to_string(),
                format_picture_control_string(&name),
            ));
        }
    }

    // PictureControlBase (string[20])
    if data.len() >= base_offset + 20 {
        let base = String::from_utf8_lossy(&data[base_offset..base_offset + 20])
            .trim_end_matches('\0')
            .to_string();
        if !base.is_empty() {
            tags.push((
                "PictureControlBase".to_string(),
                format_picture_control_string(&base),
            ));
        }
    }
}

/// Format PictureControl string like ExifTool's FormatString
/// - Words with vowels: title case (Vivid, Standard, etc.)
/// - Words without vowels: keep uppercase (KR, JP, etc.)
fn format_picture_control_string(s: &str) -> String {
    fn has_vowel(s: &str) -> bool {
        s.chars()
            .any(|c| matches!(c.to_ascii_uppercase(), 'A' | 'E' | 'I' | 'O' | 'U' | 'Y'))
    }

    fn format_segment(seg: &str) -> String {
        if seg.is_empty() {
            return String::new();
        }
        if has_vowel(seg) {
            // Title case: first letter uppercase, rest lowercase
            let mut chars = seg.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        } else {
            // No vowels: keep uppercase
            seg.to_uppercase()
        }
    }

    // Split by whitespace first
    s.split_whitespace()
        .map(|word| {
            // Within each word, handle hyphens
            word.split('-')
                .map(format_segment)
                .collect::<Vec<_>>()
                .join("-")
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Helper to parse a PictureControl value field (int8u with 0x80 offset)
/// Special values: 0xff → "n/a", 0x00 → "Auto", 0x80 → "Normal" or "None"
fn parse_pc_value(
    tags: &mut Vec<(String, String)>,
    data: &[u8],
    offset: usize,
    name: &str,
    use_none: bool,
) {
    if data.len() > offset {
        let raw = data[offset];
        // ValueConv: raw - 0x80
        // PrintPC special values: 0x7f (127) = n/a, -128 = Auto, -127 = User, 0 = Normal/$norm
        let formatted = if raw == 0xff {
            // raw 0xff → value 127 → "n/a"
            "n/a".to_string()
        } else if raw == 0x00 {
            // raw 0x00 → value -128 → "Auto"
            "Auto".to_string()
        } else if raw == 0x01 {
            // raw 0x01 → value -127 → "User"
            "User".to_string()
        } else if raw == 0x80 {
            // raw 0x80 → value 0 → "Normal" or "None"
            if use_none {
                "None".to_string()
            } else {
                "Normal".to_string()
            }
        } else {
            // Other values: output as signed integer
            let value = raw.wrapping_sub(128) as i8;
            value.to_string()
        };
        tags.push((name.to_string(), formatted));
    }
}

/// Helper to parse a PictureControl value field with divisor (for Clarity)
/// Uses format "%.2f" and divides by divisor
fn parse_pc_value_div(
    tags: &mut Vec<(String, String)>,
    data: &[u8],
    offset: usize,
    name: &str,
    divisor: f64,
) {
    if data.len() > offset {
        let raw = data[offset];
        // ValueConv: raw - 0x80
        let formatted = if raw == 0xff {
            // raw 0xff → value 127 → "n/a"
            "n/a".to_string()
        } else if raw == 0x00 {
            // raw 0x00 → value -128 → "Auto"
            "Auto".to_string()
        } else if raw == 0x01 {
            // raw 0x01 → value -127 → "User"
            "User".to_string()
        } else if raw == 0x80 {
            // raw 0x80 → value 0 → "None" (Clarity uses "None" for zero)
            "None".to_string()
        } else {
            // Other values: divide by divisor and format as %.2f
            let value = raw.wrapping_sub(128) as i8;
            let divided = f64::from(value) / divisor;
            format!("{:+.2}", divided)
        };
        tags.push((name.to_string(), formatted));
    }
}

/// Decode ToningEffect value
fn decode_toning_effect(value: u8) -> &'static str {
    match value {
        0x80 => "B&W",
        0x81 => "Sepia",
        0x82 => "Cyanotype",
        0x83 => "Red",
        0x84 => "Yellow",
        0x85 => "Green",
        0x86 => "Blue-green",
        0x87 => "Blue",
        0x88 => "Purple-blue",
        0x89 => "Red-purple",
        0xff => "n/a",
        _ => "Unknown",
    }
}

/// Decode FlashControlMode value to string
fn decode_flash_control_mode(value: u8) -> &'static str {
    match value {
        0 => "Off",
        1 => "iTTL-BL",
        2 => "iTTL",
        3 => "Auto Aperture",
        4 => "Automatic",
        5 => "GN (distance priority)",
        6 => "Manual",
        7 => "Repeating Flash",
        _ => "Unknown",
    }
}

/// Format ExternalFlashFirmware with model name lookup
/// From ExifTool Nikon.pm %flashFirmware hash
fn format_flash_firmware(major: u8, minor: u8) -> String {
    let model = match (major, minor) {
        (1, 1) => Some("SB-800 or Metz 58 AF-1"),
        (1, 3) => Some("SB-800"),
        (2, 1) => Some("SB-800"),
        (2, 4) => Some("SB-600"),
        (2, 5) => Some("SB-600"),
        (3, 1) => Some("SU-800 Remote Commander"),
        (4, 1) => Some("SB-400"),
        (4, 2) => Some("SB-400"),
        (4, 4) => Some("SB-400"),
        (5, 1) => Some("SB-900"),
        (5, 2) => Some("SB-900"),
        (6, 1) => Some("SB-700"),
        (7, 1) => Some("SB-910"),
        (14, 3) => Some("SB-5000"),
        _ => None,
    };

    match model {
        Some(name) => format!("{}.{:02} ({})", major, minor, name),
        None => format!("{}.{:02} (Unknown model)", major, minor),
    }
}

/// Parse FaceDetect tag data (tag 0x0021)
/// Extracts FaceDetectFrameSize and FacesDetected
fn parse_face_detect(data: &[u8], endian: Endianness) -> Vec<(String, String)> {
    let mut tags = Vec::new();

    // FaceDetect structure (FORMAT => 'int16u' in ExifTool):
    // Index 0x00: unknown (bytes 0-1)
    // Index 0x01-0x02: FaceDetectFrameSize int16u[2] (bytes 2-5)
    // Index 0x03: FacesDetected int16u (bytes 6-7)
    if data.len() >= 6 {
        // FaceDetectFrameSize: Width x Height
        let width = match endian {
            Endianness::Big => u16::from_be_bytes([data[2], data[3]]),
            Endianness::Little => u16::from_le_bytes([data[2], data[3]]),
        };
        let height = match endian {
            Endianness::Big => u16::from_be_bytes([data[4], data[5]]),
            Endianness::Little => u16::from_le_bytes([data[4], data[5]]),
        };
        tags.push((
            "FaceDetectFrameSize".to_string(),
            format!("{} {}", width, height),
        ));
    }

    if data.len() >= 8 {
        let faces = match endian {
            Endianness::Big => u16::from_be_bytes([data[6], data[7]]),
            Endianness::Little => u16::from_le_bytes([data[6], data[7]]),
        };
        tags.push(("FacesDetected".to_string(), faces.to_string()));
    }

    tags
}

/// Parse FlashInfo tag data (tag 0x00A8)
/// Version-dependent structure - supports versions 0100-0108, 0200
/// For Z cameras, FlashControlMode comes from MenuSettings (ShotInfo), not FlashInfo,
/// so we skip outputting it here to avoid conflicting values
fn parse_flash_info(data: &[u8], model: Option<&str>) -> Vec<(String, String)> {
    let is_z_camera = is_nikon_z_series(model);
    let mut tags = Vec::new();

    if data.len() < 4 {
        return tags;
    }

    // Offset 0x00: FlashInfoVersion (string[4])
    let version = String::from_utf8_lossy(&data[0..4]).to_string();
    tags.push(("FlashInfoVersion".to_string(), version.clone()));

    // Handle different FlashInfo versions
    // 0100/0101: D2H, D2Hs, D2X, D2Xs, D50, D70, D70s, D80, D200
    // 0102: D3 firmware 1.x, D300 firmware 1.00
    // 0103/0104/0105: D3 fw2.x, D3X, D3S, D4, D90, D300 fw1.10, D300S, D600, D700, D800, D3000-D5200, D7000
    // 0106: Df, D610, D3300, D5300, D7100
    // 0107/0108: D4S, D750, D810, D5500, D7200, D5, D500, D3400
    // 0200: Nikon 1 cameras

    // All versions from 0100-0108 share the same basic structure for common tags
    let is_standard_version = version.starts_with("010");

    if is_standard_version && data.len() >= 10 {
        // Offset 0x04: FlashSource (int8u) - common to all 01xx versions
        let source = data[4];
        let source_str = match source {
            0 => "None",
            1 => "External",
            2 => "Internal",
            _ => "Unknown",
        };
        tags.push(("FlashSource".to_string(), source_str.to_string()));

        // Offset 0x06-0x07: ExternalFlashFirmware (int8u[2])
        if data.len() >= 8 {
            if data[6] == 0 && data[7] == 0 {
                tags.push(("ExternalFlashFirmware".to_string(), "n/a".to_string()));
            } else {
                tags.push((
                    "ExternalFlashFirmware".to_string(),
                    format_flash_firmware(data[6], data[7]),
                ));
            }
        }

        // Offset 0x08: ExternalFlashFlags (int8u)
        // BITMASK: bit0=Fired, bit2=Bounce Flash, bit4=Wide Flash Adapter, bit5=Dome Diffuser
        // Unknown bits are output as [N] to match ExifTool behavior
        if data.len() >= 9 {
            let flags = data[8];
            if flags == 0 {
                tags.push(("ExternalFlashFlags".to_string(), "(none)".to_string()));
            } else {
                let mut flag_strs: Vec<String> = Vec::new();
                let mut known_bits: u8 = 0;
                if flags & 0x01 != 0 {
                    flag_strs.push("Fired".to_string());
                    known_bits |= 0x01;
                }
                if flags & 0x04 != 0 {
                    flag_strs.push("Bounce Flash".to_string());
                    known_bits |= 0x04;
                }
                if flags & 0x10 != 0 {
                    flag_strs.push("Wide Flash Adapter".to_string());
                    known_bits |= 0x10;
                }
                if flags & 0x20 != 0 {
                    flag_strs.push("Dome Diffuser".to_string());
                    known_bits |= 0x20;
                }
                // Add unknown bits as [N]
                let unknown_bits = flags & !known_bits;
                for bit in 0..8 {
                    if unknown_bits & (1 << bit) != 0 {
                        flag_strs.push(format!("[{}]", bit));
                    }
                }
                tags.push(("ExternalFlashFlags".to_string(), flag_strs.join(", ")));
            }
        }

        // Offset 0x09.1: FlashCommanderMode (bit 7, mask 0x80)
        if data.len() >= 10 {
            let commander_mode = (data[9] & 0x80) >> 7;
            let commander_str = if commander_mode == 0 { "Off" } else { "On" };
            tags.push(("FlashCommanderMode".to_string(), commander_str.to_string()));

            // Offset 0x09.2: FlashControlMode (bits 0-6, mask 0x7F)
            // For Z cameras, FlashControlMode comes from MenuSettings with different mapping,
            // so skip outputting from FlashInfo to avoid conflicts
            if !is_z_camera {
                let control_mode = data[9] & 0x7F;
                tags.push((
                    "FlashControlMode".to_string(),
                    decode_flash_control_mode(control_mode).to_string(),
                ));
            }
        }

        // Offset 0x0A (10): FlashCompensation (int8s) - when FlashControlMode < 6
        // Or FlashOutput when FlashControlMode >= 6
        if data.len() > 10 {
            let comp = data[10] as i8;
            // ExifTool uses -$val/6 for compensation
            let ev = -(comp as f64) / 6.0;
            tags.push((
                "FlashCompensation".to_string(),
                format_flash_compensation(ev),
            ));
        }

        // Version-specific parsing for extended tags
        // 0102 and later have group control modes and compensations
        let is_extended_version = version == "0102"
            || version == "0103"
            || version == "0104"
            || version == "0105"
            || version == "0106"
            || version == "0107"
            || version == "0108";

        if is_extended_version {
            // Offset 0x0C (12): FlashFocalLength (int8u)
            if data.len() > 12 {
                let fl = data[12];
                if fl != 0 && fl != 255 {
                    tags.push(("FlashFocalLength".to_string(), format!("{} mm", fl)));
                }
            }

            // Offset 0x0D (13): RepeatingFlashRate (int8u)
            if data.len() > 13 {
                let rate = data[13];
                if rate != 0 && rate != 255 {
                    tags.push(("RepeatingFlashRate".to_string(), format!("{} Hz", rate)));
                }
            }

            // Offset 0x0E (14): RepeatingFlashCount (int8u)
            if data.len() > 14 {
                let count = data[14];
                if count != 0 && count != 255 {
                    tags.push(("RepeatingFlashCount".to_string(), count.to_string()));
                }
            }

            // Offset 0x0F (15): FlashGNDistance (int8u)
            if data.len() > 15 {
                let gn = data[15];
                // Convert to distance string (direct lookup, no +3 adjustment)
                let gn_str = match gn {
                    0 => "0".to_string(),
                    1 => "0.1 m".to_string(),
                    2 => "0.2 m".to_string(),
                    3 => "0.3 m".to_string(),
                    4 => "0.4 m".to_string(),
                    5 => "0.5 m".to_string(),
                    6 => "0.6 m".to_string(),
                    7 => "0.7 m".to_string(),
                    8 => "0.8 m".to_string(),
                    9 => "0.9 m".to_string(),
                    10 => "1.0 m".to_string(),
                    11 => "1.1 m".to_string(),
                    12 => "1.3 m".to_string(),
                    13 => "1.4 m".to_string(),
                    14 => "1.6 m".to_string(),
                    15 => "1.8 m".to_string(),
                    16 => "2.0 m".to_string(),
                    17 => "2.2 m".to_string(),
                    18 => "2.5 m".to_string(),
                    19 => "2.8 m".to_string(),
                    20 => "3.2 m".to_string(),
                    21 => "3.6 m".to_string(),
                    22 => "4.0 m".to_string(),
                    23 => "4.5 m".to_string(),
                    24 => "5.0 m".to_string(),
                    25 => "5.6 m".to_string(),
                    26 => "6.3 m".to_string(),
                    27 => "7.1 m".to_string(),
                    28 => "8.0 m".to_string(),
                    29 => "9.0 m".to_string(),
                    30 => "10.0 m".to_string(),
                    31 => "11.0 m".to_string(),
                    32 => "13.0 m".to_string(),
                    33 => "14.0 m".to_string(),
                    34 => "16.0 m".to_string(),
                    35 => "18.0 m".to_string(),
                    36 => "20.0 m".to_string(),
                    255 => "n/a".to_string(),
                    _ => gn.to_string(), // Unknown values pass through
                };
                tags.push(("FlashGNDistance".to_string(), gn_str));
            }

            // FlashColorFilter at offset 0x10 (16) for versions 0103+
            if data.len() > 16 {
                let color_filter = data[16];
                let color_str = match color_filter {
                    0x00 => "None",
                    1 => "FL-GL1 or SZ-2FL Fluorescent",
                    2 => "FL-GL2",
                    9 => "TN-A1 or SZ-2TN Incandescent",
                    10 => "TN-A2",
                    65 => "Red",
                    66 => "Blue",
                    67 => "Yellow",
                    68 => "Amber",
                    128 => "Incandescent",
                    _ => "Unknown",
                };
                tags.push(("FlashColorFilter".to_string(), color_str.to_string()));
            }

            // Group control modes differ slightly by version
            // 0102: Groups at offsets 0x10/0x11
            // 0103+: Groups at offsets 0x11/0x12 (shifted by 1)
            let group_offset = if version == "0102" { 16 } else { 17 };

            // FlashGroupAControlMode (low nibble)
            if data.len() > group_offset {
                let group_a = data[group_offset] & 0x0F;
                tags.push((
                    "FlashGroupAControlMode".to_string(),
                    decode_flash_control_mode(group_a).to_string(),
                ));
            }

            // FlashGroupBControlMode (high nibble) and FlashGroupCControlMode (low nibble)
            if data.len() > group_offset + 1 {
                let group_b = (data[group_offset + 1] & 0xF0) >> 4;
                tags.push((
                    "FlashGroupBControlMode".to_string(),
                    decode_flash_control_mode(group_b).to_string(),
                ));

                let group_c = data[group_offset + 1] & 0x0F;
                tags.push((
                    "FlashGroupCControlMode".to_string(),
                    decode_flash_control_mode(group_c).to_string(),
                ));
            }

            // Group compensations
            let comp_offset = if version == "0102" { 18 } else { 19 };

            // FlashGroupACompensation (int8s)
            if data.len() > comp_offset {
                let comp = data[comp_offset] as i8;
                let ev = -(comp as f64) / 6.0;
                tags.push((
                    "FlashGroupACompensation".to_string(),
                    format_flash_compensation(ev),
                ));
            }

            // FlashGroupBCompensation (int8s)
            if data.len() > comp_offset + 1 {
                let comp = data[comp_offset + 1] as i8;
                let ev = -(comp as f64) / 6.0;
                tags.push((
                    "FlashGroupBCompensation".to_string(),
                    format_flash_compensation(ev),
                ));
            }

            // FlashGroupCCompensation (int8s)
            if data.len() > comp_offset + 2 {
                let comp = data[comp_offset + 2] as i8;
                let ev = -(comp as f64) / 6.0;
                tags.push((
                    "FlashGroupCCompensation".to_string(),
                    format_flash_compensation(ev),
                ));
            }

            // ExternalFlashCompensation at offset 0x1b (27) - versions 0103+
            if version != "0102" && data.len() > 0x1b {
                let comp = data[0x1b] as i8;
                let ev = -(comp as f64) / 6.0;
                tags.push((
                    "ExternalFlashCompensation".to_string(),
                    format_flash_compensation(ev),
                ));
            }

            // FlashExposureComp3 at offset 0x1d (29) - does not include bracketing
            if version != "0102" && data.len() > 0x1d {
                let comp = data[0x1d] as i8;
                let ev = -(comp as f64) / 6.0;
                tags.push((
                    "FlashExposureComp3".to_string(),
                    format_flash_compensation(ev),
                ));
            }

            // FlashExposureComp4 at offset 0x27 (39) - includes flash bracketing effect
            // Available in versions 0103+ (not 0102)
            if version != "0102" && data.len() > 0x27 {
                let comp = data[0x27] as i8;
                let ev = -(comp as f64) / 6.0;
                tags.push((
                    "FlashExposureComp4".to_string(),
                    format_flash_compensation(ev),
                ));
            }
        } else if (version == "0100" || version == "0101") && data.len() >= 19 {
            // FlashInfo 0100/0101 structure (D200, D50, D70, D80, etc.)
            // FlashGNDistance at offset 14
            if data.len() > 14 {
                let gn = data[14];
                let gn_str = match gn {
                    0 => "0".to_string(),
                    1 => "0.1 m".to_string(),
                    2 => "0.2 m".to_string(),
                    3 => "0.3 m".to_string(),
                    4 => "0.4 m".to_string(),
                    5 => "0.5 m".to_string(),
                    6 => "0.6 m".to_string(),
                    7 => "0.7 m".to_string(),
                    8 => "0.8 m".to_string(),
                    9 => "0.9 m".to_string(),
                    10 => "1.0 m".to_string(),
                    11 => "1.1 m".to_string(),
                    12 => "1.3 m".to_string(),
                    13 => "1.4 m".to_string(),
                    14 => "1.6 m".to_string(),
                    15 => "1.8 m".to_string(),
                    16 => "2.0 m".to_string(),
                    17 => "2.2 m".to_string(),
                    18 => "2.5 m".to_string(),
                    19 => "2.8 m".to_string(),
                    20 => "3.2 m".to_string(),
                    21 => "3.6 m".to_string(),
                    22 => "4.0 m".to_string(),
                    23 => "4.5 m".to_string(),
                    24 => "5.0 m".to_string(),
                    25 => "5.6 m".to_string(),
                    26 => "6.3 m".to_string(),
                    27 => "7.1 m".to_string(),
                    28 => "8.0 m".to_string(),
                    29 => "9.0 m".to_string(),
                    30 => "10.0 m".to_string(),
                    31 => "11.0 m".to_string(),
                    32 => "13.0 m".to_string(),
                    33 => "14.0 m".to_string(),
                    34 => "16.0 m".to_string(),
                    35 => "18.0 m".to_string(),
                    36 => "20.0 m".to_string(),
                    255 => "n/a".to_string(),
                    _ => gn.to_string(),
                };
                tags.push(("FlashGNDistance".to_string(), gn_str));
            }

            // FlashGroupAControlMode at offset 15 (mask 0x0f)
            if data.len() > 15 {
                let group_a = data[15] & 0x0F;
                tags.push((
                    "FlashGroupAControlMode".to_string(),
                    decode_flash_control_mode(group_a).to_string(),
                ));
            }

            // FlashGroupBControlMode at offset 16 (mask 0x0f)
            if data.len() > 16 {
                let group_b = data[16] & 0x0F;
                tags.push((
                    "FlashGroupBControlMode".to_string(),
                    decode_flash_control_mode(group_b).to_string(),
                ));
            }

            // FlashGroupACompensation at offset 17
            if data.len() > 17 {
                let comp = data[17] as i8;
                let ev = -(comp as f64) / 6.0;
                tags.push((
                    "FlashGroupACompensation".to_string(),
                    format_flash_compensation(ev),
                ));
            }

            // FlashGroupBCompensation at offset 18
            if data.len() > 18 {
                let comp = data[18] as i8;
                let ev = -(comp as f64) / 6.0;
                tags.push((
                    "FlashGroupBCompensation".to_string(),
                    format_flash_compensation(ev),
                ));
            }
        }
    }

    tags
}

/// Parse ShotInfo tag data (tag 0x0091)
/// Extracts the ShotInfoVersion and FirmwareVersion from the (encrypted) blob
fn parse_shot_info_basic(data: &[u8]) -> Vec<(String, String)> {
    let mut tags = Vec::new();

    if data.len() < 4 {
        return tags;
    }

    // Offset 0x00: ShotInfoVersion (string[4]) - NOT encrypted
    let version = String::from_utf8_lossy(&data[0..4]).to_string();
    // Validate that it looks like a valid version string (4 digits)
    if version.chars().all(|c| c.is_ascii_digit()) {
        tags.push(("ShotInfoVersion".to_string(), version.clone()));

        // Offset 0x04: FirmwareVersion
        // Note: FirmwareVersion is stored unencrypted at offset 0x04 for all versions
        // The encryption in 02xx versions starts after FirmwareVersion
        // - Versions 0246 (D6), 0249 (Z6/Z7), 0251 (Z6_3): 8 bytes
        // - Earlier versions: 5 bytes
        let firmware_len = match version.as_str() {
            "0246" | "0249" | "0250" | "0251" | "0252" | "0253" | "0254" | "0255" => 8,
            _ => 5,
        };
        let firmware_end = 4 + firmware_len;
        if data.len() >= firmware_end {
            let firmware = String::from_utf8_lossy(&data[4..firmware_end]);
            // Check if it looks like a valid firmware version (typically like "1.00k" or "01.11.d0")
            if firmware.chars().any(|c| c.is_ascii_digit()) {
                tags.push((
                    "FirmwareVersion".to_string(),
                    firmware.trim_end_matches('\0').trim_end().to_string(),
                ));
            }
        }
    }

    tags
}

/// Parse MultiExposure tag data (tag 0x00B0)
/// Versions: 0100 (big-endian), 0101 (little-endian), 0102/0103 (MultiExposure2)
fn parse_multi_exposure(data: &[u8]) -> Vec<(String, String)> {
    let mut tags = Vec::new();

    if data.len() < 4 {
        return tags;
    }

    // Offset 0x00: MultiExposureVersion (string[4])
    let version = String::from_utf8_lossy(&data[0..4]).to_string();
    tags.push(("MultiExposureVersion".to_string(), version.clone()));

    // Data format is int32u, so each field is 4 bytes
    // Version determines endianness: 0100 = BE, 0101+ = LE
    let is_le = version != "0100";

    if data.len() >= 8 {
        // Offset 4-7: MultiExposureMode
        let mode = if is_le {
            u32::from_le_bytes([data[4], data[5], data[6], data[7]])
        } else {
            u32::from_be_bytes([data[4], data[5], data[6], data[7]])
        };

        let mode_str = match mode {
            0 => "Off",
            1 => "Multiple Exposure",
            2 => "Image Overlay",
            3 => "HDR",
            _ => "Unknown",
        };
        tags.push(("MultiExposureMode".to_string(), mode_str.to_string()));
    }

    if data.len() >= 12 {
        // Offset 8-11: MultiExposureShots
        let shots = if is_le {
            u32::from_le_bytes([data[8], data[9], data[10], data[11]])
        } else {
            u32::from_be_bytes([data[8], data[9], data[10], data[11]])
        };
        tags.push(("MultiExposureShots".to_string(), shots.to_string()));
    }

    if data.len() >= 16 {
        // Offset 12-15: MultiExposureAutoGain
        let auto_gain = if is_le {
            u32::from_le_bytes([data[12], data[13], data[14], data[15]])
        } else {
            u32::from_be_bytes([data[12], data[13], data[14], data[15]])
        };

        let auto_gain_str = if auto_gain == 0 { "Off" } else { "On" };
        tags.push((
            "MultiExposureAutoGain".to_string(),
            auto_gain_str.to_string(),
        ));
    }

    tags
}

/// Parse HDRInfo tag data (tag 0x0035)
/// Structure: HDRInfoVersion (string[4]) + HDR (byte) + HDRLevel (byte) + HDRSmoothing (byte) + HDRLevel2 (byte)
fn parse_hdr_info(data: &[u8]) -> Vec<(String, String)> {
    let mut tags = Vec::new();

    if data.len() < 4 {
        return tags;
    }

    // Offset 0x00: HDRInfoVersion (string[4])
    let version = String::from_utf8_lossy(&data[0..4]).to_string();
    tags.push(("HDRInfoVersion".to_string(), version.clone()));

    // Offset 0x04: HDR (int8u)
    if data.len() > 4 {
        let hdr = data[4];
        let hdr_str = match hdr {
            0 => "Off",
            1 => "On (normal)",
            48 => "Auto",
            _ => "Unknown",
        };
        tags.push(("HDR".to_string(), hdr_str.to_string()));
    }

    // Offset 0x05: HDRLevel (int8u)
    if data.len() > 5 {
        let level = data[5];
        let level_str = match level {
            0 => "Auto",
            1 => "1 EV",
            2 => "2 EV",
            3 => "3 EV",
            255 => "n/a",
            _ => "Unknown",
        };
        tags.push(("HDRLevel".to_string(), level_str.to_string()));
    }

    // Offset 0x06: HDRSmoothing (int8u)
    if data.len() > 6 {
        let smoothing = data[6];
        let smoothing_str = match smoothing {
            0 => "Off",
            1 => "Normal",
            2 => "Low",
            3 => "High",
            48 => "Auto",
            _ => "Unknown",
        };
        tags.push(("HDRSmoothing".to_string(), smoothing_str.to_string()));
    }

    // Offset 0x07: HDRLevel2 (int8u) - for HDRInfoVersion 0101+
    if data.len() > 7 && version != "0100" {
        let level2 = data[7];
        let level2_str = match level2 {
            0 => "Auto",
            1 => "1 EV",
            2 => "2 EV",
            3 => "3 EV",
            255 => "n/a",
            _ => "Unknown",
        };
        tags.push(("HDRLevel2".to_string(), level2_str.to_string()));
    }

    tags
}

/// Parse FileInfo tag data (tag 0x00B8)
/// Endianness varies by camera model
fn parse_file_info(data: &[u8]) -> Vec<(String, String)> {
    let mut tags = Vec::new();

    if data.len() < 4 {
        return tags;
    }

    // Offset 0x00: FileInfoVersion (string[4])
    let version = String::from_utf8_lossy(&data[0..4]).to_string();
    if version.chars().all(|c| c.is_ascii_digit()) {
        tags.push(("FileInfoVersion".to_string(), version.clone()));
    }

    // The rest of the structure uses int16u format
    // Need to detect endianness based on valid DirectoryNumber range (100-999)
    if data.len() >= 10 {
        // Try little-endian first
        let dir_le = u16::from_le_bytes([data[6], data[7]]);
        let file_le = u16::from_le_bytes([data[8], data[9]]);
        let dir_be = u16::from_be_bytes([data[6], data[7]]);
        let file_be = u16::from_be_bytes([data[8], data[9]]);

        let is_le = (99..=999).contains(&dir_le) && file_le <= 9999;
        let is_be = (99..=999).contains(&dir_be) && file_be <= 9999;

        let (mem_card, dir, file) = if is_le && !is_be {
            (u16::from_le_bytes([data[4], data[5]]), dir_le, file_le)
        } else {
            // Default to big-endian or if both appear valid
            (u16::from_be_bytes([data[4], data[5]]), dir_be, file_be)
        };

        tags.push(("MemoryCardNumber".to_string(), mem_card.to_string()));
        tags.push(("DirectoryNumber".to_string(), format!("{:03}", dir)));
        tags.push(("FileNumber".to_string(), format!("{:04}", file)));
    }

    tags
}

/// Parse RetouchInfo tag data (tag 0x00BB)
fn parse_retouch_info(data: &[u8]) -> Vec<(String, String)> {
    let mut tags = Vec::new();

    if data.len() < 4 {
        return tags;
    }

    // Offset 0x00: RetouchInfoVersion (string[4])
    let version = String::from_utf8_lossy(&data[0..4]).to_string();
    if version.chars().all(|c| c.is_ascii_digit()) {
        tags.push(("RetouchInfoVersion".to_string(), version.clone()));

        // Offset 0x05: RetouchNEFProcessing (int8s) - only for version >= 0200
        if version.as_str() >= "0200" && data.len() >= 6 {
            let nef_processing = data[5] as i8;
            let nef_str = match nef_processing {
                -1 => "Off",
                1 => "On",
                _ => return tags, // Unknown value
            };
            tags.push(("RetouchNEFProcessing".to_string(), nef_str.to_string()));
        }
    }

    tags
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

/// Parse LensData tag data (tag 0x0098)
/// Extracts lens parameters from the binary blob
/// Version 0100: D100, D1X (unencrypted)
/// Version 0101: D70, D70s (unencrypted)
/// Version 02xx: D200, D300, D800, etc. (encrypted after byte 4)
/// Returns (tags, lens_id_bytes) where lens_id_bytes are the 7 raw bytes needed for LensID
///
/// Parameters:
/// - data: Raw LensData bytes
/// - serial: SerialNumber from tag 0x001D (for decryption)
/// - shutter_count: ShutterCount from tag 0x00A7 (for decryption)
fn parse_lens_data(
    data: &[u8],
    serial: Option<u32>,
    shutter_count: Option<u32>,
) -> (Vec<(String, String)>, Option<[u8; 7]>) {
    let mut tags = Vec::new();

    if data.len() < 4 {
        return (tags, None);
    }

    // Get version string
    let version = match std::str::from_utf8(&data[0..4]) {
        Ok(v) => v,
        Err(_) => return (tags, None),
    };

    // Add LensDataVersion
    tags.push(("LensDataVersion".to_string(), version.to_string()));

    let mut lens_id_bytes: Option<[u8; 7]> = None;

    // Version 0100: D100, D1X - unencrypted
    if version == "0100" && data.len() >= 13 {
        // Offset 0x06: LensIDNumber
        let lens_id_number = data[6];
        tags.push(("LensIDNumber".to_string(), lens_id_number.to_string()));

        // Offset 0x07: LensFStops raw value (for lens_id_bytes)
        // Don't output here - use NIKON_LENS_F_STOPS (0x008B) tag instead to avoid duplicates
        let lens_f_stops_raw = data[7];

        // Offset 0x08: MinFocalLength (5 * 2^(value/24))
        let min_focal_raw = data[8];
        let min_focal = 5.0 * 2.0_f64.powf(min_focal_raw as f64 / 24.0);
        tags.push(("MinFocalLength".to_string(), format!("{:.1} mm", min_focal)));

        // Offset 0x09: MaxFocalLength (5 * 2^(value/24))
        let max_focal_raw = data[9];
        let max_focal = 5.0 * 2.0_f64.powf(max_focal_raw as f64 / 24.0);
        tags.push(("MaxFocalLength".to_string(), format!("{:.1} mm", max_focal)));

        // Offset 0x0a: MaxApertureAtMinFocal (2^(value/24))
        let max_ap_min_raw = data[10];
        let max_ap_min = 2.0_f64.powf(max_ap_min_raw as f64 / 24.0);
        tags.push((
            "MaxApertureAtMinFocal".to_string(),
            format!("{:.1}", max_ap_min),
        ));

        // Offset 0x0b: MaxApertureAtMaxFocal (2^(value/24))
        let max_ap_max_raw = data[11];
        let max_ap_max = 2.0_f64.powf(max_ap_max_raw as f64 / 24.0);
        tags.push((
            "MaxApertureAtMaxFocal".to_string(),
            format!("{:.1}", max_ap_max),
        ));

        // Offset 0x0c: MCUVersion
        let mcu_version = data[12];
        tags.push(("MCUVersion".to_string(), mcu_version.to_string()));

        // Store raw bytes for LensID: IDNumber, FStops, MinFocal, MaxFocal, MaxApMin, MaxApMax, MCU
        lens_id_bytes = Some([
            lens_id_number,
            lens_f_stops_raw,
            min_focal_raw,
            max_focal_raw,
            max_ap_min_raw,
            max_ap_max_raw,
            mcu_version,
        ]);
    }
    // Version 0101: D70, D70s - unencrypted
    else if version == "0101" && data.len() >= 18 {
        // LensData01 structure
        // Offset 0x04: ExitPupilPosition
        let exit_pupil_raw = data[4];
        if exit_pupil_raw > 0 {
            let exit_pupil = 2048.0 / exit_pupil_raw as f64;
            tags.push((
                "ExitPupilPosition".to_string(),
                format!("{:.1} mm", exit_pupil),
            ));
        }

        // Offset 0x05: AFAperture
        let af_aperture_raw = data[5];
        let af_aperture = 2.0_f64.powf(af_aperture_raw as f64 / 24.0);
        tags.push(("AFAperture".to_string(), format!("{:.1}", af_aperture)));

        // Offset 0x08: FocusPosition
        let focus_pos = data[8];
        tags.push(("FocusPosition".to_string(), format!("0x{:02x}", focus_pos)));

        // Offset 0x09: FocusDistance
        // Formula: 0.01 * 10^(val/40) meters
        // ExifTool: raw=0 gives 0.01m, not infinity
        // Store with high precision; reformatting to %.2f for display is done in output.rs
        let focus_dist_raw = data[9];
        let focus_dist = 0.01 * 10.0_f64.powf(focus_dist_raw as f64 / 40.0);
        tags.push(("FocusDistance".to_string(), format!("{:.6} m", focus_dist)));

        // Offset 0x0b: LensIDNumber
        let lens_id_number = data[11];
        tags.push(("LensIDNumber".to_string(), lens_id_number.to_string()));

        // Offset 0x0c: LensFStops raw value (for lens_id_bytes)
        // Don't output here - use NIKON_LENS_F_STOPS (0x008B) tag instead to avoid duplicates
        let lens_f_stops_raw = data[12];

        // Offset 0x0d: MinFocalLength
        let min_focal_raw = data[13];
        let min_focal = 5.0 * 2.0_f64.powf(min_focal_raw as f64 / 24.0);
        tags.push(("MinFocalLength".to_string(), format!("{:.1} mm", min_focal)));

        // Offset 0x0e: MaxFocalLength
        let max_focal_raw = data[14];
        let max_focal = 5.0 * 2.0_f64.powf(max_focal_raw as f64 / 24.0);
        tags.push(("MaxFocalLength".to_string(), format!("{:.1} mm", max_focal)));

        // Offset 0x0f: MaxApertureAtMinFocal
        let max_ap_min_raw = data[15];
        let max_ap_min = 2.0_f64.powf(max_ap_min_raw as f64 / 24.0);
        tags.push((
            "MaxApertureAtMinFocal".to_string(),
            format!("{:.1}", max_ap_min),
        ));

        // Offset 0x10: MaxApertureAtMaxFocal
        let max_ap_max_raw = data[16];
        let max_ap_max = 2.0_f64.powf(max_ap_max_raw as f64 / 24.0);
        tags.push((
            "MaxApertureAtMaxFocal".to_string(),
            format!("{:.1}", max_ap_max),
        ));

        // Offset 0x11: MCUVersion
        let mcu_version = data[17];
        tags.push(("MCUVersion".to_string(), mcu_version.to_string()));

        // Offset 0x12: EffectiveMaxAperture
        if data.len() >= 19 {
            let eff_max_ap_raw = data[18];
            let eff_max_ap = 2.0_f64.powf(eff_max_ap_raw as f64 / 24.0);
            tags.push((
                "EffectiveMaxAperture".to_string(),
                format!("{:.1}", eff_max_ap),
            ));
        }

        // Store raw bytes for LensID
        lens_id_bytes = Some([
            lens_id_number,
            lens_f_stops_raw,
            min_focal_raw,
            max_focal_raw,
            max_ap_min_raw,
            max_ap_max_raw,
            mcu_version,
        ]);
    }
    // Version 0201, 0202, 0203: Encrypted, same structure as LensData01 (0101)
    // Per ExifTool: '$$valPt =~ /^020[1-3]/' -> LensData01
    else if (version == "0201" || version == "0202" || version == "0203") && data.len() >= 18 {
        if let (Some(ser), Some(count)) = (serial, shutter_count) {
            // Make mutable copy for decryption
            let mut decrypted = data.to_vec();

            // Decrypt starting at byte 4 (version string is unencrypted)
            nikon_decrypt(ser, count, &mut decrypted, 4);

            // Same offsets as 0101 (LensData01)
            // Offset 0x04: ExitPupilPosition
            let exit_pupil_raw = decrypted[4];
            if exit_pupil_raw > 0 {
                let exit_pupil = 2048.0 / exit_pupil_raw as f64;
                tags.push((
                    "ExitPupilPosition".to_string(),
                    format!("{:.1} mm", exit_pupil),
                ));
            }

            // Offset 0x05: AFAperture
            let af_aperture_raw = decrypted[5];
            let af_aperture = 2.0_f64.powf(af_aperture_raw as f64 / 24.0);
            tags.push(("AFAperture".to_string(), format!("{:.1}", af_aperture)));

            // Offset 0x08: FocusPosition
            let focus_pos = decrypted[8];
            tags.push(("FocusPosition".to_string(), format!("0x{:02x}", focus_pos)));

            // Offset 0x09: FocusDistance
            // Formula: 0.01 * 10^(val/40) meters
            // Store with high precision; reformatting to %.2f for display is done in output.rs
            let focus_dist_raw = decrypted[9];
            let focus_dist = 0.01 * 10.0_f64.powf(focus_dist_raw as f64 / 40.0);
            tags.push(("FocusDistance".to_string(), format!("{:.6} m", focus_dist)));

            // Offset 0x0b (11): LensIDNumber
            let lens_id_number = decrypted[11];
            tags.push(("LensIDNumber".to_string(), lens_id_number.to_string()));

            // Offset 0x0c (12): LensFStops raw value (for lens_id_bytes)
            let lens_f_stops_raw = decrypted[12];

            // Offset 0x0d (13): MinFocalLength
            let min_focal_raw = decrypted[13];
            let min_focal = 5.0 * 2.0_f64.powf(min_focal_raw as f64 / 24.0);
            tags.push(("MinFocalLength".to_string(), format!("{:.1} mm", min_focal)));

            // Offset 0x0e (14): MaxFocalLength
            let max_focal_raw = decrypted[14];
            let max_focal = 5.0 * 2.0_f64.powf(max_focal_raw as f64 / 24.0);
            tags.push(("MaxFocalLength".to_string(), format!("{:.1} mm", max_focal)));

            // Offset 0x0f (15): MaxApertureAtMinFocal
            let max_ap_min_raw = decrypted[15];
            let max_ap_min = 2.0_f64.powf(max_ap_min_raw as f64 / 24.0);
            tags.push((
                "MaxApertureAtMinFocal".to_string(),
                format!("{:.1}", max_ap_min),
            ));

            // Offset 0x10 (16): MaxApertureAtMaxFocal
            let max_ap_max_raw = decrypted[16];
            let max_ap_max = 2.0_f64.powf(max_ap_max_raw as f64 / 24.0);
            tags.push((
                "MaxApertureAtMaxFocal".to_string(),
                format!("{:.1}", max_ap_max),
            ));

            // Offset 0x11 (17): MCUVersion
            let mcu_version = decrypted[17];
            tags.push(("MCUVersion".to_string(), mcu_version.to_string()));

            // Offset 0x12 (18): EffectiveMaxAperture
            if decrypted.len() >= 19 {
                let eff_max_ap_raw = decrypted[18];
                let eff_max_ap = 2.0_f64.powf(eff_max_ap_raw as f64 / 24.0);
                tags.push((
                    "EffectiveMaxAperture".to_string(),
                    format!("{:.1}", eff_max_ap),
                ));
            }

            // Store raw bytes for LensID composite lookup
            lens_id_bytes = Some([
                lens_id_number,
                lens_f_stops_raw,
                min_focal_raw,
                max_focal_raw,
                max_ap_min_raw,
                max_ap_max_raw,
                mcu_version,
            ]);
        }
    }
    // Version 0204+: Encrypted, uses LensData0204 structure (offsets shifted by 1)
    // Per ExifTool: '$$valPt =~ /^0204/' -> LensData0204
    else if version.starts_with("020") && version >= "0204" && data.len() >= 19 {
        if let (Some(ser), Some(count)) = (serial, shutter_count) {
            // Make mutable copy for decryption
            let mut decrypted = data.to_vec();

            // Decrypt starting at byte 4 (version string is unencrypted)
            nikon_decrypt(ser, count, &mut decrypted, 4);

            // LensData0204 structure:
            // Offset 0x04: ExitPupilPosition (same as LensData01)
            let exit_pupil_raw = decrypted[4];
            if exit_pupil_raw > 0 {
                let exit_pupil = 2048.0 / exit_pupil_raw as f64;
                tags.push((
                    "ExitPupilPosition".to_string(),
                    format!("{:.1} mm", exit_pupil),
                ));
            }

            // Offset 0x05: AFAperture (same as LensData01)
            let af_aperture_raw = decrypted[5];
            let af_aperture = 2.0_f64.powf(af_aperture_raw as f64 / 24.0);
            tags.push(("AFAperture".to_string(), format!("{:.1}", af_aperture)));

            // Offset 0x08: FocusPosition (same as LensData01)
            let focus_pos = decrypted[8];
            tags.push(("FocusPosition".to_string(), format!("0x{:02x}", focus_pos)));

            // Offset 0x0a (10): FocusDistance (extra byte at 0x09 in this version)
            // Formula: 0.01 * 10^(val/40) meters
            // Store with high precision; reformatting to %.2f for display is done in output.rs
            let focus_dist_raw = decrypted[10];
            let focus_dist = 0.01 * 10.0_f64.powf(focus_dist_raw as f64 / 40.0);
            tags.push(("FocusDistance".to_string(), format!("{:.6} m", focus_dist)));

            // Offset 0x0c (12): LensIDNumber
            let lens_id_number = decrypted[12];
            tags.push(("LensIDNumber".to_string(), lens_id_number.to_string()));

            // Offset 0x0d (13): LensFStops raw value (for lens_id_bytes)
            let lens_f_stops_raw = decrypted[13];

            // Offset 0x0e (14): MinFocalLength
            let min_focal_raw = decrypted[14];
            let min_focal = 5.0 * 2.0_f64.powf(min_focal_raw as f64 / 24.0);
            tags.push(("MinFocalLength".to_string(), format!("{:.1} mm", min_focal)));

            // Offset 0x0f (15): MaxFocalLength
            let max_focal_raw = decrypted[15];
            let max_focal = 5.0 * 2.0_f64.powf(max_focal_raw as f64 / 24.0);
            tags.push(("MaxFocalLength".to_string(), format!("{:.1} mm", max_focal)));

            // Offset 0x10 (16): MaxApertureAtMinFocal
            let max_ap_min_raw = decrypted[16];
            let max_ap_min = 2.0_f64.powf(max_ap_min_raw as f64 / 24.0);
            tags.push((
                "MaxApertureAtMinFocal".to_string(),
                format!("{:.1}", max_ap_min),
            ));

            // Offset 0x11 (17): MaxApertureAtMaxFocal
            let max_ap_max_raw = decrypted[17];
            let max_ap_max = 2.0_f64.powf(max_ap_max_raw as f64 / 24.0);
            tags.push((
                "MaxApertureAtMaxFocal".to_string(),
                format!("{:.1}", max_ap_max),
            ));

            // Offset 0x12 (18): MCUVersion
            let mcu_version = decrypted[18];
            tags.push(("MCUVersion".to_string(), mcu_version.to_string()));

            // Offset 0x13 (19): EffectiveMaxAperture (shifted by 1 from LensData01)
            if decrypted.len() >= 20 {
                let eff_max_ap_raw = decrypted[19];
                let eff_max_ap = 2.0_f64.powf(eff_max_ap_raw as f64 / 24.0);
                tags.push((
                    "EffectiveMaxAperture".to_string(),
                    format!("{:.1}", eff_max_ap),
                ));
            }

            // Store raw bytes for LensID composite lookup
            lens_id_bytes = Some([
                lens_id_number,
                lens_f_stops_raw,
                min_focal_raw,
                max_focal_raw,
                max_ap_min_raw,
                max_ap_max_raw,
                mcu_version,
            ]);
        }
    }

    (tags, lens_id_bytes)
}

/// Parse AFInfo tag data (tag 0x0088)
/// Extracts AF mode and focus point information
/// model is used to determine byte order: DSLR (NIKON D*) uses big-endian, others use little-endian
fn parse_af_info(data: &[u8], model: Option<&str>) -> Vec<(String, String)> {
    let mut tags = Vec::new();

    if data.is_empty() {
        return tags;
    }

    // Byte 0: AFAreaMode
    let af_area_mode = match data[0] {
        0 => "Single Area",
        1 => "Dynamic Area",
        2 => "Dynamic Area (closest subject)",
        3 => "Group Dynamic",
        4 => "Single Area (wide)",
        5 => "Dynamic Area (wide)",
        _ => "Unknown",
    };
    tags.push(("AFAreaMode".to_string(), af_area_mode.to_string()));

    // Byte 1: AFPoint
    if data.len() >= 2 {
        let af_point = match data[1] {
            0 => "Center",
            1 => "Top",
            2 => "Bottom",
            3 => "Mid-left",
            4 => "Mid-right",
            5 => "Upper-left",
            6 => "Upper-right",
            7 => "Lower-left",
            8 => "Lower-right",
            9 => "Far Left",
            10 => "Far Right",
            _ => "Unknown",
        };
        tags.push(("AFPoint".to_string(), af_point.to_string()));
    }

    // Bytes 2-3: AFPointsInFocus (bitmask)
    // ExifTool uses BigEndian for DSLRs (NIKON D*), LittleEndian for others (Coolpix, etc.)
    if data.len() >= 4 {
        let is_dslr = model
            .map(|m| m.to_uppercase().starts_with("NIKON D"))
            .unwrap_or(false);
        let af_points_mask = if is_dslr {
            // Big-endian for DSLRs
            ((data[2] as u16) << 8) | (data[3] as u16)
        } else {
            // Little-endian for Coolpix and other cameras
            (data[2] as u16) | ((data[3] as u16) << 8)
        };
        if af_points_mask == 0 {
            tags.push(("AFPointsInFocus".to_string(), "(none)".to_string()));
        } else {
            let mut points: Vec<String> = Vec::new();
            let point_names = [
                "Center",
                "Top",
                "Bottom",
                "Mid-left",
                "Mid-right",
                "Upper-left",
                "Upper-right",
                "Lower-left",
                "Lower-right",
                "Far Left",
                "Far Right",
            ];
            // Track which bits we've handled
            let mut handled_mask: u16 = 0;
            for (i, name) in point_names.iter().enumerate() {
                let bit_mask = 1u16 << i;
                handled_mask |= bit_mask;
                if af_points_mask & bit_mask != 0 {
                    points.push(name.to_string());
                }
            }
            // Output unknown bits as "[n]" format (matching ExifTool's DecodeBits)
            for bit in 0..16 {
                let bit_mask = 1u16 << bit;
                if (handled_mask & bit_mask) == 0 && (af_points_mask & bit_mask) != 0 {
                    points.push(format!("[{}]", bit));
                }
            }
            if points.is_empty() {
                tags.push(("AFPointsInFocus".to_string(), "(none)".to_string()));
            } else {
                tags.push(("AFPointsInFocus".to_string(), points.join(",")));
            }
        }
    }

    tags
}

/// Parse AFInfo2 tag data (tag 0x00B7)
/// For cameras with LiveView (D3, D300, D700, D5000, etc.)
/// Version 0300+ (D6, D780, Z cameras) has different structure with PrimaryAFPoint at offset 0x38
fn parse_af_info2(data: &[u8]) -> Vec<(String, String)> {
    let mut tags = Vec::new();

    if data.len() < 5 {
        return tags;
    }

    // Bytes 0-3: AFInfo2Version
    let version_str = std::str::from_utf8(&data[0..4]).unwrap_or("");
    tags.push(("AFInfo2Version".to_string(), version_str.to_string()));

    // Check if version 0300+ (different structure)
    let is_v0300_plus = version_str.starts_with("03") || version_str.starts_with("04");

    // Check if Nikon 1 camera (version 0201) - different value mappings
    let is_nikon1 = version_str == "0201";

    // Byte 4: AFDetectionMethod (also determines ContrastDetectAF)
    let af_detection = data[4];
    let af_detection_str = match af_detection {
        0 => "Phase Detect",
        1 => "Contrast Detect",
        2 => "Hybrid",
        _ => "Unknown",
    };
    tags.push((
        "AFDetectionMethod".to_string(),
        af_detection_str.to_string(),
    ));

    // ContrastDetectAF - derived from AFDetectionMethod
    // ExifTool composite tag: On only if FocusMode is not Manual AND AFDetectionMethod == 1
    // For AFDetectionMethod == 2 (Hybrid), ExifTool returns "Off"
    let contrast_detect_str = match af_detection {
        0 => "Off", // Phase Detect
        1 => "On",  // Contrast Detect
        2 => "Off", // Hybrid - treated as Off by ExifTool
        _ => "Off",
    };
    tags.push((
        "ContrastDetectAF".to_string(),
        contrast_detect_str.to_string(),
    ));

    // Byte 5: AFAreaMode (depends on ContrastDetectAF and camera type)
    if data.len() >= 6 {
        let af_area_val = data[5];
        let af_area_mode = if is_nikon1 {
            // Nikon 1 cameras only have values 128-131
            match af_area_val {
                128 => "Single".to_string(),
                129 => "Auto (41 points)".to_string(),
                130 => "Subject Tracking (41 points)".to_string(),
                131 => "Face Priority (41 points)".to_string(),
                _ => format!("Unknown ({})", af_area_val),
            }
        } else if af_detection == 0 {
            // Phase detect mode
            match af_area_val {
                0 => "Single Area".to_string(),
                1 => "Dynamic Area".to_string(),
                2 => "Dynamic Area (closest subject)".to_string(),
                3 => "Group Dynamic".to_string(),
                4 => "Dynamic Area (9 points)".to_string(),
                5 => "Dynamic Area (21 points)".to_string(),
                6 => "Dynamic Area (51 points)".to_string(),
                7 => "Dynamic Area (51 points, 3D-tracking)".to_string(),
                8 => "Auto-area".to_string(),
                9 => "Dynamic Area (3D-tracking)".to_string(),
                10 => "Single Area (wide)".to_string(),
                11 => "Dynamic Area (wide)".to_string(),
                12 => "Dynamic Area (wide, 3D-tracking)".to_string(),
                13 => "Group Area".to_string(),
                14 => "Dynamic Area (25 points)".to_string(),
                15 => "Dynamic Area (72 points)".to_string(),
                16 => "Group Area (HL)".to_string(),
                17 => "Group Area (VL)".to_string(),
                18 => "Dynamic Area (49 points)".to_string(),
                // Nikon 1 cameras (1J1-1J4, 1S1-1S2, 1V1-1V3, AW1)
                128 => "Single".to_string(),
                129 => "Auto (41 points)".to_string(),
                130 => "Subject Tracking (41 points)".to_string(),
                131 => "Face Priority (41 points)".to_string(),
                // Z cameras
                192 => "Pinpoint".to_string(),
                193 => "Single".to_string(),
                194 => "Dynamic".to_string(),
                195 => "Wide (S)".to_string(),
                196 => "Wide (L)".to_string(),
                197 => "Auto".to_string(),
                199 => "Auto".to_string(),
                _ => "Unknown".to_string(),
            }
        } else {
            // Contrast detect mode
            match af_area_val {
                0 => "Contrast-detect".to_string(),
                1 => "Contrast-detect (normal area)".to_string(),
                2 => "Contrast-detect (wide area)".to_string(),
                3 => "Contrast-detect (face priority)".to_string(),
                4 => "Contrast-detect (subject tracking)".to_string(),
                // Nikon 1 cameras
                128 => "Single".to_string(),
                129 => "Auto (41 points)".to_string(),
                130 => "Subject Tracking (41 points)".to_string(),
                131 => "Face Priority (41 points)".to_string(),
                // Z cameras
                192 => "Pinpoint".to_string(),
                193 => "Single".to_string(),
                194 => "Dynamic".to_string(),
                195 => "Wide (S)".to_string(),
                196 => "Wide (L)".to_string(),
                197 => "Auto".to_string(),
                198 => "Auto (People)".to_string(),
                199 => "Auto (Animal)".to_string(),
                // D6
                200 => "Normal-area AF".to_string(),
                201 => "Wide-area AF".to_string(),
                202 => "Face-priority AF".to_string(),
                203 => "Subject-tracking AF".to_string(),
                // Z9
                204 => "Dynamic Area (S)".to_string(),
                205 => "Dynamic Area (M)".to_string(),
                206 => "Dynamic Area (L)".to_string(),
                207 => "3D-tracking".to_string(),
                208 => "Wide-Area (C1/C2)".to_string(),
                _ => "Unknown".to_string(),
            }
        };
        tags.push(("AFAreaMode".to_string(), af_area_mode));
    }

    // Byte 6: PhaseDetectAF / FocusPointSchema
    let focus_point_schema = if data.len() >= 7 { data[6] } else { 0 };

    // PhaseDetectAF logic: ExifTool composite tag
    // Only report PhaseDetectAF as On when AFDetectionMethod == 0 (Phase Detect)
    // For Hybrid (2) or Contrast (1), report Off regardless of FocusPointSchema
    let phase_detect = if is_nikon1 {
        // Nikon 1 cameras: only values 4-6 are valid
        match focus_point_schema {
            4 => "On (73-point)".to_string(),
            5 => "On (5)".to_string(),
            6 => "On (105-point)".to_string(),
            _ => format!("Unknown ({})", focus_point_schema),
        }
    } else if af_detection != 0 {
        "Off".to_string()
    } else {
        match focus_point_schema {
            0 => "Off".to_string(),
            1 => "On (51-point)".to_string(),
            2 => "On (11-point)".to_string(),
            3 => "On (39-point)".to_string(),
            4 => "On (73-point)".to_string(),
            5 => "On (5)".to_string(),
            6 => "On (105-point)".to_string(),
            7 => "On (153-point)".to_string(),
            8 => "On (81-point)".to_string(),
            9 => "On (105-point)".to_string(),
            _ => "Unknown".to_string(),
        }
    };
    tags.push(("PhaseDetectAF".to_string(), phase_detect));

    // FocusPointSchema - derived from PhaseDetectAF value
    let schema_str = match focus_point_schema {
        0 => "Off",
        1 => "51-point",
        2 => "11-point",
        3 => "39-point",
        4 => "73-point",
        5 => "5-point",
        6 => "105-point",
        7 => "153-point",
        8 => "81-point",
        9 => "105-point",
        _ => "Unknown",
    };
    tags.push(("FocusPointSchema".to_string(), schema_str.to_string()));

    // PrimaryAFPoint and AFPointsUsed offsets depend on version
    if is_v0300_plus {
        // Version 0300+: PrimaryAFPoint at offset 0x38 (56), AFPointsUsed at offset 0x0a (10)
        // Byte 7 is AFCoordinatesAvailable in v0300+
        if data.len() >= 8 {
            let af_coords_available = data[7];
            // Only parse PrimaryAFPoint when AFCoordinatesAvailable == 0
            if af_coords_available == 0 && data.len() >= 57 {
                let primary_point = data[0x38];
                let primary_str = decode_primary_af_point(primary_point, focus_point_schema);
                tags.push(("PrimaryAFPoint".to_string(), primary_str));
            }
        }

        // AFPointsUsed at offset 0x0a (10) for v0300+
        if data.len() >= 24 {
            let af_points_used = decode_af_points_used(&data[0x0a..], focus_point_schema);
            tags.push(("AFPointsUsed".to_string(), af_points_used));
        }
    } else {
        // Version 0100-02xx: PrimaryAFPoint at offset 7, AFPointsUsed at offset 8
        if data.len() >= 8 {
            let primary_point = data[7];
            let primary_str = decode_primary_af_point(primary_point, focus_point_schema);
            tags.push(("PrimaryAFPoint".to_string(), primary_str));
        }

        // Bytes 8+: AFPointsUsed (variable length based on schema)
        if data.len() >= 9 {
            let af_points_used = decode_af_points_used(&data[8..], focus_point_schema);
            tags.push(("AFPointsUsed".to_string(), af_points_used));
        }
    }

    tags
}

/// Decode PrimaryAFPoint based on FocusPointSchema
fn decode_primary_af_point(point: u8, schema: u8) -> String {
    match schema {
        0 => {
            // Off / LiveView / manual focus
            if point == 0 {
                "(none)".to_string()
            } else {
                format!("{}", point)
            }
        }
        1 => {
            // 51-point AF: 5 rows (A-E) and 11 columns (1-11)
            let name = match point {
                0 => "(none)",
                1 => "C6 (Center)",
                2 => "B6",
                3 => "A5",
                4 => "D6",
                5 => "E5",
                6 => "C7",
                7 => "B7",
                8 => "A6",
                9 => "D7",
                10 => "E6",
                11 => "C5",
                12 => "B5",
                13 => "A4",
                14 => "D5",
                15 => "E4",
                16 => "C8",
                17 => "B8",
                18 => "A7",
                19 => "D8",
                20 => "E7",
                21 => "C9",
                22 => "B9",
                23 => "A8",
                24 => "D9",
                25 => "E8",
                26 => "C10",
                27 => "B10",
                28 => "A9",
                29 => "D10",
                30 => "E9",
                31 => "C11",
                32 => "B11",
                33 => "D11",
                34 => "C4",
                35 => "B4",
                36 => "A3",
                37 => "D4",
                38 => "E3",
                39 => "C3",
                40 => "B3",
                41 => "A2",
                42 => "D3",
                43 => "E2",
                44 => "C2",
                45 => "B2",
                46 => "A1",
                47 => "D2",
                48 => "E1",
                49 => "C1",
                50 => "B1",
                51 => "D1",
                _ => return format!("Unknown ({})", point),
            };
            name.to_string()
        }
        2 => {
            // 11-point AF
            let name = match point {
                0 => "(none)",
                1 => "Center",
                2 => "Top",
                3 => "Bottom",
                4 => "Mid-left",
                5 => "Upper-left",
                6 => "Lower-left",
                7 => "Far Left",
                8 => "Mid-right",
                9 => "Upper-right",
                10 => "Lower-right",
                11 => "Far Right",
                _ => return format!("Unknown ({})", point),
            };
            name.to_string()
        }
        3 => {
            // 39-point AF
            let name = match point {
                0 => "(none)",
                1 => "C6 (Center)",
                2 => "B6",
                3 => "A2",
                4 => "D6",
                5 => "E2",
                6 => "C7",
                7 => "B7",
                8 => "A3",
                9 => "D7",
                10 => "E3",
                11 => "C5",
                12 => "B5",
                13 => "A1",
                14 => "D5",
                15 => "E1",
                16 => "C8",
                17 => "B8",
                18 => "D8",
                19 => "C9",
                20 => "B9",
                21 => "D9",
                22 => "C10",
                23 => "B10",
                24 => "D10",
                25 => "C11",
                26 => "B11",
                27 => "D11",
                28 => "C4",
                29 => "B4",
                30 => "D4",
                31 => "C3",
                32 => "B3",
                33 => "D3",
                34 => "C2",
                35 => "B2",
                36 => "D2",
                37 => "C1",
                38 => "B1",
                39 => "D1",
                _ => return format!("Unknown ({})", point),
            };
            name.to_string()
        }
        4 => {
            // 73-point (Nikon 1 cameras with 135-point grid), center is E8
            let name = match point {
                0 => "(none)",
                1 => "E8 (Center)",
                2 => "D8",
                3 => "C8",
                4 => "B8",
                5 => "A8",
                6 => "F8",
                7 => "G8",
                8 => "H8",
                9 => "I8",
                10 => "E9",
                11 => "D9",
                12 => "C9",
                13 => "B9",
                14 => "A9",
                15 => "F9",
                16 => "G9",
                17 => "H9",
                18 => "I9",
                19 => "E7",
                20 => "D7",
                21 => "C7",
                22 => "B7",
                23 => "A7",
                24 => "F7",
                25 => "G7",
                26 => "H7",
                27 => "I7",
                // Continue with remaining points...
                _ => return get_af_point_135(point),
            };
            name.to_string()
        }
        5 => {
            // Nikon 1 S2 and newer with 135-point grid, 15 columns
            // 9 rows (B-J) and 15 columns (1-15), F8 is center
            if point == 0 {
                return "(none)".to_string();
            }
            if point == 82 {
                return "F8 (Center)".to_string();
            }
            // Grid calculation: row = int((val + 0.5) / ncol), col = val - ncol * row + 1
            get_af_point_grid(point, 15)
        }
        6 => {
            // Nikon 1 J4/V3 with 171-point AF, 105-point phase-detect, 21 columns
            // 9 rows (B-J) and 19 columns (2-20), F11 is center
            if point == 0 {
                return "(none)".to_string();
            }
            if point == 115 {
                return "F11 (Center)".to_string();
            }
            get_af_point_grid(point, 21)
        }
        7 => {
            // 153-point (D5, D500, D850), center is E9
            let name = match point {
                0 => "(none)",
                1 => "E9 (Center)",
                _ => return get_af_point_153(point),
            };
            name.to_string()
        }
        8 => {
            // 81-point (Z6, Z7, Z50, etc.), 9x9 grid
            // Auto-area mode, used by Z cameras
            if point == 0 {
                return "(none)".to_string();
            }
            // Use 81-point lookup table
            get_af_point_81(point)
        }
        9 => {
            // 105-point (D6), center is D8
            // 7 rows (A-G) with 15 columns (1-15)
            let name = match point {
                0 => "(none)",
                1 => "D8 (Center)",
                _ => return get_af_point_105(point),
            };
            name.to_string()
        }
        _ => format!("{}", point),
    }
}

/// Get AF point name using grid calculation (for Nikon 1 cameras)
/// Uses same algorithm as ExifTool's GetAFPointGrid
fn get_af_point_grid(point: u8, ncol: u8) -> String {
    let val = point as f32;
    let ncol_f = ncol as f32;
    let row = ((val + 0.5) / ncol_f) as u8;
    let col = point - ncol * row + 1;
    let row_char = (b'A' + row) as char;
    format!("{}{}", row_char, col)
}

/// Get AF point name for 135-point grid (Nikon 1)
fn get_af_point_135(point: u8) -> String {
    // 9x15 grid (A-I rows, 1-15 columns)
    let lookup: &[(u8, &str)] = &[
        (1, "E8"),
        (2, "D8"),
        (3, "C8"),
        (4, "B8"),
        (5, "A8"),
        (6, "F8"),
        (7, "G8"),
        (8, "H8"),
        (9, "I8"),
        (10, "E9"),
        (11, "D9"),
        (12, "C9"),
        (13, "B9"),
        (14, "A9"),
        (15, "F9"),
        (16, "G9"),
        (17, "H9"),
        (18, "I9"),
        (19, "E7"),
        (20, "D7"),
        (21, "C7"),
        (22, "B7"),
        (23, "A7"),
        (24, "F7"),
        (25, "G7"),
        (26, "H7"),
        (27, "I7"),
        (28, "E10"),
        (29, "D10"),
        (30, "C10"),
        (31, "B10"),
        (32, "A10"),
        (33, "F10"),
        (34, "G10"),
        (35, "H10"),
        (36, "I10"),
        (37, "E11"),
        (38, "D11"),
        (39, "C11"),
        (40, "B11"),
        (41, "A11"),
        (42, "F11"),
        (43, "G11"),
        (44, "H11"),
        (45, "I11"),
        (46, "E12"),
        (47, "D12"),
        (48, "C12"),
        (49, "B12"),
        (50, "A12"),
        (51, "F12"),
        (52, "G12"),
        (53, "H12"),
        (54, "I12"),
        (55, "E13"),
        (56, "D13"),
        (57, "C13"),
        (58, "B13"),
        (59, "A13"),
        (60, "F13"),
        (61, "G13"),
        (62, "H13"),
        (63, "I13"),
        (64, "E14"),
        (65, "D14"),
        (66, "C14"),
        (67, "B14"),
        (68, "A14"),
        (69, "F14"),
        (70, "G14"),
        (71, "H14"),
        (72, "I14"),
        (73, "E15"),
        (74, "D15"),
        (75, "C15"),
        (76, "B15"),
        (77, "A15"),
        (78, "F15"),
        (79, "G15"),
        (80, "H15"),
        (81, "I15"),
        (82, "E6"),
        (83, "D6"),
        (84, "C6"),
        (85, "B6"),
        (86, "A6"),
        (87, "F6"),
        (88, "G6"),
        (89, "H6"),
        (90, "I6"),
        (91, "E5"),
        (92, "D5"),
        (93, "C5"),
        (94, "B5"),
        (95, "A5"),
        (96, "F5"),
        (97, "G5"),
        (98, "H5"),
        (99, "I5"),
        (100, "E4"),
        (101, "D4"),
        (102, "C4"),
        (103, "B4"),
        (104, "A4"),
        (105, "F4"),
        (106, "G4"),
        (107, "H4"),
        (108, "I4"),
        (109, "E3"),
        (110, "D3"),
        (111, "C3"),
        (112, "B3"),
        (113, "A3"),
        (114, "F3"),
        (115, "G3"),
        (116, "H3"),
        (117, "I3"),
        (118, "E2"),
        (119, "D2"),
        (120, "C2"),
        (121, "B2"),
        (122, "A2"),
        (123, "F2"),
        (124, "G2"),
        (125, "H2"),
        (126, "I2"),
        (127, "E1"),
        (128, "D1"),
        (129, "C1"),
        (130, "B1"),
        (131, "A1"),
        (132, "F1"),
        (133, "G1"),
        (134, "H1"),
        (135, "I1"),
    ];
    for &(idx, name) in lookup {
        if idx == point {
            return name.to_string();
        }
    }
    format!("Unknown ({})", point)
}

/// Get AF point name for 153-point grid (D5, D500, D850)
/// From ExifTool Nikon.pm: 9 rows (A-I) with 17 columns (1-17), center is E9
fn get_af_point_153(point: u8) -> String {
    // Lookup table from ExifTool %afPoints153
    let lookup: &[(u8, &str)] = &[
        (1, "E9"),
        (2, "D9"),
        (3, "C9"),
        (4, "B9"),
        (5, "A9"),
        (6, "F9"),
        (7, "G9"),
        (8, "H9"),
        (9, "I9"),
        (10, "E10"),
        (11, "D10"),
        (12, "C10"),
        (13, "B10"),
        (14, "A10"),
        (15, "F10"),
        (16, "G10"),
        (17, "H10"),
        (18, "I10"),
        (19, "E11"),
        (20, "D11"),
        (21, "C11"),
        (22, "B11"),
        (23, "A11"),
        (24, "F11"),
        (25, "G11"),
        (26, "H11"),
        (27, "I11"),
        (28, "E8"),
        (29, "D8"),
        (30, "C8"),
        (31, "B8"),
        (32, "A8"),
        (33, "F8"),
        (34, "G8"),
        (35, "H8"),
        (36, "I8"),
        (37, "E7"),
        (38, "D7"),
        (39, "C7"),
        (40, "B7"),
        (41, "A7"),
        (42, "F7"),
        (43, "G7"),
        (44, "H7"),
        (45, "I7"),
        (46, "E12"),
        (47, "D12"),
        (48, "C12"),
        (49, "B12"),
        (50, "A12"),
        (51, "F12"),
        (52, "G12"),
        (53, "H12"),
        (54, "I12"),
        (55, "E13"),
        (56, "D13"),
        (57, "C13"),
        (58, "B13"),
        (59, "A13"),
        (60, "F13"),
        (61, "G13"),
        (62, "H13"),
        (63, "I13"),
        (64, "E14"),
        (65, "D14"),
        (66, "C14"),
        (67, "B14"),
        (68, "A14"),
        (69, "F14"),
        (70, "G14"),
        (71, "H14"),
        (72, "I14"),
        (73, "E15"),
        (74, "D15"),
        (75, "C15"),
        (76, "B15"),
        (77, "A15"),
        (78, "F15"),
        (79, "G15"),
        (80, "H15"),
        (81, "I15"),
        (82, "E16"),
        (83, "D16"),
        (84, "C16"),
        (85, "B16"),
        (86, "A16"),
        (87, "F16"),
        (88, "G16"),
        (89, "H16"),
        (90, "I16"),
        (91, "E17"),
        (92, "D17"),
        (93, "C17"),
        (94, "B17"),
        (95, "A17"),
        (96, "F17"),
        (97, "G17"),
        (98, "H17"),
        (99, "I17"),
        (100, "E6"),
        (101, "D6"),
        (102, "C6"),
        (103, "B6"),
        (104, "A6"),
        (105, "F6"),
        (106, "G6"),
        (107, "H6"),
        (108, "I6"),
        (109, "E5"),
        (110, "D5"),
        (111, "C5"),
        (112, "B5"),
        (113, "A5"),
        (114, "F5"),
        (115, "G5"),
        (116, "H5"),
        (117, "I5"),
        (118, "E4"),
        (119, "D4"),
        (120, "C4"),
        (121, "B4"),
        (122, "A4"),
        (123, "F4"),
        (124, "G4"),
        (125, "H4"),
        (126, "I4"),
        (127, "E3"),
        (128, "D3"),
        (129, "C3"),
        (130, "B3"),
        (131, "A3"),
        (132, "F3"),
        (133, "G3"),
        (134, "H3"),
        (135, "I3"),
        (136, "E2"),
        (137, "D2"),
        (138, "C2"),
        (139, "B2"),
        (140, "A2"),
        (141, "F2"),
        (142, "G2"),
        (143, "H2"),
        (144, "I2"),
        (145, "E1"),
        (146, "D1"),
        (147, "C1"),
        (148, "B1"),
        (149, "A1"),
        (150, "F1"),
        (151, "G1"),
        (152, "H1"),
        (153, "I1"),
    ];
    for &(idx, name) in lookup {
        if idx == point {
            return name.to_string();
        }
    }
    format!("Unknown ({})", point)
}

/// Get AF point name for 105-point grid (D6)
/// From ExifTool Nikon.pm %afPoints105: 7 rows (A-G) with 15 columns (1-15), center is D8
fn get_af_point_105(point: u8) -> String {
    // Lookup table from ExifTool %afPoints105
    let lookup: &[(u8, &str)] = &[
        (1, "D8"),
        (2, "C8"),
        (3, "B8"),
        (4, "A8"),
        (5, "E8"),
        (6, "F8"),
        (7, "G8"),
        (8, "D9"),
        (9, "C9"),
        (10, "B9"),
        (11, "A9"),
        (12, "E9"),
        (13, "F9"),
        (14, "G9"),
        (15, "D10"),
        (16, "C10"),
        (17, "B10"),
        (18, "A10"),
        (19, "E10"),
        (20, "F10"),
        (21, "G10"),
        (22, "D7"),
        (23, "C7"),
        (24, "B7"),
        (25, "A7"),
        (26, "E7"),
        (27, "F7"),
        (28, "G7"),
        (29, "D6"),
        (30, "C6"),
        (31, "B6"),
        (32, "A6"),
        (33, "E6"),
        (34, "F6"),
        (35, "G6"),
        (36, "D11"),
        (37, "C11"),
        (38, "B11"),
        (39, "A11"),
        (40, "E11"),
        (41, "F11"),
        (42, "G11"),
        (43, "D12"),
        (44, "C12"),
        (45, "B12"),
        (46, "A12"),
        (47, "E12"),
        (48, "F12"),
        (49, "G12"),
        (50, "D13"),
        (51, "C13"),
        (52, "B13"),
        (53, "A13"),
        (54, "E13"),
        (55, "F13"),
        (56, "G13"),
        (57, "D14"),
        (58, "C14"),
        (59, "B14"),
        (60, "A14"),
        (61, "E14"),
        (62, "F14"),
        (63, "G14"),
        (64, "D15"),
        (65, "C15"),
        (66, "B15"),
        (67, "A15"),
        (68, "E15"),
        (69, "F15"),
        (70, "G15"),
        (71, "D5"),
        (72, "C5"),
        (73, "B5"),
        (74, "A5"),
        (75, "E5"),
        (76, "F5"),
        (77, "G5"),
        (78, "D4"),
        (79, "C4"),
        (80, "B4"),
        (81, "A4"),
        (82, "E4"),
        (83, "F4"),
        (84, "G4"),
        (85, "D3"),
        (86, "C3"),
        (87, "B3"),
        (88, "A3"),
        (89, "E3"),
        (90, "F3"),
        (91, "G3"),
        (92, "D2"),
        (93, "C2"),
        (94, "B2"),
        (95, "A2"),
        (96, "E2"),
        (97, "F2"),
        (98, "G2"),
        (99, "D1"),
        (100, "C1"),
        (101, "B1"),
        (102, "A1"),
        (103, "E1"),
        (104, "F1"),
        (105, "G1"),
    ];
    for &(idx, name) in lookup {
        if idx == point {
            return name.to_string();
        }
    }
    format!("Unknown ({})", point)
}

/// Get AF point name for 81-point grid (Z cameras: Z5, Z6, Z7, Z50, etc.)
/// 9x9 grid for Auto-area AF mode
fn get_af_point_81(point: u8) -> String {
    // From ExifTool %afPoints81 - 9 rows (A-I) with 9 columns (1-9), center is E5
    let lookup: &[(u8, &str)] = &[
        (1, "E5"),
        (2, "D5"),
        (3, "C5"),
        (4, "B5"),
        (5, "A5"),
        (6, "F5"),
        (7, "G5"),
        (8, "H5"),
        (9, "I5"),
        (10, "E6"),
        (11, "D6"),
        (12, "C6"),
        (13, "B6"),
        (14, "A6"),
        (15, "F6"),
        (16, "G6"),
        (17, "H6"),
        (18, "I6"),
        (19, "E4"),
        (20, "D4"),
        (21, "C4"),
        (22, "B4"),
        (23, "A4"),
        (24, "F4"),
        (25, "G4"),
        (26, "H4"),
        (27, "I4"),
        (28, "E7"),
        (29, "D7"),
        (30, "C7"),
        (31, "B7"),
        (32, "A7"),
        (33, "F7"),
        (34, "G7"),
        (35, "H7"),
        (36, "I7"),
        (37, "E3"),
        (38, "D3"),
        (39, "C3"),
        (40, "B3"),
        (41, "A3"),
        (42, "F3"),
        (43, "G3"),
        (44, "H3"),
        (45, "I3"),
        (46, "E8"),
        (47, "D8"),
        (48, "C8"),
        (49, "B8"),
        (50, "A8"),
        (51, "F8"),
        (52, "G8"),
        (53, "H8"),
        (54, "I8"),
        (55, "E2"),
        (56, "D2"),
        (57, "C2"),
        (58, "B2"),
        (59, "A2"),
        (60, "F2"),
        (61, "G2"),
        (62, "H2"),
        (63, "I2"),
        (64, "E9"),
        (65, "D9"),
        (66, "C9"),
        (67, "B9"),
        (68, "A9"),
        (69, "F9"),
        (70, "G9"),
        (71, "H9"),
        (72, "I9"),
        (73, "E1"),
        (74, "D1"),
        (75, "C1"),
        (76, "B1"),
        (77, "A1"),
        (78, "F1"),
        (79, "G1"),
        (80, "H1"),
        (81, "I1"),
    ];
    for &(idx, name) in lookup {
        if idx == point {
            return name.to_string();
        }
    }
    format!("Unknown ({})", point)
}

/// Decode AFPointsUsed bitmask based on FocusPointSchema
fn decode_af_points_used(data: &[u8], schema: u8) -> String {
    match schema {
        0 => {
            // Off / LiveView
            "(none)".to_string()
        }
        1 => {
            // 51-point: 7 bytes bitmask
            if data.len() < 7 {
                return "(none)".to_string();
            }
            decode_af_points_51(data)
        }
        2 => {
            // 11-point: 2 bytes (int16u)
            if data.len() < 2 {
                return "(none)".to_string();
            }
            let bits = u16::from_le_bytes([data[0], data[1]]);
            if bits == 0 {
                return "(none)".to_string();
            }
            if bits == 0x7ff {
                return "All 11 Points".to_string();
            }
            decode_af_points_11(bits)
        }
        3 => {
            // 39-point: 5 bytes bitmask
            if data.len() < 5 {
                return "(none)".to_string();
            }
            decode_af_points_39(data)
        }
        4 => {
            // 73-point (Nikon 1 cameras): 17 bytes bitmask for 135-point grid
            if data.len() < 17 {
                return "(none)".to_string();
            }
            decode_af_points_135(data)
        }
        5 => {
            // Nikon 1 S2 with 135-point grid, 15 columns
            // 21 bytes bitmask
            if data.len() < 21 {
                return "(none)".to_string();
            }
            decode_af_points_grid(data, 15, 21)
        }
        6 => {
            // Nikon 1 J4/V3 with 171-point grid, 21 columns
            // 29 bytes bitmask
            if data.len() < 29 {
                return "(none)".to_string();
            }
            decode_af_points_grid(data, 21, 29)
        }
        7 => {
            // 153-point: 20 bytes bitmask
            if data.len() < 20 {
                return "(none)".to_string();
            }
            decode_af_points_153(data)
        }
        _ => {
            // Unknown schema - output hex
            if data.is_empty() || data.iter().all(|&b| b == 0) {
                return "(none)".to_string();
            }
            data.iter()
                .map(|b| format!("{:02X}", b))
                .collect::<Vec<_>>()
                .join(" ")
        }
    }
}

/// Decode 51-point AF bitmask
fn decode_af_points_51(data: &[u8]) -> String {
    // 51 points across 7 bytes (56 bits, only 51 used)
    // Mapping: bit position -> point name
    let point_names: &[&str] = &[
        "C6", "B6", "A5", "D6", "E5", "C7", "B7", "A6", // byte 0
        "D7", "E6", "C5", "B5", "A4", "D5", "E4", "C8", // byte 1
        "B8", "A7", "D8", "E7", "C9", "B9", "A8", "D9", // byte 2
        "E8", "C10", "B10", "A9", "D10", "E9", "C11", "B11", // byte 3
        "D11", "C4", "B4", "A3", "D4", "E3", "C3", "B3", // byte 4
        "A2", "D3", "E2", "C2", "B2", "A1", "D2", "E1", // byte 5
        "C1", "B1", "D1", // byte 6 (only 3 bits used)
    ];

    let mut points = Vec::new();
    for (byte_idx, &byte) in data.iter().take(7).enumerate() {
        for bit in 0..8 {
            let point_idx = byte_idx * 8 + bit;
            if point_idx < point_names.len() && (byte & (1 << bit)) != 0 {
                points.push(point_names[point_idx]);
            }
        }
    }

    if points.is_empty() {
        "(none)".to_string()
    } else {
        sort_af_points(&mut points);
        points.join(",")
    }
}

/// Decode 11-point AF bitmask
fn decode_af_points_11(bits: u16) -> String {
    // ExifTool bit mapping from Nikon.pm:
    // 0=Center, 1=Top, 2=Bottom, 3=Mid-left, 4=Upper-left, 5=Lower-left,
    // 6=Far Left, 7=Mid-right, 8=Upper-right, 9=Lower-right, 10=Far Right
    let point_names: &[&str] = &[
        "Center",
        "Top",
        "Bottom",
        "Mid-left",
        "Upper-left",
        "Lower-left",
        "Far Left",
        "Mid-right",
        "Upper-right",
        "Lower-right",
        "Far Right",
    ];

    let mut points = Vec::new();
    for (i, &name) in point_names.iter().enumerate() {
        if (bits & (1 << i)) != 0 {
            points.push(name);
        }
    }

    if points.is_empty() {
        "(none)".to_string()
    } else {
        // ExifTool uses comma-space for 11-point text labels
        points.join(", ")
    }
}

/// Decode 39-point AF bitmask
fn decode_af_points_39(data: &[u8]) -> String {
    // 39 points across 5 bytes (40 bits, only 39 used)
    let point_names: &[&str] = &[
        "C6", "B6", "A2", "D6", "E2", "C7", "B7", "A3", // byte 0
        "D7", "E3", "C5", "B5", "A1", "D5", "E1", "C8", // byte 1
        "B8", "D8", "C9", "B9", "D9", "C10", "B10", "D10", // byte 2
        "C11", "B11", "D11", "C4", "B4", "D4", "C3", "B3", // byte 3
        "D3", "C2", "B2", "D2", "C1", "B1", "D1", // byte 4 (only 7 bits)
    ];

    let mut points = Vec::new();
    for (byte_idx, &byte) in data.iter().take(5).enumerate() {
        for bit in 0..8 {
            let point_idx = byte_idx * 8 + bit;
            if point_idx < point_names.len() && (byte & (1 << bit)) != 0 {
                points.push(point_names[point_idx]);
            }
        }
    }

    if points.is_empty() {
        "(none)".to_string()
    } else {
        sort_af_points(&mut points);
        points.join(",")
    }
}

/// Decode 135-point AF bitmask (Nikon 1 cameras with 73-point phase detect)
fn decode_af_points_135(data: &[u8]) -> String {
    // 135 points in a 9x15 grid (A-I rows, 1-15 columns), center is E8
    // Points are indexed 1-135 in the bitmask
    let point_lookup: &[(u8, &str)] = &[
        (1, "E8"),
        (2, "D8"),
        (3, "C8"),
        (4, "B8"),
        (5, "A8"),
        (6, "F8"),
        (7, "G8"),
        (8, "H8"),
        (9, "I8"),
        (10, "E9"),
        (11, "D9"),
        (12, "C9"),
        (13, "B9"),
        (14, "A9"),
        (15, "F9"),
        (16, "G9"),
        (17, "H9"),
        (18, "I9"),
        (19, "E7"),
        (20, "D7"),
        (21, "C7"),
        (22, "B7"),
        (23, "A7"),
        (24, "F7"),
        (25, "G7"),
        (26, "H7"),
        (27, "I7"),
        (28, "E10"),
        (29, "D10"),
        (30, "C10"),
        (31, "B10"),
        (32, "A10"),
        (33, "F10"),
        (34, "G10"),
        (35, "H10"),
        (36, "I10"),
        (37, "E11"),
        (38, "D11"),
        (39, "C11"),
        (40, "B11"),
        (41, "A11"),
        (42, "F11"),
        (43, "G11"),
        (44, "H11"),
        (45, "I11"),
        (46, "E12"),
        (47, "D12"),
        (48, "C12"),
        (49, "B12"),
        (50, "A12"),
        (51, "F12"),
        (52, "G12"),
        (53, "H12"),
        (54, "I12"),
        (55, "E13"),
        (56, "D13"),
        (57, "C13"),
        (58, "B13"),
        (59, "A13"),
        (60, "F13"),
        (61, "G13"),
        (62, "H13"),
        (63, "I13"),
        (64, "E14"),
        (65, "D14"),
        (66, "C14"),
        (67, "B14"),
        (68, "A14"),
        (69, "F14"),
        (70, "G14"),
        (71, "H14"),
        (72, "I14"),
        (73, "E15"),
        (74, "D15"),
        (75, "C15"),
        (76, "B15"),
        (77, "A15"),
        (78, "F15"),
        (79, "G15"),
        (80, "H15"),
        (81, "I15"),
        (82, "E6"),
        (83, "D6"),
        (84, "C6"),
        (85, "B6"),
        (86, "A6"),
        (87, "F6"),
        (88, "G6"),
        (89, "H6"),
        (90, "I6"),
        (91, "E5"),
        (92, "D5"),
        (93, "C5"),
        (94, "B5"),
        (95, "A5"),
        (96, "F5"),
        (97, "G5"),
        (98, "H5"),
        (99, "I5"),
        (100, "E4"),
        (101, "D4"),
        (102, "C4"),
        (103, "B4"),
        (104, "A4"),
        (105, "F4"),
        (106, "G4"),
        (107, "H4"),
        (108, "I4"),
        (109, "E3"),
        (110, "D3"),
        (111, "C3"),
        (112, "B3"),
        (113, "A3"),
        (114, "F3"),
        (115, "G3"),
        (116, "H3"),
        (117, "I3"),
        (118, "E2"),
        (119, "D2"),
        (120, "C2"),
        (121, "B2"),
        (122, "A2"),
        (123, "F2"),
        (124, "G2"),
        (125, "H2"),
        (126, "I2"),
        (127, "E1"),
        (128, "D1"),
        (129, "C1"),
        (130, "B1"),
        (131, "A1"),
        (132, "F1"),
        (133, "G1"),
        (134, "H1"),
        (135, "I1"),
    ];

    let mut points = Vec::new();
    for &(idx, name) in point_lookup {
        let byte_pos = ((idx - 1) / 8) as usize;
        let bit_pos = (idx - 1) % 8;
        if byte_pos < data.len() && (data[byte_pos] & (1 << bit_pos)) != 0 {
            points.push(name);
        }
    }

    if points.is_empty() {
        "(none)".to_string()
    } else {
        sort_af_points(&mut points);
        points.join(",")
    }
}

/// Decode AF points using grid calculation (for Nikon 1 cameras with newer firmware)
/// Uses the same algorithm as ExifTool's PrintAFPointsGrid
fn decode_af_points_grid(data: &[u8], ncol: u8, size: usize) -> String {
    let mut points = Vec::new();

    for (byte_idx, &byte) in data.iter().take(size).enumerate() {
        if byte == 0 {
            continue;
        }
        for bit in 0..8 {
            if (byte & (1 << bit)) != 0 {
                let point_num = byte_idx * 8 + bit;
                let point_name = get_af_point_grid(point_num as u8, ncol);
                points.push(point_name);
            }
        }
    }

    if points.is_empty() {
        "(none)".to_string()
    } else {
        points.join(",")
    }
}

/// Decode 153-point AF bitmask (D5, D500, D850)
fn decode_af_points_153(data: &[u8]) -> String {
    // 153 points in a 9x17 grid (A-I rows, 1-17 columns), center is E9
    let point_lookup: &[(u8, &str)] = &[
        (1, "E9"),
        (2, "D9"),
        (3, "C9"),
        (4, "B9"),
        (5, "A9"),
        (6, "F9"),
        (7, "G9"),
        (8, "H9"),
        (9, "I9"),
        (10, "E10"),
        (11, "D10"),
        (12, "C10"),
        (13, "B10"),
        (14, "A10"),
        (15, "F10"),
        (16, "G10"),
        (17, "H10"),
        (18, "I10"),
        (19, "E8"),
        (20, "D8"),
        (21, "C8"),
        (22, "B8"),
        (23, "A8"),
        (24, "F8"),
        (25, "G8"),
        (26, "H8"),
        (27, "I8"),
        (28, "E11"),
        (29, "D11"),
        (30, "C11"),
        (31, "B11"),
        (32, "A11"),
        (33, "F11"),
        (34, "G11"),
        (35, "H11"),
        (36, "I11"),
        (37, "E7"),
        (38, "D7"),
        (39, "C7"),
        (40, "B7"),
        (41, "A7"),
        (42, "F7"),
        (43, "G7"),
        (44, "H7"),
        (45, "I7"),
        (46, "E12"),
        (47, "D12"),
        (48, "C12"),
        (49, "B12"),
        (50, "A12"),
        (51, "F12"),
        (52, "G12"),
        (53, "H12"),
        (54, "I12"),
        (55, "E6"),
        (56, "D6"),
        (57, "C6"),
        (58, "B6"),
        (59, "A6"),
        (60, "F6"),
        (61, "G6"),
        (62, "H6"),
        (63, "I6"),
        (64, "E13"),
        (65, "D13"),
        (66, "C13"),
        (67, "B13"),
        (68, "A13"),
        (69, "F13"),
        (70, "G13"),
        (71, "H13"),
        (72, "I13"),
        (73, "E14"),
        (74, "D14"),
        (75, "C14"),
        (76, "B14"),
        (77, "A14"),
        (78, "F14"),
        (79, "G14"),
        (80, "H14"),
        (81, "I14"),
        (82, "E5"),
        (83, "D5"),
        (84, "C5"),
        (85, "B5"),
        (86, "A5"),
        (87, "F5"),
        (88, "G5"),
        (89, "H5"),
        (90, "I5"),
        (91, "E15"),
        (92, "D15"),
        (93, "C15"),
        (94, "B15"),
        (95, "A15"),
        (96, "F15"),
        (97, "G15"),
        (98, "H15"),
        (99, "I15"),
        (100, "E4"),
        (101, "D4"),
        (102, "C4"),
        (103, "B4"),
        (104, "A4"),
        (105, "F4"),
        (106, "G4"),
        (107, "H4"),
        (108, "I4"),
        (109, "E16"),
        (110, "D16"),
        (111, "C16"),
        (112, "B16"),
        (113, "A16"),
        (114, "F16"),
        (115, "G16"),
        (116, "H16"),
        (117, "I16"),
        (118, "E3"),
        (119, "D3"),
        (120, "C3"),
        (121, "B3"),
        (122, "A3"),
        (123, "F3"),
        (124, "G3"),
        (125, "H3"),
        (126, "I3"),
        (127, "E17"),
        (128, "D17"),
        (129, "C17"),
        (130, "B17"),
        (131, "A17"),
        (132, "F17"),
        (133, "G17"),
        (134, "H17"),
        (135, "I17"),
        (136, "E2"),
        (137, "D2"),
        (138, "C2"),
        (139, "B2"),
        (140, "A2"),
        (141, "F2"),
        (142, "G2"),
        (143, "H2"),
        (144, "I2"),
        (145, "E1"),
        (146, "D1"),
        (147, "C1"),
        (148, "B1"),
        (149, "A1"),
        (150, "F1"),
        (151, "G1"),
        (152, "H1"),
        (153, "I1"),
    ];

    let mut points = Vec::new();
    for &(idx, name) in point_lookup {
        let byte_pos = ((idx - 1) / 8) as usize;
        let bit_pos = (idx - 1) % 8;
        if byte_pos < data.len() && (data[byte_pos] & (1 << bit_pos)) != 0 {
            points.push(name);
        }
    }

    if points.is_empty() {
        "(none)".to_string()
    } else {
        sort_af_points(&mut points);
        points.join(",")
    }
}

/// Sort AF points in ExifTool order (alphabetically with proper numeric handling)
fn sort_af_points(points: &mut [&str]) {
    points.sort_by(|a, b| {
        let a = *a;
        let b = *b;
        // ExifTool sorting: same length = direct compare
        // Different length: pad shorter one (e.g., "A5" -> "A05") for comparison
        if a.len() == b.len() {
            a.cmp(b)
        } else if a.len() == 2 && b.len() == 3 {
            // "A5" vs "A10" -> compare "A05" vs "A10"
            let padded_a = format!("{}0{}", &a[0..1], &a[1..]);
            padded_a.as_str().cmp(b)
        } else if a.len() == 3 && b.len() == 2 {
            // "A10" vs "A5" -> compare "A10" vs "A05"
            let padded_b = format!("{}0{}", &b[0..1], &b[1..]);
            a.cmp(padded_b.as_str())
        } else {
            a.cmp(b)
        }
    });
}

/// Check if camera has TimeZone location in MenuSettings.
/// Only specific Z cameras have MenuSettings with TimeZone including location names:
/// - Z9 (ShotInfo 0805): MenuSettingsZ9* - TimeZone at various offsets
/// - Z8 (ShotInfo 0806): MenuSettingsZ8* - TimeZone at various offsets
/// - Z6 III only (ShotInfo 0809): MenuSettingsZ6III - TimeZone at 2302
///
/// Note: Z50 II (0810) and Z5 II (0811) use ShotInfoZ6III but their MenuOffset
/// has Condition checking for Z6_3 model only, so they don't get TimeZone from MenuSettings.
fn has_timezone_location_in_shotinfo(model: Option<&str>) -> bool {
    match model {
        Some(m) => {
            let upper = m.to_uppercase();
            upper.contains("NIKON Z 8")
                || upper.contains("NIKON Z 9")
                || upper.contains("NIKON Z6_3") // Z6 III only, not Z50 II or Z5 II
        }
        None => false,
    }
}

/// Get timezone location name for Z9-era cameras based on %timeZoneZ9 table
/// Maps UTC offset to ExifTool's canonical location string
fn get_timezone_location(tz_minutes: i16) -> Option<&'static str> {
    match tz_minutes {
        600 => Some("(Sydney)"),
        540 => Some("(Tokyo)"),
        480 => Some("(Beijing, Honk Kong, Sinapore)"), // ExifTool typo preserved
        345 => Some("(Kathmandu)"),
        330 => Some("(New Dehli)"), // ExifTool typo preserved
        300 => Some("(Islamabad)"),
        270 => Some("(Kabul)"),
        240 => Some("(Abu Dhabi)"),
        210 => Some("(Tehran)"),
        180 => Some("(Moscow, Nairobi)"),
        120 => Some("(Athens, Helsinki)"),
        60 => Some("(Madrid, Paris, Berlin)"),
        0 => Some("(London)"),
        -60 => Some("(Azores)"),
        -120 => Some("(Fernando de Noronha)"),
        -180 => Some("(Buenos Aires, Sao Paulo)"),
        -210 => Some("(Newfoundland)"),
        -240 => Some("(Manaus, Caracas)"),
        -300 => Some("(New York, Toronto, Lima)"),
        -360 => Some("(Chicago, Mexico City)"),
        -420 => Some("(Denver)"),
        -480 => Some("(Los Angeles, Vancouver)"),
        -540 => Some("(Anchorage)"),
        -600 => Some("(Hawaii)"),
        _ => None,
    }
}

/// Parse WorldTime tag data (tag 0x0024)
fn parse_world_time(data: &[u8], endian: Endianness, is_z_camera: bool) -> Vec<(String, String)> {
    let mut tags = Vec::new();

    if data.len() < 4 {
        return tags;
    }

    // Bytes 0-1: Timezone (signed, in minutes)
    let tz_minutes = match endian {
        Endianness::Little => i16::from_le_bytes([data[0], data[1]]),
        Endianness::Big => i16::from_be_bytes([data[0], data[1]]),
    };
    let tz_hours = tz_minutes / 60;
    let tz_mins = (tz_minutes % 60).abs();
    let tz_str = if tz_minutes >= 0 {
        format!("+{:02}:{:02}", tz_hours, tz_mins)
    } else {
        format!("{:03}:{:02}", tz_hours, tz_mins)
    };

    // For Z9-era cameras, add location name suffix (per ExifTool %timeZoneZ9)
    let tz_with_location = if is_z_camera {
        if let Some(location) = get_timezone_location(tz_minutes) {
            format!("{} {}", tz_str, location)
        } else {
            tz_str
        }
    } else {
        tz_str
    };
    tags.push(("TimeZone".to_string(), tz_with_location));

    // Byte 2: DaylightSavings
    if data.len() >= 3 {
        let dst = if data[2] == 0 { "No" } else { "Yes" };
        tags.push(("DaylightSavings".to_string(), dst.to_string()));
    }

    // Byte 3: DateDisplayFormat
    if data.len() >= 4 {
        let date_format = match data[3] {
            0 => "Y/M/D",
            1 => "M/D/Y",
            2 => "D/M/Y",
            _ => "Unknown",
        };
        tags.push(("DateDisplayFormat".to_string(), date_format.to_string()));
    }

    tags
}

/// Parse ColorBalance tag data (tag 0x0097)
/// Extracts white balance levels
/// serial/shutter_count needed for version 02xx decryption
fn parse_color_balance(
    data: &[u8],
    endian: Endianness,
    serial: Option<u32>,
    shutter_count: Option<u32>,
) -> Vec<(String, String)> {
    let mut tags = Vec::new();

    if data.len() < 8 {
        return tags;
    }

    // Check for NRW format first (starts with "NRW ")
    let is_nrw = data.len() >= 4 && &data[0..4] == b"NRW ";

    // First 4 bytes: version (or "NRW " for NRW files)
    let version = if is_nrw {
        // For NRW files, version is at offset 0x0004
        if data.len() >= 8 {
            match std::str::from_utf8(&data[4..8]) {
                Ok(v) => v,
                Err(_) => return tags,
            }
        } else {
            return tags;
        }
    } else {
        match std::str::from_utf8(&data[0..4]) {
            Ok(v) => v,
            Err(_) => return tags,
        }
    };

    // Output version string
    tags.push(("ColorBalanceVersion".to_string(), version.to_string()));

    // ColorBalance structures have WB levels at different offsets
    // For non-NRW files:
    //   Version 0100: RBGG at offset 72 (0x48), big-endian, NOT encrypted (D100, Coolpix)
    //   Version 0102: RGGB at offset 10, NOT encrypted (D2H)
    //   Version 0103: RGBG at offset 20, NOT encrypted (D70/D70s)
    //   Version 02xx (others): Encrypted - require decryption which we don't have keys for
    // For NRW files (handled below with is_nrw check):
    //   Uses ColorBalanceC structure with int32u values at different offsets
    if !is_nrw && version == "0100" && data.len() >= 72 + 8 {
        // Version 0100: RBGG at offset 72, big-endian
        let mut cursor = Cursor::new(&data[72..]);
        let vals: [u16; 4] = [
            cursor.read_u16::<BigEndian>().unwrap_or(0),
            cursor.read_u16::<BigEndian>().unwrap_or(0),
            cursor.read_u16::<BigEndian>().unwrap_or(0),
            cursor.read_u16::<BigEndian>().unwrap_or(0),
        ];

        // Skip if all zeros
        if !vals.iter().all(|&v| v == 0) {
            tags.push((
                "WB_RBGGLevels".to_string(),
                format!("{} {} {} {}", vals[0], vals[1], vals[2], vals[3]),
            ));

            // Compute RedBalance and BlueBalance from RBGG levels
            // Order is: R=vals[0], B=vals[1], G=vals[2], G=vals[3]
            // G = average of two G values
            let g = ((vals[2] as f64) + (vals[3] as f64)) / 2.0;
            if g > 0.0 {
                let red = (vals[0] as f64) / g;
                let blue = (vals[1] as f64) / g;
                tags.push(("RedBalance".to_string(), format!("{:.6}", red)));
                tags.push(("BlueBalance".to_string(), format!("{:.6}", blue)));
            }
        }
    } else if !is_nrw && version == "0102" && data.len() >= 10 + 8 {
        // Version 0102: RGGB at offset 10 (D2H) - non-NRW only
        let mut cursor = Cursor::new(&data[10..]);
        let vals: [u16; 4] = [
            cursor.read_u16::<BigEndian>().unwrap_or(0),
            cursor.read_u16::<BigEndian>().unwrap_or(0),
            cursor.read_u16::<BigEndian>().unwrap_or(0),
            cursor.read_u16::<BigEndian>().unwrap_or(0),
        ];

        if !vals.iter().all(|&v| v == 0) {
            tags.push((
                "WB_RGGBLevels".to_string(),
                format!("{} {} {} {}", vals[0], vals[1], vals[2], vals[3]),
            ));

            // RGGB: R=vals[0], G1=vals[1], G2=vals[2], B=vals[3]
            let g = ((vals[1] as f64) + (vals[2] as f64)) / 2.0;
            if g > 0.0 {
                let red = (vals[0] as f64) / g;
                let blue = (vals[3] as f64) / g;
                tags.push(("RedBalance".to_string(), format!("{:.6}", red)));
                tags.push(("BlueBalance".to_string(), format!("{:.6}", blue)));
            }
        }
    } else if !is_nrw && version == "0103" && data.len() >= 20 + 8 {
        // Version 0103: RGBG at offset 20 (D70/D70s) - non-NRW only
        let mut cursor = Cursor::new(&data[20..]);
        let vals: [u16; 4] = [
            cursor.read_u16::<BigEndian>().unwrap_or(0),
            cursor.read_u16::<BigEndian>().unwrap_or(0),
            cursor.read_u16::<BigEndian>().unwrap_or(0),
            cursor.read_u16::<BigEndian>().unwrap_or(0),
        ];

        if !vals.iter().all(|&v| v == 0) {
            tags.push((
                "WB_RGBGLevels".to_string(),
                format!("{} {} {} {}", vals[0], vals[1], vals[2], vals[3]),
            ));

            // RGBG: R=vals[0], G1=vals[1], B=vals[2], G2=vals[3]
            let g = ((vals[1] as f64) + (vals[3] as f64)) / 2.0;
            if g > 0.0 {
                let red = (vals[0] as f64) / g;
                let blue = (vals[2] as f64) / g;
                tags.push(("RedBalance".to_string(), format!("{:.6}", red)));
                tags.push(("BlueBalance".to_string(), format!("{:.6}", blue)));
            }
        }
    } else if !is_nrw && version.starts_with("02") {
        // Version 02xx: Various encrypted ColorBalance structures
        // Different versions have different DecryptStart, DirOffset, and WB channel orders
        // Byte order follows the makernote's byte order (passed as endian parameter)
        if let (Some(ser), Some(count)) = (serial, shutter_count) {
            // Determine decrypt_start, dir_offset, wb_name, is_grbg based on version
            let (decrypt_start, dir_offset, wb_name, is_grbg): (usize, usize, &str, bool) =
                match version {
                    // ColorBalance0205: D50
                    "0205" => (4, 14, "WB_RGGBLevels", false),
                    // ColorBalance02: D2X/D2Xs(0204), D2Hs(0206), D200(0207), D40/D40X/D80(0208), D60(0210)
                    "0204" | "0206" | "0207" | "0208" | "0210" => (284, 6, "WB_RGGBLevels", false),
                    // ColorBalance0209: D3/D3X/D300/D700(0209), D300S(0212), D3S(0214)
                    "0209" | "0212" | "0214" => (284, 10, "WB_GRBGLevels", true),
                    // ColorBalance0211: D90, D5000
                    "0211" => (284, 16, "WB_GRBGLevels", true),
                    // ColorBalance0213: D3000
                    "0213" => (284, 10, "WB_RGGBLevels", false),
                    // ColorBalance0215: D3100(0215), D7000/D5100(0216), D4/D600/D800/D3200(0217)
                    "0215" | "0216" | "0217" => (284, 4, "WB_GRBGLevels", true),
                    // ColorBalance0219: D5300(0219), D3300(0221), D4S(0222), D750/D810(0223), D3400-D7200(0224)
                    "0219" | "0221" | "0222" | "0223" | "0224" => (4, 0x7c, "WB_RGGBLevels", false),
                    // Unknown versions - skip WB extraction
                    _ => return tags,
                };

            let wb_offset = decrypt_start + dir_offset;
            if data.len() < wb_offset + 8 {
                return tags;
            }

            // Make mutable copy for decryption
            let mut decrypted = data.to_vec();

            // Decrypt starting at DecryptStart
            nikon_decrypt(ser, count, &mut decrypted, decrypt_start);

            // Read WB levels at DirOffset from DecryptStart
            // Format: 4 x int16u, follows makernote byte order
            let mut cursor = Cursor::new(&decrypted[wb_offset..]);
            let vals: [u16; 4] = match endian {
                Endianness::Little => [
                    cursor.read_u16::<LittleEndian>().unwrap_or(0),
                    cursor.read_u16::<LittleEndian>().unwrap_or(0),
                    cursor.read_u16::<LittleEndian>().unwrap_or(0),
                    cursor.read_u16::<LittleEndian>().unwrap_or(0),
                ],
                Endianness::Big => [
                    cursor.read_u16::<BigEndian>().unwrap_or(0),
                    cursor.read_u16::<BigEndian>().unwrap_or(0),
                    cursor.read_u16::<BigEndian>().unwrap_or(0),
                    cursor.read_u16::<BigEndian>().unwrap_or(0),
                ],
            };

            // Skip if all zeros
            if !vals.iter().all(|&v| v == 0) {
                tags.push((
                    wb_name.to_string(),
                    format!("{} {} {} {}", vals[0], vals[1], vals[2], vals[3]),
                ));

                // Compute RedBalance and BlueBalance
                // Channel order varies: RGGB vs GRBG
                let (r, g1, g2, b) = if is_grbg {
                    // GRBG: G=vals[0], R=vals[1], B=vals[2], G=vals[3]
                    (vals[1], vals[0], vals[3], vals[2])
                } else {
                    // RGGB: R=vals[0], G1=vals[1], G2=vals[2], B=vals[3]
                    (vals[0], vals[1], vals[2], vals[3])
                };

                let g = ((g1 as f64) + (g2 as f64)) / 2.0;
                if g > 0.0 {
                    let red = (r as f64) / g;
                    let blue = (b as f64) / g;
                    tags.push(("RedBalance".to_string(), format!("{:.6}", red)));
                    tags.push(("BlueBalance".to_string(), format!("{:.6}", blue)));
                }
            }
        }
    }

    // Check for NRW ColorBalance format (starts with "NRW ")
    // ColorBalanceB (version 0100): P6000 - uses large offsets
    // ColorBalanceC (version 0101+): P1000, P7000, P7100, B700 - uses small offsets
    if is_nrw {
        // Choose offset table based on version
        let wb_offsets: &[(&str, usize)] = if version == "0100" {
            // ColorBalanceB for P6000
            &[
                ("WB_RGGBLevels", 0x13e8),
                ("WB_RGGBLevelsDaylight", 0x13f8),
                ("WB_RGGBLevelsCloudy", 0x1408),
                ("WB_RGGBLevelsTungsten", 0x1428),
                ("WB_RGGBLevelsFluorescentW", 0x1438),
                ("WB_RGGBLevelsFlash", 0x1448),
                ("WB_RGGBLevelsCustom", 0x1468),
                ("WB_RGGBLevelsAuto", 0x1478),
            ]
        } else {
            // ColorBalanceC for P7000, P7100, P1000, B700
            &[
                ("WB_RGGBLevels", 0x0038),
                ("WB_RGGBLevelsDaylight", 0x004c),
                ("WB_RGGBLevelsCloudy", 0x0060),
                ("WB_RGGBLevelsTungsten", 0x0088),
                ("WB_RGGBLevelsFluorescentW", 0x009c),
                ("WB_RGGBLevelsFluorescentN", 0x00b0),
                ("WB_RGGBLevelsFluorescentD", 0x00c4),
                ("WB_RGGBLevelsHTMercury", 0x00d8),
                ("WB_RGGBLevelsCustom", 0x0100),
                ("WB_RGGBLevelsAuto", 0x0114),
            ]
        };

        // Format: 4 x int32u at each offset, little-endian
        // ValueConv: vals[0]*=2; vals[3]*=2
        for (name, offset) in wb_offsets {
            if data.len() >= offset + 16 {
                let mut cursor = Cursor::new(&data[*offset..]);
                let mut vals: [u32; 4] = [
                    cursor.read_u32::<LittleEndian>().unwrap_or(0),
                    cursor.read_u32::<LittleEndian>().unwrap_or(0),
                    cursor.read_u32::<LittleEndian>().unwrap_or(0),
                    cursor.read_u32::<LittleEndian>().unwrap_or(0),
                ];

                // Apply ValueConv: vals[0] *= 2; vals[3] *= 2
                vals[0] = vals[0].saturating_mul(2);
                vals[3] = vals[3].saturating_mul(2);

                // Skip if all zeros (custom preset not used)
                if !vals.iter().all(|&v| v == 0) {
                    tags.push((
                        name.to_string(),
                        format!("{} {} {} {}", vals[0], vals[1], vals[2], vals[3]),
                    ));
                }
            }
        }

        // BlackLevel at offset 0x0020 (int16u) - only for ColorBalanceC
        if version != "0100" && data.len() >= 0x0022 {
            let black_level = u16::from_le_bytes([data[0x0020], data[0x0021]]);
            tags.push(("BlackLevel".to_string(), black_level.to_string()));
        }
    }

    // Note: For encrypted versions (02xx), RedBalance/BlueBalance should come from
    // WB_RBLevels (0x000C) which already contains the correct R/G and B/G ratios

    tags
}

/// Parse CropHiSpeed tag data (tag 0x001B)
/// Format: 7 x int16u: Mode, OrigWidth, OrigHeight, CropWidth, CropHeight, CropX, CropY
fn parse_crop_hi_speed(data: &[u8], endian: Endianness) -> Vec<(String, String)> {
    let mut tags = Vec::new();

    if data.len() < 14 {
        return tags;
    }

    let mut cursor = Cursor::new(data);

    // Read all 7 values
    let read_u16 = |cursor: &mut Cursor<&[u8]>| -> u16 {
        match endian {
            Endianness::Little => cursor.read_u16::<LittleEndian>().unwrap_or(0),
            Endianness::Big => cursor.read_u16::<BigEndian>().unwrap_or(0),
        }
    };

    let mode = read_u16(&mut cursor);
    let orig_width = read_u16(&mut cursor);
    let orig_height = read_u16(&mut cursor);
    let crop_width = read_u16(&mut cursor);
    let crop_height = read_u16(&mut cursor);
    let crop_x = read_u16(&mut cursor);
    let crop_y = read_u16(&mut cursor);

    let mode_str = match mode {
        0 => "Off",
        1 => "1.3x Crop",
        2 => "DX Crop",
        3 => "5:4 Crop",
        4 => "3:2 Crop",
        6 => "16:9 Crop",
        8 => "2.7x Crop",
        9 => "DX Movie Crop",
        10 => "1.3x Movie Crop",
        11 => "FX Uncropped",
        12 => "DX Uncropped",
        15 => "1.5x Movie Crop",
        17 => "1:1 Crop",
        _ => "Unknown",
    };

    // ExifTool format: "Mode (OrigWxOrigH cropped to CropWxCropH at pixel X,Y)"
    let formatted = format!(
        "{} ({}x{} cropped to {}x{} at pixel {},{})",
        mode_str, orig_width, orig_height, crop_width, crop_height, crop_x, crop_y
    );
    tags.push(("CropHiSpeed".to_string(), formatted));

    tags
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

// ImageAuthentication (tag 0x0020): Nikon.pm / nikonmn_int.cpp nikonOffOn[]
define_tag_decoder! {
    image_authentication,
    type: u8,
    both: {
        0 => "Off",
        1 => "On",
    }
}

// SilentPhotography (tag 0x00BF): Nikon.pm
define_tag_decoder! {
    silent_photography,
    both: {
        0 => "Off",
        1 => "On",
    }
}

// CropHiSpeed (tag 0x001A): Nikon.pm cropHiSpeed hash
define_tag_decoder! {
    crop_hi_speed,
    both: {
        0 => "Off",
        1 => "1.3x Crop",
        2 => "DX Crop",
        3 => "5:4 Crop",
        4 => "3:2 Crop",
        6 => "16:9 Crop",
        8 => "2.7x Crop",
        9 => "DX Movie 16:9 Crop",
        10 => "1.3x Movie Crop",
        11 => "FX Uncropped",
        12 => "DX Uncropped",
        13 => "2.8x Movie Crop",
        14 => "1.4x Movie Crop",
        15 => "1.5x Movie Crop",
        17 => "FX 1:1 Crop",
        18 => "DX 1:1 Crop",
    }
}

// VibrationReduction: Nikon.pm VRInfo subdirectory
define_tag_decoder! {
    vibration_reduction,
    type: u8,
    both: {
        0 => "n/a",
        1 => "On",
        2 => "Off",
    }
}

/// Decode Flash Type ASCII value (tag 0x0009) - ExifTool format
/// From Nikon.pm - tag 0x0009 is ASCII string type
pub fn decode_flash_type_exiftool(value: &str) -> String {
    let val = value.trim().to_uppercase();
    let result = match val.as_str() {
        "" => "",
        "NEW" => "New",
        "NORMAL" => "Built-in",
        "EXTERNAL" | "STROBE" => "External",
        // Handle specific "Optional,AA" -> "Optional,Aa" case
        "OPTIONAL,AA" => "Optional,Aa",
        // Handle "NEW_TTL" -> "New_TTL" case
        "NEW_TTL" => "New_TTL",
        _ => return value.trim().to_string(),
    };
    result.to_string()
}

/// Decode Flash Type ASCII value (tag 0x0009) - exiv2 format
/// exiv2 treats this as plain ASCII string, using ExifTool interpretations
pub fn decode_flash_type_exiv2(value: &str) -> String {
    decode_flash_type_exiftool(value)
}

/// Decode Color Mode ASCII value (tag 0x0003) - ExifTool format
/// From Nikon.pm - tag 0x0003 is ASCII string type
/// This is already handled in decode_nikon_ascii_value, but provided here for completeness
pub fn decode_color_mode_exiftool(value: &str) -> String {
    let val = value.trim().to_uppercase();
    let result = match val.as_str() {
        "COLOR" | "COLOUR" => "Color",
        "B&W" | "B & W" | "BLACK & WHITE" => "Black & White",
        _ => return value.trim().to_string(),
    };
    result.to_string()
}

/// Decode Color Mode ASCII value (tag 0x0003) - exiv2 format
/// exiv2 treats this as plain ASCII string, using ExifTool interpretations
pub fn decode_color_mode_exiv2(value: &str) -> String {
    decode_color_mode_exiftool(value)
}

/// Decode Noise Reduction ASCII value (tag 0x0095) - ExifTool format
/// From Nikon.pm - tag 0x0095 is ASCII string type
/// Values: "Off" or "FPNR" (long exposure noise reduction)
pub fn decode_noise_reduction_exiftool(value: &str) -> String {
    let val = value.trim().to_uppercase();
    let result = match val.as_str() {
        "OFF" => "Off",
        "FPNR" => "FPNR", // ExifTool outputs as-is
        "ON" => "On",
        _ => return value.trim().to_string(),
    };
    result.to_string()
}

/// Decode Noise Reduction ASCII value (tag 0x0095) - exiv2 format
/// exiv2 treats this as plain ASCII string, using ExifTool interpretations
pub fn decode_noise_reduction_exiv2(value: &str) -> String {
    decode_noise_reduction_exiftool(value)
}

/// Decode Image Processing ASCII value (tag 0x0019/0x001A) - ExifTool format
/// From Nikon.pm - tag is ASCII string type
pub fn decode_image_processing_exiftool(value: &str) -> String {
    let val = value.trim().to_uppercase();
    let result = match val.as_str() {
        "OFF" => "Off",
        "ON" => "On",
        "MINIMAL" => "Minimal",
        "AUTO" => "Auto",
        _ => return value.trim().to_string(),
    };
    result.to_string()
}

/// Decode Image Processing ASCII value - exiv2 format
/// exiv2 treats this as plain ASCII string, using ExifTool interpretations
pub fn decode_image_processing_exiv2(value: &str) -> String {
    decode_image_processing_exiftool(value)
}

/// Decode Lens Type bitfield (tag 0x0083) - ExifTool format
/// Bit 0 = MF, Bit 1 = D, Bit 2 = G, Bit 3 = VR, Bit 4 = 1, Bit 5 = FT-1, Bit 6 = E, Bit 7 = AF-P
/// Special handling: "D G" -> "G", "E" replaces "G", "1" goes first, "FT-1" goes last
pub fn decode_lens_type_exiftool(value: u8) -> String {
    if value == 0 {
        return "AF".to_string();
    }

    let mut features = Vec::new();

    // Bit checks
    let has_mf = value & 0x01 != 0;
    let has_d = value & 0x02 != 0;
    let has_g = value & 0x04 != 0;
    let has_vr = value & 0x08 != 0;
    let has_1 = value & 0x10 != 0;
    let has_ft1 = value & 0x20 != 0;
    let has_e = value & 0x40 != 0;
    let has_afp = value & 0x80 != 0;

    // Special handling for "1" - goes first
    if has_1 {
        features.push("1");
    }

    // Special handling for "E" - replaces "G" at start
    if has_e {
        features.push("E");
    } else if has_g {
        // Only add "G" if not "E"
        features.push("G");
    } else if has_d {
        // Only add "D" if not part of "D G" combo
        features.push("D");
    }

    if has_mf {
        features.push("MF");
    }
    if has_vr {
        features.push("VR");
    }
    if has_afp {
        features.push("AF-P");
    }

    // "FT-1" goes last
    if has_ft1 {
        features.push("FT-1");
    }

    if features.is_empty() {
        "AF".to_string()
    } else {
        features.join(" ")
    }
}

/// Decode Lens Type bitfield (tag 0x0083) - exiv2 format
/// exiv2 only decodes bits 0-3 (MF, D, G, VR)
pub fn decode_lens_type_exiv2(value: u8) -> String {
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

    if features.is_empty() {
        String::new()
    } else {
        features.join(" ")
    }
}

/// Format flash compensation value like ExifTool PrintFraction
/// - 0 is displayed as just "0"
/// - Values like -0.67 become "-2/3", -0.33 becomes "-1/3", etc.
fn format_flash_compensation(ev: f64) -> String {
    if ev.abs() < 0.001 {
        return "0".to_string();
    }

    let sign = if ev < 0.0 { "-" } else { "+" };
    let abs_val = ev.abs();
    let frac = abs_val.fract();
    let whole = abs_val.trunc() as i32;

    // Check for common EV fractions (1/3, 2/3, 1/2)
    if (frac - 0.33).abs() < 0.05 || (frac - 0.34).abs() < 0.05 {
        // 1/3 step
        if whole == 0 {
            format!("{}1/3", sign)
        } else {
            format!("{}{} 1/3", sign, whole)
        }
    } else if (frac - 0.67).abs() < 0.05 || (frac - 0.66).abs() < 0.05 {
        // 2/3 step
        if whole == 0 {
            format!("{}2/3", sign)
        } else {
            format!("{}{} 2/3", sign, whole)
        }
    } else if (frac - 0.5).abs() < 0.05 {
        // 1/2 step
        if whole == 0 {
            format!("{}1/2", sign)
        } else {
            format!("{}{} 1/2", sign, whole)
        }
    } else if frac < 0.05 {
        // Whole number
        format!("{}{}", sign, whole)
    } else {
        // Fallback to decimal
        format!("{:+.1}", ev)
    }
}

/// Format a lens focal length or aperture value like ExifTool
/// - If value is an integer, show without decimal (e.g., 11)
/// - If value has fractional part, show with one decimal (e.g., 27.5)
fn format_lens_value(val: f64) -> String {
    // Check if the value is effectively an integer
    let rounded = val.round();
    if (val - rounded).abs() < 0.01 {
        format!("{:.0}", rounded)
    } else {
        format!("{:.1}", val)
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

    // Format focal lengths using smart formatting (no decimal if integer)
    let min_f = format_lens_value(min_focal);
    let max_f = format_lens_value(max_focal);

    // Format the lens description
    if (min_focal - max_focal).abs() < 0.1 {
        // Prime lens
        Some(format!("{}mm f/{}", min_f, format_lens_value(min_aperture)))
    } else {
        // Zoom lens
        if (min_aperture - max_aperture).abs() < 0.1 {
            // Constant aperture
            Some(format!(
                "{}-{}mm f/{}",
                min_f,
                max_f,
                format_lens_value(min_aperture)
            ))
        } else {
            // Variable aperture
            Some(format!(
                "{}-{}mm f/{}-{}",
                min_f,
                max_f,
                format_lens_value(min_aperture),
                format_lens_value(max_aperture)
            ))
        }
    }
}

/// Parse Nikon maker notes
pub fn parse_nikon_maker_notes(
    data: &[u8],
    endian: Endianness,
    model: Option<&str>,
    tiff_data: Option<&[u8]>,
    tiff_offset: usize,
    makernote_file_offset: Option<usize>,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    // Variables for LensID composite lookup
    let mut lens_type_raw: Option<u8> = None;
    let mut lens_id_bytes: Option<[u8; 7]> = None;

    // Variables for LensData decryption
    let mut serial_number: Option<u32> = None;
    let mut shutter_count_val: Option<u32> = None;
    // Store LensData for deferred processing (needs serial/shutter_count which come later)
    let mut lens_data_bytes: Option<Vec<u8>> = None;
    // Store ColorBalance for deferred processing (version 02xx needs serial/shutter_count for decryption)
    let mut color_balance_bytes: Option<Vec<u8>> = None;
    // Store ShotInfo for deferred processing (version 02xx needs serial/shutter_count for decryption)
    let mut shot_info_bytes: Option<Vec<u8>> = None;

    // Nikon maker notes often start with "Nikon\0" header
    if data.len() < 18 {
        return Ok(tags);
    }

    let base_offset;
    let ifd_offset;
    let maker_endian;
    // Type 1/2 maker notes (no Nikon header): offsets are relative to file's TIFF header
    // Type 3 maker notes (with "Nikon\0" header): offsets are relative to internal TIFF header
    let is_type3;

    // Check for Nikon Type 3 header: "Nikon\0" + version + TIFF header
    if data.starts_with(b"Nikon\0") {
        // Structure: "Nikon\0" (6 bytes) + version (4 bytes) + TIFF header
        base_offset = 10; // Start of TIFF header (after "Nikon\0" + version)
        is_type3 = true;

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
        // No Nikon header (Type 1/2) - use the data as-is
        // Offsets in IFD entries are relative to file's TIFF header, not maker notes
        base_offset = 0;
        maker_endian = endian;
        ifd_offset = 0;
        is_type3 = false;
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
                1 => count as usize,      // BYTE
                2 => count as usize,      // ASCII
                3 => count as usize * 2,  // SHORT
                4 => count as usize * 4,  // LONG
                5 => count as usize * 8,  // RATIONAL
                7 => count as usize,      // UNDEFINED
                8 => count as usize * 2,  // SSHORT (signed short)
                9 => count as usize * 4,  // SLONG (signed long)
                10 => count as usize * 8, // SRATIONAL
                _ => 0,
            };

            // Determine if value is inline or at offset
            let value_bytes = if value_size <= 4 {
                // Inline value in the value_offset field
                match maker_endian {
                    Endianness::Little => value_offset.to_le_bytes().to_vec(),
                    Endianness::Big => value_offset.to_be_bytes().to_vec(),
                }
            } else if is_type3 {
                // Type 3: Value at offset relative to maker notes' internal TIFF header
                let abs_offset = base_offset + value_offset as usize;
                if abs_offset + value_size <= data.len() {
                    data[abs_offset..abs_offset + value_size].to_vec()
                } else if let (Some(tiff), Some(mn_offset)) = (tiff_data, makernote_file_offset) {
                    // Some Nikon cameras store tag data past the declared maker notes boundary
                    // value_offset is still relative to maker notes' internal TIFF header (at base_offset)
                    // So actual file offset = makernote_file_offset + base_offset + value_offset
                    let file_offset = mn_offset + base_offset + value_offset as usize;
                    if file_offset + value_size <= tiff.len() {
                        tiff[file_offset..file_offset + value_size].to_vec()
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            } else if let Some(tiff) = tiff_data {
                // Type 1/2: Value at offset relative to file's TIFF header (not maker notes)
                // tiff_data includes "Exif\0\0" prefix, so add tiff_offset to get actual position
                let file_offset = tiff_offset + value_offset as usize;
                if file_offset + value_size <= tiff.len() {
                    tiff[file_offset..file_offset + value_size].to_vec()
                } else {
                    continue;
                }
            } else {
                continue;
            };

            // Parse the value based on type
            let value = match tag_type {
                1 => {
                    // BYTE
                    let bytes = value_bytes[..count as usize].to_vec();
                    // Apply decoder for specific tags
                    if tag_id == NIKON_LENS_TYPE && !bytes.is_empty() {
                        // Store raw byte for LensID composite lookup
                        lens_type_raw = Some(bytes[0]);
                        ExifValue::Ascii(decode_lens_type_exiftool(bytes[0]))
                    } else if tag_id == NIKON_FLASH_MODE && !bytes.is_empty() {
                        ExifValue::Ascii(decode_flash_mode_exiftool(bytes[0]))
                    } else if tag_id == NIKON_IMAGE_AUTHENTICATION && !bytes.is_empty() {
                        ExifValue::Ascii(decode_image_authentication_exiftool(bytes[0]).to_string())
                    } else if tag_id == NIKON_SHOOTING_MODE && !bytes.is_empty() {
                        // ShootingMode can be stored as Byte on older cameras
                        ExifValue::Ascii(decode_shooting_mode_exiftool(bytes[0] as u16))
                    } else if tag_id == NIKON_IMAGE_PROCESSING {
                        // ImageProcessing can be stored as BYTE
                        // If 1 byte, output as decimal; otherwise decode as ASCII
                        if bytes.len() == 1 {
                            ExifValue::Ascii(bytes[0].to_string())
                        } else {
                            let s = String::from_utf8_lossy(&bytes)
                                .trim_end_matches('\0')
                                .to_string();
                            ExifValue::Ascii(decode_image_processing_exiftool(&s))
                        }
                    } else {
                        ExifValue::Byte(bytes)
                    }
                }
                2 => {
                    // ASCII
                    let s = String::from_utf8_lossy(&value_bytes[..count as usize])
                        .trim_end_matches('\0')
                        .to_string();

                    // Capture SerialNumber for LensData decryption
                    if tag_id == NIKON_SERIAL_NUMBER {
                        // Try to parse as u32, otherwise use fallback
                        // D50 uses 0x22 as fallback, others use 0x60 (per ExifTool)
                        let fallback = if model.is_some_and(|m| m.contains("D50")) {
                            0x22
                        } else {
                            0x60
                        };
                        serial_number = s.trim().parse::<u32>().ok().or(Some(fallback));
                    }

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
                    // Special handling for ISO tag - extract the actual ISO value
                    // ExifTool: first value 1 means "Hi ISO" mode, prefix "Hi " to value
                    if tag_id == NIKON_ISO_SETTING && values.len() >= 2 {
                        let iso = if values[1] > 0 { values[1] } else { values[0] };
                        if values[0] == 1 && iso > 0 {
                            // Hi ISO mode - output as "Hi XXXXX"
                            ExifValue::Ascii(format!("Hi {}", iso))
                        } else {
                            ExifValue::Short(vec![iso])
                        }
                    } else if tag_id == NIKON_NEF_BIT_DEPTH && !values.is_empty() {
                        // NEFBitDepth: ExifTool outputs just the first value as the bit depth
                        ExifValue::Short(vec![values[0]])
                    } else if values.len() == 1 {
                        // Apply value decoders for single SHORT values
                        let v = values[0];
                        let decoded = match tag_id {
                            NIKON_ACTIVE_D_LIGHTING => {
                                Some(decode_active_d_lighting_exiftool(v).to_string())
                            }
                            NIKON_COLOR_SPACE => Some(decode_color_space_exiftool(v).to_string()),
                            NIKON_VIGNETTE_CONTROL => {
                                Some(decode_vignette_control_exiftool(v).to_string())
                            }
                            NIKON_HIGH_ISO_NOISE_REDUCTION => {
                                Some(decode_high_iso_noise_reduction_exiftool(v).to_string())
                            }
                            NIKON_DATE_STAMP_MODE => {
                                Some(decode_date_stamp_mode_exiftool(v).to_string())
                            }
                            NIKON_NEF_COMPRESSION => {
                                Some(decode_nef_compression_exiftool(v).to_string())
                            }
                            NIKON_RETOUCH_HISTORY => {
                                Some(decode_retouch_history_exiftool(v).to_string())
                            }
                            NIKON_SHOOTING_MODE => Some(decode_shooting_mode_exiftool(v)),
                            NIKON_AUTO_BRACKET_RELEASE => {
                                Some(decode_auto_bracket_release_exiftool(v).to_string())
                            }
                            NIKON_SILENT_PHOTOGRAPHY => {
                                Some(decode_silent_photography_exiftool(v).to_string())
                            }
                            NIKON_SHUTTER_MODE => Some(decode_shutter_mode_exiftool(v).to_string()),
                            _ => None,
                        };

                        if let Some(s) = decoded {
                            ExifValue::Ascii(s)
                        } else {
                            ExifValue::Short(values)
                        }
                    } else if tag_id == NIKON_CROP_HI_SPEED && values.len() >= 7 {
                        // CropHiSpeed is a 7-element array:
                        // [0]=Mode, [1]=OrigWidth, [2]=OrigHeight, [3]=CropWidth, [4]=CropHeight, [5]=CropX, [6]=CropY
                        let mode_str = decode_crop_hi_speed_exiftool(values[0]);
                        let formatted = format!(
                            "{} ({}x{} cropped to {}x{} at pixel {},{})",
                            mode_str,
                            values[1],
                            values[2],
                            values[3],
                            values[4],
                            values[5],
                            values[6]
                        );
                        ExifValue::Ascii(formatted)
                    } else if tag_id == NIKON_WB_RB_LEVELS && values.len() >= 2 {
                        // WB_RBLevels as SHORT: values are R and B relative to 256
                        // RedBalance = R/256, BlueBalance = B/256
                        let red = values[0] as f64 / 256.0;
                        let blue = values[1] as f64 / 256.0;
                        // Add RedBalance and BlueBalance as separate tags
                        tags.insert(
                            0xFE00,
                            MakerNoteTag::new(
                                0xFE00,
                                Some("RedBalance"),
                                ExifValue::Ascii(format!("{:.6}", red)),
                            ),
                        );
                        tags.insert(
                            0xFE01,
                            MakerNoteTag::new(
                                0xFE01,
                                Some("BlueBalance"),
                                ExifValue::Ascii(format!("{:.6}", blue)),
                            ),
                        );
                        // Return WB_RBLevels as space-separated values
                        ExifValue::Ascii(
                            values
                                .iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<_>>()
                                .join(" "),
                        )
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
                        // Capture ShutterCount for LensData decryption
                        if v != 4294965247 {
                            shutter_count_val = Some(v);
                        }
                        if v == 4294965247 {
                            ExifValue::Ascii("n/a".to_string())
                        } else {
                            ExifValue::Long(values)
                        }
                    } else if tag_id == NIKON_IMAGE_PROCESSING && values.len() == 1 {
                        // ImageProcessing stored as LONG - output as decimal string
                        ExifValue::Ascii(values[0].to_string())
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

                    // Apply decoder for specific tags
                    if tag_id == NIKON_LENS {
                        if let Some(formatted) = format_lens_info(&values) {
                            ExifValue::Ascii(formatted)
                        } else {
                            ExifValue::Rational(values)
                        }
                    } else if tag_id == NIKON_SENSOR_PIXEL_SIZE && values.len() == 2 {
                        // SensorPixelSize: format as "7.8 x 7.8 um"
                        let x = if values[0].1 != 0 {
                            values[0].0 as f64 / values[0].1 as f64
                        } else {
                            values[0].0 as f64
                        };
                        let y = if values[1].1 != 0 {
                            values[1].0 as f64 / values[1].1 as f64
                        } else {
                            values[1].0 as f64
                        };
                        ExifValue::Ascii(format!("{} x {} um", x, y))
                    } else if tag_id == NIKON_WB_RB_LEVELS && values.len() >= 2 {
                        // WB_RBLevels: First two rationals are RedBalance and BlueBalance
                        // Compute the values and add as extra tags
                        let red = if values[0].1 != 0 {
                            values[0].0 as f64 / values[0].1 as f64
                        } else {
                            values[0].0 as f64
                        };
                        let blue = if values[1].1 != 0 {
                            values[1].0 as f64 / values[1].1 as f64
                        } else {
                            values[1].0 as f64
                        };
                        // Add RedBalance and BlueBalance as separate tags
                        tags.insert(
                            0xFE00,
                            MakerNoteTag::new(
                                0xFE00,
                                Some("RedBalance"),
                                ExifValue::Ascii(format!("{:.6}", red)),
                            ),
                        );
                        tags.insert(
                            0xFE01,
                            MakerNoteTag::new(
                                0xFE01,
                                Some("BlueBalance"),
                                ExifValue::Ascii(format!("{:.6}", blue)),
                            ),
                        );
                        // Also return the WB_RBLevels as formatted string
                        let formatted: Vec<String> = values
                            .iter()
                            .map(|(n, d)| format_rational_like_exiftool(*n, *d))
                            .collect();
                        ExifValue::Ascii(formatted.join(" "))
                    } else {
                        ExifValue::Rational(values)
                    }
                }
                7 => {
                    // UNDEFINED - handle packed rational values specially
                    match tag_id {
                        // MakerNoteVersion - format as "X.Y" or "X.YZ"
                        // Can be either ASCII "0200" or binary [0, 2, 0, 0]
                        NIKON_VERSION => {
                            if value_bytes.len() >= 4 {
                                // Check if binary format (first byte is 0-9, not '0'-'9')
                                let is_binary = value_bytes[0] <= 9;

                                if is_binary {
                                    // Binary format: [major_tens, major_ones, minor_tens, minor_ones]
                                    // e.g., [0, 2, 1, 0] -> version 2.10
                                    let major = value_bytes[0] * 10 + value_bytes[1];
                                    let minor = value_bytes[2] * 10 + value_bytes[3];
                                    // Format as "X.YZ" - keep two digits for minor
                                    let formatted = format!("{}.{:02}", major, minor);
                                    ExifValue::Ascii(formatted)
                                } else if let Ok(s) = std::str::from_utf8(&value_bytes[..4]) {
                                    // ASCII format: "0210" -> version 2.10
                                    if s.len() == 4 && s.chars().all(|c| c.is_ascii_digit()) {
                                        let major: u8 = s[0..2].parse().unwrap_or(0);
                                        let minor: u8 = s[2..4].parse().unwrap_or(0);
                                        // Format as "X.YZ" - keep two digits for minor
                                        let formatted = format!("{}.{:02}", major, minor);
                                        ExifValue::Ascii(formatted)
                                    } else {
                                        // Fallback: return trimmed string
                                        ExifValue::Ascii(s.trim().to_string())
                                    }
                                } else {
                                    // Invalid - return raw bytes
                                    ExifValue::Undefined(value_bytes)
                                }
                            } else {
                                ExifValue::Undefined(value_bytes)
                            }
                        }
                        // Signed packed rationals (a * b/c where a,b,c are signed)
                        NIKON_PROGRAM_SHIFT
                        | NIKON_EXPOSURE_DIFFERENCE
                        | NIKON_FLASH_EXPOSURE_COMP
                        | NIKON_EXTERNAL_FLASH_EXPOSURE_COMP
                        | NIKON_FLASH_EXPOSURE_BRACKET_VALUE
                        | NIKON_EXPOSURE_TUNING => {
                            if let Some(val) = decode_packed_rational_signed(&value_bytes) {
                                // Store as SRational for proper JSON number output
                                // Convert to rational with denominator 100 for precision
                                let int_val = (val * 100.0).round() as i32;
                                ExifValue::SRational(vec![(int_val, 100)])
                            } else {
                                ExifValue::Undefined(value_bytes)
                            }
                        }
                        // Unsigned packed rationals (a * b/c where a,b,c are unsigned)
                        NIKON_LENS_F_STOPS => {
                            if let Some(val) = decode_packed_rational_unsigned(&value_bytes) {
                                // Store as Rational for proper JSON number output
                                let int_val = (val * 100.0).round() as u32;
                                ExifValue::Rational(vec![(int_val, 100)])
                            } else {
                                ExifValue::Undefined(value_bytes)
                            }
                        }
                        // ImageProcessing can be stored as UNDEFINED
                        // If 1 byte (count==1), output as decimal; otherwise decode as ASCII
                        NIKON_IMAGE_PROCESSING => {
                            if count == 1 {
                                // Single byte - output as decimal (matches ExifTool)
                                ExifValue::Ascii(value_bytes[0].to_string())
                            } else {
                                // Multiple bytes - decode as ASCII string
                                let s = String::from_utf8_lossy(&value_bytes[..count as usize])
                                    .trim_end_matches('\0')
                                    .to_string();
                                ExifValue::Ascii(decode_image_processing_exiftool(&s))
                            }
                        }
                        _ => ExifValue::Undefined(value_bytes),
                    }
                }
                8 => {
                    // SSHORT (signed short)
                    let mut values = Vec::new();
                    let mut cursor = Cursor::new(&value_bytes);
                    for _ in 0..count {
                        if let Ok(v) = match maker_endian {
                            Endianness::Little => cursor.read_i16::<LittleEndian>(),
                            Endianness::Big => cursor.read_i16::<BigEndian>(),
                        } {
                            values.push(v);
                        } else {
                            break;
                        }
                    }
                    if values.len() == 1 {
                        ExifValue::Short(vec![values[0] as u16])
                    } else {
                        // Multiple values - format as space-separated
                        let formatted: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                        ExifValue::Ascii(formatted.join(" "))
                    }
                }
                9 => {
                    // SLONG (signed long)
                    let mut values = Vec::new();
                    let mut cursor = Cursor::new(&value_bytes);
                    for _ in 0..count {
                        if let Ok(v) = match maker_endian {
                            Endianness::Little => cursor.read_i32::<LittleEndian>(),
                            Endianness::Big => cursor.read_i32::<BigEndian>(),
                        } {
                            values.push(v);
                        } else {
                            break;
                        }
                    }
                    if values.len() == 1 {
                        ExifValue::Long(vec![values[0] as u32])
                    } else {
                        // Multiple values - format as space-separated
                        let formatted: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                        ExifValue::Ascii(formatted.join(" "))
                    }
                }
                10 => {
                    // SRATIONAL
                    let mut values = Vec::new();
                    let mut cursor = Cursor::new(&value_bytes);
                    for _ in 0..count {
                        let numerator = match maker_endian {
                            Endianness::Little => cursor.read_i32::<LittleEndian>(),
                            Endianness::Big => cursor.read_i32::<BigEndian>(),
                        };
                        let denominator = match maker_endian {
                            Endianness::Little => cursor.read_i32::<LittleEndian>(),
                            Endianness::Big => cursor.read_i32::<BigEndian>(),
                        };

                        if let (Ok(num), Ok(den)) = (numerator, denominator) {
                            values.push((num, den));
                        } else {
                            break;
                        }
                    }
                    ExifValue::SRational(values)
                }
                _ => {
                    // Unsupported type
                    continue;
                }
            };

            // For tag 0x0002 (ISO), output as "ISO" with the extracted value
            // Older cameras (D1, D1X, D100) don't have ISOInfo tag so this is the only source
            if tag_id == NIKON_ISO_SETTING {
                let should_output = match &value {
                    ExifValue::Short(vals) => !vals.is_empty() && vals[0] > 0,
                    ExifValue::Ascii(s) => !s.is_empty(), // Hi ISO mode
                    _ => false,
                };
                if should_output {
                    // Create synthetic ISO tag with ID 0xFFFF to avoid conflict
                    let iso_tag = MakerNoteTag::new(0xFFFF, Some("ISO"), value.clone());
                    tags.insert(0xFFFF, iso_tag);
                }
                continue;
            }

            // Special formatting for ISOSetting (0x0013) - output only second value
            // ExifTool: PrintConv => '$_=$val;s/^0 //;$_' (removes leading "0 ")
            if tag_id == NIKON_ISO_SETTING_2 {
                if let ExifValue::Short(vals) = &value {
                    if vals.len() >= 2 && vals[1] != 0 {
                        tags.insert(
                            tag_id,
                            MakerNoteTag::new(
                                tag_id,
                                get_nikon_tag_name(tag_id),
                                ExifValue::Short(vec![vals[1]]),
                            ),
                        );
                    }
                    // Skip if values are 0 0 (ExifTool outputs empty string)
                    continue;
                }
                // Skip undefined data (ExifTool outputs empty string for 00 00 00 00)
                if let ExifValue::Undefined(_) = &value {
                    continue;
                }
            }

            tags.insert(
                tag_id,
                MakerNoteTag::new(tag_id, get_nikon_tag_name(tag_id), value.clone()),
            );

            // Parse sub-structures and insert extracted fields as separate tags
            match tag_id {
                NIKON_ISO_INFO => {
                    if let ExifValue::Undefined(ref bytes) = value {
                        // Check if we have a "Hi" prefixed ISO from tag 0x0002
                        let has_hi_iso = tags.get(&0xFFFF).is_some_and(
                            |t| matches!(&t.value, ExifValue::Ascii(s) if s.starts_with("Hi ")),
                        );
                        for (name, val) in parse_iso_info(bytes) {
                            // Skip ISOInfo-derived ISO if we already have a "Hi" prefixed ISO
                            if name == "ISO" && has_hi_iso {
                                continue;
                            }
                            let tag_id = 0x9000 + tags.len() as u16;
                            let tag = if let Some((exiv2_group, exiv2_name)) =
                                get_exiv2_nikon_subfield(NIKON_ISO_INFO, &name)
                            {
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val.clone()),
                                    ExifValue::Ascii(val),
                                    exiv2_group,
                                    exiv2_name,
                                )
                            } else {
                                MakerNoteTag::new(
                                    tag_id,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val),
                                )
                            };
                            tags.insert(tag_id, tag);
                        }
                    }
                }
                NIKON_VR_INFO => {
                    if let ExifValue::Undefined(ref bytes) = value {
                        for (name, val) in parse_vr_info(bytes, maker_endian, model) {
                            let tag_id = 0x9100 + tags.len() as u16;
                            let tag = if let Some((exiv2_group, exiv2_name)) =
                                get_exiv2_nikon_subfield(NIKON_VR_INFO, &name)
                            {
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val.clone()),
                                    ExifValue::Ascii(val),
                                    exiv2_group,
                                    exiv2_name,
                                )
                            } else {
                                MakerNoteTag::new(
                                    tag_id,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val),
                                )
                            };
                            tags.insert(tag_id, tag);
                        }
                    }
                }
                NIKON_HDR_INFO => {
                    if let ExifValue::Undefined(ref bytes) = value {
                        for (name, val) in parse_hdr_info(bytes) {
                            let tag_id = 0x9180 + tags.len() as u16;
                            let tag = MakerNoteTag::new(
                                tag_id,
                                Some(Box::leak(name.into_boxed_str())),
                                ExifValue::Ascii(val),
                            );
                            tags.insert(tag_id, tag);
                        }
                    }
                }
                NIKON_PICTURE_CONTROL_DATA | NIKON_PICTURE_CONTROL_DATA_2 => {
                    if let ExifValue::Undefined(ref bytes) = value {
                        for (name, val) in parse_picture_control(bytes) {
                            let tag_id = 0x9200 + tags.len() as u16;
                            let tag = if let Some((exiv2_group, exiv2_name)) =
                                get_exiv2_nikon_subfield(NIKON_PICTURE_CONTROL_DATA, &name)
                            {
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val.clone()),
                                    ExifValue::Ascii(val),
                                    exiv2_group,
                                    exiv2_name,
                                )
                            } else {
                                MakerNoteTag::new(
                                    tag_id,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val),
                                )
                            };
                            tags.insert(tag_id, tag);
                        }
                    }
                }
                NIKON_DISTORTION_CONTROL => {
                    if let ExifValue::Undefined(ref bytes) = value {
                        for (name, val) in parse_distort_info(bytes) {
                            let tag_id = 0x9150 + tags.len() as u16;
                            let tag = MakerNoteTag::new(
                                tag_id,
                                Some(Box::leak(name.into_boxed_str())),
                                ExifValue::Ascii(val),
                            );
                            tags.insert(tag_id, tag);
                        }
                    }
                }
                NIKON_AF_TUNE => {
                    if let ExifValue::Undefined(ref bytes) = value {
                        for (name, val) in parse_af_tune(bytes) {
                            let tag_id = 0x9160 + tags.len() as u16;
                            let tag = MakerNoteTag::new(
                                tag_id,
                                Some(Box::leak(name.into_boxed_str())),
                                ExifValue::Ascii(val),
                            );
                            tags.insert(tag_id, tag);
                        }
                    }
                }
                NIKON_POWER_UP_TIME_2 => {
                    // PowerUpTime: 7 bytes = year(2) + month + day + hour + min + sec
                    if let ExifValue::Undefined(ref bytes) = value {
                        if bytes.len() >= 7 {
                            let year = match maker_endian {
                                Endianness::Little => u16::from_le_bytes([bytes[0], bytes[1]]),
                                Endianness::Big => u16::from_be_bytes([bytes[0], bytes[1]]),
                            };
                            let month = bytes[2];
                            let day = bytes[3];
                            let hour = bytes[4];
                            let min = bytes[5];
                            let sec = bytes[6];
                            let formatted = format!(
                                "{:04}:{:02}:{:02} {:02}:{:02}:{:02}",
                                year, month, day, hour, min, sec
                            );
                            let tag = MakerNoteTag::new(
                                tag_id,
                                Some("PowerUpTime"),
                                ExifValue::Ascii(formatted),
                            );
                            tags.insert(tag_id, tag);
                        }
                    }
                }
                NIKON_FLASH_INFO => {
                    if let ExifValue::Undefined(ref bytes) = value {
                        for (name, val) in parse_flash_info(bytes, model) {
                            let tag_id = 0x9300 + tags.len() as u16;
                            let tag = if let Some((exiv2_group, exiv2_name)) =
                                get_exiv2_nikon_subfield(NIKON_FLASH_INFO, &name)
                            {
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val.clone()),
                                    ExifValue::Ascii(val),
                                    exiv2_group,
                                    exiv2_name,
                                )
                            } else {
                                MakerNoteTag::new(
                                    tag_id,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val),
                                )
                            };
                            tags.insert(tag_id, tag);
                        }
                    }
                }
                NIKON_FACE_DETECT => {
                    if let ExifValue::Undefined(ref bytes) = value {
                        for (name, val) in parse_face_detect(bytes, endian) {
                            let tag_id = 0x9340 + tags.len() as u16;
                            tags.insert(
                                tag_id,
                                MakerNoteTag::new(
                                    tag_id,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val),
                                ),
                            );
                        }
                    }
                }
                NIKON_MULTI_EXPOSURE => {
                    if let ExifValue::Undefined(ref bytes) = value {
                        for (name, val) in parse_multi_exposure(bytes) {
                            let tag_id = 0x9350 + tags.len() as u16;
                            let tag = if let Some((exiv2_group, exiv2_name)) =
                                get_exiv2_nikon_subfield(NIKON_MULTI_EXPOSURE, &name)
                            {
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val.clone()),
                                    ExifValue::Ascii(val),
                                    exiv2_group,
                                    exiv2_name,
                                )
                            } else {
                                MakerNoteTag::new(
                                    tag_id,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val),
                                )
                            };
                            tags.insert(tag_id, tag);
                        }
                    }
                }
                NIKON_SHOT_INFO => {
                    if let ExifValue::Undefined(ref bytes) = value {
                        // Check version to determine if we need decryption
                        let version = if bytes.len() >= 4 {
                            String::from_utf8_lossy(&bytes[0..4]).to_string()
                        } else {
                            String::new()
                        };

                        // Version 02xx needs decryption - defer processing
                        if version.starts_with("02") {
                            // Parse basic info (version) immediately since it's unencrypted
                            if version.chars().all(|c| c.is_ascii_digit()) {
                                let tag_id = 0x9380_u16;
                                tags.insert(
                                    tag_id,
                                    MakerNoteTag::new(
                                        tag_id,
                                        Some("ShotInfoVersion"),
                                        ExifValue::Ascii(version),
                                    ),
                                );
                            }
                            // Store for deferred decryption processing
                            shot_info_bytes = Some(bytes.clone());
                        } else {
                            // Version 01xx or other - not encrypted, parse directly
                            for (name, val) in parse_shot_info_basic(bytes) {
                                let tag_id = 0x9380 + tags.len() as u16;
                                let tag = MakerNoteTag::new(
                                    tag_id,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val),
                                );
                                tags.insert(tag_id, tag);
                            }
                        }
                    }
                }
                NIKON_FILE_INFO => {
                    if let ExifValue::Undefined(ref bytes) = value {
                        for (name, val) in parse_file_info(bytes) {
                            let tag_id = 0x9390 + tags.len() as u16;
                            let tag = if let Some((exiv2_group, exiv2_name)) =
                                get_exiv2_nikon_subfield(NIKON_FILE_INFO, &name)
                            {
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val.clone()),
                                    ExifValue::Ascii(val),
                                    exiv2_group,
                                    exiv2_name,
                                )
                            } else {
                                MakerNoteTag::new(
                                    tag_id,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val),
                                )
                            };
                            tags.insert(tag_id, tag);
                        }
                    }
                }
                NIKON_RETOUCH_INFO => {
                    if let ExifValue::Undefined(ref bytes) = value {
                        for (name, val) in parse_retouch_info(bytes) {
                            tags.insert(
                                0x93A0 + tags.len() as u16, // Pseudo tag ID
                                MakerNoteTag::new(
                                    0x93A0 + tags.len() as u16,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val),
                                ),
                            );
                        }
                    }
                }
                NIKON_LENS_DATA => {
                    // Store LensData for deferred processing after serial/shutter_count are captured
                    if let ExifValue::Undefined(ref data_bytes) = value {
                        lens_data_bytes = Some(data_bytes.clone());
                    }
                }
                NIKON_AF_INFO => {
                    if let ExifValue::Undefined(ref bytes) = value {
                        for (name, val) in parse_af_info(bytes, model) {
                            let tag_id = 0x9500 + tags.len() as u16;
                            let tag = if let Some((exiv2_group, exiv2_name)) =
                                get_exiv2_nikon_subfield(NIKON_AF_INFO, &name)
                            {
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val.clone()),
                                    ExifValue::Ascii(val),
                                    exiv2_group,
                                    exiv2_name,
                                )
                            } else {
                                MakerNoteTag::new(
                                    tag_id,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val),
                                )
                            };
                            tags.insert(tag_id, tag);
                        }
                    }
                }
                NIKON_AF_INFO_2 => {
                    if let ExifValue::Undefined(ref bytes) = value {
                        for (name, val) in parse_af_info2(bytes) {
                            let tag_id = 0x9600 + tags.len() as u16;
                            let tag = if let Some((exiv2_group, exiv2_name)) =
                                get_exiv2_nikon_subfield(NIKON_AF_INFO_2, &name)
                            {
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val.clone()),
                                    ExifValue::Ascii(val),
                                    exiv2_group,
                                    exiv2_name,
                                )
                            } else {
                                MakerNoteTag::new(
                                    tag_id,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val),
                                )
                            };
                            tags.insert(tag_id, tag);
                        }
                    }
                }
                NIKON_WORLD_TIME => {
                    if let ExifValue::Undefined(ref bytes) = value {
                        // Only Z9-era cameras (Z9, Z8, Z6III, etc.) have timezone location names
                        let has_tz_location = has_timezone_location_in_shotinfo(model);
                        for (name, val) in parse_world_time(bytes, maker_endian, has_tz_location) {
                            let tag_id = 0x9700 + tags.len() as u16;
                            let tag = if let Some((exiv2_group, exiv2_name)) =
                                get_exiv2_nikon_subfield(NIKON_WORLD_TIME, &name)
                            {
                                MakerNoteTag::with_exiv2(
                                    tag_id,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val.clone()),
                                    ExifValue::Ascii(val),
                                    exiv2_group,
                                    exiv2_name,
                                )
                            } else {
                                MakerNoteTag::new(
                                    tag_id,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val),
                                )
                            };
                            tags.insert(tag_id, tag);
                        }
                    }
                }
                NIKON_COLOR_BALANCE => {
                    if let ExifValue::Undefined(ref bytes) = value {
                        // Defer processing - version 02xx needs serial/shutter_count for decryption
                        color_balance_bytes = Some(bytes.clone());
                    }
                }
                NIKON_COLOR_BALANCE_A => {
                    // NRWData for P1000, P7000, P7100, B700 - uses ColorBalanceC structure
                    // NRW files don't need decryption
                    if let ExifValue::Undefined(ref bytes) = value {
                        for (name, val) in parse_color_balance(bytes, maker_endian, None, None) {
                            tags.insert(
                                0x9A00 + tags.len() as u16,
                                MakerNoteTag::new(
                                    0x9A00 + tags.len() as u16,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val),
                                ),
                            );
                        }
                    }
                }
                NIKON_CROP_HI_SPEED => {
                    if let ExifValue::Short(ref vals) = value {
                        // Convert shorts to bytes for parsing
                        let bytes: Vec<u8> =
                            vals.iter().flat_map(|v| v.to_le_bytes().to_vec()).collect();
                        for (name, val) in parse_crop_hi_speed(&bytes, maker_endian) {
                            tags.insert(
                                0x9900 + tags.len() as u16,
                                MakerNoteTag::new(
                                    0x9900 + tags.len() as u16,
                                    Some(Box::leak(name.into_boxed_str())),
                                    ExifValue::Ascii(val),
                                ),
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Deferred ColorBalance processing - version 02xx needs serial/shutter_count for decryption
    if let Some(bytes) = color_balance_bytes {
        for (name, val) in
            parse_color_balance(&bytes, maker_endian, serial_number, shutter_count_val)
        {
            tags.insert(
                0x9800 + tags.len() as u16,
                MakerNoteTag::new(
                    0x9800 + tags.len() as u16,
                    Some(Box::leak(name.into_boxed_str())),
                    ExifValue::Ascii(val),
                ),
            );
        }
    }

    // Deferred ShotInfo processing - version 02xx needs serial/shutter_count for decryption
    if let Some(bytes) = shot_info_bytes {
        if let (Some(ser), Some(count)) = (serial_number, shutter_count_val) {
            // Get version string (unencrypted)
            let version = if bytes.len() >= 4 {
                String::from_utf8_lossy(&bytes[0..4]).to_string()
            } else {
                String::new()
            };

            // Determine FirmwareVersion length based on version
            // - Versions 0246 (D6), 0249 (Z6/Z7), 0251 (Z6_3): 8 bytes
            // - Earlier versions: 5 bytes
            let firmware_len = match version.as_str() {
                "0246" | "0249" | "0250" | "0251" | "0252" | "0253" | "0254" | "0255" => 8,
                _ => 5,
            };
            let firmware_end = 4 + firmware_len;

            // Decrypt ShotInfo starting at offset 4 (version string is unencrypted)
            if bytes.len() >= firmware_end {
                let mut decrypted = bytes.clone();
                nikon_decrypt(ser, count, &mut decrypted, 4);

                // Extract FirmwareVersion from decrypted data
                let firmware = String::from_utf8_lossy(&decrypted[4..firmware_end]);
                if firmware.chars().any(|c| c.is_ascii_digit()) {
                    let tag_id = 0x9381_u16;
                    tags.insert(
                        tag_id,
                        MakerNoteTag::new(
                            tag_id,
                            Some("FirmwareVersion"),
                            ExifValue::Ascii(
                                firmware.trim_end_matches('\0').trim_end().to_string(),
                            ),
                        ),
                    );
                }
            }
        }
    }

    // Deferred LensData processing - need serial_number and shutter_count_val
    // which may appear after LensData in the IFD (ShutterCount is 0x00A7, LensData is 0x0098)
    if let Some(data) = lens_data_bytes {
        let (parsed_tags, id_bytes) = parse_lens_data(&data, serial_number, shutter_count_val);
        lens_id_bytes = id_bytes;
        for (name, val) in parsed_tags {
            let tag_id = 0x9400 + tags.len() as u16;
            let tag = if let Some((exiv2_group, exiv2_name)) =
                get_exiv2_nikon_subfield(NIKON_LENS_DATA, &name)
            {
                MakerNoteTag::with_exiv2(
                    tag_id,
                    Some(Box::leak(name.into_boxed_str())),
                    ExifValue::Ascii(val.clone()),
                    ExifValue::Ascii(val),
                    exiv2_group,
                    exiv2_name,
                )
            } else {
                MakerNoteTag::new(
                    tag_id,
                    Some(Box::leak(name.into_boxed_str())),
                    ExifValue::Ascii(val),
                )
            };
            tags.insert(tag_id, tag);
        }
    }

    // Add computed AutoFocus tag based on FocusMode
    // AutoFocus is "On" unless FocusMode starts with "Manual"
    if let Some(focus_mode_tag) = tags.get(&NIKON_FOCUS_MODE) {
        if let ExifValue::Ascii(ref mode) = focus_mode_tag.value {
            let auto_focus = if mode.starts_with("Manual") {
                "Off"
            } else {
                "On"
            };
            tags.insert(
                0xA000, // Pseudo tag ID for AutoFocus
                MakerNoteTag::new(
                    0xA000,
                    Some("AutoFocus"),
                    ExifValue::Ascii(auto_focus.to_string()),
                ),
            );
        }
    }

    // Add LensID composite lookup
    // LensID is constructed from 8 bytes: 7 from LensData + LensType
    if let (Some(id_bytes), Some(lens_type)) = (lens_id_bytes, lens_type_raw) {
        let lens_id = format!(
            "{:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
            id_bytes[0], // LensIDNumber
            id_bytes[1], // LensFStops
            id_bytes[2], // MinFocalLength
            id_bytes[3], // MaxFocalLength
            id_bytes[4], // MaxApertureAtMinFocal
            id_bytes[5], // MaxApertureAtMaxFocal
            id_bytes[6], // MCUVersion
            lens_type,   // LensType
        );

        // Look up lens name - if found, output as LensID (matches ExifTool)
        // ExifTool shows "Lens ID" as the full resolved lens name
        let lookup_result = get_nikon_lens_name(&lens_id);
        if let Some(lens_name) = lookup_result {
            tags.insert(
                0xA003, // Pseudo tag ID for LensID (avoiding 0xA001 ColorSpace conflict)
                MakerNoteTag::new(
                    0xA003,
                    Some("LensID"),
                    ExifValue::Ascii(lens_name.to_string()),
                ),
            );
        } else {
            // If not found in database, output as "Unknown (hex)" to match ExifTool
            tags.insert(
                0xA003,
                MakerNoteTag::new(
                    0xA003,
                    Some("LensID"),
                    ExifValue::Ascii(format!("Unknown ({})", lens_id)),
                ),
            );
        }
    }

    Ok(tags)
}
