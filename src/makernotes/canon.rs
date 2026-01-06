// makernotes/canon.rs - Canon maker notes parsing

use crate::data_types::{Endianness, ExifValue};
use crate::define_tag_decoder;
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use std::collections::HashMap;

// Canon MakerNote tag IDs
pub const CANON_CAMERA_SETTINGS: u16 = 0x0001;
pub const CANON_FOCAL_LENGTH: u16 = 0x0002;
pub const CANON_FLASH_INFO: u16 = 0x0003;
pub const CANON_SHOT_INFO: u16 = 0x0004;
pub const CANON_PANORAMA: u16 = 0x0005;
pub const CANON_IMAGE_TYPE: u16 = 0x0006;
pub const CANON_FIRMWARE_VERSION: u16 = 0x0007;
pub const CANON_FILE_NUMBER: u16 = 0x0008;
pub const CANON_OWNER_NAME: u16 = 0x0009;
pub const CANON_SERIAL_NUMBER: u16 = 0x000C;
pub const CANON_CAMERA_INFO: u16 = 0x000D;
pub const CANON_FILE_LENGTH: u16 = 0x000E;
pub const CANON_CUSTOM_FUNCTIONS: u16 = 0x000F;
pub const CANON_MODEL_ID: u16 = 0x0010;
pub const CANON_MOVIE_INFO: u16 = 0x0011;
pub const CANON_AF_INFO: u16 = 0x0012;
pub const CANON_THUMBNAIL_IMAGE_VALID_AREA: u16 = 0x0013;
pub const CANON_SERIAL_NUMBER_FORMAT: u16 = 0x0015;
pub const CANON_SUPER_MACRO: u16 = 0x001A;
pub const CANON_DATE_STAMP_MODE: u16 = 0x001C;
pub const CANON_MY_COLORS: u16 = 0x001D;
pub const CANON_FIRMWARE_REVISION: u16 = 0x001E;
pub const CANON_CATEGORIES: u16 = 0x0023;
pub const CANON_FACE_DETECT: u16 = 0x0024;
pub const CANON_FACE_DETECT_2: u16 = 0x0025;
pub const CANON_AF_INFO_2: u16 = 0x0026;
pub const CANON_CONTRAST_INFO: u16 = 0x0027;
pub const CANON_IMAGE_UNIQUE_ID: u16 = 0x0028;
pub const CANON_WB_INFO: u16 = 0x0029;
pub const CANON_FACE_DETECT_3: u16 = 0x002F;
pub const CANON_TIME_INFO: u16 = 0x0035;
pub const CANON_BATTERY_TYPE: u16 = 0x0038;
pub const CANON_AF_INFO_3: u16 = 0x003C;
pub const CANON_RAW_DATA_OFFSET: u16 = 0x0081;
pub const CANON_ORIGINAL_DECISION_DATA_OFFSET: u16 = 0x0083;
pub const CANON_PERSONAL_FUNCTIONS: u16 = 0x0090;
pub const CANON_PERSONAL_FUNCTION_VALUES: u16 = 0x0091;
pub const CANON_FILE_INFO: u16 = 0x0093;
pub const CANON_AF_POINTS_IN_FOCUS_1D: u16 = 0x0094;
pub const CANON_LENS_MODEL: u16 = 0x0095;
pub const CANON_SERIAL_INFO: u16 = 0x0096;
pub const CANON_DUST_REMOVAL_DATA: u16 = 0x0097;
pub const CANON_CROP_INFO: u16 = 0x0098;
pub const CANON_CUSTOM_FUNCTIONS_2: u16 = 0x0099;
pub const CANON_ASPECT_INFO: u16 = 0x009A;
pub const CANON_PROCESSING_INFO: u16 = 0x00A0;
pub const CANON_TONE_CURVE_TABLE: u16 = 0x00A1;
pub const CANON_SHARPNESS_TABLE: u16 = 0x00A2;
pub const CANON_SHARPNESS_FREQ_TABLE: u16 = 0x00A3;
pub const CANON_WHITE_BALANCE_TABLE: u16 = 0x00A4;
pub const CANON_COLOR_BALANCE: u16 = 0x00A9;
pub const CANON_MEASURED_COLOR: u16 = 0x00AA;
pub const CANON_COLOR_TEMPERATURE: u16 = 0x00AE;
pub const CANON_CANON_FLAGS: u16 = 0x00B0;
pub const CANON_MODIFIED_INFO: u16 = 0x00B1;
pub const CANON_TONE_CURVE_MATCHING: u16 = 0x00B2;
pub const CANON_WHITE_BALANCE_MATCHING: u16 = 0x00B3;
pub const CANON_COLOR_SPACE: u16 = 0x00B4;
pub const CANON_PREVIEW_IMAGE_INFO: u16 = 0x00B6;
pub const CANON_VRD_OFFSET: u16 = 0x00D0;
pub const CANON_SENSOR_INFO: u16 = 0x00E0;
pub const CANON_COLOR_DATA: u16 = 0x4001;
pub const CANON_CRWPARAM: u16 = 0x4002;
pub const CANON_COLOR_INFO: u16 = 0x4003;
pub const CANON_FLAVOR: u16 = 0x4005;
pub const CANON_PICTURE_STYLE_USER_DEF: u16 = 0x4008;
pub const CANON_PICTURE_STYLE_PC: u16 = 0x4009;
pub const CANON_CUSTOM_PICTURE_STYLE_FILE_NAME: u16 = 0x4010;
pub const CANON_AF_MICRO_ADJ: u16 = 0x4013;
pub const CANON_VIGNETTING_CORR: u16 = 0x4015;
pub const CANON_VIGNETTING_CORR_2: u16 = 0x4016;
pub const CANON_LIGHTING_OPT: u16 = 0x4018;
pub const CANON_LENS_INFO: u16 = 0x4019;
pub const CANON_AMBIANCE_INFO: u16 = 0x4020;
pub const CANON_MULTI_EXP: u16 = 0x4021;
pub const CANON_FILTER_INFO: u16 = 0x4024;
pub const CANON_HDR_INFO: u16 = 0x4025;
pub const CANON_AF_CONFIG: u16 = 0x4028;
pub const CANON_RAW_BURST_MODE_ROLL: u16 = 0x4029;
pub const CANON_LOG_INFO: u16 = 0x4040;

// CameraSettings sub-array tags (indices in tag 0x0001)
pub const CS_MACRO_MODE: u16 = 0x0001;
pub const CS_SELF_TIMER: u16 = 0x0002;
pub const CS_QUALITY: u16 = 0x0003;
pub const CS_FLASH_MODE: u16 = 0x0004;
pub const CS_CONTINUOUS_DRIVE: u16 = 0x0005;
pub const CS_FOCUS_MODE: u16 = 0x0007;
pub const CS_RECORD_MODE: u16 = 0x0009;
pub const CS_IMAGE_SIZE: u16 = 0x000A;
pub const CS_EASY_MODE: u16 = 0x000B;
pub const CS_DIGITAL_ZOOM: u16 = 0x000C;
pub const CS_CONTRAST: u16 = 0x000D;
pub const CS_SATURATION: u16 = 0x000E;
pub const CS_SHARPNESS: u16 = 0x000F;
pub const CS_ISO_SPEED: u16 = 0x0010;
pub const CS_METERING_MODE: u16 = 0x0011;
pub const CS_FOCUS_TYPE: u16 = 0x0012;
pub const CS_AF_POINT: u16 = 0x0013;
pub const CS_EXPOSURE_PROGRAM: u16 = 0x0014;
pub const CS_LENS_TYPE: u16 = 0x0016;
pub const CS_MAX_FOCAL_LENGTH: u16 = 0x0017;
pub const CS_MIN_FOCAL_LENGTH: u16 = 0x0018;
pub const CS_FOCAL_UNITS: u16 = 0x0019;
pub const CS_MAX_APERTURE: u16 = 0x001A;
pub const CS_MIN_APERTURE: u16 = 0x001B;
pub const CS_FLASH_ACTIVITY: u16 = 0x001C;
pub const CS_FLASH_BITS: u16 = 0x001D;
pub const CS_FOCUS_CONTINUOUS: u16 = 0x0020;
pub const CS_AE_SETTING: u16 = 0x0021;
pub const CS_IMAGE_STABILIZATION: u16 = 0x0022;
pub const CS_DISPLAY_APERTURE: u16 = 0x0023;
pub const CS_ZOOM_SOURCE_WIDTH: u16 = 0x0024;
pub const CS_ZOOM_TARGET_WIDTH: u16 = 0x0025;
pub const CS_SPOT_METERING_MODE: u16 = 0x0027;
pub const CS_PHOTO_EFFECT: u16 = 0x0028;
pub const CS_MANUAL_FLASH_OUTPUT: u16 = 0x0029;
pub const CS_COLOR_TONE: u16 = 0x002A;
pub const CS_SRAW_QUALITY: u16 = 0x002E;
pub const CS_FOCUS_BRACKETING: u16 = 0x0032; // Index 50
pub const CS_CLARITY: u16 = 0x0033; // Index 51
pub const CS_HDR_PQ: u16 = 0x0034; // Index 52

// ShotInfo sub-array tags (indices in tag 0x0004)
pub const SI_AUTO_ISO: u16 = 0x0001;
pub const SI_BASE_ISO: u16 = 0x0002;
pub const SI_MEASURED_EV: u16 = 0x0003;
pub const SI_TARGET_APERTURE: u16 = 0x0004;
pub const SI_TARGET_EXPOSURE_TIME: u16 = 0x0005;
pub const SI_EXPOSURE_COMPENSATION: u16 = 0x0006;
pub const SI_WHITE_BALANCE: u16 = 0x0007;
pub const SI_SLOW_SHUTTER: u16 = 0x0008;
pub const SI_SEQUENCE_NUMBER: u16 = 0x0009;
pub const SI_OPTICAL_ZOOM_CODE: u16 = 0x000A;
pub const SI_CAMERA_TEMPERATURE: u16 = 0x000C;
pub const SI_FLASH_GUIDE_NUMBER: u16 = 0x000D;
pub const SI_AF_POINTS_IN_FOCUS: u16 = 0x000E;
pub const SI_FLASH_EXPOSURE_COMP: u16 = 0x000F;
pub const SI_AUTO_EXPOSURE_BRACKETING: u16 = 0x0010;
pub const SI_AEB_BRACKET_VALUE: u16 = 0x0011;
pub const SI_CONTROL_MODE: u16 = 0x0012;
pub const SI_FOCUS_DISTANCE_UPPER: u16 = 0x0013;
pub const SI_FOCUS_DISTANCE_LOWER: u16 = 0x0014;
pub const SI_FNUMBER: u16 = 0x0015; // Index 21
pub const SI_EXPOSURE_TIME: u16 = 0x0016; // Index 22
pub const SI_MEASURED_EV2: u16 = 0x0017; // Index 23
pub const SI_BULB_DURATION: u16 = 0x0018; // Index 24
pub const SI_CAMERA_TYPE: u16 = 0x001A; // Index 26
pub const SI_AUTO_ROTATE: u16 = 0x001B; // Index 27
pub const SI_ND_FILTER: u16 = 0x001C; // Index 28
pub const SI_SELF_TIMER_2: u16 = 0x001D; // Index 29
pub const SI_FLASH_OUTPUT: u16 = 0x0021; // Index 33

// FocalLength sub-array tags (indices in tag 0x0002)
pub const FL_FOCAL_TYPE: u16 = 0x0000;
pub const FL_FOCAL_LENGTH: u16 = 0x0001;
pub const FL_FOCAL_PLANE_X_SIZE: u16 = 0x0002;
pub const FL_FOCAL_PLANE_Y_SIZE: u16 = 0x0003;

// ProcessingInfo sub-array tags (indices in tag 0x00A0)
pub const PR_TONE_CURVE: u16 = 0x0001;
pub const PR_SHARPNESS: u16 = 0x0002;
pub const PR_SHARPNESS_FREQUENCY: u16 = 0x0003;
pub const PR_SENSOR_RED_LEVEL: u16 = 0x0004;
pub const PR_SENSOR_BLUE_LEVEL: u16 = 0x0005;
pub const PR_WHITE_BALANCE_RED: u16 = 0x0006;
pub const PR_WHITE_BALANCE_BLUE: u16 = 0x0007;
pub const PR_WHITE_BALANCE: u16 = 0x0008;
pub const PR_COLOR_TEMPERATURE: u16 = 0x0009;
pub const PR_PICTURE_STYLE: u16 = 0x000A;
pub const PR_DIGITAL_GAIN: u16 = 0x000B;
pub const PR_WB_SHIFT_AB: u16 = 0x000C;
pub const PR_WB_SHIFT_GM: u16 = 0x000D;

// FileInfo sub-array tags (indices in tag 0x0093)
pub const FI_FILE_NUMBER: u16 = 0x0001;
pub const FI_BRACKET_MODE: u16 = 0x0003;
pub const FI_BRACKET_VALUE: u16 = 0x0004;
pub const FI_BRACKET_SHOT_NUMBER: u16 = 0x0005;
pub const FI_RAW_JPG_QUALITY: u16 = 0x0006;
pub const FI_RAW_JPG_SIZE: u16 = 0x0007;
pub const FI_NOISE_REDUCTION: u16 = 0x0008;
pub const FI_WB_BRACKET_MODE: u16 = 0x0009;
pub const FI_WB_BRACKET_VALUE_AB: u16 = 0x000C;
pub const FI_WB_BRACKET_VALUE_GM: u16 = 0x000D;
pub const FI_FILTER_EFFECT: u16 = 0x000E;
pub const FI_TONING_EFFECT: u16 = 0x000F;
pub const FI_MACRO_MAGNIFICATION: u16 = 0x0010;
pub const FI_LIVE_VIEW_SHOOTING: u16 = 0x0013;
pub const FI_FOCUS_DISTANCE_UPPER_2: u16 = 0x0014;
pub const FI_FOCUS_DISTANCE_LOWER_2: u16 = 0x0015;
pub const FI_FLASH_EXPOSURE_LOCK: u16 = 0x0019;

// LightingOpt sub-array tags (indices in tag 0x4018)
pub const LO_PERIPHERAL_LIGHTING: u16 = 0x0000;
pub const LO_PERIPHERAL_LIGHTING_VALUE: u16 = 0x0001;
pub const LO_AUTO_LIGHTING_OPTIMIZER: u16 = 0x0002;
pub const LO_HIGH_ISO_NOISE_REDUCTION: u16 = 0x0003;
pub const LO_HIGHLIGHT_TONE_PRIORITY: u16 = 0x0004;
pub const LO_LONG_EXPOSURE_NOISE_REDUCTION: u16 = 0x0005;

// AFConfig sub-array tags (indices in tag 0x4028)
pub const AFC_AF_CONFIG_TOOL: u16 = 0x0000;
pub const AFC_AF_TRACK_SENSITIVITY: u16 = 0x0001;
pub const AFC_AF_ACCEL_TRACK: u16 = 0x0002;
pub const AFC_AF_POINT_SWITCHING: u16 = 0x0003;
pub const AFC_AI_SERVO_FIRST_IMAGE: u16 = 0x0004;
pub const AFC_AI_SERVO_SECOND_IMAGE: u16 = 0x0005;
pub const AFC_USABLE_AF_POINTS: u16 = 0x0006;

// VignettingCorr sub-array tags (indices in tag 0x4015)
pub const VC_PERIPHERAL_ILLUMINATION_CORR: u16 = 0x0000;
pub const VC_DISTORTION_CORRECTION: u16 = 0x0001;
pub const VC_CHROMATIC_ABERRATION_CORR: u16 = 0x0002;
pub const VC_PERIPHERAL_ILLUMINATION_CORR_2: u16 = 0x0003;
pub const VC_DIFFRACTION_CORRECTION: u16 = 0x0004;

// LensInfo sub-array tags (indices in tag 0x4019)
pub const LI_LENS_SERIAL_NUMBER: u16 = 0x0000;

// MultiExp sub-array tags (indices in tag 0x4021)
pub const ME_MULTI_EXPOSURE_MODE: u16 = 0x0000;
pub const ME_MULTI_EXPOSURE_SHOTS: u16 = 0x0001;
pub const ME_MULTI_EXPOSURE_CONTROL: u16 = 0x0002;

// HDRInfo sub-array tags (indices in tag 0x4025)
pub const HDR_MODE: u16 = 0x0000;
pub const HDR_STRENGTH: u16 = 0x0001;
pub const HDR_EFFECT: u16 = 0x0002;

// Additional CameraSettings tags
pub const CS_FLASH_DETAILS: u16 = 0x001E;
pub const CS_FOCUS_MODE_2: u16 = 0x0032;
pub const CS_MANUAL_AF_POINT_SEL: u16 = 0x0034;
pub const CS_AF_AREA_MODE: u16 = 0x0033;

// AspectInfo sub-array tags (indices in tag 0x009A)
pub const AI_ASPECT_RATIO: u16 = 0x0000;
pub const AI_CROPPED_IMAGE_WIDTH: u16 = 0x0001;
pub const AI_CROPPED_IMAGE_HEIGHT: u16 = 0x0002;
pub const AI_CROPPED_IMAGE_LEFT: u16 = 0x0003;
pub const AI_CROPPED_IMAGE_TOP: u16 = 0x0004;

// CustomFunctions tags
pub const CF_SET_BUTTON_FUNCTION: u16 = 0x0101;
pub const CF_LONG_EXPOSURE_NOISE_REDUCTION: u16 = 0x0102;
pub const CF_FLASH_SYNC_SPEED: u16 = 0x0103;
pub const CF_SHUTTER_AE_LOCK_BUTTON: u16 = 0x0104;
pub const CF_MIRROR_LOCKUP: u16 = 0x0105;
pub const CF_TV_AV_SETTING: u16 = 0x0106;
pub const CF_AF_ASSIST_BEAM_FIRING: u16 = 0x0107;
pub const CF_SAFETY_SHIFT: u16 = 0x0108;
pub const CF_LCD_DISPLAY_AT_POWER_ON: u16 = 0x0109;
pub const CF_AEB_SEQUENCE: u16 = 0x0110;
pub const CF_SHUTTER_CURTAIN_SYNC: u16 = 0x0111;

// MovieInfo sub-array tags (indices in tag 0x0011)
pub const MI_MOVIE_FRAME_WIDTH: u16 = 0x0000;
pub const MI_MOVIE_FRAME_HEIGHT: u16 = 0x0001;
pub const MI_MOVIE_FRAME_RATE: u16 = 0x0002;
pub const MI_MOVIE_FRAME_COUNT: u16 = 0x0003;
pub const MI_MOVIE_AUDIO_DURATION: u16 = 0x0004;

// AFInfo2 sub-array tags (indices in tag 0x0026)
pub const AF2_NUM_AF_POINTS: u16 = 0x0000;
pub const AF2_VALID_AF_POINTS: u16 = 0x0001;
pub const AF2_IMG_WIDTH: u16 = 0x0002;
pub const AF2_IMG_HEIGHT: u16 = 0x0003;
pub const AF2_AF_IMG_WIDTH: u16 = 0x0004;
pub const AF2_AF_IMG_HEIGHT: u16 = 0x0005;
pub const AF2_AF_AREA_WIDTHS: u16 = 0x0006;
pub const AF2_AF_AREA_HEIGHTS: u16 = 0x0007;
pub const AF2_AF_AREA_X_POSITIONS: u16 = 0x0008;
pub const AF2_AF_AREA_Y_POSITIONS: u16 = 0x0009;
pub const AF2_AF_POINTS_IN_FOCUS: u16 = 0x000A;
pub const AF2_AF_POINTS_SELECTED: u16 = 0x000B;
pub const AF2_PRIMARY_AF_POINT: u16 = 0x000C;

// ColorBalance sub-array tags (indices in tag 0x00A9)
pub const CB_WB_RGGB_LEVELS_AS_SHOT: u16 = 0x0000;
pub const CB_COLOR_TEMP_AS_SHOT: u16 = 0x0001;
pub const CB_WB_RGGB_LEVELS_AUTO: u16 = 0x0002;
pub const CB_COLOR_TEMP_AUTO: u16 = 0x0003;
pub const CB_WB_RGGB_LEVELS_MEASURED: u16 = 0x0004;
pub const CB_COLOR_TEMP_MEASURED: u16 = 0x0005;
pub const CB_WB_RGGB_LEVELS_DAYLIGHT: u16 = 0x0006;
pub const CB_COLOR_TEMP_DAYLIGHT: u16 = 0x0007;
pub const CB_WB_RGGB_LEVELS_SHADE: u16 = 0x0008;
pub const CB_COLOR_TEMP_SHADE: u16 = 0x0009;
pub const CB_WB_RGGB_LEVELS_CLOUDY: u16 = 0x000A;
pub const CB_COLOR_TEMP_CLOUDY: u16 = 0x000B;
pub const CB_WB_RGGB_LEVELS_TUNGSTEN: u16 = 0x000C;
pub const CB_COLOR_TEMP_TUNGSTEN: u16 = 0x000D;
pub const CB_WB_RGGB_LEVELS_FLUORESCENT: u16 = 0x000E;
pub const CB_COLOR_TEMP_FLUORESCENT: u16 = 0x000F;
pub const CB_WB_RGGB_LEVELS_KELVIN: u16 = 0x0010;
pub const CB_COLOR_TEMP_KELVIN: u16 = 0x0011;
pub const CB_WB_RGGB_LEVELS_FLASH: u16 = 0x0012;
pub const CB_COLOR_TEMP_FLASH: u16 = 0x0013;

// SensorInfo sub-array tags (indices in tag 0x00E0)
pub const SE_SENSOR_WIDTH: u16 = 0x0001;
pub const SE_SENSOR_HEIGHT: u16 = 0x0002;
pub const SE_SENSOR_LEFT_BORDER: u16 = 0x0005;
pub const SE_SENSOR_TOP_BORDER: u16 = 0x0006;
pub const SE_SENSOR_RIGHT_BORDER: u16 = 0x0007;
pub const SE_SENSOR_BOTTOM_BORDER: u16 = 0x0008;
pub const SE_BLACK_MASK_LEFT_BORDER: u16 = 0x0009;
pub const SE_BLACK_MASK_TOP_BORDER: u16 = 0x000A;
pub const SE_BLACK_MASK_RIGHT_BORDER: u16 = 0x000B;
pub const SE_BLACK_MASK_BOTTOM_BORDER: u16 = 0x000C;

// FilterInfo sub-array tags (indices in tag 0x4024)
pub const FLT_MINIATURE_FILTER_ORIENTATION: u16 = 0x0000;
pub const FLT_MINIATURE_FILTER_POSITION: u16 = 0x0001;
pub const FLT_MINIATURE_FILTER_PARAMETER: u16 = 0x0002;
pub const FLT_FISHEYE_EFFECT: u16 = 0x0003;
pub const FLT_TOY_CAMERA_EFFECT: u16 = 0x0004;
pub const FLT_SOFT_FOCUS_EFFECT: u16 = 0x0005;

// AmbianceInfo sub-array tags (indices in tag 0x4020)
pub const AM_AMBIANCE_SELECTION: u16 = 0x0000;
pub const AM_AMBIANCE_EFFECT: u16 = 0x0001;

// PictureStyleUserDef sub-array tags (indices in tag 0x4008)
pub const PS_USER_DEF_1: u16 = 0x0000;
pub const PS_USER_DEF_2: u16 = 0x0001;
pub const PS_USER_DEF_3: u16 = 0x0002;

// AFMicroAdj sub-array tags (indices in tag 0x4013)
pub const AFMA_AF_MICRO_ADJ_MODE: u16 = 0x0000;
pub const AFMA_AF_MICRO_ADJ_VALUE: u16 = 0x0001;

// Additional main IFD tags
pub const CANON_FACE_INFO_1: u16 = 0x0030;
pub const CANON_FACE_INFO_2: u16 = 0x0031;
pub const CANON_ORIGINAL_FILENAME: u16 = 0x003A;
pub const CANON_EMBEDDED_IMAGE_QUALITY: u16 = 0x0071;
pub const CANON_MV_INFO: u16 = 0x4007;
pub const CANON_CANON_FLASHINFO: u16 = 0x00C4;
pub const CANON_INTERNAL_SERIAL_NUMBER: u16 = 0x0096;

// Additional CropInfo tags (indices in tag 0x0098)
pub const CR_CROP_LEFT_MARGIN: u16 = 0x0000;
pub const CR_CROP_RIGHT_MARGIN: u16 = 0x0001;
pub const CR_CROP_TOP_MARGIN: u16 = 0x0002;
pub const CR_CROP_BOTTOM_MARGIN: u16 = 0x0003;

// Panorama sub-array tags (indices in tag 0x0005)
pub const PA_PANORAMA_FRAME_NUMBER: u16 = 0x0000;
pub const PA_PANORAMA_DIRECTION: u16 = 0x0001;

// FaceDetect sub-array tags (indices in tag 0x0024)
pub const FD_NUM_FACES_DETECTED: u16 = 0x0000;
pub const FD_FACE_DETECT_FRAME_SIZE: u16 = 0x0001;
pub const FD_FACE_1_POSITION: u16 = 0x0002;
pub const FD_FACE_2_POSITION: u16 = 0x0003;
pub const FD_FACE_3_POSITION: u16 = 0x0004;
pub const FD_FACE_4_POSITION: u16 = 0x0005;
pub const FD_FACE_5_POSITION: u16 = 0x0006;

// PreviewImageInfo sub-array tags (indices in tag 0x00B6)
pub const PI_PREVIEW_QUALITY: u16 = 0x0000;
pub const PI_PREVIEW_IMAGE_LENGTH: u16 = 0x0001;
pub const PI_PREVIEW_IMAGE_WIDTH: u16 = 0x0002;
pub const PI_PREVIEW_IMAGE_HEIGHT: u16 = 0x0003;
pub const PI_PREVIEW_IMAGE_START: u16 = 0x0004;
pub const PI_FOCAL_PLANE_X_RESOLUTION: u16 = 0x0005;
pub const PI_FOCAL_PLANE_Y_RESOLUTION: u16 = 0x0006;

// ModifiedInfo sub-array tags (indices in tag 0x00B1)
pub const MD_MODIFIED_TONE_CURVE: u16 = 0x0000;
pub const MD_MODIFIED_SHARPNESS: u16 = 0x0001;
pub const MD_MODIFIED_SHARPNESS_FREQ: u16 = 0x0002;
pub const MD_MODIFIED_SENSOR_RED_LEVEL: u16 = 0x0003;
pub const MD_MODIFIED_SENSOR_BLUE_LEVEL: u16 = 0x0004;
pub const MD_MODIFIED_WHITE_BALANCE_RED: u16 = 0x0005;
pub const MD_MODIFIED_WHITE_BALANCE_BLUE: u16 = 0x0006;
pub const MD_MODIFIED_WHITE_BALANCE: u16 = 0x0007;
pub const MD_MODIFIED_COLOR_TEMP: u16 = 0x0008;
pub const MD_MODIFIED_PICTURE_STYLE: u16 = 0x0009;
pub const MD_MODIFIED_DIGITAL_GAIN: u16 = 0x000A;

// VignettingCorr2 sub-array tags (indices in tag 0x4016)
pub const VC2_PERIPHERAL_LIGHTING_SETTING: u16 = 0x0000;
pub const VC2_CHROMATIC_ABERRATION_SETTING: u16 = 0x0001;
pub const VC2_DISTORTION_SETTING: u16 = 0x0002;

// FlashInfo sub-array tags (indices in tag 0x0003)
pub const FLS_FLASH_GUIDE_NUMBER: u16 = 0x0000;
pub const FLS_FLASH_THRESHOLD: u16 = 0x0001;

// TimeInfo sub-array tags (indices in tag 0x0035)
pub const TI_TIME_ZONE: u16 = 0x0000;
pub const TI_TIME_ZONE_CITY: u16 = 0x0001;
pub const TI_DAYLIGHT_SAVINGS: u16 = 0x0002;

// ContrastInfo sub-array tags (indices in tag 0x0027)
pub const CI_INCREMENTAL_COLOR_TEMP: u16 = 0x0004;
pub const CI_INCREMENTAL_TINT: u16 = 0x0005;

// Additional FileInfo tags
pub const FI_SHUTTER_MODE: u16 = 0x0016;
pub const FI_RAW_BURST_IMAGE_COUNT: u16 = 0x0017;
pub const FI_RAW_BURST_SHOT_NUM: u16 = 0x0018;
pub const FI_HDR: u16 = 0x001E;

// Additional LightingOpt tags
pub const LO_DIGITAL_LENS_OPTIMIZER: u16 = 0x0006;
pub const LO_DLO_DATA: u16 = 0x0007;
pub const LO_DLO_SETTING_APPLIED: u16 = 0x0009;

// Extended ColorData indices (tag 0x4001)
pub const CD_COLOR_DATA_VERSION: u16 = 0x0000;
pub const CD_WB_RGGB_LEVELS_AS_SHOT: u16 = 0x0003;
pub const CD_COLOR_TEMP_AS_SHOT: u16 = 0x0007;
pub const CD_WB_RGGB_LEVELS_AUTO: u16 = 0x0008;
pub const CD_COLOR_TEMP_AUTO: u16 = 0x000C;
pub const CD_WB_RGGB_LEVELS_MEASURED: u16 = 0x000D;
pub const CD_COLOR_TEMP_MEASURED: u16 = 0x0011;
pub const CD_WB_RGGB_BLACK_LEVELS: u16 = 0x0012;
pub const CD_LINEAR_RESPONSE_LIMIT: u16 = 0x0016;
pub const CD_CAMERA_COLOR_CALIBRATION_01: u16 = 0x0017;
pub const CD_CAMERA_COLOR_CALIBRATION_02: u16 = 0x0018;
pub const CD_CAMERA_COLOR_CALIBRATION_03: u16 = 0x0019;
pub const CD_CAMERA_COLOR_CALIBRATION_04: u16 = 0x001A;
pub const CD_CAMERA_COLOR_CALIBRATION_05: u16 = 0x001B;
pub const CD_CAMERA_COLOR_CALIBRATION_06: u16 = 0x001C;
pub const CD_CAMERA_COLOR_CALIBRATION_07: u16 = 0x001D;
pub const CD_CAMERA_COLOR_CALIBRATION_08: u16 = 0x001E;
pub const CD_CAMERA_COLOR_CALIBRATION_09: u16 = 0x001F;
pub const CD_CAMERA_COLOR_CALIBRATION_10: u16 = 0x0020;
pub const CD_CAMERA_COLOR_CALIBRATION_11: u16 = 0x0021;
pub const CD_CAMERA_COLOR_CALIBRATION_12: u16 = 0x0022;
pub const CD_CAMERA_COLOR_CALIBRATION_13: u16 = 0x0023;
pub const CD_CAMERA_COLOR_CALIBRATION_14: u16 = 0x0024;
pub const CD_CAMERA_COLOR_CALIBRATION_15: u16 = 0x0025;
pub const CD_REDUCED_RESOLUTION_CALIBRATION_01: u16 = 0x0026;
pub const CD_REDUCED_RESOLUTION_CALIBRATION_02: u16 = 0x0027;
pub const CD_REDUCED_RESOLUTION_CALIBRATION_03: u16 = 0x0028;
pub const CD_REDUCED_RESOLUTION_CALIBRATION_04: u16 = 0x0029;
pub const CD_REDUCED_RESOLUTION_CALIBRATION_05: u16 = 0x002A;
pub const CD_REDUCED_RESOLUTION_CALIBRATION_06: u16 = 0x002B;
pub const CD_REDUCED_RESOLUTION_CALIBRATION_07: u16 = 0x002C;
pub const CD_REDUCED_RESOLUTION_CALIBRATION_08: u16 = 0x002D;
pub const CD_REDUCED_RESOLUTION_CALIBRATION_09: u16 = 0x002E;
pub const CD_REDUCED_RESOLUTION_CALIBRATION_10: u16 = 0x002F;
pub const CD_REDUCED_RESOLUTION_CALIBRATION_11: u16 = 0x0030;
pub const CD_REDUCED_RESOLUTION_CALIBRATION_12: u16 = 0x0031;
pub const CD_REDUCED_RESOLUTION_CALIBRATION_13: u16 = 0x0032;
pub const CD_REDUCED_RESOLUTION_CALIBRATION_14: u16 = 0x0033;
pub const CD_REDUCED_RESOLUTION_CALIBRATION_15: u16 = 0x0034;
pub const CD_AVERAGE_BLACK_LEVEL: u16 = 0x0035;

// Additional AFInfo2 tags
pub const AF2_AF_STATUS_ARRAY: u16 = 0x000D;

// Extended CameraSettings tags
pub const CS_ND_FILTER: u16 = 0x0015;
pub const CS_FOCUS_RANGE: u16 = 0x0033;

// Extended ShotInfo tags
pub const SI_AF_AREA_WIDTH: u16 = 0x0019;
pub const SI_AF_AREA_HEIGHT: u16 = 0x001A;
pub const SI_AF_AREA_X_POSITIONS: u16 = 0x001B;
pub const SI_AF_AREA_Y_POSITIONS: u16 = 0x001C;

// Additional main tags
pub const CANON_MEASURED_EV_2: u16 = 0x00A6;
pub const CANON_FLASH_INFO_2: u16 = 0x0018;
pub const CANON_MEASURED_RGGB: u16 = 0x00AB;
pub const CANON_WB_PACKET: u16 = 0x00AD;
pub const CANON_HIGHLIGHT_SHADOW_ADJUST: u16 = 0x00AC;
pub const CANON_SENSOR_CALIBRATION: u16 = 0x00E5;
pub const CANON_ROLLING_SHUTTER_INFO: u16 = 0x00E6;
pub const CANON_CCDT_INFO: u16 = 0x00E2;
pub const CANON_VRD_STAMP_INFO: u16 = 0x00D1;
pub const CANON_DIGITAL_INFO: u16 = 0x00E4;
pub const CANON_LENS_SERIAL_INFO: u16 = 0x00E8;
pub const CANON_CANON_VRD_RECIPE: u16 = 0x00DF;
pub const CANON_MK_NOTES_VERSION: u16 = 0x0001;
pub const CANON_ISO_SPEED_RATING: u16 = 0x000A;
pub const CANON_LONG_EXP_NR_ON: u16 = 0x00E7;
pub const CANON_USER_COMMENT: u16 = 0x00F7;
pub const CANON_WB_INFO_2: u16 = 0x002A;
pub const CANON_AF_POINTS_IN_FOCUS: u16 = 0x002C;
pub const CANON_MY_COLOR_MODE: u16 = 0x002D;
pub const CANON_FACE_DETECT_INFO: u16 = 0x002E;
pub const CANON_THUMBNAIL_IMAGE: u16 = 0x0032;

/// Convert Canon hex-based EV value to standard EV
/// Canon uses special encoding where lower 5 bits contain fractional part
/// with special codes 0x0c (12) for 1/3 and 0x14 (20) for 2/3 stops
/// Reference: ExifTool Canon.pm CanonEv function
fn canon_ev(val: i16) -> f64 {
    let sign = if val < 0 { -1.0 } else { 1.0 };
    let abs_val = val.unsigned_abs();

    let frac = abs_val & 0x1f;
    let whole = abs_val - frac;

    // Convert 1/3 and 2/3 codes
    let frac_val = match frac {
        0x0c => 32.0 / 3.0, // 1/3 stop
        0x14 => 64.0 / 3.0, // 2/3 stop
        _ => frac as f64,
    };

    sign * (whole as f64 + frac_val) / 32.0
}

/// Convert Canon EV value to f-number and format with %.2g like ExifTool
fn canon_ev_to_fnumber(val: u16) -> f64 {
    let ev = canon_ev(val as i16);
    2f64.powf(ev / 2.0)
}

/// Format f-number using %.2g format like ExifTool
fn format_fnumber_2g(f_number: f64) -> f64 {
    // %.2g means 2 significant figures
    // For values like 1.3, 3.6, 11, etc.
    if f_number >= 10.0 {
        f_number.round()
    } else if f_number >= 1.0 {
        (f_number * 10.0).round() / 10.0
    } else {
        (f_number * 100.0).round() / 100.0
    }
}

/// Get Canon lens name from lens type ID
/// Based on ExifTool's canonLensTypes database
pub fn get_canon_lens_name(lens_id: u16) -> Option<&'static str> {
    match lens_id {
        // Most common Canon EF/EF-S lenses
        1 => Some("Canon EF 50mm f/1.8"),
        2 => Some("Canon EF 28mm f/2.8"),
        3 => Some("Canon EF 135mm f/2.8 Soft"),
        4 => Some("Canon EF 35-105mm f/3.5-4.5"),
        5 => Some("Canon EF 35-70mm f/3.5-4.5"),
        6 => Some("Canon EF 28-70mm f/3.5-4.5"),
        7 => Some("Canon EF 100-300mm f/5.6L"),
        8 => Some("Canon EF 100-300mm f/5.6"),
        9 => Some("Canon EF 70-210mm f/4"),
        10 => Some("Canon EF 50mm f/2.5 Macro"),
        11 => Some("Canon EF 35mm f/2"),
        13 => Some("Canon EF 15mm f/2.8 Fisheye"),
        21 => Some("Canon EF 80-200mm f/2.8L"),
        22 => Some("Canon EF 20-35mm f/2.8L"),
        26 => Some("Canon EF 100mm f/2.8 Macro"),
        29 => Some("Canon EF 50mm f/1.8 II"),
        31 => Some("Canon EF 75-300mm f/4-5.6"),
        32 => Some("Canon EF 24mm f/2.8"),
        35 => Some("Canon EF 35-80mm f/4-5.6"),
        37 => Some("Canon EF 35-80mm f/4-5.6"),
        39 => Some("Canon EF 75-300mm f/4-5.6"),
        40 => Some("Canon EF 28-80mm f/3.5-5.6"),
        43 => Some("Canon EF 28-105mm f/4-5.6"),
        45 => Some("Canon EF-S 18-55mm f/3.5-5.6"),
        48 => Some("Canon EF-S 18-55mm f/3.5-5.6 IS"),
        49 => Some("Canon EF-S 55-250mm f/4-5.6 IS"),
        50 => Some("Canon EF-S 18-200mm f/3.5-5.6 IS"),
        51 => Some("Canon EF-S 18-135mm f/3.5-5.6 IS"),
        52 => Some("Canon EF-S 18-55mm f/3.5-5.6 IS II"),
        53 => Some("Canon EF-S 18-55mm f/3.5-5.6 III"),
        54 => Some("Canon EF-S 55-250mm f/4-5.6 IS II"),
        94 => Some("Canon TS-E 17mm f/4L"),
        95 => Some("Canon TS-E 24mm f/3.5L II"),
        124 => Some("Canon MP-E 65mm f/2.8 1-5x Macro Photo"),
        125 => Some("Canon TS-E 24mm f/3.5L"),
        126 => Some("Canon TS-E 45mm f/2.8"),
        127 => Some("Canon TS-E 90mm f/2.8"),
        129 => Some("Canon EF 300mm f/2.8L USM"),
        130 => Some("Canon EF 50mm f/1.0L USM"),
        131 => Some("Canon EF 28-80mm f/2.8-4L USM"),
        132 => Some("Canon EF 1200mm f/5.6L USM"),
        134 => Some("Canon EF 600mm f/4L IS USM"),
        135 => Some("Canon EF 200mm f/1.8L USM"),
        136 => Some("Canon EF 300mm f/2.8L USM"),
        137 => Some("Canon EF 85mm f/1.2L USM"),
        138 => Some("Canon EF 28-80mm f/2.8-4L"),
        139 => Some("Canon EF 400mm f/2.8L USM"),
        140 => Some("Canon EF 500mm f/4.5L USM"),
        141 => Some("Canon EF 500mm f/4.5L USM"),
        142 => Some("Canon EF 300mm f/2.8L IS USM"),
        143 => Some("Canon EF 500mm f/4L IS USM"),
        144 => Some("Canon EF 35-135mm f/4-5.6 USM"),
        145 => Some("Canon EF 100-300mm f/4.5-5.6 USM"),
        146 => Some("Canon EF 70-210mm f/3.5-4.5 USM"),
        147 => Some("Canon EF 35-135mm f/4-5.6 USM"),
        148 => Some("Canon EF 28-80mm f/3.5-5.6 USM"),
        149 => Some("Canon EF 100mm f/2 USM"),
        150 => Some("Canon EF 14mm f/2.8L USM"),
        151 => Some("Canon EF 200mm f/2.8L USM"),
        152 => Some("Canon EF 300mm f/4L IS USM"),
        153 => Some("Canon EF 35-350mm f/3.5-5.6L USM"),
        154 => Some("Canon EF 20mm f/2.8 USM"),
        155 => Some("Canon EF 85mm f/1.8 USM"),
        156 => Some("Canon EF 28-105mm f/3.5-4.5 USM"),
        160 => Some("Canon EF 20-35mm f/3.5-4.5 USM"),
        161 => Some("Canon EF 28-70mm f/2.8L USM"),
        162 => Some("Canon EF 200mm f/2.8L USM"),
        163 => Some("Canon EF 300mm f/4L USM"),
        164 => Some("Canon EF 400mm f/5.6L USM"),
        165 => Some("Canon EF 70-200mm f/2.8L USM"),
        166 => Some("Canon EF 70-200mm f/2.8L IS USM"),
        167 => Some("Canon EF 70-200mm f/4L USM"),
        168 => Some("Canon EF 28mm f/1.8 USM"),
        169 => Some("Canon EF 17-35mm f/2.8L USM"),
        170 => Some("Canon EF 200mm f/2.8L II USM"),
        171 => Some("Canon EF 300mm f/4L IS USM"),
        172 => Some("Canon EF 400mm f/2.8L II USM"),
        173 => Some("Canon EF 180mm f/3.5L Macro USM"),
        174 => Some("Canon EF 135mm f/2L USM"),
        175 => Some("Canon EF 400mm f/2.8L USM"),
        176 => Some("Canon EF 24-85mm f/3.5-4.5 USM"),
        177 => Some("Canon EF 300mm f/4L IS USM"),
        178 => Some("Canon EF 28-135mm f/3.5-5.6 IS USM"),
        179 => Some("Canon EF 24mm f/1.4L USM"),
        180 => Some("Canon EF 35mm f/1.4L USM"),
        181 => Some("Canon EF 100-400mm f/4.5-5.6L IS USM"),
        182 => Some("Canon EF 100-400mm f/4.5-5.6L IS USM"),
        183 => Some("Canon EF 100mm f/2.8 Macro USM"),
        184 => Some("Canon EF 400mm f/2.8L IS USM"),
        185 => Some("Canon EF 600mm f/4L IS USM"),
        186 => Some("Canon EF 70-200mm f/4L IS USM"),
        187 => Some("Canon EF 70-200mm f/4L IS USM"),
        188 => Some("Canon EF 28-300mm f/3.5-5.6L IS USM"),
        189 => Some("Canon EF 600mm f/4L IS USM"),
        190 => Some("Canon EF 100mm f/2.8L Macro IS USM"),
        191 => Some("Canon EF 400mm f/4 DO IS II USM"),
        193 => Some("Canon EF 35-80mm f/4-5.6 III"),
        194 => Some("Canon EF 80-200mm f/4.5-5.6"),
        195 => Some("Canon EF 35-105mm f/4.5-5.6"),
        196 => Some("Canon EF 75-300mm f/4-5.6"),
        197 => Some("Canon EF 75-300mm f/4-5.6 IS USM"),
        198 => Some("Canon EF 50mm f/1.4 USM"),
        199 => Some("Canon EF 28-80mm f/3.5-5.6 V USM"),
        200 => Some("Canon EF 75-300mm f/4-5.6 III USM"),
        201 => Some("Canon EF 28-80mm f/3.5-5.6"),
        202 => Some("Canon EF 28-80mm f/3.5-5.6 IV USM"),
        208 => Some("Canon EF 22-55mm f/4-5.6 USM"),
        209 => Some("Canon EF 55-200mm f/4.5-5.6"),
        210 => Some("Canon EF 28-90mm f/4-5.6 III"),
        211 => Some("Canon EF 28-200mm f/3.5-5.6 USM"),
        212 => Some("Canon EF 28-105mm f/4-5.6 USM"),
        213 => Some("Canon EF 90-300mm f/4.5-5.6 USM"),
        214 => Some("Canon EF-S 18-55mm f/3.5-5.6 USM"),
        215 => Some("Canon EF 55-200mm f/4.5-5.6 II USM"),
        217 => Some("Canon EF 35-80mm f/4-5.6 III"),
        224 => Some("Canon EF 70-200mm f/2.8L IS USM"),
        225 => Some("Canon EF 70-200mm f/2.8L IS III USM"),
        226 => Some("Canon EF 28-90mm f/4-5.6 III"),
        227 => Some("Canon EF-S 55-250mm f/4-5.6 IS STM"),
        228 => Some("Canon EF 28-105mm f/4-5.6"),
        229 => Some("Canon EF 16-35mm f/2.8L USM"),
        230 => Some("Canon EF 24-70mm f/2.8L USM"),
        231 => Some("Canon EF 17-40mm f/4L USM"),
        232 => Some("Canon EF 70-300mm f/4.5-5.6 DO IS USM"),
        233 => Some("Canon EF 28-300mm f/3.5-5.6L IS USM"),
        234 => Some("Canon EF-S 17-85mm f/4-5.6 IS USM"),
        235 => Some("Canon EF-S 10-22mm f/3.5-4.5 USM"),
        236 => Some("Canon EF-S 60mm f/2.8 Macro USM"),
        237 => Some("Canon EF 24-105mm f/4L IS USM"),
        238 => Some("Canon EF 70-300mm f/4-5.6 IS USM"),
        239 => Some("Canon EF 85mm f/1.2L II USM"),
        240 => Some("Canon EF-S 17-55mm f/2.8 IS USM"),
        241 => Some("Canon EF 50mm f/1.2L USM"),
        242 => Some("Canon EF 70-200mm f/4L IS USM"),
        243 => Some("Canon EF 70-200mm f/4L IS II USM"),
        244 => Some("Canon EF 100mm f/2.8L Macro IS USM"),
        245 => Some("Canon EF 24mm f/1.4L II USM"),
        246 => Some("Canon EF-S 55-250mm f/4-5.6 IS II"),
        247 => Some("Canon EF 35mm f/2 IS USM"),
        248 => Some("Canon EF 24-70mm f/2.8L II USM"),
        249 => Some("Canon EF 300mm f/2.8L IS II USM"),
        250 => Some("Canon EF 400mm f/2.8L IS II USM"),
        251 => Some("Canon EF 500mm f/4L IS II USM"),
        252 => Some("Canon EF 600mm f/4L IS II USM"),
        253 => Some("Canon EF 24-70mm f/4L IS USM"),
        254 => Some("Canon EF 16-35mm f/4L IS USM"),
        255 => Some("Canon EF 35mm f/1.4L II USM"),
        488 => Some("Canon EF-S 15-85mm f/3.5-5.6 IS USM"),
        489 => Some("Canon EF 70-300mm f/4-5.6L IS USM"),
        490 => Some("Canon EF 8-15mm f/4L Fisheye USM"),
        491 => Some("Canon EF 300mm f/2.8L IS II USM"),
        492 => Some("Canon EF-S 55-250mm f/4-5.6 IS STM"),
        493 => Some("Canon EF 40mm f/2.8 STM"),
        494 => Some("Canon EF-S 18-135mm f/3.5-5.6 IS STM"),
        495 => Some("Canon EF-S 18-55mm f/3.5-5.6 IS STM"),
        496 => Some("Canon EF 24-105mm f/3.5-5.6 IS STM"),
        499 => Some("Canon EF 200-400mm f/4L IS USM"),
        502 => Some("Canon EF 100-400mm f/4.5-5.6L IS II USM"),
        503 => Some("Canon EF 24-105mm f/4L IS II USM"),
        506 => Some("Canon EF 35mm f/1.4L II USM"),
        507 => Some("Canon EF 16-35mm f/2.8L III USM"),
        508 => Some("Canon EF 11-24mm f/4L USM"),
        747 => Some("Canon EF 100-400mm f/4.5-5.6L IS II USM"),
        748 => Some("Canon EF 100-400mm f/4.5-5.6L IS II USM + 1.4x III"),
        749 => Some("Canon EF 100-400mm f/4.5-5.6L IS II USM + 2x III"),
        750 => Some("Canon EF 35mm f/1.4L II USM"),
        751 => Some("Canon EF 16-35mm f/2.8L III USM"),
        752 => Some("Canon EF 24-105mm f/4L IS II USM"),
        // RF Mount Lenses (ID 61182 is generic RF)
        61182 => Some("Canon RF Lens"),
        // RF Lenses with specific IDs
        61491 => Some("Canon RF 50mm F1.2L USM"),
        61492 => Some("Canon RF 24-105mm F4L IS USM"),
        61493 => Some("Canon RF 28-70mm F2L USM"),
        61494 => Some("Canon RF 35mm F1.8 Macro IS STM"),
        61495 => Some("Canon RF 85mm F1.2L USM"),
        61496 => Some("Canon RF 85mm F1.2L USM DS"),
        61497 => Some("Canon RF 24-240mm F4-6.3 IS USM"),
        61498 => Some("Canon RF 70-200mm F2.8L IS USM"),
        61499 => Some("Canon RF 15-35mm F2.8L IS USM"),
        61500 => Some("Canon RF 24-70mm F2.8L IS USM"),
        61501 => Some("Canon RF 100-500mm F4.5-7.1L IS USM"),
        61502 => Some("Canon RF 600mm F11 IS STM"),
        61503 => Some("Canon RF 800mm F11 IS STM"),
        61504 => Some("Canon RF 85mm F2 Macro IS STM"),
        61505 => Some("Canon RF 100mm F2.8L Macro IS USM"),
        61506 => Some("Canon RF 50mm F1.8 STM"),
        61507 => Some("Canon RF 70-200mm F4L IS USM"),
        61508 => Some("Canon RF 14-35mm F4L IS USM"),
        61509 => Some("Canon RF 16mm F2.8 STM"),
        61510 => Some("Canon RF 100-400mm F5.6-8 IS USM"),
        61511 => Some("Canon RF 400mm F2.8L IS USM"),
        61512 => Some("Canon RF 600mm F4L IS USM"),
        _ => None,
    }
}

/// Get picture style name from ID
pub fn get_picture_style_name(style_id: u16) -> &'static str {
    match style_id {
        0x00 => "None",
        0x01 => "Standard",
        0x02 => "Portrait",
        0x03 => "High Saturation",
        0x04 => "Adobe RGB",
        0x05 => "Low Saturation",
        0x06 => "CM Set 1",
        0x07 => "CM Set 2",
        0x21 => "User Def. 1",
        0x22 => "User Def. 2",
        0x23 => "User Def. 3",
        0x41 => "PC 1",
        0x42 => "PC 2",
        0x43 => "PC 3",
        0x81 => "Standard",
        0x82 => "Portrait",
        0x83 => "Landscape",
        0x84 => "Neutral",
        0x85 => "Faithful",
        0x86 => "Monochrome",
        0x87 => "Auto",
        0x88 => "Fine Detail",
        0xff => "n/a",
        0xffff => "n/a",
        _ => "Unknown",
    }
}

/// Get the Canon model name from model ID
/// Based on ExifTool Canon.pm %canonModelID
pub fn get_canon_model_name(model_id: u32) -> Option<&'static str> {
    match model_id {
        // EOS DSLRs and Mirrorless (0x80000xxx range)
        0x80000001 => Some("EOS-1D"),
        0x80000167 => Some("EOS-1DS"),
        0x80000168 => Some("EOS 10D"),
        0x80000169 => Some("EOS-1D Mark III"),
        0x80000170 => Some("EOS Digital Rebel / 300D / Kiss Digital"),
        0x80000174 => Some("EOS-1D Mark II"),
        0x80000175 => Some("EOS 20D"),
        0x80000176 => Some("EOS Digital Rebel XSi / 450D / Kiss X2"),
        0x80000188 => Some("EOS-1Ds Mark II"),
        0x80000189 => Some("EOS Digital Rebel XT / 350D / Kiss Digital N"),
        0x80000190 => Some("EOS 40D"),
        0x80000213 => Some("EOS 5D"),
        0x80000215 => Some("EOS-1Ds Mark III"),
        0x80000218 => Some("EOS 5D Mark II"),
        0x80000232 => Some("EOS-1D Mark II N"),
        0x80000234 => Some("EOS 30D"),
        0x80000236 => Some("EOS Digital Rebel XTi / 400D / Kiss Digital X"),
        0x80000250 => Some("EOS 7D"),
        0x80000252 => Some("EOS Rebel T1i / 500D / Kiss X3"),
        0x80000254 => Some("EOS Rebel XS / 1000D / Kiss F"),
        0x80000261 => Some("EOS 50D"),
        0x80000269 => Some("EOS-1D X"),
        0x80000270 => Some("EOS Rebel T2i / 550D / Kiss X4"),
        0x80000281 => Some("EOS-1D Mark IV"),
        0x80000285 => Some("EOS 5D Mark III"),
        0x80000286 => Some("EOS Rebel T3i / 600D / Kiss X5"),
        0x80000287 => Some("EOS 60D"),
        0x80000288 => Some("EOS Rebel T3 / 1100D / Kiss X50"),
        0x80000289 => Some("EOS 7D Mark II"),
        0x80000301 => Some("EOS Rebel T4i / 650D / Kiss X6i"),
        0x80000302 => Some("EOS 6D"),
        0x80000324 => Some("EOS-1D C"),
        0x80000325 => Some("EOS 70D"),
        0x80000326 => Some("EOS Rebel T5i / 700D / Kiss X7i"),
        0x80000327 => Some("EOS Rebel T5 / 1200D / Kiss X70 / Hi"),
        0x80000328 => Some("EOS-1D X Mark II"),
        0x80000331 => Some("EOS M"),
        0x80000346 => Some("EOS Rebel SL1 / 100D / Kiss X7"),
        0x80000347 => Some("EOS Rebel T6s / 760D / 8000D"),
        0x80000349 => Some("EOS 5D Mark IV"),
        0x80000350 => Some("EOS 80D"),
        0x80000355 => Some("EOS M2"),
        0x80000382 => Some("EOS 5DS"),
        0x80000393 => Some("EOS Rebel T6i / 750D / Kiss X8i"),
        0x80000401 => Some("EOS 5DS R"),
        0x80000404 => Some("EOS Rebel T6 / 1300D / Kiss X80"),
        0x80000405 => Some("EOS Rebel T7i / 800D / Kiss X9i"),
        0x80000406 => Some("EOS 6D Mark II"),
        0x80000408 => Some("EOS 77D / 9000D"),
        0x80000417 => Some("EOS Rebel SL2 / 200D / Kiss X9"),
        0x80000421 => Some("EOS R5"),
        0x80000422 => Some("EOS Rebel T100 / 4000D / 3000D"),
        0x80000424 => Some("EOS R"),
        0x80000428 => Some("EOS-1D X Mark III"),
        0x80000432 => Some("EOS Rebel T7 / 2000D / 1500D / Kiss X90"),
        0x80000433 => Some("EOS RP"),
        0x80000435 => Some("EOS Rebel T8i / 850D / X10i"),
        0x80000436 => Some("EOS SL3 / 250D / Kiss X10"),
        0x80000437 => Some("EOS 90D"),
        0x80000450 => Some("EOS R3"),
        0x80000453 => Some("EOS R6"),
        0x80000464 => Some("EOS R7"),
        0x80000465 => Some("EOS R10"),
        0x80000467 => Some("PowerShot ZOOM"),
        0x80000468 => Some("EOS M50 Mark II / Kiss M2"),
        0x80000480 => Some("EOS R50"),
        0x80000481 => Some("EOS R6 Mark II"),
        0x80000487 => Some("EOS R8"),
        0x80000495 => Some("EOS R1"),
        0x80000496 => Some("EOS R5 Mark II"),
        0x80000497 => Some("PowerShot V1"),
        0x80000498 => Some("EOS R100"),
        0x80000516 => Some("EOS R50 V"),
        0x80000518 => Some("EOS R6 Mark III"),
        // PowerShot and compact models (0x0xxx0000 range)
        0x01010000 => Some("PowerShot A30"),
        0x01040000 => Some("PowerShot S300 / Digital IXUS 300 / IXY Digital 300"),
        0x01060000 => Some("PowerShot A20"),
        0x01080000 => Some("PowerShot A10"),
        0x01090000 => Some("PowerShot S110 / Digital IXUS v / IXY Digital 200"),
        0x01100000 => Some("PowerShot G2"),
        0x01110000 => Some("PowerShot S40"),
        0x01120000 => Some("PowerShot S30"),
        0x01130000 => Some("PowerShot A40"),
        0x01140000 => Some("EOS D30"),
        0x01150000 => Some("PowerShot A100"),
        0x01160000 => Some("PowerShot S200 / Digital IXUS v2 / IXY Digital 200a"),
        0x01170000 => Some("PowerShot A200"),
        0x01180000 => Some("PowerShot S330 / Digital IXUS 330 / IXY Digital 300a"),
        0x01190000 => Some("PowerShot G3"),
        0x01210000 => Some("PowerShot S45"),
        0x01230000 => Some("PowerShot SD100 / Digital IXUS II / IXY Digital 30"),
        0x01240000 => Some("PowerShot S230 / Digital IXUS v3 / IXY Digital 320"),
        0x01250000 => Some("PowerShot A70"),
        0x01260000 => Some("PowerShot A60"),
        0x01270000 => Some("PowerShot S400 / Digital IXUS 400 / IXY Digital 400"),
        0x01290000 => Some("PowerShot G5"),
        0x01300000 => Some("PowerShot A300"),
        0x01310000 => Some("PowerShot S50"),
        0x01340000 => Some("PowerShot A80"),
        0x01350000 => Some("PowerShot SD10 / Digital IXUS i / IXY Digital L"),
        0x01360000 => Some("PowerShot S1 IS"),
        0x01370000 => Some("PowerShot Pro1"),
        0x01380000 => Some("PowerShot S70"),
        0x01390000 => Some("PowerShot S60"),
        0x01400000 => Some("PowerShot G6"),
        0x01410000 => Some("PowerShot S500 / Digital IXUS 500 / IXY Digital 500"),
        0x01420000 => Some("PowerShot A75"),
        0x01440000 => Some("PowerShot SD110 / Digital IXUS IIs / IXY Digital 30a"),
        0x01450000 => Some("PowerShot A400"),
        0x01470000 => Some("PowerShot A310"),
        0x01490000 => Some("PowerShot A85"),
        0x01520000 => Some("PowerShot S410 / Digital IXUS 430 / IXY Digital 450"),
        0x01530000 => Some("PowerShot A95"),
        0x01540000 => Some("PowerShot SD300 / Digital IXUS 40 / IXY Digital 50"),
        0x01550000 => Some("PowerShot SD200 / Digital IXUS 30 / IXY Digital 40"),
        0x01560000 => Some("PowerShot A520"),
        0x01570000 => Some("PowerShot A510"),
        0x01590000 => Some("PowerShot SD20 / Digital IXUS i5 / IXY Digital L2"),
        0x01640000 => Some("PowerShot S2 IS"),
        0x01650000 => Some("PowerShot SD430 / Digital IXUS Wireless / IXY Digital Wireless"),
        0x01660000 => Some("PowerShot SD500 / Digital IXUS 700 / IXY Digital 600"),
        0x01700000 => Some("PowerShot SD30 / Digital IXUS i Zoom / IXY Digital L3"),
        0x01740000 => Some("PowerShot A430"),
        0x01750000 => Some("PowerShot A410"),
        0x01760000 => Some("PowerShot S80"),
        0x01780000 => Some("PowerShot A620"),
        0x01790000 => Some("PowerShot A610"),
        0x01800000 => Some("PowerShot SD630 / Digital IXUS 65 / IXY Digital 80"),
        0x01810000 => Some("PowerShot SD450 / Digital IXUS 55 / IXY Digital 60"),
        0x01820000 => Some("PowerShot TX1"),
        0x01870000 => Some("PowerShot SD400 / Digital IXUS 50 / IXY Digital 55"),
        0x01880000 => Some("PowerShot A420"),
        0x01890000 => Some("PowerShot SD900 / Digital IXUS 900 Ti / IXY Digital 1000"),
        0x01900000 => Some("PowerShot SD550 / Digital IXUS 750 / IXY Digital 700"),
        0x01920000 => Some("PowerShot A700"),
        0x01940000 => Some("PowerShot SD700 IS / Digital IXUS 800 IS / IXY Digital 800 IS"),
        0x01950000 => Some("PowerShot S3 IS"),
        0x01960000 => Some("PowerShot A540"),
        0x01970000 => Some("PowerShot SD600 / Digital IXUS 60 / IXY Digital 70"),
        0x01980000 => Some("PowerShot G7"),
        0x01990000 => Some("PowerShot A530"),
        0x02000000 => Some("PowerShot SD800 IS / Digital IXUS 850 IS / IXY Digital 900 IS"),
        0x02010000 => Some("PowerShot SD40 / Digital IXUS i7 / IXY Digital L4"),
        0x02020000 => Some("PowerShot A710 IS"),
        0x02030000 => Some("PowerShot A640"),
        0x02040000 => Some("PowerShot A630"),
        0x02090000 => Some("PowerShot S5 IS"),
        0x02100000 => Some("PowerShot A460"),
        0x02120000 => Some("PowerShot SD850 IS / Digital IXUS 950 IS / IXY Digital 810 IS"),
        0x02130000 => Some("PowerShot A570 IS"),
        0x02140000 => Some("PowerShot A560"),
        0x02150000 => Some("PowerShot SD750 / Digital IXUS 75 / IXY Digital 90"),
        0x02160000 => Some("PowerShot SD1000 / Digital IXUS 70 / IXY Digital 10"),
        0x02180000 => Some("PowerShot A550"),
        0x02190000 => Some("PowerShot A450"),
        0x02230000 => Some("PowerShot G9"),
        0x02240000 => Some("PowerShot A650 IS"),
        0x02260000 => Some("PowerShot A720 IS"),
        0x02290000 => Some("PowerShot SX100 IS"),
        0x02300000 => Some("PowerShot SD950 IS / Digital IXUS 960 IS / IXY Digital 2000 IS"),
        0x02310000 => Some("PowerShot SD870 IS / Digital IXUS 860 IS / IXY Digital 910 IS"),
        0x02320000 => Some("PowerShot SD890 IS / Digital IXUS 970 IS / IXY Digital 820 IS"),
        0x02360000 => Some("PowerShot SD790 IS / Digital IXUS 90 IS / IXY Digital 95 IS"),
        0x02370000 => Some("PowerShot SD770 IS / Digital IXUS 85 IS / IXY Digital 25 IS"),
        0x02380000 => Some("PowerShot A590 IS"),
        0x02390000 => Some("PowerShot A580"),
        0x02420000 => Some("PowerShot A470"),
        0x02430000 => Some("PowerShot SD1100 IS / Digital IXUS 80 IS / IXY Digital 20 IS"),
        0x02460000 => Some("PowerShot SX1 IS"),
        0x02470000 => Some("PowerShot SX10 IS"),
        0x02480000 => Some("PowerShot A1000 IS"),
        0x02490000 => Some("PowerShot G10"),
        0x02510000 => Some("PowerShot A2000 IS"),
        0x02520000 => Some("PowerShot SX110 IS"),
        0x02530000 => Some("PowerShot SD990 IS / Digital IXUS 980 IS / IXY Digital 3000 IS"),
        0x02540000 => Some("PowerShot SD880 IS / Digital IXUS 870 IS / IXY Digital 920 IS"),
        0x02550000 => Some("PowerShot E1"),
        0x02560000 => Some("PowerShot D10"),
        0x02570000 => Some("PowerShot SD960 IS / Digital IXUS 110 IS / IXY Digital 510 IS"),
        0x02580000 => Some("PowerShot A2100 IS"),
        0x02590000 => Some("PowerShot A480"),
        0x02600000 => Some("PowerShot SX200 IS"),
        0x02610000 => Some("PowerShot SD970 IS / Digital IXUS 990 IS / IXY Digital 830 IS"),
        0x02620000 => Some("PowerShot SD780 IS / Digital IXUS 100 IS / IXY Digital 210 IS"),
        0x02630000 => Some("PowerShot A1100 IS"),
        0x02640000 => Some("PowerShot SD1200 IS / Digital IXUS 95 IS / IXY Digital 110 IS"),
        0x02700000 => Some("PowerShot G11"),
        0x02710000 => Some("PowerShot SX120 IS"),
        0x02720000 => Some("PowerShot S90"),
        0x02750000 => Some("PowerShot SX20 IS"),
        0x02760000 => Some("PowerShot SD980 IS / Digital IXUS 200 IS / IXY Digital 930 IS"),
        0x02770000 => Some("PowerShot SD940 IS / Digital IXUS 120 IS / IXY Digital 220 IS"),
        0x02800000 => Some("PowerShot A495"),
        0x02810000 => Some("PowerShot A490"),
        0x02820000 => Some("PowerShot A3100/A3150 IS"),
        0x02830000 => Some("PowerShot A3000 IS"),
        0x02840000 => Some("PowerShot SD1400 IS / IXUS 130 / IXY 400F"),
        0x02850000 => Some("PowerShot SD1300 IS / IXUS 105 / IXY 200F"),
        0x02860000 => Some("PowerShot SD3500 IS / IXUS 210 / IXY 10S"),
        0x02870000 => Some("PowerShot SX210 IS"),
        0x02880000 => Some("PowerShot SD4000 IS / IXUS 300 HS / IXY 30S"),
        0x02890000 => Some("PowerShot SD4500 IS / IXUS 1000 HS / IXY 50S"),
        0x02920000 => Some("PowerShot G12"),
        0x02930000 => Some("PowerShot SX30 IS"),
        0x02940000 => Some("PowerShot SX130 IS"),
        0x02950000 => Some("PowerShot S95"),
        0x02980000 => Some("PowerShot A3300 IS"),
        0x02990000 => Some("PowerShot A3200 IS"),
        0x03000000 => Some("PowerShot ELPH 500 HS / IXUS 310 HS / IXY 31S"),
        0x03010000 => Some("PowerShot Pro90 IS"),
        0x03020000 => Some("PowerShot ELPH 100 HS / IXUS 115 HS / IXY 210F"),
        0x03030000 => Some("PowerShot SX230 HS"),
        0x03040000 => Some("PowerShot ELPH 300 HS / IXUS 220 HS / IXY 410F"),
        0x03050000 => Some("PowerShot A2200"),
        0x03060000 => Some("PowerShot A1200"),
        0x03070000 => Some("PowerShot SX220 HS"),
        0x03080000 => Some("PowerShot G1 X"),
        0x03090000 => Some("PowerShot SX150 IS"),
        0x03100000 => Some("PowerShot ELPH 510 HS / IXUS 1100 HS / IXY 51S"),
        0x03110000 => Some("PowerShot S100 (new)"),
        0x03120000 => Some("PowerShot ELPH 310 HS / IXUS 230 HS / IXY 600F"),
        0x03130000 => Some("PowerShot SX40 HS"),
        0x03140000 => Some("IXY 32S"),
        0x03160000 => Some("PowerShot A1300"),
        0x03170000 => Some("PowerShot A810"),
        0x03180000 => Some("PowerShot ELPH 320 HS / IXUS 240 HS / IXY 420F"),
        0x03190000 => Some("PowerShot ELPH 110 HS / IXUS 125 HS / IXY 220F"),
        0x03200000 => Some("PowerShot D20"),
        0x03210000 => Some("PowerShot A4000 IS"),
        0x03220000 => Some("PowerShot SX260 HS"),
        0x03230000 => Some("PowerShot SX240 HS"),
        0x03240000 => Some("PowerShot ELPH 530 HS / IXUS 510 HS / IXY 1"),
        0x03250000 => Some("PowerShot ELPH 520 HS / IXUS 500 HS / IXY 3"),
        0x03260000 => Some("PowerShot A3400 IS"),
        0x03270000 => Some("PowerShot A2400 IS"),
        0x03280000 => Some("PowerShot A2300"),
        0x03320000 => Some("PowerShot S100V"),
        0x03330000 => Some("PowerShot G15"),
        0x03340000 => Some("PowerShot SX50 HS"),
        0x03350000 => Some("PowerShot SX160 IS"),
        0x03360000 => Some("PowerShot S110 (new)"),
        0x03370000 => Some("PowerShot SX500 IS"),
        0x03380000 => Some("PowerShot N"),
        0x03390000 => Some("IXUS 245 HS / IXY 430F"),
        0x03400000 => Some("PowerShot SX280 HS"),
        0x03410000 => Some("PowerShot SX270 HS"),
        0x03420000 => Some("PowerShot A3500 IS"),
        0x03430000 => Some("PowerShot A2600"),
        0x03440000 => Some("PowerShot SX275 HS"),
        0x03450000 => Some("PowerShot A1400"),
        0x03460000 => Some("PowerShot ELPH 130 IS / IXUS 140 / IXY 110F"),
        0x03470000 => Some("PowerShot ELPH 115/120 IS / IXUS 132/135 / IXY 90F/100F"),
        0x03490000 => Some("PowerShot ELPH 330 HS / IXUS 255 HS / IXY 610F"),
        0x03510000 => Some("PowerShot A2500"),
        0x03540000 => Some("PowerShot G16"),
        0x03550000 => Some("PowerShot S120"),
        0x03560000 => Some("PowerShot SX170 IS"),
        0x03580000 => Some("PowerShot SX510 HS"),
        0x03590000 => Some("PowerShot S200 (new)"),
        0x03600000 => Some("IXY 620F"),
        0x03610000 => Some("PowerShot N100"),
        0x03640000 => Some("PowerShot G1 X Mark II"),
        0x03650000 => Some("PowerShot D30"),
        0x03660000 => Some("PowerShot SX700 HS"),
        0x03670000 => Some("PowerShot SX600 HS"),
        0x03680000 => Some("PowerShot ELPH 140 IS / IXUS 150 / IXY 130"),
        0x03690000 => Some("PowerShot ELPH 135 / IXUS 145 / IXY 120"),
        0x03700000 => Some("PowerShot ELPH 340 HS / IXUS 265 HS / IXY 630"),
        0x03710000 => Some("PowerShot ELPH 150 IS / IXUS 155 / IXY 140"),
        0x03740000 => Some("EOS M3"),
        0x03750000 => Some("PowerShot SX60 HS"),
        0x03760000 => Some("PowerShot SX520 HS"),
        0x03770000 => Some("PowerShot SX400 IS"),
        0x03780000 => Some("PowerShot G7 X"),
        0x03790000 => Some("PowerShot N2"),
        0x03800000 => Some("PowerShot SX530 HS"),
        0x03820000 => Some("PowerShot SX710 HS"),
        0x03830000 => Some("PowerShot SX610 HS"),
        0x03840000 => Some("EOS M10"),
        0x03850000 => Some("PowerShot G3 X"),
        0x03860000 => Some("PowerShot ELPH 165 HS / IXUS 165 / IXY 160"),
        0x03870000 => Some("PowerShot ELPH 160 / IXUS 160"),
        0x03880000 => Some("PowerShot ELPH 350 HS / IXUS 275 HS / IXY 640"),
        0x03890000 => Some("PowerShot ELPH 170 IS / IXUS 170"),
        0x03910000 => Some("PowerShot SX410 IS"),
        0x03930000 => Some("PowerShot G9 X"),
        0x03940000 => Some("EOS M5"),
        0x03950000 => Some("PowerShot G5 X"),
        0x03970000 => Some("PowerShot G7 X Mark II"),
        0x03980000 => Some("EOS M100"),
        0x03990000 => Some("PowerShot ELPH 360 HS / IXUS 285 HS / IXY 650"),
        0x04010000 => Some("PowerShot SX540 HS"),
        0x04020000 => Some("PowerShot SX420 IS"),
        0x04030000 => Some("PowerShot ELPH 190 IS / IXUS 180 / IXY 190"),
        0x04040000 => Some("PowerShot G1"),
        0x04050000 => Some("PowerShot SX720 HS"),
        0x04060000 => Some("PowerShot SX620 HS"),
        0x04070000 => Some("EOS M6"),
        0x04100000 => Some("PowerShot G9 X Mark II"),
        0x04150000 => Some("PowerShot ELPH 185 / IXUS 185 / IXY 200"),
        0x04160000 => Some("PowerShot SX430 IS"),
        0x04170000 => Some("PowerShot SX730 HS"),
        0x04180000 => Some("PowerShot G1 X Mark III"),
        0x06040000 => Some("PowerShot S100 / Digital IXUS / IXY Digital"),
        _ => None,
    }
}

/// Get the name of a Canon MakerNote tag
pub fn get_canon_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        CANON_CAMERA_SETTINGS => Some("CanonCameraSettings"),
        CANON_FOCAL_LENGTH => Some("CanonFocalLength"),
        CANON_FLASH_INFO => Some("CanonFlashInfo"),
        CANON_SHOT_INFO => Some("CanonShotInfo"),
        CANON_PANORAMA => Some("CanonPanorama"),
        CANON_IMAGE_TYPE => Some("CanonImageType"),
        CANON_FIRMWARE_VERSION => Some("CanonFirmwareVersion"),
        CANON_FILE_NUMBER => Some("FileNumber"),
        CANON_OWNER_NAME => Some("OwnerName"),
        CANON_SERIAL_NUMBER => Some("SerialNumber"),
        CANON_CAMERA_INFO => Some("CanonCameraInfo"),
        CANON_FILE_LENGTH => Some("FileLength"),
        CANON_CUSTOM_FUNCTIONS => Some("CanonCustomFunctions"),
        CANON_MODEL_ID => Some("CanonModelID"),
        CANON_MOVIE_INFO => Some("CanonMovieInfo"),
        CANON_AF_INFO => Some("CanonAFInfo"),
        CANON_THUMBNAIL_IMAGE_VALID_AREA => Some("ThumbnailImageValidArea"),
        CANON_SERIAL_NUMBER_FORMAT => Some("SerialNumberFormat"),
        CANON_SUPER_MACRO => Some("SuperMacro"),
        CANON_DATE_STAMP_MODE => Some("DateStampMode"),
        CANON_MY_COLORS => Some("MyColors"),
        CANON_FIRMWARE_REVISION => Some("FirmwareRevision"),
        CANON_CATEGORIES => Some("Categories"),
        CANON_FACE_DETECT => Some("FaceDetect1"),
        CANON_FACE_DETECT_2 => Some("FaceDetect2"),
        CANON_AF_INFO_2 => Some("AFInfo2"),
        CANON_CONTRAST_INFO => Some("ContrastInfo"),
        CANON_IMAGE_UNIQUE_ID => Some("ImageUniqueID"),
        CANON_WB_INFO => Some("WBInfo"),
        CANON_FACE_DETECT_3 => Some("FaceDetect3"),
        CANON_TIME_INFO => Some("TimeInfo"),
        CANON_BATTERY_TYPE => Some("BatteryType"),
        CANON_AF_INFO_3 => Some("AFInfo3"),
        CANON_RAW_DATA_OFFSET => Some("RawDataOffset"),
        CANON_ORIGINAL_DECISION_DATA_OFFSET => Some("OriginalDecisionDataOffset"),
        CANON_PERSONAL_FUNCTIONS => Some("PersonalFunctions"),
        CANON_PERSONAL_FUNCTION_VALUES => Some("PersonalFunctionValues"),
        CANON_FILE_INFO => Some("FileInfo"),
        CANON_AF_POINTS_IN_FOCUS_1D => Some("AFPointsInFocus1D"),
        CANON_LENS_MODEL => Some("LensModel"),
        CANON_SERIAL_INFO => Some("InternalSerialNumber"),
        CANON_DUST_REMOVAL_DATA => Some("DustRemovalData"),
        CANON_CROP_INFO => Some("CropInfo"),
        CANON_CUSTOM_FUNCTIONS_2 => Some("CustomFunctions2"),
        CANON_ASPECT_INFO => Some("AspectInfo"),
        CANON_PROCESSING_INFO => Some("ProcessingInfo"),
        CANON_TONE_CURVE_TABLE => Some("ToneCurveTable"),
        CANON_SHARPNESS_TABLE => Some("SharpnessTable"),
        CANON_SHARPNESS_FREQ_TABLE => Some("SharpnessFreqTable"),
        CANON_WHITE_BALANCE_TABLE => Some("WhiteBalanceTable"),
        CANON_COLOR_BALANCE => Some("ColorBalance"),
        CANON_MEASURED_COLOR => Some("MeasuredColor"),
        CANON_COLOR_TEMPERATURE => Some("ColorTemperature"),
        CANON_CANON_FLAGS => Some("CanonFlags"),
        CANON_MODIFIED_INFO => Some("ModifiedInfo"),
        CANON_TONE_CURVE_MATCHING => Some("ToneCurveMatching"),
        CANON_WHITE_BALANCE_MATCHING => Some("WhiteBalanceMatching"),
        CANON_COLOR_SPACE => Some("CanonColorSpace"),
        CANON_PREVIEW_IMAGE_INFO => Some("PreviewImageInfo"),
        CANON_VRD_OFFSET => Some("VRDOffset"),
        CANON_SENSOR_INFO => Some("SensorInfo"),
        CANON_COLOR_DATA => Some("ColorData"),
        CANON_CRWPARAM => Some("CRWParam"),
        CANON_COLOR_INFO => Some("ColorInfo"),
        CANON_FLAVOR => Some("Flavor"),
        CANON_PICTURE_STYLE_USER_DEF => Some("PictureStyleUserDef"),
        CANON_PICTURE_STYLE_PC => Some("PictureStylePC"),
        CANON_CUSTOM_PICTURE_STYLE_FILE_NAME => Some("CustomPictureStyleFileName"),
        CANON_AF_MICRO_ADJ => Some("AFMicroAdj"),
        CANON_VIGNETTING_CORR => Some("VignettingCorr"),
        CANON_VIGNETTING_CORR_2 => Some("VignettingCorr2"),
        CANON_LIGHTING_OPT => Some("LightingOpt"),
        CANON_LENS_INFO => Some("LensInfo"),
        CANON_AMBIANCE_INFO => Some("AmbianceInfo"),
        CANON_MULTI_EXP => Some("MultiExp"),
        CANON_FILTER_INFO => Some("FilterInfo"),
        CANON_HDR_INFO => Some("HDRInfo"),
        CANON_AF_CONFIG => Some("AFConfig"),
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

/// Format EV compensation as a fraction like ExifTool's PrintFraction
/// Examples: 0 -> "0", 1.0 -> "+1", -1.0 -> "-1", 0.333 -> "+1/3", 0.5 -> "+1/2"
fn format_ev_fraction(val: f64) -> String {
    if val == 0.0 {
        return "0".to_string();
    }

    // Avoid floating point errors
    let val = val * 1.00001;

    // Check if it's a whole number
    if ((val.round() / val) - 1.0).abs() < 0.001 {
        return format!("{:+}", val.round() as i32);
    }

    // Check if it's n/2
    let times2 = val * 2.0;
    if ((times2.round() / times2) - 1.0).abs() < 0.001 {
        return format!("{:+}/2", times2.round() as i32);
    }

    // Check if it's n/3
    let times3 = val * 3.0;
    if ((times3.round() / times3) - 1.0).abs() < 0.001 {
        return format!("{:+}/3", times3.round() as i32);
    }

    // Fall back to decimal format
    format!("{:+.3}", val)
}

// =============================================================================
// Dual-format decode functions (ExifTool and exiv2)
// Only _exiv2 versions created where values actually differ from ExifTool
// =============================================================================

// MacroMode: Canon.pm PrintConv / canonmn_int.cpp canonCsMacro
// Differs: exiftool uses "Macro"/"Normal", exiv2 uses "On"/"Off"
define_tag_decoder! {
    macro_mode,
    exiftool: {
        1 => "Macro",
        2 => "Normal",
    },
    exiv2: {
        1 => "On",
        2 => "Off",
    }
}

// Quality: Canon.pm PrintConv / canonmn_int.cpp canonCsQuality
define_tag_decoder! {
    quality,
    type: i16,
    exiftool: {
        -1 => "n/a",
        1 => "Economy",
        2 => "Normal",
        3 => "Fine",
        4 => "RAW",
        5 => "Superfine",
        7 => "CRAW",
        130 => "Light (RAW)",
        131 => "Standard (RAW)",
    },
    exiv2: {
        -1 => "n/a",
        0 => "unknown",
        1 => "Economy",
        2 => "Normal",
        3 => "Fine",
        4 => "RAW",
        5 => "Superfine",
        7 => "CRAW",
        130 => "Light (RAW)",
        131 => "Standard (RAW)",
    }
}

// FlashMode: Canon.pm PrintConv / canonmn_int.cpp canonCsFlashMode
// Note: ExifTool uses "Off" for value 0 per actual -j output
// Note: 65535 (-1 as signed) means "n/a" for EOS M MOV video etc.
define_tag_decoder! {
    flash_mode,
    exiftool: {
        0 => "Off",
        1 => "Auto",
        2 => "On",
        3 => "Red-eye reduction",
        4 => "Slow-sync",
        5 => "Red-eye reduction (Auto)",
        6 => "Red-eye reduction (On)",
        16 => "External flash",
        65535 => "n/a",
    },
    exiv2: {
        0 => "Off",
        1 => "Auto",
        2 => "On",
        3 => "Red-eye",
        4 => "Slow sync",
        5 => "Auto + red-eye",
        6 => "On + red-eye",
        16 => "External",
        65535 => "n/a",
    }
}

// DriveMode: Canon.pm PrintConv / canonmn_int.cpp canonCsDriveMode
define_tag_decoder! {
    drive_mode,
    exiftool: {
        0 => "Single",
        1 => "Continuous",
        2 => "Movie",
        3 => "Continuous, Speed Priority",
        4 => "Continuous, Low",
        5 => "Continuous, High",
        6 => "Silent Single",
        8 => "Continuous, High+",
        9 => "Single, Silent",
        10 => "Continuous, Silent",
    },
    exiv2: {
        0 => "Single / timer",
        1 => "Continuous",
        2 => "Movie",
        3 => "Continuous, speed priority",
        4 => "Continuous, low",
        5 => "Continuous, high",
        6 => "Silent Single",
        8 => "Continuous, high+",
        9 => "Single, Silent",
        10 => "Continuous, Silent",
    }
}

// FocusMode: Canon.pm PrintConv / canonmn_int.cpp canonCsFocusMode
define_tag_decoder! {
    focus_mode,
    exiftool: {
        0 => "One-shot AF",
        1 => "AI Servo AF",
        2 => "AI Focus AF",
        3 => "Manual Focus (3)",
        4 => "Single",
        5 => "Continuous",
        6 => "Manual Focus (6)",
        16 => "Pan Focus",
        256 => "One-shot AF (Live View)",
        257 => "AI Servo AF (Live View)",
        258 => "AI Focus AF (Live View)",
        512 => "Movie Snap Focus",
        519 => "Movie Servo AF",
    },
    exiv2: {
        0 => "One shot AF",
        1 => "AI servo AF",
        2 => "AI focus AF",
        3 => "Manual focus (3)",
        4 => "Single",
        5 => "Continuous",
        6 => "Manual focus (6)",
        16 => "Pan focus",
        256 => "AF + MF",
        512 => "Movie Snap Focus",
        519 => "Movie Servo AF",
    }
}

// MeteringMode: Canon.pm PrintConv / canonmn_int.cpp canonCsMeteringMode
define_tag_decoder! {
    metering_mode,
    exiftool: {
        0 => "Default",
        1 => "Spot",
        2 => "Average",
        3 => "Evaluative",
        4 => "Partial",
        5 => "Center-weighted average",
    },
    exiv2: {
        0 => "Default",
        1 => "Spot",
        2 => "Average",
        3 => "Evaluative",
        4 => "Partial",
        5 => "Center-weighted average",
    }
}

// FocusRange: Canon.pm PrintConv / canonmn_int.cpp canonCsFocusType
define_tag_decoder! {
    focus_range,
    exiftool: {
        0 => "Manual",
        1 => "Auto",
        2 => "Not Known",
        3 => "Macro",
        7 => "Very Close",
        8 => "Close",
        9 => "Middle Range",
        10 => "Far Range",
        11 => "Pan Focus",
        14 => "Super Macro",
        15 => "Infinity",
    },
    exiv2: {
        0 => "Manual",
        1 => "Auto",
        2 => "Not known",
        3 => "Macro",
        4 => "Very close",
        5 => "Close",
        6 => "Middle range",
        7 => "Far range",
        8 => "Pan focus",
        9 => "Super macro",
        10 => "Infinity",
    }
}

// ExposureMode: Canon.pm PrintConv / canonmn_int.cpp canonCsExposureProgram
define_tag_decoder! {
    exposure_mode,
    exiftool: {
        0 => "Easy",
        1 => "Program AE",
        2 => "Shutter speed priority AE",
        3 => "Aperture-priority AE",
        4 => "Manual",
        5 => "Depth-of-field AE",
        6 => "M-Dep",
        7 => "Bulb",
    },
    exiv2: {
        0 => "Easy shooting (Auto)",
        1 => "Program (P)",
        2 => "Shutter priority (Tv)",
        3 => "Aperture priority (Av)",
        4 => "Manual (M)",
        5 => "A-DEP",
        6 => "M-DEP",
        7 => "Bulb",
    }
}

// ImageStabilization: Canon.pm PrintConv (same for exiv2)
define_tag_decoder! {
    image_stabilization,
    both: {
        0 => "Off",
        1 => "On",
        2 => "Shoot Only",
        3 => "Panning",
        4 => "Dynamic",
        256 => "Off (2)",
        257 => "On (2)",
        258 => "Shoot Only (2)",
        259 => "Panning (2)",
        260 => "Dynamic (2)",
        0xFFFF => "n/a",
    }
}

// ManualFlashOutput: Canon.pm PrintConv (same for exiv2)
define_tag_decoder! {
    manual_flash_output,
    both: {
        0x0000 => "n/a",
        0x0500 => "Full",
        0x0502 => "Medium",
        0x0504 => "Low",
        0x7FFF => "n/a",
    }
}

// WhiteBalance: Canon.pm PrintConv / canonmn_int.cpp canonSiWhiteBalance
define_tag_decoder! {
    white_balance,
    exiftool: {
        0 => "Auto",
        1 => "Daylight",
        2 => "Cloudy",
        3 => "Tungsten",
        4 => "Fluorescent",
        5 => "Flash",
        6 => "Custom",
        7 => "Black & White",
        8 => "Shade",
        9 => "Manual Temperature",
        10 => "PC Set1",
        11 => "PC Set2",
        12 => "PC Set3",
        14 => "Daylight Fluorescent",
        15 => "Custom 1",
        16 => "Custom 2",
        17 => "Underwater",
    },
    exiv2: {
        0 => "Auto",
        1 => "Daylight",
        2 => "Cloudy",
        3 => "Tungsten",
        4 => "Fluorescent",
        5 => "Flash",
        6 => "Custom",
        7 => "Black & White",
        8 => "Shade",
        9 => "Manual Temperature (Kelvin)",
        10 => "PC Set 1",
        11 => "PC Set 2",
        12 => "PC Set 3",
        14 => "Daylight Fluorescent",
        15 => "Custom 1",
        16 => "Custom 2",
        17 => "Underwater",
        18 => "Custom 3",
        19 => "Custom 3",
        20 => "PC Set 4",
        21 => "PC Set 5",
        23 => "Auto (ambience priority)",
    }
}

// FocalType: Canon.pm PrintConv (same for exiv2)
define_tag_decoder! {
    focal_type,
    both: {
        0 => "n/a",
        1 => "Fixed",
        2 => "Zoom",
    }
}

// SlowShutter: Canon.pm PrintConv / canonmn_int.cpp slowShutter
define_tag_decoder! {
    slow_shutter,
    both: {
        0 => "Off",
        1 => "Night Scene",
        2 => "On",
        3 => "None",
        0xFFFF => "n/a",
    }
}

// AutoExposureBracketing: Canon.pm PrintConv / canonmn_int.cpp autoExposureBracketing
define_tag_decoder! {
    auto_exposure_bracketing,
    both: {
        0 => "Off",
        1 => "On (shot 1)",
        2 => "On (shot 2)",
        3 => "On (shot 3)",
        0xFFFF => "On",
    }
}

// AFPointsInFocus: Canon.pm (D30, D60, some PowerShot/Ixus)
define_tag_decoder! {
    af_points_in_focus,
    both: {
        0x3000 => "None (MF)",
        0x3001 => "Right",
        0x3002 => "Center",
        0x3003 => "Center+Right",
        0x3004 => "Left",
        0x3005 => "Left+Right",
        0x3006 => "Left+Center",
        0x3007 => "All",
    }
}

// ControlMode: Canon.pm ShotInfo index 18
define_tag_decoder! {
    control_mode,
    both: {
        0 => "n/a",
        1 => "Camera Local Control",
        3 => "Computer Remote Control",
    }
}

// CameraType: Canon.pm PrintConv / canonmn_int.cpp cameraType
define_tag_decoder! {
    camera_type,
    both: {
        0 => "n/a",
        248 => "EOS High-end",
        250 => "Compact",
        252 => "EOS Mid-range",
        255 => "DV Camera",
    }
}

// NDFilter: Canon.pm ShotInfo index 28
define_tag_decoder! {
    nd_filter,
    type: i16,
    both: {
        -1 => "n/a",
        0 => "Off",
        1 => "On",
    }
}

// AutoRotate: Canon.pm ShotInfo index 27
define_tag_decoder! {
    auto_rotate,
    type: i16,
    both: {
        -1 => "n/a",
        0 => "None",
        1 => "Rotate 90 CW",
        2 => "Rotate 180",
        3 => "Rotate 270 CW",
    }
}

// BracketMode: Canon.pm PrintConv
define_tag_decoder! {
    bracket_mode,
    both: {
        0 => "Off",
        1 => "AEB",
        2 => "FEB",
        3 => "ISO",
        4 => "WB",
    }
}

// AFAreaMode: Canon.pm PrintConv / canonmn_int.cpp canonAFAreaMode
define_tag_decoder! {
    af_area_mode,
    exiftool: {
        0 => "Off (Manual Focus)",
        1 => "AF Point Expansion (surround)",
        2 => "Single-point AF",
        4 => "Auto",
        5 => "Face Detect AF",
        6 => "Face + Tracking",
        7 => "Zone AF",
        8 => "AF Point Expansion (4 point)",
        9 => "Spot AF",
        10 => "AF Point Expansion (8 point)",
        11 => "Flexizone Multi",
        13 => "Flexizone Single",
        14 => "Large Zone AF",
        0x0060 => "Face AiAF",
        0x1001 => "Single-point AF (Compact)",
        0x1002 => "Tracking AF (Compact)",
        0x1003 => "Face + Tracking (Compact)",
        0xFFFF => "n/a",
    },
    exiv2: {
        0 => "Off (Manual Focus)",
        1 => "AF Point Expansion (surround)",
        2 => "Single-point AF",
        4 => "Multi-point AF",
        5 => "Face Detect AF",
        6 => "Face + Tracking",
        7 => "Zone AF",
        8 => "AF Point Expansion (4 point)",
        9 => "Spot AF",
        10 => "AF Point Expansion (8 point)",
        11 => "Flexizone Multi (49 point)",
        12 => "Flexizone Multi (9 point)",
        13 => "Flexizone Single",
        14 => "Large Zone AF",
    }
}

// DateStampMode: Canon.pm PrintConv
define_tag_decoder! {
    date_stamp_mode,
    both: {
        0 => "Off",
        1 => "Date",
        2 => "Date & Time",
    }
}

// DigitalZoom: Canon.pm PrintConv (same for exiv2)
define_tag_decoder! {
    digital_zoom,
    both: {
        0 => "None",
        1 => "2x",
        2 => "4x",
        3 => "Other",
    }
}

/// Decode Contrast - ExifTool format (from Canon.pm using printParameter)
pub fn decode_contrast_exiftool(value: u16) -> String {
    if value == 0x7fff {
        return "n/a".to_string();
    }
    let signed_val = if value > 0xfff0 {
        (value as i32) - 0x10000
    } else {
        value as i32
    };
    if signed_val == 0 {
        "Normal".to_string()
    } else if signed_val > 0 {
        format!("+{}", signed_val)
    } else {
        signed_val.to_string()
    }
}

// Contrast exiv2: canonmn_int.cpp canonCsLnh
define_tag_decoder! {
    contrast_lnh,
    both: {
        0xffff => "Low",
        0x0000 => "Normal",
        0x0001 => "High",
    }
}

/// Decode Saturation - ExifTool format
pub fn decode_saturation_exiftool(value: u16) -> String {
    if value == 0x7fff {
        return "n/a".to_string();
    }
    let signed_val = if value > 0xfff0 {
        (value as i32) - 0x10000
    } else {
        value as i32
    };
    if signed_val == 0 {
        "Normal".to_string()
    } else if signed_val > 0 {
        format!("+{}", signed_val)
    } else {
        signed_val.to_string()
    }
}

// Saturation exiv2: canonmn_int.cpp canonCsLnh
define_tag_decoder! {
    saturation_lnh,
    both: {
        0xffff => "Low",
        0x0000 => "Normal",
        0x0001 => "High",
    }
}

/// Decode Sharpness - ExifTool format
pub fn decode_sharpness_exiftool(value: u16) -> String {
    if value == 0x7fff {
        return "n/a".to_string();
    }
    if value > 0 {
        format!("+{}", value)
    } else {
        value.to_string()
    }
}

// Sharpness exiv2: canonmn_int.cpp canonCsLnh
define_tag_decoder! {
    sharpness_lnh,
    both: {
        0xffff => "Low",
        0x0000 => "Normal",
        0x0001 => "High",
    }
}

// RecordMode: Canon.pm PrintConv / canonmn_int.cpp
define_tag_decoder! {
    record_mode,
    type: i16,
    exiftool: {
        -1 => "n/a",
        1 => "JPEG",
        2 => "CRW+THM",
        3 => "AVI+THM",
        4 => "TIF",
        5 => "TIF+JPEG",
        6 => "CR2",
        7 => "CR2+JPEG",
        9 => "MOV",
        10 => "MP4",
        11 => "CRM",
        12 => "CR3",
        13 => "CR3+JPEG",
        14 => "HIF",
        15 => "CR3+HIF",
    },
    exiv2: {
        1 => "JPEG",
        2 => "CRW+THM",
        3 => "AVI+THM",
        4 => "TIF",
        5 => "TIF+JPEG",
        6 => "CR2",
        7 => "CR2+JPEG",
        9 => "MOV",
        10 => "MP4",
        11 => "CRM",
        12 => "CR3",
        13 => "CR3+JPEG",
        14 => "HIF",
        15 => "CR3+HIF",
    }
}

// CanonImageSize: Canon.pm PrintConv / canonmn_int.cpp
define_tag_decoder! {
    canon_image_size,
    type: i16,
    exiftool: {
        -1 => "n/a",
        0 => "Large",
        1 => "Medium",
        2 => "Small",
        5 => "Medium 1",
        6 => "Medium 2",
        7 => "Medium 3",
        8 => "Postcard",
        9 => "Widescreen",
        10 => "Medium Widescreen",
        14 => "Small 1",
        15 => "Small 2",
        16 => "Small 3",
        128 => "640x480 Movie",
        129 => "Medium Movie",
        130 => "Small Movie",
        137 => "1280x720 Movie",
        142 => "1920x1080 Movie",
        143 => "4096x2160 Movie",
    },
    exiv2: {
        0 => "Large",
        1 => "Medium",
        2 => "Small",
        5 => "Medium 1",
        6 => "Medium 2",
        7 => "Medium 3",
        8 => "Postcard",
        9 => "Widescreen",
        10 => "Medium Widescreen",
        14 => "Small 1",
        15 => "Small 2",
        16 => "Small 3",
        128 => "640x480 Movie",
        129 => "Medium Movie",
        130 => "Small Movie",
        137 => "1280x720 Movie",
        142 => "1920x1080 Movie",
        143 => "4096x2160 Movie",
    }
}

// EasyMode: Canon.pm (same for exiv2)
define_tag_decoder! {
    easy_mode,
    both: {
        0 => "Full auto",
        1 => "Manual",
        2 => "Landscape",
        3 => "Fast shutter",
        4 => "Slow shutter",
        5 => "Night",
        6 => "Gray Scale",
        7 => "Sepia",
        8 => "Portrait",
        9 => "Sports",
        10 => "Macro",
        11 => "Black & White",
        12 => "Pan focus",
        13 => "Vivid",
        14 => "Neutral",
        15 => "Flash Off",
        16 => "Long Shutter",
        17 => "Super Macro",
        18 => "Foliage",
        19 => "Indoor",
        20 => "Fireworks",
        21 => "Beach",
        22 => "Underwater",
        23 => "Snow",
        24 => "Kids & Pets",
        25 => "Night Snapshot",
        26 => "Digital Macro",
        27 => "My Colors",
        28 => "Movie Snap",
        29 => "Super Macro 2",
        30 => "Color Accent",
        31 => "Color Swap",
        32 => "Aquarium",
        33 => "ISO 3200",
        34 => "ISO 6400",
        35 => "Creative Light Effect",
        36 => "Easy",
        37 => "Quick Shot",
        38 => "Creative Auto",
        39 => "Zoom Blur",
        40 => "Low Light",
        41 => "Nostalgic",
        42 => "Super Vivid",
        43 => "Poster Effect",
        44 => "Face Self-timer",
        45 => "Smile",
        46 => "Wink Self-timer",
        47 => "Fisheye Effect",
        48 => "Miniature Effect",
        49 => "High-speed Burst",
        50 => "Best Image Selection",
        51 => "High Dynamic Range",
        52 => "Handheld Night Scene",
        53 => "Movie Digest",
        54 => "Live View Control",
        55 => "Discreet",
        56 => "Blur Reduction",
        57 => "Monochrome",
        58 => "Toy Camera Effect",
        59 => "Scene Intelligent Auto",
        60 => "High-speed Burst HQ",
        61 => "Smooth Skin",
        62 => "Soft Focus",
        68 => "Food",
        84 => "HDR Art Standard",
        85 => "HDR Art Vivid",
        93 => "HDR Art Bold",
        257 => "Spotlight",
        258 => "Night 2",
        259 => "Night+",
        260 => "Super Night",
        261 => "Sunset",
        263 => "Night Scene",
        264 => "Surface",
        265 => "Low Light 2",
    }
}

// AFPoint: Canon.pm PrintConv / canonmn_int.cpp
define_tag_decoder! {
    af_point,
    exiftool: {
        0x2005 => "Manual AF point selection",
        0x3000 => "None (MF)",
        0x3001 => "Auto AF point selection",
        0x3002 => "Right",
        0x3003 => "Center",
        0x3004 => "Left",
        0x4001 => "Auto AF point selection",
        0x4006 => "Face Detect",
    },
    exiv2: {
        0x2005 => "Manual AF point selection",
        0x3000 => "None (MF)",
        0x3001 => "Auto-selected",
        0x3002 => "Right",
        0x3003 => "Center",
        0x3004 => "Left",
        0x4001 => "Auto AF point selection",
        0x4006 => "Face Detect",
    }
}

// FocusContinuous: Canon.pm PrintConv
define_tag_decoder! {
    focus_continuous,
    type: i16,
    both: {
        -1 => "n/a",
        0 => "Single",
        1 => "Continuous",
        8 => "Manual",
    }
}

// AESetting: Canon.pm PrintConv
define_tag_decoder! {
    ae_setting,
    type: i16,
    both: {
        -1 => "n/a",
        0 => "Normal AE",
        1 => "Exposure Compensation",
        2 => "AE Lock",
        3 => "AE Lock + Exposure Comp.",
        4 => "No AE",
    }
}

// SpotMeteringMode: Canon.pm PrintConv
define_tag_decoder! {
    spot_metering_mode,
    type: i16,
    both: {
        -1 => "n/a",
        0 => "Center",
        1 => "AF Point",
    }
}

// PhotoEffect: Canon.pm PrintConv
define_tag_decoder! {
    photo_effect,
    type: i16,
    both: {
        -1 => "n/a",
        0 => "Off",
        1 => "Vivid",
        2 => "Neutral",
        3 => "Smooth",
        4 => "Sepia",
        5 => "B&W",
        6 => "Custom",
        100 => "My Color Data",
    }
}

/// Decode ColorTone - ExifTool format (uses printParameter like Contrast/Saturation)
/// printParameter: 0 => "Normal", positive => "+N", negative => "-N"
pub fn decode_color_tone_exiftool(value: u16) -> String {
    if value == 0x7fff {
        return "n/a".to_string();
    }
    if value == 0 {
        "Normal".to_string()
    } else {
        // Treat as signed value for display
        let signed = value as i16;
        if signed > 0 {
            format!("+{}", signed)
        } else {
            signed.to_string()
        }
    }
}

// ColorTone exiv2: canonmn_int.cpp canonCsLnh
define_tag_decoder! {
    color_tone_lnh,
    both: {
        0xffff => "Low",
        0x0000 => "Normal",
        0x0001 => "High",
    }
}

// SRAWQuality: Canon.pm PrintConv
define_tag_decoder! {
    sraw_quality,
    type: i16,
    both: {
        -1 => "n/a",
        0 => "n/a",
        1 => "sRAW1 (mRAW)",
        2 => "sRAW2 (sRAW)",
    }
}

// FocusBracketing: Canon.pm PrintConv (same for exiv2)
define_tag_decoder! {
    focus_bracketing,
    both: {
        0 => "Disable",
        1 => "Enable",
    }
}

// DaylightSavings: Canon.pm PrintConv (same for exiv2)
define_tag_decoder! {
    daylight_savings,
    both: {
        0 => "Off",
        60 => "On",
    }
}

// PictureStyle: Canon.pm pictureStyles (same for exiv2)
define_tag_decoder! {
    picture_style,
    both: {
        0x00 => "None",
        0x01 => "Standard",
        0x02 => "Portrait",
        0x03 => "High Saturation",
        0x04 => "Adobe RGB",
        0x05 => "Low Saturation",
        0x06 => "CM Set 1",
        0x07 => "CM Set 2",
        0x21 => "User Def. 1",
        0x22 => "User Def. 2",
        0x23 => "User Def. 3",
        0x41 => "PC 1",
        0x42 => "PC 2",
        0x43 => "PC 3",
        0x81 => "Standard",
        0x82 => "Portrait",
        0x83 => "Landscape",
        0x84 => "Neutral",
        0x85 => "Faithful",
        0x86 => "Monochrome",
        0x87 => "Auto",
        0x88 => "Fine Detail",
        0xff => "n/a",
        0xffff => "n/a",
    }
}

// ToneCurve: Canon.pm toneCurve
define_tag_decoder! {
    tone_curve,
    both: {
        0 => "Standard",
        1 => "Manual",
        2 => "Custom",
    }
}

// SharpnessFrequency: Canon.pm sharpnessFrequency
define_tag_decoder! {
    sharpness_frequency,
    both: {
        0 => "n/a",
        1 => "Lowest",
        2 => "Low",
        3 => "Standard",
        4 => "High",
        5 => "Highest",
    }
}

/// Decode MeasuredEV - ExifTool format (from Canon.pm PrintConv)
/// Simple passthrough that returns the value as a string
pub fn decode_measured_ev_exiftool(value: i16) -> String {
    value.to_string()
}

/// Decode MeasuredEV - exiv2 format (same as ExifTool)
pub fn decode_measured_ev_exiv2(value: i16) -> String {
    decode_measured_ev_exiftool(value)
}

/// Decode AFPointsInFocus1D - ExifTool format (from Canon.pm PrintAFPoints1D)
/// EOS 1D has 45 AF points in 5 rows: A1-7, B1-10, C1-11, D1-10, E1-7
/// Center point is C6
pub fn decode_af_points_in_focus_1d(data: &[u8]) -> String {
    if data.len() != 8 {
        return "Unknown".to_string();
    }

    // Focus point values for decoding the first byte
    // They are x/y positions: y is upper 3 bits, x is lower 5 bits
    const FOCUS_PTS: [u8; 56] = [
        0x00, 0x00, // padding
        0x04, 0x06, 0x08, 0x0a, 0x0c, 0x0e, 0x10, 0x00, 0x00, // Row A (7 points)
        0x21, 0x23, 0x25, 0x27, 0x29, 0x2b, 0x2d, 0x2f, 0x31, 0x33, // Row B (10 points)
        0x40, 0x42, 0x44, 0x46, 0x48, 0x4a, 0x4c, 0x4d, 0x50, 0x52, 0x54, // Row C (11 points)
        0x61, 0x63, 0x65, 0x67, 0x69, 0x6b, 0x6d, 0x6f, 0x71, 0x73, 0x00,
        0x00, // Row D (10 points)
        0x84, 0x86, 0x88, 0x8a, 0x8c, 0x8e, 0x90, 0x00, 0x00, 0x00, 0x00,
        0x00, // Row E (7 points)
    ];

    const ROWS: [char; 56] = [
        ' ', ' ', // padding
        'A', 'A', 'A', 'A', 'A', 'A', 'A', ' ', ' ', // Row A
        'B', 'B', 'B', 'B', 'B', 'B', 'B', 'B', 'B', 'B', // Row B
        'C', 'C', 'C', 'C', 'C', 'C', 'C', 'C', 'C', 'C', 'C', // Row C
        'D', 'D', 'D', 'D', 'D', 'D', 'D', 'D', 'D', 'D', ' ', ' ', // Row D
        'E', 'E', 'E', 'E', 'E', 'E', 'E', ' ', ' ', ' ', ' ', ' ', // Row E
    ];

    let focus = data[0];

    // Parse the bitmask from bytes 1-7
    let mut bits = Vec::with_capacity(56);
    for &byte in data.iter().take(8).skip(1) {
        for bit_idx in 0..8 {
            bits.push((byte >> bit_idx) & 1 != 0);
        }
    }

    let mut focusing = String::new();
    let mut points = Vec::new();
    let mut last_row = ' ';
    let mut col = 0u8;

    for (idx, &focus_pt) in FOCUS_PTS.iter().enumerate() {
        let row = ROWS[idx];
        if row == last_row {
            col += 1;
        } else {
            col = 1;
        }
        last_row = row;

        if row != ' ' {
            if focus == focus_pt {
                focusing = format!("{}{}", row, col);
            }
            if idx < bits.len() && bits[idx] {
                points.push(format!("{}{}", row, col));
            }
        }
    }

    if focusing.is_empty() {
        if focus == 0xff {
            focusing = "Auto".to_string();
        } else {
            focusing = format!("Unknown (0x{:02x})", focus);
        }
    }

    if points.is_empty() {
        format!("{} ()", focusing)
    } else {
        format!("{} ({})", focusing, points.join(","))
    }
}

// SuperMacro: Canon.pm / canonmn_int.cpp canonSuperMacro
define_tag_decoder! {
    super_macro,
    both: {
        0 => "Off",
        1 => "On (1)",
        2 => "On (2)",
    }
}

// AutoLightingOptimizer: Canon.pm / canonmn_int.cpp canonAutoLightingOptimizer
define_tag_decoder! {
    auto_lighting_optimizer,
    both: {
        0 => "Standard",
        1 => "Low",
        2 => "Strong",
        3 => "Off",
    }
}

// LongExposureNoiseReduction: Canon.pm / canonmn_int.cpp canonLongExposureNoiseReduction
define_tag_decoder! {
    long_exposure_noise_reduction,
    both: {
        0 => "Off",
        1 => "Auto",
        2 => "On",
    }
}

// HighISONoiseReduction: Canon.pm / canonmn_int.cpp canonHighISONoiseReduction
define_tag_decoder! {
    high_iso_noise_reduction,
    both: {
        0 => "Standard",
        1 => "Low",
        2 => "Strong",
        3 => "Off",
    }
}

// DigitalLensOptimizer: Canon.pm / canonmn_int.cpp canonDigitalLensOptimizer
define_tag_decoder! {
    digital_lens_optimizer,
    both: {
        0 => "Off",
        1 => "Standard",
        2 => "High",
    }
}

// ColorSpace: Canon.pm / canonmn_int.cpp canonColorSpace
define_tag_decoder! {
    color_space,
    exiftool: {
        1 => "sRGB",
        2 => "Adobe RGB",
    },
    exiv2: {
        1 => "sRGB",
        2 => "Adobe RGB",
        65535 => "n/a",
    }
}

// HDR: Canon.pm / canonmn_int.cpp canonHdr
define_tag_decoder! {
    hdr,
    exiftool: {
        0 => "Off",
        1 => "Auto",
        2 => "On",
    },
    exiv2: {
        0 => "Off",
        1 => "On",
        2 => "On (RAW)",
    }
}

// HDREffect: Canon.pm / canonmn_int.cpp canonHdrEffect
define_tag_decoder! {
    hdr_effect,
    both: {
        0 => "Natural",
        1 => "Art (standard)",
        2 => "Art (vivid)",
        3 => "Art (bold)",
        4 => "Art (embossed)",
    }
}

// CameraOrientation: Canon.pm
define_tag_decoder! {
    camera_orientation,
    both: {
        0 => "Horizontal (normal)",
        1 => "Rotate 90 CW",
        2 => "Rotate 270 CW",
    }
}

// DualPixelRaw: Canon.pm / canonmn_int.cpp canonDualPixelRaw
define_tag_decoder! {
    dual_pixel_raw,
    both: {
        0 => "Off",
        1 => "On",
    }
}

// SerialNumberFormat: Canon.pm / canonmn_int.cpp canonSerialNumberFormat
define_tag_decoder! {
    serial_number_format,
    type: u32,
    both: {
        0x90000000 => "Format 1",
        0xa0000000 => "Format 2",
    }
}

// MultiExposure: Canon.pm / canonmn_int.cpp canonMultiExposure
define_tag_decoder! {
    multi_exposure,
    both: {
        0 => "Off",
        1 => "On",
        2 => "On (RAW)",
    }
}

// MultiExposureControl: Canon.pm / canonmn_int.cpp canonMultiExposureControl
define_tag_decoder! {
    multi_exposure_control,
    both: {
        0 => "Additive",
        1 => "Average",
        2 => "Bright (comparative)",
        3 => "Dark (comparative)",
    }
}

// AmbienceSelection: Canon.pm / canonmn_int.cpp canonAmbienceSelection
define_tag_decoder! {
    ambience_selection,
    both: {
        0 => "Standard",
        1 => "Vivid",
        2 => "Warm",
        3 => "Soft",
        4 => "Cool",
        5 => "Intense",
        6 => "Brighter",
        7 => "Darker",
        8 => "Monochrome",
    }
}

// LongExposureNoiseReduction2: Canon.pm FileInfo tag 8
define_tag_decoder! {
    long_exposure_noise_reduction2,
    type: i16,
    both: {
        -1 => "n/a",
        0 => "Off",
        1 => "On (1D)",
        3 => "On",
        4 => "Auto",
    }
}

// WBBracketMode: Canon.pm FileInfo tag 9
define_tag_decoder! {
    wb_bracket_mode,
    both: {
        0 => "Off",
        1 => "On (shift AB)",
        2 => "On (shift GM)",
    }
}

// FilterEffect: Canon.pm FileInfo tag 14
define_tag_decoder! {
    filter_effect,
    type: i16,
    both: {
        -1 => "n/a",
        0 => "None",
        1 => "Yellow",
        2 => "Orange",
        3 => "Red",
        4 => "Green",
    }
}

// ToningEffect: Canon.pm FileInfo tag 15
define_tag_decoder! {
    toning_effect,
    type: i16,
    both: {
        -1 => "n/a",
        0 => "None",
        1 => "Sepia",
        2 => "Blue",
        3 => "Purple",
        4 => "Green",
    }
}

// ShutterMode: Canon.pm FileInfo tag 23
define_tag_decoder! {
    shutter_mode,
    both: {
        0 => "Mechanical",
        1 => "Electronic First Curtain",
        2 => "Electronic",
    }
}

// AntiFlicker: Canon.pm FileInfo tag 32 (same as OffOn)
define_tag_decoder! {
    anti_flicker,
    both: {
        0 => "Off",
        1 => "On",
    }
}

/// Parse a single IFD entry and return the tag value
///
/// Canon maker notes use offsets relative to the TIFF header, not the MakerNote data.
/// If tiff_data is provided, we use it to resolve offsets; otherwise we try the MakerNote data.
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
    // Canon maker notes use offsets relative to the TIFF header
    let value_data: &[u8] = if total_size <= 4 {
        value_offset_bytes
    } else {
        let offset = read_u32(value_offset_bytes, endian) as usize;

        // Try to use TIFF data first (Canon uses TIFF-relative offsets)
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
        1 => {
            // BYTE
            ExifValue::Byte(value_data[..count.min(value_data.len())].to_vec())
        }
        2 => {
            // ASCII
            let s = value_data[..count.min(value_data.len())]
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
                if i * 2 + 2 <= value_data.len() {
                    values.push(read_u16(&value_data[i * 2..], endian));
                }
            }
            ExifValue::Short(values)
        }
        4 => {
            // LONG
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                if i * 4 + 4 <= value_data.len() {
                    values.push(read_u32(&value_data[i * 4..], endian));
                }
            }
            ExifValue::Long(values)
        }
        5 => {
            // RATIONAL
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
        6 => {
            // SBYTE
            ExifValue::SByte(
                value_data[..count.min(value_data.len())]
                    .iter()
                    .map(|&b| b as i8)
                    .collect(),
            )
        }
        7 => {
            // UNDEFINED
            ExifValue::Undefined(value_data[..count.min(value_data.len())].to_vec())
        }
        8 => {
            // SSHORT
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                if i * 2 + 2 <= value_data.len() {
                    values.push(read_i16(&value_data[i * 2..], endian));
                }
            }
            ExifValue::SShort(values)
        }
        9 => {
            // SLONG
            let mut values = Vec::with_capacity(count);
            for i in 0..count {
                if i * 4 + 4 <= value_data.len() {
                    values.push(read_i32(&value_data[i * 4..], endian));
                }
            }
            ExifValue::SLong(values)
        }
        10 => {
            // SRATIONAL
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
        _ => return None,
    };

    Some((tag_id, value))
}

/// Decode Canon CameraSettings array sub-fields
///
/// This function decodes the CanonCameraSettings array into individual fields
/// that can be easily interpreted. Uses ExifTool format by default.
pub fn decode_camera_settings(data: &[u16]) -> HashMap<String, ExifValue> {
    decode_camera_settings_exiftool(data)
}

/// Decode Canon CameraSettings array sub-fields - ExifTool format
pub fn decode_camera_settings_exiftool(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // Macro mode (index 1)
    if data.len() > 1 {
        decoded.insert(
            "MacroMode".to_string(),
            ExifValue::Ascii(decode_macro_mode_exiftool(data[1]).to_string()),
        );
    }

    // Self timer (index 2)
    if data.len() > 2 {
        let self_timer = if data[2] == 0 {
            "Off".to_string()
        } else {
            // Value is in 1/10 seconds
            format!("{} s", data[2] as f64 / 10.0)
        };
        decoded.insert("SelfTimer".to_string(), ExifValue::Ascii(self_timer));
    }

    // Quality (index 3)
    if data.len() > 3 {
        decoded.insert(
            "Quality".to_string(),
            ExifValue::Ascii(decode_quality_exiftool(data[3] as i16).to_string()),
        );
    }

    // Flash mode (index 4) - ExifTool calls this "CanonFlashMode"
    if data.len() > 4 {
        decoded.insert(
            "CanonFlashMode".to_string(),
            ExifValue::Ascii(decode_flash_mode_exiftool(data[4]).to_string()),
        );
    }

    // Continuous drive (index 5)
    // Note: ExifTool calls this "ContinuousDrive", not "DriveMode"
    // The composite "DriveMode" tag is computed elsewhere
    if data.len() > 5 {
        decoded.insert(
            "ContinuousDrive".to_string(),
            ExifValue::Ascii(decode_drive_mode_exiftool(data[5]).to_string()),
        );
    }

    // Focus mode (index 7)
    if data.len() > 7 {
        decoded.insert(
            "FocusMode".to_string(),
            ExifValue::Ascii(decode_focus_mode_exiftool(data[7]).to_string()),
        );
    }

    // Record mode (index 9)
    if data.len() > 9 {
        let value = data[9] as i16;
        decoded.insert(
            "RecordMode".to_string(),
            ExifValue::Ascii(decode_record_mode_exiftool(value).to_string()),
        );
    }

    // Canon image size (index 10)
    if data.len() > 10 {
        let value = data[10] as i16;
        decoded.insert(
            "CanonImageSize".to_string(),
            ExifValue::Ascii(decode_canon_image_size_exiftool(value).to_string()),
        );
    }

    // Easy mode (index 11)
    if data.len() > 11 {
        decoded.insert(
            "EasyMode".to_string(),
            ExifValue::Ascii(decode_easy_mode_exiftool(data[11]).to_string()),
        );
    }

    // Digital zoom (index 12)
    if data.len() > 12 {
        decoded.insert(
            "DigitalZoom".to_string(),
            ExifValue::Ascii(decode_digital_zoom_exiftool(data[12]).to_string()),
        );
    }

    // Contrast (index 13)
    if data.len() > 13 && data[13] != 0x7fff {
        decoded.insert(
            "Contrast".to_string(),
            ExifValue::Ascii(decode_contrast_exiftool(data[13])),
        );
    }

    // Saturation (index 14)
    if data.len() > 14 && data[14] != 0x7fff {
        decoded.insert(
            "Saturation".to_string(),
            ExifValue::Ascii(decode_saturation_exiftool(data[14])),
        );
    }

    // Sharpness (index 15)
    if data.len() > 15 && data[15] != 0x7fff {
        decoded.insert(
            "Sharpness".to_string(),
            ExifValue::Ascii(decode_sharpness_exiftool(data[15])),
        );
    }

    // Metering mode (index 17)
    if data.len() > 17 {
        decoded.insert(
            "MeteringMode".to_string(),
            ExifValue::Ascii(decode_metering_mode_exiftool(data[17]).to_string()),
        );
    }

    // Focus range (index 18)
    if data.len() > 18 {
        decoded.insert(
            "FocusRange".to_string(),
            ExifValue::Ascii(decode_focus_range_exiftool(data[18]).to_string()),
        );
    }

    // AF point (index 19)
    if data.len() > 19 && data[19] != 0 {
        decoded.insert(
            "AFPoint".to_string(),
            ExifValue::Ascii(decode_af_point_exiftool(data[19]).to_string()),
        );
    }

    // Exposure mode (index 20) - ExifTool calls this "CanonExposureMode"
    if data.len() > 20 {
        decoded.insert(
            "CanonExposureMode".to_string(),
            ExifValue::Ascii(decode_exposure_mode_exiftool(data[20]).to_string()),
        );
    }

    // Lens type (index 22) - decode to lens name
    if data.len() > 22 && data[22] > 0 {
        if let Some(lens_name) = get_canon_lens_name(data[22]) {
            decoded.insert(
                "LensType".to_string(),
                ExifValue::Ascii(lens_name.to_string()),
            );
        }
    }

    // Focal units per mm (index 25) - needed for calculating actual focal lengths
    let focal_units = if data.len() > 25 && data[25] > 0 {
        data[25] as f64
    } else {
        1.0
    };

    // Max focal length (index 23)
    if data.len() > 23 {
        let fl = data[23] as f64 / focal_units;
        decoded.insert(
            "MaxFocalLength".to_string(),
            ExifValue::Ascii(format!("{} mm", fl as u32)),
        );
    }

    // Min focal length (index 24)
    if data.len() > 24 {
        let fl = data[24] as f64 / focal_units;
        decoded.insert(
            "MinFocalLength".to_string(),
            ExifValue::Ascii(format!("{} mm", fl as u32)),
        );
    }

    // Max aperture (index 26) - Convert Canon aperture value to f-number
    // Uses Canon EV encoding with special 1/3 and 2/3 stop codes
    if data.len() > 26 && data[26] > 0 {
        let f_number = canon_ev_to_fnumber(data[26]);
        let rounded = format_fnumber_2g(f_number);
        decoded.insert("MaxAperture".to_string(), ExifValue::Double(vec![rounded]));
    }

    // Min aperture (index 27) - Convert Canon aperture value to f-number
    if data.len() > 27 && data[27] > 0 {
        let f_number = canon_ev_to_fnumber(data[27]);
        // ExifTool rounds to integer for MinAperture
        let rounded = f_number.round();
        decoded.insert("MinAperture".to_string(), ExifValue::Double(vec![rounded]));
    }

    // Camera ISO (index 16)
    // Canon uses a special encoding: if bit 0x4000 is set, the lower 14 bits are the ISO value
    // Otherwise it's a lookup table value (0=n/a, 14=Auto High, 15=Auto, 16=50, 17=100, etc.)
    if data.len() > 16 && data[16] != 0x7FFF {
        let raw_iso = data[16];
        if raw_iso & 0x4000 != 0 {
            // Direct ISO value in lower 14 bits
            let iso_value = raw_iso & 0x3FFF;
            decoded.insert("CameraISO".to_string(), ExifValue::Short(vec![iso_value]));
        } else {
            // Lookup table value - some return strings, others return numbers
            match raw_iso {
                0 => {
                    decoded.insert("CameraISO".to_string(), ExifValue::Ascii("n/a".to_string()));
                }
                14 => {
                    decoded.insert(
                        "CameraISO".to_string(),
                        ExifValue::Ascii("Auto High".to_string()),
                    );
                }
                15 => {
                    decoded.insert(
                        "CameraISO".to_string(),
                        ExifValue::Ascii("Auto".to_string()),
                    );
                }
                16 => {
                    decoded.insert("CameraISO".to_string(), ExifValue::Short(vec![50]));
                }
                17 => {
                    decoded.insert("CameraISO".to_string(), ExifValue::Short(vec![100]));
                }
                18 => {
                    decoded.insert("CameraISO".to_string(), ExifValue::Short(vec![200]));
                }
                19 => {
                    decoded.insert("CameraISO".to_string(), ExifValue::Short(vec![400]));
                }
                20 => {
                    decoded.insert("CameraISO".to_string(), ExifValue::Short(vec![800]));
                }
                _ => {
                    // Unknown value - output as "Unknown (N)"
                    decoded.insert(
                        "CameraISO".to_string(),
                        ExifValue::Ascii(format!("Unknown ({})", raw_iso)),
                    );
                }
            }
        }
    }

    // Focal units per mm (index 25) - already calculated above as focal_units
    if data.len() > 25 && data[25] > 0 {
        decoded.insert(
            "FocalUnits".to_string(),
            ExifValue::Ascii(format!("{}/mm", data[25])),
        );
    }

    // Flash activity (index 28)
    if data.len() > 28 {
        decoded.insert(
            "FlashActivity".to_string(),
            ExifValue::Short(vec![data[28]]),
        );
    }

    // Flash bits (index 29)
    if data.len() > 29 {
        let bits = data[29];
        let flash_bits_desc = if bits == 0 {
            "(none)".to_string()
        } else {
            let mut descriptions = Vec::new();
            if bits & 0x0001 != 0 {
                descriptions.push("Manual");
            }
            if bits & 0x0002 != 0 {
                descriptions.push("TTL");
            }
            if bits & 0x0004 != 0 {
                descriptions.push("A-TTL");
            }
            if bits & 0x0008 != 0 {
                descriptions.push("E-TTL");
            }
            if bits & 0x0010 != 0 {
                descriptions.push("FP sync enabled");
            }
            if bits & 0x4000 != 0 {
                descriptions.push("External");
            }
            if bits & 0x2000 != 0 {
                descriptions.push("Built-in");
            }
            if descriptions.is_empty() {
                format!("Unknown (0x{:04x})", bits)
            } else {
                descriptions.join(", ")
            }
        };
        decoded.insert("FlashBits".to_string(), ExifValue::Ascii(flash_bits_desc));
    }

    // Focus continuous (index 32)
    if data.len() > 32 {
        let value = data[32] as i16;
        decoded.insert(
            "FocusContinuous".to_string(),
            ExifValue::Ascii(decode_focus_continuous_exiftool(value).to_string()),
        );
    }

    // AE setting (index 33)
    if data.len() > 33 {
        let value = data[33] as i16;
        decoded.insert(
            "AESetting".to_string(),
            ExifValue::Ascii(decode_ae_setting_exiftool(value).to_string()),
        );
    }

    // Image stabilization (index 34)
    if data.len() > 34 {
        decoded.insert(
            "ImageStabilization".to_string(),
            ExifValue::Ascii(decode_image_stabilization_exiftool(data[34]).to_string()),
        );
    }

    // Display aperture (index 35)
    if data.len() > 35 && data[35] > 0 {
        let aperture = data[35] as f64 / 10.0;
        decoded.insert(
            "DisplayAperture".to_string(),
            ExifValue::Ascii(format!("{:.1}", aperture)),
        );
    }

    // Zoom source width (index 36)
    if data.len() > 36 {
        decoded.insert(
            "ZoomSourceWidth".to_string(),
            ExifValue::Short(vec![data[36]]),
        );
    }

    // Zoom target width (index 37)
    if data.len() > 37 {
        decoded.insert(
            "ZoomTargetWidth".to_string(),
            ExifValue::Short(vec![data[37]]),
        );
    }

    // Spot metering mode (index 39)
    if data.len() > 39 {
        let value = data[39] as i16;
        decoded.insert(
            "SpotMeteringMode".to_string(),
            ExifValue::Ascii(decode_spot_metering_mode_exiftool(value).to_string()),
        );
    }

    // Photo effect (index 40)
    if data.len() > 40 {
        let value = data[40] as i16;
        decoded.insert(
            "PhotoEffect".to_string(),
            ExifValue::Ascii(decode_photo_effect_exiftool(value).to_string()),
        );
    }

    // Manual flash output (index 41)
    if data.len() > 41 {
        decoded.insert(
            "ManualFlashOutput".to_string(),
            ExifValue::Ascii(decode_manual_flash_output_exiftool(data[41]).to_string()),
        );
    }

    // Color tone (index 42)
    if data.len() > 42 && data[42] != 0x7fff {
        decoded.insert(
            "ColorTone".to_string(),
            ExifValue::Ascii(decode_color_tone_exiftool(data[42])),
        );
    }

    // SRAW quality (index 46)
    if data.len() > 46 {
        let value = data[46] as i16;
        decoded.insert(
            "SRAWQuality".to_string(),
            ExifValue::Ascii(decode_sraw_quality_exiftool(value).to_string()),
        );
    }

    // Focus bracketing (index 50) - EOS R models
    if data.len() > 50 {
        let focus_bracketing = match data[50] {
            0 => "Disable",
            1 => "Enable",
            _ => "Unknown",
        };
        decoded.insert(
            "FocusBracketing".to_string(),
            ExifValue::Ascii(focus_bracketing.to_string()),
        );
    }

    // Clarity (index 51) - EOS R models
    if data.len() > 51 && data[51] != 0x7fff {
        let value = data[51] as i16;
        let clarity = if value == 0 {
            "0".to_string()
        } else if value > 0 {
            format!("+{}", value)
        } else {
            format!("{}", value)
        };
        decoded.insert("Clarity".to_string(), ExifValue::Ascii(clarity));
    }

    // HDR-PQ (index 52)
    if data.len() > 52 {
        let hdr_pq = match data[52] as i16 {
            -1 => "n/a",
            0 => "Off",
            1 => "On",
            _ => "Unknown",
        };
        decoded.insert("HDR-PQ".to_string(), ExifValue::Ascii(hdr_pq.to_string()));
    }

    decoded
}

/// Decode Canon CameraSettings array sub-fields - exiv2 format
pub fn decode_camera_settings_exiv2(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // Macro mode (index 1)
    if data.len() > 1 {
        decoded.insert(
            "MacroMode".to_string(),
            ExifValue::Ascii(decode_macro_mode_exiv2(data[1]).to_string()),
        );
    }

    // Self timer (index 2)
    if data.len() > 2 {
        let self_timer = if data[2] == 0 {
            "Off".to_string()
        } else {
            format!("{} s", data[2] as f64 / 10.0)
        };
        decoded.insert("SelfTimer".to_string(), ExifValue::Ascii(self_timer));
    }

    // Quality (index 3)
    if data.len() > 3 {
        decoded.insert(
            "Quality".to_string(),
            ExifValue::Ascii(decode_quality_exiv2(data[3] as i16).to_string()),
        );
    }

    // Flash mode (index 4) - CanonFlashMode for consistency
    if data.len() > 4 {
        decoded.insert(
            "CanonFlashMode".to_string(),
            ExifValue::Ascii(decode_flash_mode_exiv2(data[4]).to_string()),
        );
    }

    // Continuous drive (index 5)
    // Note: This is "ContinuousDrive", not "DriveMode"
    if data.len() > 5 {
        decoded.insert(
            "ContinuousDrive".to_string(),
            ExifValue::Ascii(decode_drive_mode_exiv2(data[5]).to_string()),
        );
    }

    // Focus mode (index 7)
    if data.len() > 7 {
        decoded.insert(
            "FocusMode".to_string(),
            ExifValue::Ascii(decode_focus_mode_exiv2(data[7]).to_string()),
        );
    }

    // Record mode (index 9)
    if data.len() > 9 {
        decoded.insert(
            "RecordMode".to_string(),
            ExifValue::Ascii(decode_record_mode_exiv2(data[9] as i16).to_string()),
        );
    }

    // Canon image size (index 10)
    if data.len() > 10 {
        decoded.insert(
            "CanonImageSize".to_string(),
            ExifValue::Ascii(decode_canon_image_size_exiv2(data[10] as i16).to_string()),
        );
    }

    // Easy mode (index 11)
    if data.len() > 11 {
        decoded.insert(
            "EasyMode".to_string(),
            ExifValue::Ascii(decode_easy_mode_exiv2(data[11]).to_string()),
        );
    }

    // Digital zoom (index 12)
    if data.len() > 12 {
        decoded.insert(
            "DigitalZoom".to_string(),
            ExifValue::Ascii(decode_digital_zoom_exiv2(data[12]).to_string()),
        );
    }

    // Contrast (index 13)
    if data.len() > 13 {
        decoded.insert(
            "Contrast".to_string(),
            ExifValue::Ascii(decode_contrast_lnh_exiv2(data[13]).to_string()),
        );
    }

    // Saturation (index 14)
    if data.len() > 14 {
        decoded.insert(
            "Saturation".to_string(),
            ExifValue::Ascii(decode_saturation_lnh_exiv2(data[14]).to_string()),
        );
    }

    // Sharpness (index 15)
    if data.len() > 15 {
        decoded.insert(
            "Sharpness".to_string(),
            ExifValue::Ascii(decode_sharpness_lnh_exiv2(data[15]).to_string()),
        );
    }

    // Metering mode (index 17)
    if data.len() > 17 {
        decoded.insert(
            "MeteringMode".to_string(),
            ExifValue::Ascii(decode_metering_mode_exiv2(data[17]).to_string()),
        );
    }

    // Focus range (index 18)
    if data.len() > 18 {
        decoded.insert(
            "FocusRange".to_string(),
            ExifValue::Ascii(decode_focus_range_exiv2(data[18]).to_string()),
        );
    }

    // AF point (index 19)
    if data.len() > 19 && data[19] != 0 {
        decoded.insert(
            "AFPoint".to_string(),
            ExifValue::Ascii(decode_af_point_exiv2(data[19]).to_string()),
        );
    }

    // Exposure mode (index 20) - CanonExposureMode for consistency
    if data.len() > 20 {
        decoded.insert(
            "CanonExposureMode".to_string(),
            ExifValue::Ascii(decode_exposure_mode_exiv2(data[20]).to_string()),
        );
    }

    // Lens type (index 22) - decode to lens name
    if data.len() > 22 && data[22] > 0 {
        if let Some(lens_name) = get_canon_lens_name(data[22]) {
            decoded.insert(
                "LensType".to_string(),
                ExifValue::Ascii(lens_name.to_string()),
            );
        }
    }

    // Focal units per mm (index 25)
    let focal_units = if data.len() > 25 && data[25] > 0 {
        data[25] as f64
    } else {
        1.0
    };

    // Max focal length (index 23)
    if data.len() > 23 {
        let fl = data[23] as f64 / focal_units;
        decoded.insert(
            "MaxFocalLength".to_string(),
            ExifValue::Ascii(format!("{} mm", fl as u32)),
        );
    }

    // Min focal length (index 24)
    if data.len() > 24 {
        let fl = data[24] as f64 / focal_units;
        decoded.insert(
            "MinFocalLength".to_string(),
            ExifValue::Ascii(format!("{} mm", fl as u32)),
        );
    }

    // Max aperture (index 26) - Uses Canon EV encoding
    if data.len() > 26 && data[26] > 0 {
        let f_number = canon_ev_to_fnumber(data[26]);
        let rounded = format_fnumber_2g(f_number);
        decoded.insert("MaxAperture".to_string(), ExifValue::Double(vec![rounded]));
    }

    // Min aperture (index 27)
    if data.len() > 27 && data[27] > 0 {
        let f_number = canon_ev_to_fnumber(data[27]);
        let rounded = f_number.round();
        decoded.insert("MinAperture".to_string(), ExifValue::Double(vec![rounded]));
    }

    // Camera ISO (index 16)
    // Canon uses a special encoding: if bit 0x4000 is set, the lower 14 bits are the ISO value
    // Otherwise it's a lookup table value (0=n/a, 14=Auto High, 15=Auto, 16=50, 17=100, etc.)
    if data.len() > 16 && data[16] != 0x7FFF {
        let raw_iso = data[16];
        if raw_iso & 0x4000 != 0 {
            // Direct ISO value in lower 14 bits
            let iso_value = raw_iso & 0x3FFF;
            decoded.insert("CameraISO".to_string(), ExifValue::Short(vec![iso_value]));
        } else {
            // Lookup table value - some return strings, others return numbers
            match raw_iso {
                0 => {
                    decoded.insert("CameraISO".to_string(), ExifValue::Ascii("n/a".to_string()));
                }
                14 => {
                    decoded.insert(
                        "CameraISO".to_string(),
                        ExifValue::Ascii("Auto High".to_string()),
                    );
                }
                15 => {
                    decoded.insert(
                        "CameraISO".to_string(),
                        ExifValue::Ascii("Auto".to_string()),
                    );
                }
                16 => {
                    decoded.insert("CameraISO".to_string(), ExifValue::Short(vec![50]));
                }
                17 => {
                    decoded.insert("CameraISO".to_string(), ExifValue::Short(vec![100]));
                }
                18 => {
                    decoded.insert("CameraISO".to_string(), ExifValue::Short(vec![200]));
                }
                19 => {
                    decoded.insert("CameraISO".to_string(), ExifValue::Short(vec![400]));
                }
                20 => {
                    decoded.insert("CameraISO".to_string(), ExifValue::Short(vec![800]));
                }
                _ => {
                    // Unknown value - output as "Unknown (N)"
                    decoded.insert(
                        "CameraISO".to_string(),
                        ExifValue::Ascii(format!("Unknown ({})", raw_iso)),
                    );
                }
            }
        }
    }

    // Focal units per mm (index 25)
    if data.len() > 25 && data[25] > 0 {
        decoded.insert(
            "FocalUnits".to_string(),
            ExifValue::Ascii(format!("{}/mm", data[25])),
        );
    }

    // Flash activity (index 28)
    if data.len() > 28 {
        decoded.insert(
            "FlashActivity".to_string(),
            ExifValue::Short(vec![data[28]]),
        );
    }

    // Flash bits (index 29)
    if data.len() > 29 {
        let bits = data[29];
        let flash_bits_desc = if bits == 0 {
            "(none)".to_string()
        } else {
            let mut descriptions = Vec::new();
            if bits & 0x0001 != 0 {
                descriptions.push("Manual");
            }
            if bits & 0x0002 != 0 {
                descriptions.push("TTL");
            }
            if bits & 0x0004 != 0 {
                descriptions.push("A-TTL");
            }
            if bits & 0x0008 != 0 {
                descriptions.push("E-TTL");
            }
            if bits & 0x0010 != 0 {
                descriptions.push("FP sync enabled");
            }
            if bits & 0x4000 != 0 {
                descriptions.push("External");
            }
            if bits & 0x2000 != 0 {
                descriptions.push("Built-in");
            }
            if descriptions.is_empty() {
                format!("Unknown (0x{:04x})", bits)
            } else {
                descriptions.join(", ")
            }
        };
        decoded.insert("FlashBits".to_string(), ExifValue::Ascii(flash_bits_desc));
    }

    // Focus continuous (index 32)
    if data.len() > 32 {
        let value = data[32] as i16;
        decoded.insert(
            "FocusContinuous".to_string(),
            ExifValue::Ascii(decode_focus_continuous_exiv2(value).to_string()),
        );
    }

    // AE setting (index 33)
    if data.len() > 33 {
        let value = data[33] as i16;
        decoded.insert(
            "AESetting".to_string(),
            ExifValue::Ascii(decode_ae_setting_exiv2(value).to_string()),
        );
    }

    // Image stabilization (index 34) - same as exiftool
    if data.len() > 34 {
        decoded.insert(
            "ImageStabilization".to_string(),
            ExifValue::Ascii(decode_image_stabilization_exiftool(data[34]).to_string()),
        );
    }

    // Display aperture (index 35)
    if data.len() > 35 && data[35] > 0 {
        let aperture = data[35] as f64 / 10.0;
        decoded.insert(
            "DisplayAperture".to_string(),
            ExifValue::Ascii(format!("{:.1}", aperture)),
        );
    }

    // Zoom source width (index 36)
    if data.len() > 36 {
        decoded.insert(
            "ZoomSourceWidth".to_string(),
            ExifValue::Short(vec![data[36]]),
        );
    }

    // Zoom target width (index 37)
    if data.len() > 37 {
        decoded.insert(
            "ZoomTargetWidth".to_string(),
            ExifValue::Short(vec![data[37]]),
        );
    }

    // Spot metering mode (index 39)
    if data.len() > 39 {
        let value = data[39] as i16;
        decoded.insert(
            "SpotMeteringMode".to_string(),
            ExifValue::Ascii(decode_spot_metering_mode_exiv2(value).to_string()),
        );
    }

    // Photo effect (index 40)
    if data.len() > 40 {
        let value = data[40] as i16;
        decoded.insert(
            "PhotoEffect".to_string(),
            ExifValue::Ascii(decode_photo_effect_exiv2(value).to_string()),
        );
    }

    // Manual flash output (index 41) - same as exiftool
    if data.len() > 41 {
        decoded.insert(
            "ManualFlashOutput".to_string(),
            ExifValue::Ascii(decode_manual_flash_output_exiftool(data[41]).to_string()),
        );
    }

    // Color tone (index 42)
    if data.len() > 42 && data[42] != 0x7fff {
        decoded.insert(
            "ColorTone".to_string(),
            ExifValue::Ascii(decode_color_tone_lnh_exiv2(data[42]).to_string()),
        );
    }

    // SRAW quality (index 46)
    if data.len() > 46 {
        let value = data[46] as i16;
        decoded.insert(
            "SRAWQuality".to_string(),
            ExifValue::Ascii(decode_sraw_quality_exiv2(value).to_string()),
        );
    }

    // Focus bracketing (index 50) - EOS R models
    if data.len() > 50 {
        decoded.insert(
            "FocusBracketing".to_string(),
            ExifValue::Ascii(decode_focus_bracketing_exiv2(data[50]).to_string()),
        );
    }

    // Clarity (index 51) - EOS R models
    if data.len() > 51 && data[51] != 0x7fff {
        let value = data[51] as i16;
        let clarity = if value == 0 {
            "0".to_string()
        } else if value > 0 {
            format!("+{}", value)
        } else {
            format!("{}", value)
        };
        decoded.insert("Clarity".to_string(), ExifValue::Ascii(clarity));
    }

    // HDR-PQ (index 52)
    if data.len() > 52 {
        let hdr_pq = match data[52] as i16 {
            -1 => "n/a",
            0 => "Off",
            1 => "On",
            _ => "Unknown",
        };
        decoded.insert("HDR-PQ".to_string(), ExifValue::Ascii(hdr_pq.to_string()));
    }

    decoded
}

/// Decode Canon ShotInfo array sub-fields
///
/// This function decodes the CanonShotInfo array into individual fields
/// that can be easily interpreted. Uses ExifTool format by default.
pub fn decode_shot_info(data: &[u16]) -> HashMap<String, ExifValue> {
    decode_shot_info_exiftool(data)
}

/// Decode Canon ShotInfo array sub-fields - ExifTool format
pub fn decode_shot_info_exiftool(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // Auto ISO (index 1)
    // Formula: exp(val/32*log(2))*100
    if data.len() > 1 && data[1] as i16 != -1 {
        let val = data[1] as i16;
        let auto_iso = ((val as f64 / 32.0) * 2f64.ln()).exp() * 100.0;
        decoded.insert(
            "AutoISO".to_string(),
            ExifValue::Ascii(format!("{:.0}", auto_iso)),
        );
    }

    // Base ISO (index 2)
    // Formula: exp(val/32*log(2))*100/32
    if data.len() > 2 && data[2] > 0 {
        let val = data[2] as i16;
        let base_iso = ((val as f64 / 32.0) * 2f64.ln()).exp() * 100.0 / 32.0;
        decoded.insert(
            "BaseISO".to_string(),
            ExifValue::Ascii(format!("{:.0}", base_iso)),
        );
    }

    // Measured EV (index 3) - Canon APEX value, stored as value/32 with +5.0 offset
    // MeasuredEV represents the metered exposure value
    // ExifTool uses: MeasuredEV = (raw / 32.0) + 5.0
    // ExifTool uses banker's rounding (round half to even)
    if data.len() > 3 {
        let raw = data[3] as i16; // Treat as signed for negative EV values
        let ev = (raw as f64 / 32.0) + 5.0;
        // Banker's rounding to 2 decimal places
        let scaled = ev * 100.0;
        let rounded = if scaled.fract() == 0.5 {
            // Round half to even
            if (scaled.floor() as i64) % 2 == 0 {
                scaled.floor()
            } else {
                scaled.ceil()
            }
        } else {
            scaled.round()
        };
        decoded.insert(
            "MeasuredEV".to_string(),
            ExifValue::Double(vec![rounded / 100.0]),
        );
    }

    // Target aperture (index 4) - Canon APEX aperture value with Canon EV encoding
    if data.len() > 4 && data[4] > 0 {
        let f_number = canon_ev_to_fnumber(data[4]);
        let rounded = format_fnumber_2g(f_number);
        decoded.insert(
            "TargetAperture".to_string(),
            ExifValue::Double(vec![rounded]),
        );
    }

    // Target exposure time (index 5) - Canon APEX time value
    // exposure time = 2^(-CanonEv(value)), displayed as fraction
    if data.len() > 5 && data[5] > 0 {
        let ev = canon_ev(data[5] as i16);
        let time = 2f64.powf(-ev);
        // Format as fraction if less than 1 second
        let formatted = if time < 1.0 {
            let denominator = (1.0 / time).round() as u32;
            format!("1/{}", denominator)
        } else {
            format!("{}", time.round() as u32)
        };
        decoded.insert(
            "TargetExposureTime".to_string(),
            ExifValue::Ascii(formatted),
        );
    }

    // White balance (index 7)
    if data.len() > 7 {
        decoded.insert(
            "WhiteBalance".to_string(),
            ExifValue::Ascii(decode_white_balance_exiftool(data[7]).to_string()),
        );
    }

    // Slow shutter (index 8)
    if data.len() > 8 {
        decoded.insert(
            "SlowShutter".to_string(),
            ExifValue::Ascii(decode_slow_shutter_exiftool(data[8]).to_string()),
        );
    }

    // Sequence number (index 9)
    if data.len() > 9 {
        decoded.insert(
            "SequenceNumber".to_string(),
            ExifValue::Short(vec![data[9]]),
        );
    }

    // Optical zoom code (index 10)
    if data.len() > 10 {
        let zoom_str = if data[10] == 8 {
            "n/a".to_string()
        } else {
            data[10].to_string()
        };
        decoded.insert("OpticalZoomCode".to_string(), ExifValue::Ascii(zoom_str));
    }

    // Camera temperature (index 12)
    // Value stored as: actual_temp + 128
    // Only valid for some EOS models (not 1D/1DS)
    if data.len() > 12 && data[12] > 0 {
        let temp = (data[12] as i16) - 128;
        decoded.insert(
            "CameraTemperature".to_string(),
            ExifValue::Ascii(format!("{} C", temp)),
        );
    }

    // AF points in focus (index 14)
    // Used by D30, D60 and some PowerShot/Ixus models
    if data.len() > 14 && data[14] != 0 {
        decoded.insert(
            "AFPointsInFocus".to_string(),
            ExifValue::Ascii(decode_af_points_in_focus_exiftool(data[14]).to_string()),
        );
    }

    // Auto exposure bracketing (index 16)
    if data.len() > 16 {
        decoded.insert(
            "AutoExposureBracketing".to_string(),
            ExifValue::Ascii(decode_auto_exposure_bracketing_exiftool(data[16]).to_string()),
        );
    }

    // Flash exposure compensation (index 15)
    if data.len() > 15 {
        decoded.insert(
            "FlashExposureComp".to_string(),
            ExifValue::Short(vec![data[15]]),
        );
    }

    // AEB Bracket Value (index 17) - using CanonEv encoding
    // Note: index 16 is AutoExposureBracketing (On/Off), index 17 is the actual bracket value
    if data.len() > 17 {
        let raw = data[17] as i16;
        let aeb_value = canon_ev(raw);
        // Format with sign prefix ("+0.0" format), but for 0 output just "0"
        let formatted = if aeb_value == 0.0 {
            "0".to_string()
        } else if aeb_value > 0.0 {
            format!("+{:.1}", aeb_value)
        } else {
            format!("{:.1}", aeb_value)
        };
        decoded.insert("AEBBracketValue".to_string(), ExifValue::Ascii(formatted));
    }

    // Control mode (index 18)
    if data.len() > 18 {
        decoded.insert(
            "ControlMode".to_string(),
            ExifValue::Ascii(decode_control_mode_exiftool(data[18]).to_string()),
        );
    }

    // Measured EV 2 (index 23)
    // Formula: val / 8 - 6
    if data.len() > 23 && data[23] != 0 {
        let ev2 = (data[23] as f64 / 8.0) - 6.0;
        // Round to 3 decimal places
        let rounded = (ev2 * 1000.0).round() / 1000.0;
        decoded.insert("MeasuredEV2".to_string(), ExifValue::Double(vec![rounded]));
    }

    // Bulb duration (index 24)
    // Value stored as: duration * 10
    if data.len() > 24 {
        let duration = data[24] as f64 / 10.0;
        decoded.insert(
            "BulbDuration".to_string(),
            ExifValue::Ascii(format!("{}", duration as u32)),
        );
    }

    // Camera type (index 26)
    if data.len() > 26 {
        decoded.insert(
            "CameraType".to_string(),
            ExifValue::Ascii(decode_camera_type_exiftool(data[26]).to_string()),
        );
    }

    // Auto rotate (index 27)
    if data.len() > 27 && data[27] as i16 >= 0 {
        decoded.insert(
            "AutoRotate".to_string(),
            ExifValue::Ascii(decode_auto_rotate_exiftool(data[27] as i16).to_string()),
        );
    }

    // ND filter (index 28)
    if data.len() > 28 {
        decoded.insert(
            "NDFilter".to_string(),
            ExifValue::Ascii(decode_nd_filter_exiftool(data[28] as i16).to_string()),
        );
    }

    // Exposure compensation (index 6)
    if data.len() > 6 {
        // Canon stores exposure compensation using CanonEv encoding
        let raw = data[6] as i16;
        let ev_comp = canon_ev(raw);
        decoded.insert(
            "ExposureCompensation".to_string(),
            ExifValue::Ascii(format_ev_fraction(ev_comp)),
        );
    }

    // Flash guide number (index 13)
    if data.len() > 13 {
        decoded.insert(
            "FlashGuideNumber".to_string(),
            ExifValue::Short(vec![data[13]]),
        );
    }

    // Flash exposure compensation (index 15)
    if data.len() > 15 {
        let raw = data[15] as i16;
        let flash_comp = canon_ev(raw);
        decoded.insert(
            "FlashExposureComp".to_string(),
            ExifValue::Ascii(format_ev_fraction(flash_comp)),
        );
    }

    // Subject distance (index 19)
    if data.len() > 19 {
        decoded.insert(
            "SubjectDistance".to_string(),
            ExifValue::Short(vec![data[19]]),
        );
    }

    // FNumber (index 21) - Canon APEX aperture value
    // f-number = exp(CanonEv(value)*log(2)/2)
    if data.len() > 21 && data[21] != 0 {
        let raw = data[21] as i16;
        let f_number = (canon_ev(raw) * 2f64.ln() / 2.0).exp();
        let rounded = (f_number * 100.0).round() / 100.0;
        decoded.insert(
            "FNumber".to_string(),
            ExifValue::Ascii(format!("{:.2}", rounded)),
        );
    }

    // ExposureTime (index 22) - Canon APEX time value
    // exposure time = exp(-CanonEv(value)*log(2))
    if data.len() > 22 && data[22] != 0 {
        let raw = data[22] as i16;
        let time = (-canon_ev(raw) * 2f64.ln()).exp();
        // Format as fraction if less than 1 second
        let formatted = if time < 1.0 && time > 0.0 {
            let denominator = (1.0 / time).round() as u32;
            format!("1/{}", denominator)
        } else {
            format!("{:.1}", time)
        };
        decoded.insert("ExposureTime".to_string(), ExifValue::Ascii(formatted));
    }

    // FlashOutput (index 33) - PowerShot models only
    if data.len() > 33 && data[33] != 0 {
        decoded.insert("FlashOutput".to_string(), ExifValue::Short(vec![data[33]]));
    }

    decoded
}

/// Decode Canon ShotInfo array sub-fields - exiv2 format
pub fn decode_shot_info_exiv2(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // Auto ISO (index 1)
    if data.len() > 1 && data[1] as i16 != -1 {
        let val = data[1] as i16;
        let auto_iso = ((val as f64 / 32.0) * 2f64.ln()).exp() * 100.0;
        decoded.insert(
            "AutoISO".to_string(),
            ExifValue::Ascii(format!("{:.0}", auto_iso)),
        );
    }

    // Base ISO (index 2)
    if data.len() > 2 && data[2] > 0 {
        let val = data[2] as i16;
        let base_iso = ((val as f64 / 32.0) * 2f64.ln()).exp() * 100.0 / 32.0;
        decoded.insert(
            "BaseISO".to_string(),
            ExifValue::Ascii(format!("{:.0}", base_iso)),
        );
    }

    // Measured EV (index 3) - ExifTool uses banker's rounding
    if data.len() > 3 {
        let raw = data[3] as i16;
        let ev = (raw as f64 / 32.0) + 5.0;
        let scaled = ev * 100.0;
        let rounded = if scaled.fract() == 0.5 {
            if (scaled.floor() as i64) % 2 == 0 {
                scaled.floor()
            } else {
                scaled.ceil()
            }
        } else {
            scaled.round()
        };
        decoded.insert(
            "MeasuredEV".to_string(),
            ExifValue::Double(vec![rounded / 100.0]),
        );
    }

    // Target aperture (index 4) - Uses Canon EV encoding
    if data.len() > 4 && data[4] > 0 {
        let f_number = canon_ev_to_fnumber(data[4]);
        let rounded = format_fnumber_2g(f_number);
        decoded.insert(
            "TargetAperture".to_string(),
            ExifValue::Double(vec![rounded]),
        );
    }

    // Target exposure time (index 5)
    if data.len() > 5 && data[5] > 0 {
        let ev = canon_ev(data[5] as i16);
        let time = 2f64.powf(-ev);
        let formatted = if time < 1.0 {
            let denominator = (1.0 / time).round() as u32;
            format!("1/{}", denominator)
        } else {
            format!("{}", time.round() as u32)
        };
        decoded.insert(
            "TargetExposureTime".to_string(),
            ExifValue::Ascii(formatted),
        );
    }

    // White balance (index 7) - uses exiv2 format
    if data.len() > 7 {
        decoded.insert(
            "WhiteBalance".to_string(),
            ExifValue::Ascii(decode_white_balance_exiv2(data[7]).to_string()),
        );
    }

    // Slow shutter (index 8)
    if data.len() > 8 {
        decoded.insert(
            "SlowShutter".to_string(),
            ExifValue::Ascii(decode_slow_shutter_exiv2(data[8]).to_string()),
        );
    }

    // Sequence number (index 9)
    if data.len() > 9 {
        decoded.insert(
            "SequenceNumber".to_string(),
            ExifValue::Short(vec![data[9]]),
        );
    }

    // Optical zoom code (index 10)
    if data.len() > 10 {
        let zoom_str = if data[10] == 8 {
            "n/a".to_string()
        } else {
            data[10].to_string()
        };
        decoded.insert("OpticalZoomCode".to_string(), ExifValue::Ascii(zoom_str));
    }

    // Camera temperature (index 12)
    if data.len() > 12 && data[12] > 0 {
        let temp = (data[12] as i16) - 128;
        decoded.insert(
            "CameraTemperature".to_string(),
            ExifValue::Ascii(format!("{} C", temp)),
        );
    }

    // AF points in focus (index 14)
    if data.len() > 14 && data[14] != 0 {
        decoded.insert(
            "AFPointsInFocus".to_string(),
            ExifValue::Ascii(decode_af_points_in_focus_exiftool(data[14]).to_string()),
        );
    }

    // Auto exposure bracketing (index 16)
    if data.len() > 16 {
        decoded.insert(
            "AutoExposureBracketing".to_string(),
            ExifValue::Ascii(decode_auto_exposure_bracketing_exiv2(data[16]).to_string()),
        );
    }

    // Flash exposure compensation (index 15)
    if data.len() > 15 {
        decoded.insert(
            "FlashExposureComp".to_string(),
            ExifValue::Short(vec![data[15]]),
        );
    }

    // AEB Bracket Value (index 17)
    if data.len() > 17 {
        let raw = data[17] as i16;
        let aeb_value = canon_ev(raw);
        let formatted = if aeb_value == 0.0 {
            "0".to_string()
        } else if aeb_value > 0.0 {
            format!("+{:.1}", aeb_value)
        } else {
            format!("{:.1}", aeb_value)
        };
        decoded.insert("AEBBracketValue".to_string(), ExifValue::Ascii(formatted));
    }

    // Control mode (index 18) - same as exiftool
    if data.len() > 18 {
        decoded.insert(
            "ControlMode".to_string(),
            ExifValue::Ascii(decode_control_mode_exiftool(data[18]).to_string()),
        );
    }

    // Measured EV 2 (index 23)
    if data.len() > 23 && data[23] != 0 {
        let ev2 = (data[23] as f64 / 8.0) - 6.0;
        let rounded = (ev2 * 1000.0).round() / 1000.0;
        decoded.insert("MeasuredEV2".to_string(), ExifValue::Double(vec![rounded]));
    }

    // Bulb duration (index 24)
    if data.len() > 24 {
        let duration = data[24] as f64 / 10.0;
        decoded.insert(
            "BulbDuration".to_string(),
            ExifValue::Ascii(format!("{}", duration as u32)),
        );
    }

    // Camera type (index 26) - uses exiv2 format
    if data.len() > 26 {
        decoded.insert(
            "CameraType".to_string(),
            ExifValue::Ascii(decode_camera_type_exiv2(data[26]).to_string()),
        );
    }

    // Auto rotate (index 27) - same as exiftool
    if data.len() > 27 && data[27] as i16 >= 0 {
        decoded.insert(
            "AutoRotate".to_string(),
            ExifValue::Ascii(decode_auto_rotate_exiftool(data[27] as i16).to_string()),
        );
    }

    // ND filter (index 28) - same as exiftool
    if data.len() > 28 {
        decoded.insert(
            "NDFilter".to_string(),
            ExifValue::Ascii(decode_nd_filter_exiftool(data[28] as i16).to_string()),
        );
    }

    // Exposure compensation (index 6)
    if data.len() > 6 {
        let raw = data[6] as i16;
        let ev_comp = canon_ev(raw);
        decoded.insert(
            "ExposureCompensation".to_string(),
            ExifValue::Ascii(format_ev_fraction(ev_comp)),
        );
    }

    // Flash guide number (index 13)
    if data.len() > 13 {
        decoded.insert(
            "FlashGuideNumber".to_string(),
            ExifValue::Short(vec![data[13]]),
        );
    }

    // Flash exposure compensation (index 15)
    if data.len() > 15 {
        let raw = data[15] as i16;
        let flash_comp = canon_ev(raw);
        decoded.insert(
            "FlashExposureComp".to_string(),
            ExifValue::Ascii(format_ev_fraction(flash_comp)),
        );
    }

    // Subject distance (index 19)
    if data.len() > 19 {
        decoded.insert(
            "SubjectDistance".to_string(),
            ExifValue::Short(vec![data[19]]),
        );
    }

    // FNumber (index 21) - Canon APEX aperture value
    if data.len() > 21 && data[21] != 0 {
        let raw = data[21] as i16;
        let f_number = (canon_ev(raw) * 2f64.ln() / 2.0).exp();
        let rounded = (f_number * 100.0).round() / 100.0;
        decoded.insert(
            "FNumber".to_string(),
            ExifValue::Ascii(format!("{:.2}", rounded)),
        );
    }

    // ExposureTime (index 22) - Canon APEX time value
    if data.len() > 22 && data[22] != 0 {
        let raw = data[22] as i16;
        let time = (-canon_ev(raw) * 2f64.ln()).exp();
        let formatted = if time < 1.0 && time > 0.0 {
            let denominator = (1.0 / time).round() as u32;
            format!("1/{}", denominator)
        } else {
            format!("{:.1}", time)
        };
        decoded.insert("ExposureTime".to_string(), ExifValue::Ascii(formatted));
    }

    // FlashOutput (index 33) - PowerShot models only
    if data.len() > 33 && data[33] != 0 {
        decoded.insert("FlashOutput".to_string(), ExifValue::Short(vec![data[33]]));
    }

    decoded
}

/// Decode Canon FocalLength array sub-fields
///
/// This function decodes the CanonFocalLength array into individual fields
/// that can be easily interpreted. Uses ExifTool format by default.
pub fn decode_focal_length(data: &[u16]) -> HashMap<String, ExifValue> {
    decode_focal_length_exiftool(data)
}

/// Decode Canon FocalLength array sub-fields - ExifTool format
pub fn decode_focal_length_exiftool(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // Focal type (index 0)
    if !data.is_empty() {
        decoded.insert(
            "FocalType".to_string(),
            ExifValue::Ascii(decode_focal_type_exiftool(data[0]).to_string()),
        );
    }

    // Focal length (index 1)
    // Canon stores focal length multiplied by FocalUnits (typically 1)
    // ExifTool shows "55.0 mm" with one decimal place
    if data.len() > 1 {
        let fl = data[1] as f64;
        decoded.insert(
            "FocalLength".to_string(),
            ExifValue::Ascii(format!("{:.1} mm", fl)),
        );
    }

    // FocalPlaneXSize (index 2) - sensor width in mm
    // Note: Only valid for some models, affected by digital zoom
    // Value is stored in 1/1000 inch units, convert to mm using: value * 25.4 / 1000
    if data.len() > 2 && data[2] > 0 {
        let size_mm = data[2] as f64 * 25.4 / 1000.0;
        decoded.insert(
            "FocalPlaneXSize".to_string(),
            ExifValue::Ascii(format!("{:.2} mm", size_mm)),
        );
    }

    // FocalPlaneYSize (index 3) - sensor height in mm
    // Note: Only valid for some models, affected by digital zoom
    // Value is stored in 1/1000 inch units, convert to mm using: value * 25.4 / 1000
    if data.len() > 3 && data[3] > 0 {
        let size_mm = data[3] as f64 * 25.4 / 1000.0;
        decoded.insert(
            "FocalPlaneYSize".to_string(),
            ExifValue::Ascii(format!("{:.2} mm", size_mm)),
        );
    }

    decoded
}

// decode_focal_length_exiv2 - same as exiftool (FocalType values are identical)
// Use decode_focal_length_exiftool for both formats

/// Decode Canon FileInfo array sub-fields
///
/// This function decodes the CanonFileInfo array into individual fields
/// that can be easily interpreted. Uses ExifTool format by default.
pub fn decode_file_info(data: &[u16]) -> HashMap<String, ExifValue> {
    decode_file_info_exiftool(data)
}

/// Decode Canon FileInfo array sub-fields - ExifTool format
pub fn decode_file_info_exiftool(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // File number (index 1) - SKIP
    // The FileNumber at index 1 requires complex model-specific bit manipulation.
    // It's redundant with the main FileNumber tag (0x0008) which is already decoded.
    // See Canon.pm lines 6793-6850 for the complex conditional decoding logic.

    // Bracket mode (index 3)
    if data.len() > 3 {
        decoded.insert(
            "BracketMode".to_string(),
            ExifValue::Ascii(decode_bracket_mode_exiftool(data[3]).to_string()),
        );
    }

    // Bracket value (index 4)
    if data.len() > 4 {
        decoded.insert("BracketValue".to_string(), ExifValue::Short(vec![data[4]]));
    }

    // Bracket shot number (index 5)
    if data.len() > 5 {
        decoded.insert(
            "BracketShotNumber".to_string(),
            ExifValue::Short(vec![data[5]]),
        );
    }

    // RawJpgQuality (index 6)
    if data.len() > 6 {
        let val = data[6] as i16;
        if val > 0 {
            decoded.insert(
                "RawJpgQuality".to_string(),
                ExifValue::Ascii(decode_quality_exiftool(val).to_string()),
            );
        }
    }

    // RawJpgSize (index 7)
    if data.len() > 7 {
        let val = data[7] as i16;
        if val >= 0 {
            decoded.insert(
                "RawJpgSize".to_string(),
                ExifValue::Ascii(decode_canon_image_size_exiftool(val).to_string()),
            );
        }
    }

    // LongExposureNoiseReduction2 (index 8)
    if data.len() > 8 {
        let val = data[8] as i16;
        if val >= 0 {
            decoded.insert(
                "LongExposureNoiseReduction2".to_string(),
                ExifValue::Ascii(decode_long_exposure_noise_reduction2_exiftool(val).to_string()),
            );
        }
    }

    // WBBracketMode (index 9)
    if data.len() > 9 {
        decoded.insert(
            "WBBracketMode".to_string(),
            ExifValue::Ascii(decode_wb_bracket_mode_exiftool(data[9]).to_string()),
        );
    }

    // WBBracketValueAB (index 12)
    if data.len() > 12 {
        decoded.insert(
            "WBBracketValueAB".to_string(),
            ExifValue::Short(vec![data[12]]),
        );
    }

    // WBBracketValueGM (index 13)
    if data.len() > 13 {
        decoded.insert(
            "WBBracketValueGM".to_string(),
            ExifValue::Short(vec![data[13]]),
        );
    }

    // FilterEffect (index 14)
    if data.len() > 14 {
        let val = data[14] as i16;
        if val != -1 {
            decoded.insert(
                "FilterEffect".to_string(),
                ExifValue::Ascii(decode_filter_effect_exiftool(val).to_string()),
            );
        }
    }

    // ToningEffect (index 15)
    if data.len() > 15 {
        let val = data[15] as i16;
        if val != -1 {
            decoded.insert(
                "ToningEffect".to_string(),
                ExifValue::Ascii(decode_toning_effect_exiftool(val).to_string()),
            );
        }
    }

    // MacroMagnification (index 16) - skip for now
    // Requires lens type check and model-specific validation
    // See Canon.pm lines 6942-6954

    // LiveViewShooting (index 19)
    if data.len() > 19 {
        decoded.insert(
            "LiveViewShooting".to_string(),
            ExifValue::Ascii(if data[19] == 0 { "Off" } else { "On" }.to_string()),
        );
    }

    // FocusDistanceUpper (index 20)
    if data.len() > 20 {
        let val = data[20];
        if val > 0 {
            let meters = val as f64 / 100.0;
            if meters > 655.345 {
                decoded.insert(
                    "FocusDistanceUpper".to_string(),
                    ExifValue::Ascii("inf".to_string()),
                );
            } else {
                decoded.insert(
                    "FocusDistanceUpper".to_string(),
                    ExifValue::Ascii(format!("{:.2} m", meters)),
                );
            }
        }
    }

    // FocusDistanceLower (index 21)
    if data.len() > 21 && data.len() > 20 && data[20] > 0 {
        let val = data[21];
        let meters = val as f64 / 100.0;
        if meters > 655.345 {
            decoded.insert(
                "FocusDistanceLower".to_string(),
                ExifValue::Ascii("inf".to_string()),
            );
        } else {
            decoded.insert(
                "FocusDistanceLower".to_string(),
                ExifValue::Ascii(format!("{:.2} m", meters)),
            );
        }
    }

    // ShutterMode (index 23)
    if data.len() > 23 {
        decoded.insert(
            "ShutterMode".to_string(),
            ExifValue::Ascii(decode_shutter_mode_exiftool(data[23]).to_string()),
        );
    }

    // FlashExposureLock (index 25)
    if data.len() > 25 {
        decoded.insert(
            "FlashExposureLock".to_string(),
            ExifValue::Ascii(if data[25] == 0 { "Off" } else { "On" }.to_string()),
        );
    }

    // AntiFlicker (index 32)
    if data.len() > 32 {
        decoded.insert(
            "AntiFlicker".to_string(),
            ExifValue::Ascii(decode_anti_flicker_exiftool(data[32]).to_string()),
        );
    }

    // RFLensType (index 61/0x3d) - skip for now
    // This is a large lookup table and may be rarely used
    // See Canon.pm lines 7004-7083

    decoded
}

// decode_file_info_exiv2 - same as exiftool (BracketMode values are identical)
// Use decode_file_info_exiftool for both formats

/// Decode Canon ProcessingInfo array sub-fields
///
/// This function decodes the CanonProcessingInfo array (tag 0x00A0).
pub fn decode_processing_info(data: &[u16]) -> HashMap<String, ExifValue> {
    decode_processing_info_exiftool(data)
}

/// Decode Canon ProcessingInfo array sub-fields - ExifTool format
pub fn decode_processing_info_exiftool(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // ProcessingInfo structure (indices are 1-based in ExifTool docs):
    // Index 1: ToneCurve
    // Index 2: Sharpness
    // Index 3: SharpnessFrequency
    // Index 4: SensorRedLevel
    // Index 5: SensorBlueLevel
    // Index 6: WhiteBalanceRed
    // Index 7: WhiteBalanceBlue
    // Index 8: WhiteBalance
    // Index 9: ColorTemperature
    // Index 10: PictureStyle
    // Index 11: DigitalGain
    // Index 12: WBShiftAB
    // Index 13: WBShiftGM

    // ToneCurve (index 1)
    if data.len() > 1 {
        decoded.insert(
            "ToneCurve".to_string(),
            ExifValue::Ascii(decode_tone_curve_exiftool(data[1]).to_string()),
        );
    }

    // Sharpness (index 2) - numeric value
    // Note: ExifTool formatting varies by source block (CameraSettings uses +, ColorData doesn't)
    if data.len() > 2 {
        let sharpness = data[2] as i16;
        decoded.insert("Sharpness".to_string(), ExifValue::SShort(vec![sharpness]));
    }

    // SharpnessFrequency (index 3)
    if data.len() > 3 {
        decoded.insert(
            "SharpnessFrequency".to_string(),
            ExifValue::Ascii(decode_sharpness_frequency_exiftool(data[3]).to_string()),
        );
    }

    // SensorRedLevel (index 4)
    if data.len() > 4 {
        decoded.insert(
            "SensorRedLevel".to_string(),
            ExifValue::Short(vec![data[4]]),
        );
    }

    // SensorBlueLevel (index 5)
    if data.len() > 5 {
        decoded.insert(
            "SensorBlueLevel".to_string(),
            ExifValue::Short(vec![data[5]]),
        );
    }

    // WhiteBalanceRed (index 6)
    if data.len() > 6 {
        decoded.insert(
            "WhiteBalanceRed".to_string(),
            ExifValue::Short(vec![data[6]]),
        );
    }

    // WhiteBalanceBlue (index 7)
    if data.len() > 7 {
        decoded.insert(
            "WhiteBalanceBlue".to_string(),
            ExifValue::Short(vec![data[7]]),
        );
    }

    // ColorTemperature (index 9)
    if data.len() > 9 {
        decoded.insert(
            "ColorTemperature".to_string(),
            ExifValue::Short(vec![data[9]]),
        );
    }

    // PictureStyle (index 10)
    if data.len() > 10 {
        decoded.insert(
            "PictureStyle".to_string(),
            ExifValue::Ascii(decode_picture_style_exiftool(data[10]).to_string()),
        );
    }

    // DigitalGain (index 11)
    if data.len() > 11 && data[11] != 0 {
        decoded.insert("DigitalGain".to_string(), ExifValue::Short(vec![data[11]]));
    }

    // WBShiftAB (index 12) - signed
    if data.len() > 12 {
        let shift = data[12] as i16;
        decoded.insert("WBShiftAB".to_string(), ExifValue::SShort(vec![shift]));
    }

    // WBShiftGM (index 13) - signed
    if data.len() > 13 {
        let shift = data[13] as i16;
        decoded.insert("WBShiftGM".to_string(), ExifValue::SShort(vec![shift]));
    }

    decoded
}

/// Decode Canon AFInfo2 array sub-fields
///
/// This function decodes the CanonAFInfo2 array which contains AF area information.
/// Uses ExifTool format by default.
pub fn decode_af_info2(data: &[u16]) -> HashMap<String, ExifValue> {
    decode_af_info2_exiftool(data)
}

/// Decode Canon AFInfo2 array sub-fields - ExifTool format
pub fn decode_af_info2_exiftool(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // AFInfo2 structure (from ExifTool Canon.pm):
    // Index 0: Byte count/header
    // Index 1: AFAreaMode
    // Index 2: NumAFPoints
    // Index 3: ValidAFPoints
    // Index 4: CanonImageWidth (full sensor width)
    // Index 5: CanonImageHeight (full sensor height)
    // Index 6: AFImageWidth (AF area width)
    // Index 7: AFImageHeight (AF area height)
    // Index 8+: Widths[N], Heights[N], XPositions[N], YPositions[N]

    // AF area mode (index 1, after header)
    if data.len() > 1 {
        decoded.insert(
            "AFAreaMode".to_string(),
            ExifValue::Ascii(decode_af_area_mode_exiftool(data[1]).to_string()),
        );
    }

    // Number of AF points (index 2)
    if data.len() > 2 {
        let num_af_points = data[2];
        decoded.insert(
            "NumAFPoints".to_string(),
            ExifValue::Short(vec![num_af_points]),
        );

        // ValidAFPoints (index 3)
        if data.len() > 3 {
            decoded.insert("ValidAFPoints".to_string(), ExifValue::Short(vec![data[3]]));
        }

        // CanonImageWidth (index 4) - full sensor dimensions
        if data.len() > 4 {
            decoded.insert(
                "CanonImageWidth".to_string(),
                ExifValue::Short(vec![data[4]]),
            );
        }

        // CanonImageHeight (index 5)
        if data.len() > 5 {
            decoded.insert(
                "CanonImageHeight".to_string(),
                ExifValue::Short(vec![data[5]]),
            );
        }

        // AFImageWidth (index 6) - AF area dimensions
        if data.len() > 6 {
            decoded.insert("AFImageWidth".to_string(), ExifValue::Short(vec![data[6]]));
        }

        // AFImageHeight (index 7)
        if data.len() > 7 {
            decoded.insert("AFImageHeight".to_string(), ExifValue::Short(vec![data[7]]));
        }

        // AF area arrays start at index 8 (after the header fields)
        let base = 8usize;
        let n = num_af_points as usize;

        // Widths (index 3 to 3+N-1) - signed values
        if data.len() >= base + n {
            let widths: Vec<i16> = data[base..base + n].iter().map(|&v| v as i16).collect();
            // Format as space-separated string like ExifTool
            let widths_str = widths
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            decoded.insert("AFAreaWidths".to_string(), ExifValue::Ascii(widths_str));
        }

        // Heights (index 3+N to 3+2N-1) - signed values
        if data.len() >= base + 2 * n {
            let heights: Vec<i16> = data[base + n..base + 2 * n]
                .iter()
                .map(|&v| v as i16)
                .collect();
            let heights_str = heights
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            decoded.insert("AFAreaHeights".to_string(), ExifValue::Ascii(heights_str));
        }

        // X positions (index 3+2N to 3+3N-1) - signed values
        if data.len() >= base + 3 * n {
            let x_positions: Vec<i16> = data[base + 2 * n..base + 3 * n]
                .iter()
                .map(|&v| v as i16)
                .collect();
            let xpos_str = x_positions
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            decoded.insert("AFAreaXPositions".to_string(), ExifValue::Ascii(xpos_str));
        }

        // Y positions (index 3+3N to 3+4N-1) - signed values
        if data.len() >= base + 4 * n {
            let y_positions: Vec<i16> = data[base + 3 * n..base + 4 * n]
                .iter()
                .map(|&v| v as i16)
                .collect();
            let ypos_str = y_positions
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            decoded.insert("AFAreaYPositions".to_string(), ExifValue::Ascii(ypos_str));
        }
    }

    decoded
}

/// Decode Canon AFInfo2 array sub-fields - exiv2 format
pub fn decode_af_info2_exiv2(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // AFInfo2 structure:
    // Index 0: Byte count/header
    // Index 1: AFAreaMode
    // Index 2: NumAFPoints
    // Index 3-7: ValidAFPoints, AFImageWidth/Height, CanonImageWidth/Height
    // Index 8+: Widths[N], Heights[N], XPositions[N], YPositions[N]

    // AF area mode (index 1) - uses exiv2 format
    if data.len() > 1 {
        decoded.insert(
            "AFAreaMode".to_string(),
            ExifValue::Ascii(decode_af_area_mode_exiv2(data[1]).to_string()),
        );
    }

    // Number of AF points (index 2)
    if data.len() > 2 {
        let num_af_points = data[2];
        decoded.insert(
            "NumAFPoints".to_string(),
            ExifValue::Short(vec![num_af_points]),
        );

        let base = 8usize;
        let n = num_af_points as usize;

        // Widths - signed values
        if data.len() >= base + n {
            let widths: Vec<i16> = data[base..base + n].iter().map(|&v| v as i16).collect();
            let widths_str = widths
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            decoded.insert("AFAreaWidths".to_string(), ExifValue::Ascii(widths_str));
        }

        // Heights - signed values
        if data.len() >= base + 2 * n {
            let heights: Vec<i16> = data[base + n..base + 2 * n]
                .iter()
                .map(|&v| v as i16)
                .collect();
            let heights_str = heights
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            decoded.insert("AFAreaHeights".to_string(), ExifValue::Ascii(heights_str));
        }

        // X positions (signed)
        if data.len() >= base + 3 * n {
            let x_positions: Vec<i16> = data[base + 2 * n..base + 3 * n]
                .iter()
                .map(|&v| v as i16)
                .collect();
            let xpos_str = x_positions
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            decoded.insert("AFAreaXPositions".to_string(), ExifValue::Ascii(xpos_str));
        }

        // Y positions (signed)
        if data.len() >= base + 4 * n {
            let y_positions: Vec<i16> = data[base + 3 * n..base + 4 * n]
                .iter()
                .map(|&v| v as i16)
                .collect();
            let ypos_str = y_positions
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            decoded.insert("AFAreaYPositions".to_string(), ExifValue::Ascii(ypos_str));
        }
    }

    decoded
}

/// Decode Canon ColorData array sub-fields - ExifTool format
/// ColorData structure is version-dependent
pub fn decode_color_data(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    if data.is_empty() {
        return decoded;
    }

    // ColorDataVersion (index 0) - can be signed or unsigned
    let version = data[0] as i16;
    let version_str = match version {
        -5 => "-5 (M50V)",
        -4 => "-4 (R50)",
        -3 => "-3 (M10/M3)",
        -2 => "-2 (SX280HS/SX60HS)",
        -1 => "-1 (600HS/D10/SX260HS)",
        1 => "1 (1DmkIIN/5D/30D/400D)",
        2 => "2 (1DmkIII)",
        3 => "3 (40D)",
        4 => "4 (1DSmkIII)",
        5 => "5 (450D/1000D)",
        6 => "6 (50D/5DmkII)",
        7 => "7 (500D/550D/7D/1DmkIV)",
        9 => "9 (60D/1100D)",
        10 => "10 (1DX/5DmkIII/6D/70D/100D/650D/700D/M/M2)",
        11 => "11 (7DmkII/750D/760D/8000D)",
        12 => "12 (1DXmkII/5DS/5DSR)",
        13 => "13 (80D/5DmkIV)",
        14 => "14 (1300D/2000D/4000D)",
        15 => "15 (6DmkII/77D/200D/800D,9000D)",
        16 => "16 (M50)",
        17 => "17 (R)",
        18 => "18 (RP/250D)",
        19 => "19 (90D/850D/M6mkII/M200)",
        32 => "32 (1DXmkIII)",
        33 => "33 (R5/R6)",
        34 => "34 (R3)",
        48 => "48 (R7/R10/R50/R6mkII)",
        64 => "64 (R1/R5mkII)",
        65 => "65 (R50V)",
        _ => "Unknown",
    };
    decoded.insert(
        "ColorDataVersion".to_string(),
        ExifValue::Ascii(version_str.to_string()),
    );

    // ColorData4 (versions 2, 3, 4, 5, 6, 7, 9) - 1DmkIII, 40D, 1DSmkIII, 450D, 50D, 5DmkII, 7D, 60D, etc.
    // Has ColorCoefs subdirectory at offset 0x3f
    if matches!(version, 2 | 3 | 4 | 5 | 6 | 7 | 9) && data.len() > 0x3f + 0x68 {
        // ColorCoefs starts at index 0x3f (63)
        let colorcoefs_base = 0x3f;

        // WB_RGGBLevelsAsShot at ColorCoefs[0x00-0x03]
        let idx = colorcoefs_base;
        if data.len() > idx + 3 {
            let levels_str = format!(
                "{} {} {} {}",
                data[idx],
                data[idx + 1],
                data[idx + 2],
                data[idx + 3]
            );
            decoded.insert(
                "WB_RGGBLevelsAsShot".to_string(),
                ExifValue::Ascii(levels_str.clone()),
            );
            decoded.insert("WB_RGGBLevels".to_string(), ExifValue::Ascii(levels_str));
        }

        // ColorTempAsShot at ColorCoefs[0x04]
        if data.len() > colorcoefs_base + 4 {
            decoded.insert(
                "ColorTempAsShot".to_string(),
                ExifValue::Short(vec![data[colorcoefs_base + 4]]),
            );
        }

        // WB_RGGBLevelsAuto at ColorCoefs[0x05-0x08]
        let idx = colorcoefs_base + 5;
        if data.len() > idx + 3 {
            let levels_str = format!(
                "{} {} {} {}",
                data[idx],
                data[idx + 1],
                data[idx + 2],
                data[idx + 3]
            );
            decoded.insert(
                "WB_RGGBLevelsAuto".to_string(),
                ExifValue::Ascii(levels_str),
            );
        }

        // ColorTempAuto at ColorCoefs[0x09]
        if data.len() > colorcoefs_base + 9 {
            decoded.insert(
                "ColorTempAuto".to_string(),
                ExifValue::Short(vec![data[colorcoefs_base + 9]]),
            );
        }

        // WB_RGGBLevelsMeasured at ColorCoefs[0x0a-0x0d]
        let idx = colorcoefs_base + 0x0a;
        if data.len() > idx + 3 {
            let levels_str = format!(
                "{} {} {} {}",
                data[idx],
                data[idx + 1],
                data[idx + 2],
                data[idx + 3]
            );
            decoded.insert(
                "WB_RGGBLevelsMeasured".to_string(),
                ExifValue::Ascii(levels_str),
            );
        }

        // ColorTempMeasured at ColorCoefs[0x0e]
        if data.len() > colorcoefs_base + 0x0e {
            decoded.insert(
                "ColorTempMeasured".to_string(),
                ExifValue::Short(vec![data[colorcoefs_base + 0x0e]]),
            );
        }

        // WB_RGGBLevelsDaylight at ColorCoefs[0x14-0x17]
        let idx = colorcoefs_base + 0x14;
        if data.len() > idx + 3 {
            let levels_str = format!(
                "{} {} {} {}",
                data[idx],
                data[idx + 1],
                data[idx + 2],
                data[idx + 3]
            );
            decoded.insert(
                "WB_RGGBLevelsDaylight".to_string(),
                ExifValue::Ascii(levels_str),
            );
        }

        // ColorTempDaylight at ColorCoefs[0x18]
        if data.len() > colorcoefs_base + 0x18 {
            decoded.insert(
                "ColorTempDaylight".to_string(),
                ExifValue::Short(vec![data[colorcoefs_base + 0x18]]),
            );
        }

        // WB_RGGBLevelsShade at ColorCoefs[0x19-0x1c]
        let idx = colorcoefs_base + 0x19;
        if data.len() > idx + 3 {
            let levels_str = format!(
                "{} {} {} {}",
                data[idx],
                data[idx + 1],
                data[idx + 2],
                data[idx + 3]
            );
            decoded.insert(
                "WB_RGGBLevelsShade".to_string(),
                ExifValue::Ascii(levels_str),
            );
        }

        // ColorTempShade at ColorCoefs[0x1d]
        if data.len() > colorcoefs_base + 0x1d {
            decoded.insert(
                "ColorTempShade".to_string(),
                ExifValue::Short(vec![data[colorcoefs_base + 0x1d]]),
            );
        }

        // WB_RGGBLevelsCloudy at ColorCoefs[0x1e-0x21]
        let idx = colorcoefs_base + 0x1e;
        if data.len() > idx + 3 {
            let levels_str = format!(
                "{} {} {} {}",
                data[idx],
                data[idx + 1],
                data[idx + 2],
                data[idx + 3]
            );
            decoded.insert(
                "WB_RGGBLevelsCloudy".to_string(),
                ExifValue::Ascii(levels_str),
            );
        }

        // ColorTempCloudy at ColorCoefs[0x22]
        if data.len() > colorcoefs_base + 0x22 {
            decoded.insert(
                "ColorTempCloudy".to_string(),
                ExifValue::Short(vec![data[colorcoefs_base + 0x22]]),
            );
        }

        // WB_RGGBLevelsTungsten at ColorCoefs[0x23-0x26]
        let idx = colorcoefs_base + 0x23;
        if data.len() > idx + 3 {
            let levels_str = format!(
                "{} {} {} {}",
                data[idx],
                data[idx + 1],
                data[idx + 2],
                data[idx + 3]
            );
            decoded.insert(
                "WB_RGGBLevelsTungsten".to_string(),
                ExifValue::Ascii(levels_str),
            );
        }

        // ColorTempTungsten at ColorCoefs[0x27]
        if data.len() > colorcoefs_base + 0x27 {
            decoded.insert(
                "ColorTempTungsten".to_string(),
                ExifValue::Short(vec![data[colorcoefs_base + 0x27]]),
            );
        }

        // WB_RGGBLevelsFluorescent at ColorCoefs[0x28-0x2b]
        let idx = colorcoefs_base + 0x28;
        if data.len() > idx + 3 {
            let levels_str = format!(
                "{} {} {} {}",
                data[idx],
                data[idx + 1],
                data[idx + 2],
                data[idx + 3]
            );
            decoded.insert(
                "WB_RGGBLevelsFluorescent".to_string(),
                ExifValue::Ascii(levels_str),
            );
        }

        // ColorTempFluorescent at ColorCoefs[0x2c]
        if data.len() > colorcoefs_base + 0x2c {
            decoded.insert(
                "ColorTempFluorescent".to_string(),
                ExifValue::Short(vec![data[colorcoefs_base + 0x2c]]),
            );
        }

        // WB_RGGBLevelsKelvin at ColorCoefs[0x2d-0x30]
        let idx = colorcoefs_base + 0x2d;
        if data.len() > idx + 3 {
            let levels_str = format!(
                "{} {} {} {}",
                data[idx],
                data[idx + 1],
                data[idx + 2],
                data[idx + 3]
            );
            decoded.insert(
                "WB_RGGBLevelsKelvin".to_string(),
                ExifValue::Ascii(levels_str),
            );
        }

        // ColorTempKelvin at ColorCoefs[0x31]
        if data.len() > colorcoefs_base + 0x31 {
            decoded.insert(
                "ColorTempKelvin".to_string(),
                ExifValue::Short(vec![data[colorcoefs_base + 0x31]]),
            );
        }

        // WB_RGGBLevelsFlash at ColorCoefs[0x32-0x35]
        let idx = colorcoefs_base + 0x32;
        if data.len() > idx + 3 {
            let levels_str = format!(
                "{} {} {} {}",
                data[idx],
                data[idx + 1],
                data[idx + 2],
                data[idx + 3]
            );
            decoded.insert(
                "WB_RGGBLevelsFlash".to_string(),
                ExifValue::Ascii(levels_str),
            );
        }

        // ColorTempFlash at ColorCoefs[0x36]
        if data.len() > colorcoefs_base + 0x36 {
            decoded.insert(
                "ColorTempFlash".to_string(),
                ExifValue::Short(vec![data[colorcoefs_base + 0x36]]),
            );
        }

        // AverageBlackLevel at 0xe7 (for ColorData4)
        if data.len() > 0xe7 + 3 {
            let levels_str = format!(
                "{} {} {} {}",
                data[0xe7], data[0xe8], data[0xe9], data[0xea]
            );
            decoded.insert(
                "AverageBlackLevel".to_string(),
                ExifValue::Ascii(levels_str),
            );
        }

        // RawMeasuredRGGB at 0x280
        // Canon stores as int32u[4] where each int32u contains the same 16-bit value in both words
        // e.g., 0x05200520 for value 1312 (0x0520). Just extract one of the u16 values.
        if data.len() > 0x280 + 7 {
            let r = data[0x280] as u32;
            let g1 = data[0x282] as u32;
            let g2 = data[0x284] as u32;
            let b = data[0x286] as u32;
            let rggb_str = format!("{} {} {} {}", r, g1, g2, b);
            decoded.insert("RawMeasuredRGGB".to_string(), ExifValue::Ascii(rggb_str));
        }

        // PerChannelBlackLevel - varies by version
        if version == 6 || version == 7 {
            if data.len() > 0x2cb + 3 {
                let levels_str = format!(
                    "{} {} {} {}",
                    data[0x2cb], data[0x2cc], data[0x2cd], data[0x2ce]
                );
                decoded.insert(
                    "PerChannelBlackLevel".to_string(),
                    ExifValue::Ascii(levels_str),
                );
            }
            if data.len() > 0x2cf {
                let val = data[0x2cf];
                if val != 0 {
                    decoded.insert("NormalWhiteLevel".to_string(), ExifValue::Short(vec![val]));
                }
            }
            if data.len() > 0x2d0 {
                decoded.insert(
                    "SpecularWhiteLevel".to_string(),
                    ExifValue::Short(vec![data[0x2d0]]),
                );
            }
            if data.len() > 0x2d1 {
                decoded.insert(
                    "LinearityUpperMargin".to_string(),
                    ExifValue::Short(vec![data[0x2d1]]),
                );
            }
        } else if version == 9 {
            if data.len() > 0x2cf + 3 {
                let levels_str = format!(
                    "{} {} {} {}",
                    data[0x2cf], data[0x2d0], data[0x2d1], data[0x2d2]
                );
                decoded.insert(
                    "PerChannelBlackLevel".to_string(),
                    ExifValue::Ascii(levels_str),
                );
            }
            if data.len() > 0x2d3 {
                let val = data[0x2d3];
                if val != 0 {
                    decoded.insert("NormalWhiteLevel".to_string(), ExifValue::Short(vec![val]));
                }
            }
            if data.len() > 0x2d4 {
                decoded.insert(
                    "SpecularWhiteLevel".to_string(),
                    ExifValue::Short(vec![data[0x2d4]]),
                );
            }
            if data.len() > 0x2d5 {
                decoded.insert(
                    "LinearityUpperMargin".to_string(),
                    ExifValue::Short(vec![data[0x2d5]]),
                );
            }
        } else if version == 4 || version == 5 {
            if data.len() > 0x2b4 + 3 {
                let levels_str = format!(
                    "{} {} {} {}",
                    data[0x2b4], data[0x2b5], data[0x2b6], data[0x2b7]
                );
                decoded.insert(
                    "PerChannelBlackLevel".to_string(),
                    ExifValue::Ascii(levels_str),
                );
            }
            if data.len() > 0x2b8 {
                let val = data[0x2b8];
                if val != 0 {
                    decoded.insert("NormalWhiteLevel".to_string(), ExifValue::Short(vec![val]));
                }
            }
            if data.len() > 0x2b9 {
                decoded.insert(
                    "SpecularWhiteLevel".to_string(),
                    ExifValue::Short(vec![data[0x2b9]]),
                );
            }
            if data.len() > 0x2ba {
                decoded.insert(
                    "LinearityUpperMargin".to_string(),
                    ExifValue::Short(vec![data[0x2ba]]),
                );
            }
        }
    }
    // ColorData6 (version 10) - 600D/1200D
    else if version == 10 && data.len() > 0x3f + 0x68 {
        // Same structure as ColorData6 from Canon.pm
        decode_colordata6_or_7(data, &mut decoded, 0x3f);
    }
    // ColorData7 (version 10, 11) - 1DX/5DmkIII/7DmkII - uses different offsets than ColorData6
    // This is tricky because both ColorData6 and ColorData7 can have version 10
    // We'll handle this by checking array length (ColorData7 is 1312+ elements vs ColorData6 is 1273+)
    else if (version == 10 || version == 11) && data.len() >= 1312 {
        decode_colordata6_or_7(data, &mut decoded, 0x3f);

        // Additional fields for ColorData7
        if data.len() > 0x114 + 3 {
            let levels_str = format!(
                "{} {} {} {}",
                data[0x114], data[0x115], data[0x116], data[0x117]
            );
            decoded.insert(
                "AverageBlackLevel".to_string(),
                ExifValue::Ascii(levels_str),
            );
        }

        if version == 10 {
            if data.len() > 0x1ad + 7 {
                // Same format: extract one u16 value per color channel
                let r = data[0x1ad] as u32;
                let g1 = data[0x1af] as u32;
                let g2 = data[0x1b1] as u32;
                let b = data[0x1b3] as u32;
                let rggb_str = format!("{} {} {} {}", r, g1, g2, b);
                decoded.insert("RawMeasuredRGGB".to_string(), ExifValue::Ascii(rggb_str));
            }
            if data.len() > 0x1f8 + 3 {
                let levels_str = format!(
                    "{} {} {} {}",
                    data[0x1f8], data[0x1f9], data[0x1fa], data[0x1fb]
                );
                decoded.insert(
                    "PerChannelBlackLevel".to_string(),
                    ExifValue::Ascii(levels_str),
                );
            }
            if data.len() > 0x1fc {
                let val = data[0x1fc];
                if val != 0 {
                    decoded.insert("NormalWhiteLevel".to_string(), ExifValue::Short(vec![val]));
                }
            }
            if data.len() > 0x1fd {
                decoded.insert(
                    "SpecularWhiteLevel".to_string(),
                    ExifValue::Short(vec![data[0x1fd]]),
                );
            }
            if data.len() > 0x1fe {
                decoded.insert(
                    "LinearityUpperMargin".to_string(),
                    ExifValue::Short(vec![data[0x1fe]]),
                );
            }
        }
    }

    decoded
}

/// Helper to decode ColorData6/7 WB coefficients (common structure)
fn decode_colordata6_or_7(data: &[u16], decoded: &mut HashMap<String, ExifValue>, base: usize) {
    // WB_RGGBLevelsAsShot at base + 0x00
    let idx = base;
    if data.len() > idx + 3 {
        let levels_str = format!(
            "{} {} {} {}",
            data[idx],
            data[idx + 1],
            data[idx + 2],
            data[idx + 3]
        );
        decoded.insert(
            "WB_RGGBLevelsAsShot".to_string(),
            ExifValue::Ascii(levels_str.clone()),
        );
        decoded.insert("WB_RGGBLevels".to_string(), ExifValue::Ascii(levels_str));
    }

    // ColorTempAsShot at base + 0x04
    if data.len() > base + 4 {
        decoded.insert(
            "ColorTempAsShot".to_string(),
            ExifValue::Short(vec![data[base + 4]]),
        );
    }

    // WB_RGGBLevelsAuto at base + 0x05
    let idx = base + 5;
    if data.len() > idx + 3 {
        let levels_str = format!(
            "{} {} {} {}",
            data[idx],
            data[idx + 1],
            data[idx + 2],
            data[idx + 3]
        );
        decoded.insert(
            "WB_RGGBLevelsAuto".to_string(),
            ExifValue::Ascii(levels_str),
        );
    }

    // ColorTempAuto at base + 0x09
    if data.len() > base + 9 {
        decoded.insert(
            "ColorTempAuto".to_string(),
            ExifValue::Short(vec![data[base + 9]]),
        );
    }

    // WB_RGGBLevelsMeasured at base + 0x0a
    let idx = base + 0x0a;
    if data.len() > idx + 3 {
        let levels_str = format!(
            "{} {} {} {}",
            data[idx],
            data[idx + 1],
            data[idx + 2],
            data[idx + 3]
        );
        decoded.insert(
            "WB_RGGBLevelsMeasured".to_string(),
            ExifValue::Ascii(levels_str),
        );
    }

    // ColorTempMeasured at base + 0x0e
    if data.len() > base + 0x0e {
        decoded.insert(
            "ColorTempMeasured".to_string(),
            ExifValue::Short(vec![data[base + 0x0e]]),
        );
    }

    // WB_RGGBLevelsDaylight at base + 0x28 (for ColorData6/7)
    let idx = base + 0x28;
    if data.len() > idx + 3 {
        let levels_str = format!(
            "{} {} {} {}",
            data[idx],
            data[idx + 1],
            data[idx + 2],
            data[idx + 3]
        );
        decoded.insert(
            "WB_RGGBLevelsDaylight".to_string(),
            ExifValue::Ascii(levels_str),
        );
    }

    // ColorTempDaylight at base + 0x2c
    if data.len() > base + 0x2c {
        decoded.insert(
            "ColorTempDaylight".to_string(),
            ExifValue::Short(vec![data[base + 0x2c]]),
        );
    }

    // WB_RGGBLevelsShade at base + 0x2d
    let idx = base + 0x2d;
    if data.len() > idx + 3 {
        let levels_str = format!(
            "{} {} {} {}",
            data[idx],
            data[idx + 1],
            data[idx + 2],
            data[idx + 3]
        );
        decoded.insert(
            "WB_RGGBLevelsShade".to_string(),
            ExifValue::Ascii(levels_str),
        );
    }

    // ColorTempShade at base + 0x31
    if data.len() > base + 0x31 {
        decoded.insert(
            "ColorTempShade".to_string(),
            ExifValue::Short(vec![data[base + 0x31]]),
        );
    }

    // WB_RGGBLevelsCloudy at base + 0x32
    let idx = base + 0x32;
    if data.len() > idx + 3 {
        let levels_str = format!(
            "{} {} {} {}",
            data[idx],
            data[idx + 1],
            data[idx + 2],
            data[idx + 3]
        );
        decoded.insert(
            "WB_RGGBLevelsCloudy".to_string(),
            ExifValue::Ascii(levels_str),
        );
    }

    // ColorTempCloudy at base + 0x36
    if data.len() > base + 0x36 {
        decoded.insert(
            "ColorTempCloudy".to_string(),
            ExifValue::Short(vec![data[base + 0x36]]),
        );
    }

    // WB_RGGBLevelsTungsten at base + 0x37
    let idx = base + 0x37;
    if data.len() > idx + 3 {
        let levels_str = format!(
            "{} {} {} {}",
            data[idx],
            data[idx + 1],
            data[idx + 2],
            data[idx + 3]
        );
        decoded.insert(
            "WB_RGGBLevelsTungsten".to_string(),
            ExifValue::Ascii(levels_str),
        );
    }

    // ColorTempTungsten at base + 0x3b
    if data.len() > base + 0x3b {
        decoded.insert(
            "ColorTempTungsten".to_string(),
            ExifValue::Short(vec![data[base + 0x3b]]),
        );
    }

    // WB_RGGBLevelsFluorescent at base + 0x3c
    let idx = base + 0x3c;
    if data.len() > idx + 3 {
        let levels_str = format!(
            "{} {} {} {}",
            data[idx],
            data[idx + 1],
            data[idx + 2],
            data[idx + 3]
        );
        decoded.insert(
            "WB_RGGBLevelsFluorescent".to_string(),
            ExifValue::Ascii(levels_str),
        );
    }

    // ColorTempFluorescent at base + 0x40
    if data.len() > base + 0x40 {
        decoded.insert(
            "ColorTempFluorescent".to_string(),
            ExifValue::Short(vec![data[base + 0x40]]),
        );
    }

    // WB_RGGBLevelsKelvin at base + 0x41
    let idx = base + 0x41;
    if data.len() > idx + 3 {
        let levels_str = format!(
            "{} {} {} {}",
            data[idx],
            data[idx + 1],
            data[idx + 2],
            data[idx + 3]
        );
        decoded.insert(
            "WB_RGGBLevelsKelvin".to_string(),
            ExifValue::Ascii(levels_str),
        );
    }

    // ColorTempKelvin at base + 0x45
    if data.len() > base + 0x45 {
        decoded.insert(
            "ColorTempKelvin".to_string(),
            ExifValue::Short(vec![data[base + 0x45]]),
        );
    }

    // WB_RGGBLevelsFlash at base + 0x46
    let idx = base + 0x46;
    if data.len() > idx + 3 {
        let levels_str = format!(
            "{} {} {} {}",
            data[idx],
            data[idx + 1],
            data[idx + 2],
            data[idx + 3]
        );
        decoded.insert(
            "WB_RGGBLevelsFlash".to_string(),
            ExifValue::Ascii(levels_str),
        );
    }

    // ColorTempFlash at base + 0x4a
    if data.len() > base + 0x4a {
        decoded.insert(
            "ColorTempFlash".to_string(),
            ExifValue::Short(vec![data[base + 0x4a]]),
        );
    }
}

/// Decode Canon AFMicroAdj array sub-fields
pub fn decode_af_micro_adj(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    if data.is_empty() {
        return decoded;
    }

    // AFMicroAdj structure:
    // Index 0: Version (e.g., 20)
    // Index 1: AFMicroAdjMode
    // Index 2: AFMicroAdjValue (for mode 1)
    // Index 3-4: Additional adjustment values

    // AFMicroAdjMode (index 1)
    if data.len() > 1 {
        let mode = data[1];
        let mode_str = match mode {
            0 => "Disable",
            1 => "Adjust all by same amount",
            2 => "Adjust by lens",
            _ => "Unknown",
        };
        decoded.insert(
            "AFMicroAdjMode".to_string(),
            ExifValue::Ascii(mode_str.to_string()),
        );

        // AFMicroAdjValue (index 2)
        // Note: ExifTool's AFMicroadjustment composite comes from CanonCustom (0x0507),
        // not from this tag (0x4013), so we don't generate the composite here.
        if data.len() > 2 {
            let value = data[2] as i16;
            decoded.insert(
                "AFMicroAdjValue".to_string(),
                ExifValue::SShort(vec![value]),
            );
        }
    }

    decoded
}

/// Decode Canon VignettingCorr (tag 0x4015)
/// ExifTool: Canon.pm VignettingCorr
/// exiv2: canonmn_int.cpp tagInfoVigCor_
/// FORMAT: int16s
pub fn decode_vignetting_corr(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    if data.is_empty() {
        return decoded;
    }

    // Index 0: VignettingCorrVersion (int8u - first byte)
    if !data.is_empty() {
        let version = (data[0] & 0xFF) as u8;
        decoded.insert(
            "VignettingCorrVersion".to_string(),
            ExifValue::Byte(vec![version]),
        );
    }

    // Index 2: PeripheralLighting
    if data.len() > 2 {
        let value = data[2] as i16;
        let str_val = match value {
            0 => "Off",
            1 => "On",
            _ => "Unknown",
        };
        decoded.insert(
            "PeripheralLighting".to_string(),
            ExifValue::Ascii(str_val.to_string()),
        );
    }

    // Index 3: DistortionCorrection
    if data.len() > 3 {
        let value = data[3] as i16;
        let str_val = match value {
            0 => "Off",
            1 => "On",
            _ => "Unknown",
        };
        decoded.insert(
            "DistortionCorrection".to_string(),
            ExifValue::Ascii(str_val.to_string()),
        );
    }

    // Index 4: ChromaticAberrationCorr
    if data.len() > 4 {
        let value = data[4] as i16;
        let str_val = match value {
            0 => "Off",
            1 => "On",
            _ => "Unknown",
        };
        decoded.insert(
            "ChromaticAberrationCorr".to_string(),
            ExifValue::Ascii(str_val.to_string()),
        );
    }

    // Index 6: PeripheralLightingValue
    if data.len() > 6 {
        decoded.insert(
            "PeripheralLightingValue".to_string(),
            ExifValue::SShort(vec![data[6] as i16]),
        );
    }

    // Index 9: DistortionCorrectionValue
    if data.len() > 9 {
        decoded.insert(
            "DistortionCorrectionValue".to_string(),
            ExifValue::SShort(vec![data[9] as i16]),
        );
    }

    // Index 11: OriginalImageWidth
    if data.len() > 11 {
        decoded.insert(
            "OriginalImageWidth".to_string(),
            ExifValue::SShort(vec![data[11] as i16]),
        );
    }

    // Index 12: OriginalImageHeight
    if data.len() > 12 {
        decoded.insert(
            "OriginalImageHeight".to_string(),
            ExifValue::SShort(vec![data[12] as i16]),
        );
    }

    decoded
}

/// Decode Canon VignettingCorr2 (tag 0x4016)
/// ExifTool: Canon.pm VignettingCorr2
/// exiv2: canonmn_int.cpp tagInfoVigCor2_
/// FORMAT: int32s
pub fn decode_vignetting_corr2(data: &[u32]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    if data.is_empty() {
        return decoded;
    }

    // Index 5: PeripheralLightingSetting
    if data.len() > 5 {
        let value = data[5] as i32;
        let str_val = match value {
            0 => "Off",
            1 => "On",
            _ => "Unknown",
        };
        decoded.insert(
            "PeripheralLightingSetting".to_string(),
            ExifValue::Ascii(str_val.to_string()),
        );
    }

    // Index 6: ChromaticAberrationSetting
    if data.len() > 6 {
        let value = data[6] as i32;
        let str_val = match value {
            0 => "Off",
            1 => "On",
            _ => "Unknown",
        };
        decoded.insert(
            "ChromaticAberrationSetting".to_string(),
            ExifValue::Ascii(str_val.to_string()),
        );
    }

    // Index 7: DistortionCorrectionSetting
    if data.len() > 7 {
        let value = data[7] as i32;
        let str_val = match value {
            0 => "Off",
            1 => "On",
            _ => "Unknown",
        };
        decoded.insert(
            "DistortionCorrectionSetting".to_string(),
            ExifValue::Ascii(str_val.to_string()),
        );
    }

    // Index 9: DigitalLensOptimizerSetting
    if data.len() > 9 {
        let value = data[9] as i32;
        let str_val = match value {
            0 => "Off",
            1 => "On",
            _ => "Unknown",
        };
        decoded.insert(
            "DigitalLensOptimizerSetting".to_string(),
            ExifValue::Ascii(str_val.to_string()),
        );
    }

    decoded
}

/// Decode Canon LightingOpt (tag 0x4018)
/// ExifTool: Canon.pm LightingOpt
/// exiv2: canonmn_int.cpp tagInfoLiOp_
/// FORMAT: int32s
pub fn decode_lighting_opt(data: &[u32]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    if data.is_empty() {
        return decoded;
    }

    // Index 1: PeripheralIlluminationCorr
    if data.len() > 1 {
        let value = data[1] as i32;
        let str_val = match value {
            0 => "Off",
            1 => "On",
            _ => "Unknown",
        };
        decoded.insert(
            "PeripheralIlluminationCorr".to_string(),
            ExifValue::Ascii(str_val.to_string()),
        );
    }

    // Index 2: AutoLightingOptimizer
    if data.len() > 2 {
        let value = data[2] as i32;
        let alo_str = match value {
            0 => "Standard",
            1 => "Low",
            2 => "Strong",
            3 => "Off",
            _ => "Unknown",
        };
        decoded.insert(
            "AutoLightingOptimizer".to_string(),
            ExifValue::Ascii(alo_str.to_string()),
        );
    }

    // Index 3: HighlightTonePriority
    if data.len() > 3 {
        let value = data[3] as i32;
        let htp_str = match value {
            0 => "Off",
            1 => "On",
            2 => "Enhanced",
            _ => "Unknown",
        };
        decoded.insert(
            "HighlightTonePriority".to_string(),
            ExifValue::Ascii(htp_str.to_string()),
        );
    }

    // Index 4: LongExposureNoiseReduction
    if data.len() > 4 {
        let value = data[4] as i32;
        let lenr_str = match value {
            0 => "Off",
            1 => "Auto",
            2 => "On",
            _ => "Unknown",
        };
        decoded.insert(
            "LongExposureNoiseReduction".to_string(),
            ExifValue::Ascii(lenr_str.to_string()),
        );
    }

    // Index 5: HighISONoiseReduction
    if data.len() > 5 {
        let value = data[5] as i32;
        let hisonr_str = match value {
            0 => "Standard",
            1 => "Low",
            2 => "Strong",
            3 => "Off",
            _ => "Unknown",
        };
        decoded.insert(
            "HighISONoiseReduction".to_string(),
            ExifValue::Ascii(hisonr_str.to_string()),
        );
    }

    // Index 10: DigitalLensOptimizer
    if data.len() > 10 {
        let value = data[10] as i32;
        let dlo_str = match value {
            0 => "Off",
            1 => "Standard",
            2 => "High",
            _ => "Unknown",
        };
        decoded.insert(
            "DigitalLensOptimizer".to_string(),
            ExifValue::Ascii(dlo_str.to_string()),
        );
    }

    // Index 11: DualPixelRaw
    if data.len() > 11 {
        let value = data[11] as i32;
        let str_val = match value {
            0 => "Off",
            1 => "On",
            _ => "Unknown",
        };
        decoded.insert(
            "DualPixelRaw".to_string(),
            ExifValue::Ascii(str_val.to_string()),
        );
    }

    decoded
}

/// Decode Canon CameraInfo for 50D - extract PictureStyleInfo
/// PictureStyleInfo starts at offset 0x2d7 (727) within CameraInfo
/// Each value is int32s (4 bytes, signed)
pub fn decode_camera_info_50d(data: &[u8]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // PictureStyleInfo starts at offset 0x2d7 (727)
    const PS_INFO_OFFSET: usize = 0x2d7;

    // Check if we have enough data
    if data.len() < PS_INFO_OFFSET + 0xdc {
        return decoded;
    }

    // Helper to read int32s at offset (little-endian)
    let read_i32 = |offset: usize| -> i32 {
        let base = PS_INFO_OFFSET + offset;
        if base + 4 <= data.len() {
            i32::from_le_bytes([data[base], data[base + 1], data[base + 2], data[base + 3]])
        } else {
            0
        }
    };

    // PSInfo structure offsets (from ExifTool Canon.pm)
    let fields: &[(usize, &str)] = &[
        (0x00, "ContrastStandard"),
        (0x04, "SharpnessStandard"),
        (0x08, "SaturationStandard"),
        (0x0c, "ColorToneStandard"),
        (0x18, "ContrastPortrait"),
        (0x1c, "SharpnessPortrait"),
        (0x20, "SaturationPortrait"),
        (0x24, "ColorTonePortrait"),
        (0x30, "ContrastLandscape"),
        (0x34, "SharpnessLandscape"),
        (0x38, "SaturationLandscape"),
        (0x3c, "ColorToneLandscape"),
        (0x48, "ContrastNeutral"),
        (0x4c, "SharpnessNeutral"),
        (0x50, "SaturationNeutral"),
        (0x54, "ColorToneNeutral"),
        (0x60, "ContrastFaithful"),
        (0x64, "SharpnessFaithful"),
        (0x68, "SaturationFaithful"),
        (0x6c, "ColorToneFaithful"),
        (0x78, "ContrastMonochrome"),
        (0x7c, "SharpnessMonochrome"),
        (0x90, "ContrastUserDef1"),
        (0x94, "SharpnessUserDef1"),
        (0x98, "SaturationUserDef1"),
        (0x9c, "ColorToneUserDef1"),
        (0xa8, "ContrastUserDef2"),
        (0xac, "SharpnessUserDef2"),
        (0xb0, "SaturationUserDef2"),
        (0xb4, "ColorToneUserDef2"),
        (0xc0, "ContrastUserDef3"),
        (0xc4, "SharpnessUserDef3"),
        (0xc8, "SaturationUserDef3"),
        (0xcc, "ColorToneUserDef3"),
    ];

    for &(offset, name) in fields {
        let value = read_i32(offset);
        // Skip "n/a" marker values (0xdeadbeef = -559038737)
        if value != -559038737 {
            decoded.insert(name.to_string(), ExifValue::SShort(vec![value as i16]));
        }
    }

    // FilterEffect and ToningEffect for Monochrome
    let filter_effect = read_i32(0x88);
    if filter_effect != -559038737 {
        let filter_str = match filter_effect {
            0 => "None",
            1 => "Yellow",
            2 => "Orange",
            3 => "Red",
            4 => "Green",
            _ => "Unknown",
        };
        decoded.insert(
            "FilterEffectMonochrome".to_string(),
            ExifValue::Ascii(filter_str.to_string()),
        );
    }

    let toning_effect = read_i32(0x8c);
    if toning_effect != -559038737 {
        let toning_str = match toning_effect {
            0 => "None",
            1 => "Sepia",
            2 => "Blue",
            3 => "Purple",
            4 => "Green",
            _ => "Unknown",
        };
        decoded.insert(
            "ToningEffectMonochrome".to_string(),
            ExifValue::Ascii(toning_str.to_string()),
        );
    }

    // UserDef1 FilterEffect and ToningEffect
    let filter_ud1 = read_i32(0xa0);
    if filter_ud1 != -559038737 {
        let filter_str = match filter_ud1 {
            0 => "None",
            1 => "Yellow",
            2 => "Orange",
            3 => "Red",
            4 => "Green",
            _ => "Unknown",
        };
        decoded.insert(
            "FilterEffectUserDef1".to_string(),
            ExifValue::Ascii(filter_str.to_string()),
        );
    }

    let toning_ud1 = read_i32(0xa4);
    if toning_ud1 != -559038737 {
        let toning_str = match toning_ud1 {
            0 => "None",
            1 => "Sepia",
            2 => "Blue",
            3 => "Purple",
            4 => "Green",
            _ => "Unknown",
        };
        decoded.insert(
            "ToningEffectUserDef1".to_string(),
            ExifValue::Ascii(toning_str.to_string()),
        );
    }

    // UserDef2 FilterEffect and ToningEffect
    let filter_ud2 = read_i32(0xb8);
    if filter_ud2 != -559038737 {
        let filter_str = match filter_ud2 {
            0 => "None",
            1 => "Yellow",
            2 => "Orange",
            3 => "Red",
            4 => "Green",
            _ => "Unknown",
        };
        decoded.insert(
            "FilterEffectUserDef2".to_string(),
            ExifValue::Ascii(filter_str.to_string()),
        );
    }

    let toning_ud2 = read_i32(0xbc);
    if toning_ud2 != -559038737 {
        let toning_str = match toning_ud2 {
            0 => "None",
            1 => "Sepia",
            2 => "Blue",
            3 => "Purple",
            4 => "Green",
            _ => "Unknown",
        };
        decoded.insert(
            "ToningEffectUserDef2".to_string(),
            ExifValue::Ascii(toning_str.to_string()),
        );
    }

    // UserDef3 FilterEffect and ToningEffect
    let filter_ud3 = read_i32(0xd0);
    if filter_ud3 != -559038737 {
        let filter_str = match filter_ud3 {
            0 => "None",
            1 => "Yellow",
            2 => "Orange",
            3 => "Red",
            4 => "Green",
            _ => "Unknown",
        };
        decoded.insert(
            "FilterEffectUserDef3".to_string(),
            ExifValue::Ascii(filter_str.to_string()),
        );
    }

    let toning_ud3 = read_i32(0xd4);
    if toning_ud3 != -559038737 {
        let toning_str = match toning_ud3 {
            0 => "None",
            1 => "Sepia",
            2 => "Blue",
            3 => "Purple",
            4 => "Green",
            _ => "Unknown",
        };
        decoded.insert(
            "ToningEffectUserDef3".to_string(),
            ExifValue::Ascii(toning_str.to_string()),
        );
    }

    decoded
}

/// Decode Canon ColorInfo array sub-fields (tag 0x4003)
/// Contains per-style ColorTone, Contrast, Saturation, Sharpness
pub fn decode_color_info(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    if data.len() < 40 {
        return decoded;
    }

    // ColorInfo structure has style settings at specific offsets
    // Based on ExifTool Canon.pm ColorInfo structure

    // Per-style settings - each style has 4 parameters at fixed offsets
    // The structure varies by firmware, but typically:
    // ColorTone, Contrast, Saturation, Sharpness for each style

    let styles = [
        "Standard",
        "Portrait",
        "Landscape",
        "Neutral",
        "Faithful",
        "Monochrome",
        "UserDef1",
        "UserDef2",
        "UserDef3",
    ];

    // For 50D, the offsets are approximately:
    // Standard: 4-7, Portrait: 8-11, Landscape: 12-15, etc.
    for (i, style) in styles.iter().enumerate() {
        let base = 4 + i * 4;
        if data.len() > base + 3 {
            // Sharpness
            decoded.insert(
                format!("Sharpness{}", style),
                ExifValue::SShort(vec![data[base] as i16]),
            );
            // Contrast
            decoded.insert(
                format!("Contrast{}", style),
                ExifValue::SShort(vec![data[base + 1] as i16]),
            );
            // Saturation
            decoded.insert(
                format!("Saturation{}", style),
                ExifValue::SShort(vec![data[base + 2] as i16]),
            );
            // ColorTone
            decoded.insert(
                format!("ColorTone{}", style),
                ExifValue::SShort(vec![data[base + 3] as i16]),
            );
        }
    }

    // FilterEffect and ToningEffect for Monochrome and UserDef styles
    // These are at higher offsets
    let mono_styles = ["Monochrome", "UserDef1", "UserDef2", "UserDef3"];
    for (i, style) in mono_styles.iter().enumerate() {
        let base = 40 + i * 2;
        if data.len() > base + 1 {
            let filter_effect = match data[base] {
                0 => "None",
                1 => "Yellow",
                2 => "Orange",
                3 => "Red",
                4 => "Green",
                _ => "Unknown",
            };
            decoded.insert(
                format!("FilterEffect{}", style),
                ExifValue::Ascii(filter_effect.to_string()),
            );

            let toning_effect = match data[base + 1] {
                0 => "None",
                1 => "Sepia",
                2 => "Blue",
                3 => "Purple",
                4 => "Green",
                _ => "Unknown",
            };
            decoded.insert(
                format!("ToningEffect{}", style),
                ExifValue::Ascii(toning_effect.to_string()),
            );
        }
    }

    decoded
}

/// Decode Canon CropInfo array sub-fields
///
/// This function decodes the CropInfo array which contains crop margin information.
/// Uses ExifTool format by default.
pub fn decode_crop_info(data: &[u16]) -> HashMap<String, ExifValue> {
    decode_crop_info_exiftool(data)
}

/// Decode Canon CropInfo array sub-fields - ExifTool format
pub fn decode_crop_info_exiftool(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // CropInfo structure (from ExifTool Canon.pm):
    // Format: int16u
    // Index 0: CropLeftMargin
    // Index 1: CropRightMargin
    // Index 2: CropTopMargin
    // Index 3: CropBottomMargin

    if !data.is_empty() {
        decoded.insert(
            "CropLeftMargin".to_string(),
            ExifValue::Short(vec![data[0]]),
        );
    }

    if data.len() > 1 {
        decoded.insert(
            "CropRightMargin".to_string(),
            ExifValue::Short(vec![data[1]]),
        );
    }

    if data.len() > 2 {
        decoded.insert("CropTopMargin".to_string(), ExifValue::Short(vec![data[2]]));
    }

    if data.len() > 3 {
        decoded.insert(
            "CropBottomMargin".to_string(),
            ExifValue::Short(vec![data[3]]),
        );
    }

    decoded
}

/// Decode Canon AspectRatio value - ExifTool format
pub fn decode_aspect_ratio_exiftool(value: u32) -> &'static str {
    match value {
        0 => "3:2",
        1 => "1:1",
        2 => "4:3",
        7 => "16:9",
        8 => "4:5",
        12 => "3:2 (APS-H crop)",
        13 => "3:2 (APS-C crop)",
        258 => "4:3 crop",
        _ => "Unknown",
    }
}

/// Decode Canon AspectRatio value - exiv2 format
pub fn decode_aspect_ratio_exiv2(value: u32) -> &'static str {
    match value {
        0 => "3:2",
        1 => "1:1",
        2 => "4:3",
        7 => "16:9",
        8 => "4:5",
        12 => "3:2 (APS-H crop)",
        13 => "3:2 (APS-C crop)",
        _ => "Unknown",
    }
}

/// Decode Canon AspectInfo array sub-fields
///
/// This function decodes the AspectInfo array which contains aspect ratio and cropped image information.
/// Uses ExifTool format by default.
pub fn decode_aspect_info(data: &[u32]) -> HashMap<String, ExifValue> {
    decode_aspect_info_exiftool(data)
}

/// Decode Canon AspectInfo array sub-fields - ExifTool format
pub fn decode_aspect_info_exiftool(data: &[u32]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // AspectInfo structure (from ExifTool Canon.pm):
    // Format: int32u
    // Index 0: AspectRatio
    // Index 1: CroppedImageWidth
    // Index 2: CroppedImageHeight
    // Index 3: CroppedImageLeft
    // Index 4: CroppedImageTop

    if !data.is_empty() {
        decoded.insert(
            "AspectRatio".to_string(),
            ExifValue::Ascii(decode_aspect_ratio_exiftool(data[0]).to_string()),
        );
    }

    if data.len() > 1 {
        decoded.insert(
            "CroppedImageWidth".to_string(),
            ExifValue::Long(vec![data[1]]),
        );
    }

    if data.len() > 2 {
        decoded.insert(
            "CroppedImageHeight".to_string(),
            ExifValue::Long(vec![data[2]]),
        );
    }

    if data.len() > 3 {
        decoded.insert(
            "CroppedImageLeft".to_string(),
            ExifValue::Long(vec![data[3]]),
        );
    }

    if data.len() > 4 {
        decoded.insert(
            "CroppedImageTop".to_string(),
            ExifValue::Long(vec![data[4]]),
        );
    }

    decoded
}

/// Decode Canon SensorInfo array sub-fields
///
/// This function decodes the SensorInfo array which contains sensor dimensions and borders.
/// Uses ExifTool format by default.
pub fn decode_sensor_info(data: &[u16]) -> HashMap<String, ExifValue> {
    decode_sensor_info_exiftool(data)
}

/// Decode Canon SensorInfo array sub-fields - ExifTool format
pub fn decode_sensor_info_exiftool(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // SensorInfo structure (from ExifTool Canon.pm):
    // Format: int16s (signed)
    // FIRST_ENTRY => 1 (1-based indexing)
    // Index 1: SensorWidth
    // Index 2: SensorHeight
    // Index 5: SensorLeftBorder
    // Index 6: SensorTopBorder
    // Index 7: SensorRightBorder
    // Index 8: SensorBottomBorder
    // Index 9: BlackMaskLeftBorder
    // Index 10: BlackMaskTopBorder
    // Index 11: BlackMaskRightBorder
    // Index 12: BlackMaskBottomBorder

    // SensorWidth (index 1)
    if data.len() > 1 {
        decoded.insert(
            "SensorWidth".to_string(),
            ExifValue::SShort(vec![data[1] as i16]),
        );
    }

    // SensorHeight (index 2)
    if data.len() > 2 {
        decoded.insert(
            "SensorHeight".to_string(),
            ExifValue::SShort(vec![data[2] as i16]),
        );
    }

    // SensorLeftBorder (index 5)
    if data.len() > 5 {
        decoded.insert(
            "SensorLeftBorder".to_string(),
            ExifValue::SShort(vec![data[5] as i16]),
        );
    }

    // SensorTopBorder (index 6)
    if data.len() > 6 {
        decoded.insert(
            "SensorTopBorder".to_string(),
            ExifValue::SShort(vec![data[6] as i16]),
        );
    }

    // SensorRightBorder (index 7)
    if data.len() > 7 {
        decoded.insert(
            "SensorRightBorder".to_string(),
            ExifValue::SShort(vec![data[7] as i16]),
        );
    }

    // SensorBottomBorder (index 8)
    if data.len() > 8 {
        decoded.insert(
            "SensorBottomBorder".to_string(),
            ExifValue::SShort(vec![data[8] as i16]),
        );
    }

    // BlackMaskLeftBorder (index 9)
    if data.len() > 9 {
        decoded.insert(
            "BlackMaskLeftBorder".to_string(),
            ExifValue::SShort(vec![data[9] as i16]),
        );
    }

    // BlackMaskTopBorder (index 10)
    if data.len() > 10 {
        decoded.insert(
            "BlackMaskTopBorder".to_string(),
            ExifValue::SShort(vec![data[10] as i16]),
        );
    }

    // BlackMaskRightBorder (index 11)
    if data.len() > 11 {
        decoded.insert(
            "BlackMaskRightBorder".to_string(),
            ExifValue::SShort(vec![data[11] as i16]),
        );
    }

    // BlackMaskBottomBorder (index 12)
    if data.len() > 12 {
        decoded.insert(
            "BlackMaskBottomBorder".to_string(),
            ExifValue::SShort(vec![data[12] as i16]),
        );
    }

    decoded
}

/// Parse Canon maker notes
///
/// Canon maker notes use TIFF-relative offsets, so we need access to the full
/// TIFF/EXIF data to properly resolve string and array values.
pub fn parse_canon_maker_notes(
    data: &[u8],
    endian: Endianness,
    tiff_data: Option<&[u8]>,
    tiff_offset: usize,
    model: Option<&str>,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    if data.len() < 2 {
        return Ok(tags);
    }

    // Canon maker notes use standard TIFF IFD format starting immediately
    // Read number of entries
    let num_entries = read_u16(&data[0..2], endian) as usize;

    // Sanity check
    if num_entries > 500 || 2 + num_entries * 12 > data.len() {
        return Ok(tags);
    }

    // Parse each IFD entry (12 bytes each)
    // We use a counter for synthetic tag IDs for decoded sub-fields
    let mut synthetic_tag_id = 0xF000u16;

    for i in 0..num_entries {
        let entry_offset = 2 + i * 12;

        if let Some((tag_id, value)) =
            parse_ifd_entry(data, entry_offset, endian, tiff_data, tiff_offset)
        {
            // Skip large binary blobs to save memory (dust removal data)
            if tag_id == CANON_DUST_REMOVAL_DATA {
                continue;
            }

            // CameraInfo parsing is model-specific with different offsets per camera
            // Enable for known models with implemented decoders
            if tag_id == CANON_CAMERA_INFO {
                if let Some(model_str) = model {
                    let bytes_data: Option<Vec<u8>> = match &value {
                        ExifValue::Undefined(bytes) => Some(bytes.clone()),
                        ExifValue::Byte(bytes) => Some(bytes.clone()),
                        _ => None,
                    };

                    if let Some(bytes) = bytes_data {
                        let decoded = if model_str.contains("EOS 50D") {
                            decode_camera_info_50d(&bytes)
                        } else {
                            HashMap::new()
                        };

                        if !decoded.is_empty() {
                            for (field_name, field_value) in decoded {
                                tags.insert(
                                    synthetic_tag_id,
                                    MakerNoteTag {
                                        tag_id: synthetic_tag_id,
                                        tag_name: Some(Box::leak(field_name.into_boxed_str())),
                                        value: field_value,
                                    },
                                );
                                synthetic_tag_id += 1;
                            }
                        }
                    }
                }
                // Skip the raw CameraInfo blob
                continue;
            }

            // Check if this is a tag we can decode into sub-fields
            let should_decode = matches!(
                tag_id,
                CANON_CAMERA_SETTINGS
                    | CANON_SHOT_INFO
                    | CANON_FOCAL_LENGTH
                    | CANON_FILE_INFO
                    | CANON_AF_INFO_2
                    | CANON_PROCESSING_INFO
                    | CANON_COLOR_DATA
                    | CANON_AF_MICRO_ADJ
                    | CANON_COLOR_INFO
                    | CANON_CROP_INFO
                    | CANON_ASPECT_INFO
                    | CANON_SENSOR_INFO
                    | CANON_VIGNETTING_CORR
                    | CANON_VIGNETTING_CORR_2
                    | CANON_LIGHTING_OPT
            );

            if should_decode {
                // Handle tags that use u32 values (AspectInfo, VignettingCorr2, LightingOpt)
                if matches!(
                    tag_id,
                    CANON_ASPECT_INFO | CANON_VIGNETTING_CORR_2 | CANON_LIGHTING_OPT
                ) {
                    let longs_data: Option<Vec<u32>> = match &value {
                        ExifValue::Long(longs) => Some(longs.clone()),
                        ExifValue::SLong(slongs) => {
                            Some(slongs.iter().map(|&v| v as u32).collect())
                        }
                        _ => None,
                    };

                    if let Some(longs) = longs_data {
                        let decoded = match tag_id {
                            CANON_ASPECT_INFO => decode_aspect_info(&longs),
                            CANON_VIGNETTING_CORR_2 => decode_vignetting_corr2(&longs),
                            CANON_LIGHTING_OPT => decode_lighting_opt(&longs),
                            _ => HashMap::new(),
                        };

                        // Insert each decoded sub-field as a separate tag
                        for (field_name, field_value) in decoded {
                            tags.insert(
                                synthetic_tag_id,
                                MakerNoteTag {
                                    tag_id: synthetic_tag_id,
                                    tag_name: Some(Box::leak(field_name.into_boxed_str())),
                                    value: field_value,
                                },
                            );
                            synthetic_tag_id = synthetic_tag_id.wrapping_add(1);
                        }
                        // Skip inserting the raw array
                        continue;
                    }
                }

                // Extract shorts from Short, SShort, or Long values
                // Some Canon tags use LONG type but contain small values that should be decoded as shorts
                let shorts_data: Option<Vec<u16>> = match &value {
                    ExifValue::Short(shorts) => Some(shorts.clone()),
                    ExifValue::SShort(sshorts) => Some(sshorts.iter().map(|&v| v as u16).collect()),
                    ExifValue::Long(longs) => {
                        // Convert longs to u16 (clamping to 16-bit range)
                        Some(longs.iter().map(|&v| v as u16).collect())
                    }
                    ExifValue::SLong(slongs) => {
                        // Convert signed longs to u16
                        Some(slongs.iter().map(|&v| v as u16).collect())
                    }
                    _ => None,
                };

                if let Some(shorts) = shorts_data {
                    let decoded = match tag_id {
                        CANON_CAMERA_SETTINGS => decode_camera_settings(&shorts),
                        CANON_SHOT_INFO => decode_shot_info(&shorts),
                        CANON_FOCAL_LENGTH => decode_focal_length(&shorts),
                        CANON_FILE_INFO => decode_file_info(&shorts),
                        CANON_AF_INFO_2 => decode_af_info2(&shorts),
                        CANON_PROCESSING_INFO => decode_processing_info(&shorts),
                        CANON_COLOR_DATA => decode_color_data(&shorts),
                        CANON_AF_MICRO_ADJ => decode_af_micro_adj(&shorts),
                        CANON_COLOR_INFO => decode_color_info(&shorts),
                        CANON_CROP_INFO => decode_crop_info(&shorts),
                        CANON_SENSOR_INFO => decode_sensor_info(&shorts),
                        CANON_VIGNETTING_CORR => decode_vignetting_corr(&shorts),
                        _ => HashMap::new(),
                    };

                    // Insert each decoded sub-field as a separate tag
                    for (field_name, field_value) in decoded {
                        tags.insert(
                            synthetic_tag_id,
                            MakerNoteTag {
                                tag_id: synthetic_tag_id,
                                tag_name: Some(Box::leak(field_name.into_boxed_str())),
                                value: field_value,
                            },
                        );
                        synthetic_tag_id = synthetic_tag_id.wrapping_add(1);
                    }
                    // Skip inserting the raw array
                    continue;
                }
            }

            // Special handling for specific tags
            let final_value = match tag_id {
                // CanonModelID - convert to model name string
                CANON_MODEL_ID => {
                    if let ExifValue::Long(ref longs) = value {
                        if !longs.is_empty() {
                            if let Some(name) = get_canon_model_name(longs[0]) {
                                ExifValue::Ascii(name.to_string())
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
                // FirmwareRevision - decode version bytes: 0xAABBCCDD -> "A.BB rev C.DD"
                CANON_FIRMWARE_REVISION => {
                    if let ExifValue::Long(ref longs) = value {
                        if !longs.is_empty() {
                            let v = longs[0];
                            let major = (v >> 24) & 0xFF;
                            let minor = (v >> 16) & 0xFF;
                            let rev = (v >> 8) & 0xFF;
                            let sub_rev = v & 0xFF;
                            ExifValue::Ascii(format!(
                                "{}.{:02} rev {}.{:02}",
                                major, minor, rev, sub_rev
                            ))
                        } else {
                            value
                        }
                    } else {
                        value
                    }
                }
                // ImageUniqueID - convert byte array to hex string
                CANON_IMAGE_UNIQUE_ID => {
                    if let ExifValue::Undefined(ref bytes) = value {
                        if bytes.len() == 16 {
                            let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
                            ExifValue::Ascii(hex)
                        } else {
                            value
                        }
                    } else if let ExifValue::Byte(ref bytes) = value {
                        if bytes.len() == 16 {
                            let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
                            ExifValue::Ascii(hex)
                        } else {
                            value
                        }
                    } else {
                        value
                    }
                }
                // FileNumber - format as "XXX-YYYY" from numeric value
                CANON_FILE_NUMBER => {
                    if let ExifValue::Long(ref longs) = value {
                        if !longs.is_empty() {
                            let v = longs[0];
                            let dir = v / 10000;
                            let file = v % 10000;
                            ExifValue::Ascii(format!("{}-{:04}", dir, file))
                        } else {
                            value
                        }
                    } else {
                        value
                    }
                }
                // SerialNumber - format depends on camera model
                // EOS-1D series: 6 digits, all other models: 10 digits
                CANON_SERIAL_NUMBER => {
                    if let ExifValue::Long(ref longs) = value {
                        if !longs.is_empty() {
                            // Check if this is an EOS-1D series camera
                            let is_1d_series = model.map(|m| m.contains("EOS-1D")).unwrap_or(false);

                            if is_1d_series {
                                ExifValue::Ascii(format!("{:06}", longs[0]))
                            } else {
                                ExifValue::Ascii(format!("{:010}", longs[0]))
                            }
                        } else {
                            value
                        }
                    } else {
                        value
                    }
                }
                // SerialNumberFormat - decode to "Format 1" or "Format 2"
                CANON_SERIAL_NUMBER_FORMAT => {
                    if let ExifValue::Long(ref longs) = value {
                        if !longs.is_empty() {
                            let format_str = match longs[0] {
                                0x90000000 => "Format 1".to_string(),
                                0xa0000000 => "Format 2".to_string(),
                                _ => format!("Unknown (0x{:08x})", longs[0]),
                            };
                            ExifValue::Ascii(format_str)
                        } else {
                            value
                        }
                    } else {
                        value
                    }
                }
                // DateStampMode - decode using decode function
                CANON_DATE_STAMP_MODE => {
                    if let ExifValue::Short(ref shorts) = value {
                        if !shorts.is_empty() {
                            ExifValue::Ascii(decode_date_stamp_mode_exiftool(shorts[0]).to_string())
                        } else {
                            value
                        }
                    } else {
                        value
                    }
                }
                // SuperMacro - decode using decode function
                CANON_SUPER_MACRO => {
                    if let ExifValue::Short(ref shorts) = value {
                        if !shorts.is_empty() {
                            ExifValue::Ascii(decode_super_macro_exiftool(shorts[0]).to_string())
                        } else {
                            value
                        }
                    } else {
                        value
                    }
                }
                // InternalSerialNumber - strip trailing null bytes and non-printable chars
                CANON_SERIAL_INFO => {
                    if let ExifValue::Ascii(ref s) = value {
                        // Strip after first null byte and non-printable chars
                        let cleaned: String = s
                            .chars()
                            .take_while(|&c| c != '\0' && c.is_ascii_graphic())
                            .collect();
                        if !cleaned.is_empty() {
                            ExifValue::Ascii(cleaned)
                        } else {
                            value
                        }
                    } else {
                        value
                    }
                }
                // Categories - decode bit flags to "(none)" or category names
                CANON_CATEGORIES => {
                    if let ExifValue::Long(ref longs) = value {
                        // Format is 2 values: first is always 8, second is the category bitmask
                        if longs.len() >= 2 && longs[0] == 8 {
                            let categories = longs[1];
                            if categories == 0 {
                                ExifValue::Ascii("(none)".to_string())
                            } else {
                                let mut cat_names = Vec::new();
                                if categories & 0x01 != 0 {
                                    cat_names.push("People");
                                }
                                if categories & 0x02 != 0 {
                                    cat_names.push("Scenery");
                                }
                                if categories & 0x04 != 0 {
                                    cat_names.push("Events");
                                }
                                if categories & 0x08 != 0 {
                                    cat_names.push("User 1");
                                }
                                if categories & 0x10 != 0 {
                                    cat_names.push("User 2");
                                }
                                if categories & 0x20 != 0 {
                                    cat_names.push("User 3");
                                }
                                if categories & 0x40 != 0 {
                                    cat_names.push("To Do");
                                }
                                ExifValue::Ascii(cat_names.join(", "))
                            }
                        } else {
                            value
                        }
                    } else {
                        value
                    }
                }
                // PictureStyleUserDef - decode array of 3 shorts to style names
                CANON_PICTURE_STYLE_USER_DEF => {
                    if let ExifValue::Short(ref shorts) = value {
                        if shorts.len() >= 3 {
                            let styles: Vec<String> = shorts
                                .iter()
                                .take(3)
                                .map(|&s| get_picture_style_name(s).to_string())
                                .collect();
                            ExifValue::Ascii(styles.join("; "))
                        } else {
                            value
                        }
                    } else {
                        value
                    }
                }
                // PictureStylePC - decode array of 3 shorts to style names
                CANON_PICTURE_STYLE_PC => {
                    if let ExifValue::Short(ref shorts) = value {
                        if shorts.len() >= 3 {
                            let styles: Vec<String> = shorts
                                .iter()
                                .take(3)
                                .map(|&s| get_picture_style_name(s).to_string())
                                .collect();
                            ExifValue::Ascii(styles.join("; "))
                        } else {
                            value
                        }
                    } else {
                        value
                    }
                }
                // AFPointsInFocus1D - decode 8-byte bitmask to AF point names
                CANON_AF_POINTS_IN_FOCUS_1D => {
                    let bytes_data: Option<&[u8]> = match &value {
                        ExifValue::Undefined(bytes) => Some(bytes.as_slice()),
                        ExifValue::Byte(bytes) => Some(bytes.as_slice()),
                        _ => None,
                    };
                    if let Some(bytes) = bytes_data {
                        ExifValue::Ascii(decode_af_points_in_focus_1d(bytes))
                    } else {
                        value
                    }
                }
                _ => value,
            };

            tags.insert(
                tag_id,
                MakerNoteTag {
                    tag_id,
                    tag_name: get_canon_tag_name(tag_id),
                    value: final_value,
                },
            );
        }
    }

    Ok(tags)
}
