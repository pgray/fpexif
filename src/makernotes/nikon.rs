// makernotes/nikon.rs - Nikon maker notes parsing

use crate::data_types::{Endianness, ExifValue};
use crate::define_tag_decoder;
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::Cursor;

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
pub const NIKON_FLASH_EXPOSURE_BRACKET_VALUE: u16 = 0x0017;
pub const NIKON_EXPOSURE_BRACKET_VALUE: u16 = 0x0018;
pub const NIKON_IMAGE_PROCESSING: u16 = 0x0019;
pub const NIKON_CROP_HI_SPEED: u16 = 0x001A;
pub const NIKON_EXPOSURE_TUNING: u16 = 0x001B;
pub const NIKON_SERIAL_NUMBER: u16 = 0x001D;
pub const NIKON_COLOR_SPACE: u16 = 0x001E;
pub const NIKON_VR_INFO: u16 = 0x001F;
pub const NIKON_IMAGE_AUTHENTICATION: u16 = 0x0020;
pub const NIKON_FACE_DETECT: u16 = 0x0021;
pub const NIKON_ACTIVE_D_LIGHTING: u16 = 0x0022;
pub const NIKON_HIGH_ISO_NOISE_REDUCTION: u16 = 0x00B1;
pub const NIKON_PICTURE_CONTROL_DATA: u16 = 0x0023;
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
pub const NIKON_HDR_INFO: u16 = 0x00B0;
pub const NIKON_MULTI_EXPOSURE: u16 = 0x00B1;
pub const NIKON_LOCATION_INFO: u16 = 0x00B5;
pub const NIKON_BLACK_LEVEL: u16 = 0x00B6;
pub const NIKON_AF_INFO_2: u16 = 0x00B7;
pub const NIKON_FILE_INFO: u16 = 0x00B8;
pub const NIKON_AF_TUNE: u16 = 0x00B9;
pub const NIKON_RETOUCH_INFO: u16 = 0x00BB;
pub const NIKON_PICTURE_CONTROL_VERSION: u16 = 0x00BC;
pub const NIKON_SILENT_PHOTO: u16 = 0x00BD;
pub const NIKON_SHUTTER_MODE: u16 = 0x0034;
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
pub const NIKON_NEF_BIT_DEPTH: u16 = 0x009F;
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

/// Get the name of a Nikon MakerNote tag
pub fn get_nikon_tag_name(tag_id: u16) -> Option<&'static str> {
    match tag_id {
        NIKON_VERSION => Some("NikonMakerNoteVersion"),
        NIKON_ISO_SETTING => Some("ISO"),
        NIKON_COLOR_MODE => Some("ColorMode"),
        NIKON_QUALITY => Some("Quality"),
        NIKON_WHITE_BALANCE => Some("WhiteBalance"),
        NIKON_SHARPNESS => Some("Sharpness"),
        NIKON_FOCUS_MODE => Some("FocusMode"),
        NIKON_FLASH_SETTING => Some("FlashSetting"),
        NIKON_FLASH_TYPE => Some("FlashType"),
        NIKON_SERIAL_NUMBER => Some("SerialNumber"),
        NIKON_COLOR_SPACE => Some("ColorSpace"),
        NIKON_VR_INFO => Some("VRInfo"),
        NIKON_ACTIVE_D_LIGHTING => Some("ActiveDLighting"),
        NIKON_PICTURE_CONTROL_DATA => Some("PictureControlData"),
        NIKON_VIGNETTE_CONTROL => Some("VignetteControl"),
        NIKON_DISTORTION_CONTROL => Some("DistortionControl"),
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
        NIKON_SATURATION => Some("Saturation"),
        NIKON_NOISE_REDUCTION => Some("NoiseReduction"),
        NIKON_COLOR_BALANCE => Some("ColorBalance"),
        NIKON_LENS_DATA => Some("LensData"),
        NIKON_RAW_IMAGE_CENTER => Some("RawImageCenter"),
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
        NIKON_WHITE_BALANCE_FINE => Some("WhiteBalanceFine"),
        NIKON_PROGRAM_SHIFT => Some("ProgramShift"),
        NIKON_EXPOSURE_DIFFERENCE => Some("ExposureDifference"),
        NIKON_ISO_SELECTION => Some("ISOSelection"),
        NIKON_FLASH_EXPOSURE_COMP => Some("FlashExposureComp"),
        NIKON_ISO_SETTING_2 => Some("ISO2"),
        NIKON_IMAGE_BOUNDARY => Some("ImageBoundary"),
        NIKON_FLASH_EXPOSURE_BRACKET_VALUE => Some("FlashExposureBracketValue"),
        NIKON_EXPOSURE_BRACKET_VALUE => Some("ExposureBracketValue"),
        NIKON_CROP_HI_SPEED => Some("CropHiSpeed"),
        NIKON_EXPOSURE_TUNING => Some("ExposureTuning"),
        NIKON_IMAGE_AUTHENTICATION => Some("ImageAuthentication"),
        NIKON_FACE_DETECT => Some("FaceDetect"),
        NIKON_WORLD_TIME => Some("WorldTime"),
        NIKON_ISO_INFO => Some("ISOInfo"),
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
        "A1 40 18 37 2C 34 BB 06" => Some("AF-S DX Nikkor 10-24mm f/3.5-4.5G ED"),
        "A2 48 5C 80 24 24 BC 0E" => Some("AF-S Nikkor 70-200mm f/2.8G ED VR II"),
        "A3 3C 29 44 30 30 BD 0E" => Some("AF-S Nikkor 16-35mm f/4G ED VR"),
        "A4 54 37 37 0C 0C BE 06" => Some("AF-S Nikkor 24mm f/1.4G ED"),
        "A5 40 3C 8E 2C 3C BF 0E" => Some("AF-S Nikkor 28-300mm f/3.5-5.6G ED VR"),
        "A6 48 8E 8E 24 24 C0 0E" => Some("AF-S Nikkor 300mm f/2.8G ED VR II"),
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
        NIKON_COLOR_MODE => decode_color_mode_exiftool(value),
        NIKON_FLASH_TYPE => decode_flash_type_exiftool(value),
        NIKON_NOISE_REDUCTION => decode_noise_reduction_exiftool(value),
        NIKON_IMAGE_PROCESSING => decode_image_processing_exiftool(value),
        NIKON_LIGHT_SOURCE => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                "Unknown".to_string()
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
define_tag_decoder! {
    flash_mode,
    type: u8,
    both: {
        0 => "Did Not Fire",
        1 => "Fired, Manual",
        3 => "Not Ready",
        7 => "Fired, External",
        8 => "Fired, Commander Mode",
        9 => "Fired, TTL Mode",
        18 => "LED Light",
    }
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
    let mut modes = Vec::new();

    if value & 0x0001 != 0 {
        modes.push("Continuous");
    }
    if value & 0x0002 != 0 {
        modes.push("Delay");
    }
    if value & 0x0004 != 0 {
        modes.push("PC Control");
    }
    if value & 0x0008 != 0 {
        modes.push("Self-timer");
    }
    if value & 0x0010 != 0 {
        modes.push("Exposure Bracketing");
    }
    if value & 0x0020 != 0 {
        // Note: For D70, bit 5 means "Unused LE-NR Slowdown"
        // For other models it means "Auto ISO"
        modes.push("Auto ISO");
    }
    if value & 0x0040 != 0 {
        modes.push("White-Balance Bracketing");
    }
    if value & 0x0080 != 0 {
        modes.push("IR Control");
    }
    if value & 0x0100 != 0 {
        modes.push("D-Lighting Bracketing");
    }

    if modes.is_empty() {
        "Unknown".to_string()
    } else {
        modes.join(", ")
    }
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
/// From Nikon.pm - values: "Normal", "Slow", "Rear Slow", "RED-EYE", "RED-EYE SLOW"
pub fn decode_flash_setting_exiftool(value: &str) -> String {
    let val = value.trim().to_uppercase();
    let result = match val.as_str() {
        "NORMAL" => "Normal",
        "SLOW" => "Slow",
        "REAR" | "REAR SLOW" => "Rear Slow",
        "RED-EYE" | "REDEYE" => "Red-eye",
        "RED-EYE SLOW" | "REDEYE SLOW" => "Red-eye Slow",
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
        "HIGH" | "HARD" => "Hard",
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
    let iso_raw = data[0];
    if iso_raw != 0 {
        let iso = (100.0 * ((iso_raw as f64 / 12.0 - 5.0) * 2.0_f64.ln()).exp()).round() as u32;
        tags.push(("ISO".to_string(), iso.to_string()));
    }

    // Offset 0x04: ISOExpansion (int16u, BigEndian)
    let iso_expansion = u16::from_be_bytes([data[4], data[5]]);
    if iso_expansion != 0 {
        tags.push((
            "ISOExpansion".to_string(),
            decode_iso_expansion_exiftool(iso_expansion).to_string(),
        ));
    }

    // Offset 0x06: ISO2 (int8u)
    let iso2_raw = data[6];
    if iso2_raw != 0 {
        let iso2 = (100.0 * ((iso2_raw as f64 / 12.0 - 5.0) * 2.0_f64.ln()).exp()).round() as u32;
        tags.push(("ISO2".to_string(), iso2.to_string()));
    }

    // Offset 0x0A: ISOExpansion2 (int16u, BigEndian)
    let iso_expansion2 = u16::from_be_bytes([data[10], data[11]]);
    if iso_expansion2 != 0 {
        tags.push((
            "ISOExpansion2".to_string(),
            decode_iso_expansion_exiftool(iso_expansion2).to_string(),
        ));
    }

    tags
}

/// Parse VRInfo tag data (tag 0x001F)
fn parse_vr_info(data: &[u8], _endian: Endianness) -> Vec<(String, String)> {
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
    if vr != 0 {
        tags.push(("VibrationReduction".to_string(), vr_str.to_string()));
    }

    // Offset 0x06: VRMode (int8u)
    let vr_mode = data[6];
    let vr_mode_str = match vr_mode {
        0 => "Normal",
        1 => "On (1)",
        2 => "Active",
        3 => "Sport",
        _ => "Unknown",
    };
    tags.push(("VRMode".to_string(), vr_mode_str.to_string()));

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
            // Format string: capitalize first letter of each word
            let formatted_name = name
                .to_lowercase()
                .split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            tags.push(("PictureControlName".to_string(), formatted_name));
        }

        // Offset 0x18: PictureControlBase (string[20])
        let base_bytes = &data[24..44];
        let base = String::from_utf8_lossy(base_bytes)
            .trim_end_matches('\0')
            .to_string();
        if !base.is_empty() {
            let formatted_base = base
                .to_lowercase()
                .split_whitespace()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
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
            if quick != 0xff {
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
            let value = bright.wrapping_sub(128) as i8;
            if value == 0 {
                tags.push(("Brightness".to_string(), "Normal".to_string()));
            } else {
                tags.push(("Brightness".to_string(), value.to_string()));
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
    }

    tags
}

/// Parse FlashInfo tag data (tag 0x00A8)
/// Version-dependent structure
fn parse_flash_info(data: &[u8]) -> Vec<(String, String)> {
    let mut tags = Vec::new();

    if data.len() < 4 {
        return tags;
    }

    // Offset 0x00: FlashInfoVersion (string[4])
    let version = String::from_utf8_lossy(&data[0..4]).to_string();
    tags.push(("FlashInfoVersion".to_string(), version.clone()));

    // FlashInfo0102 structure (D300 uses this)
    if version == "0102" && data.len() >= 25 {
        // Offset 0x04: FlashSource (int8u)
        let source = data[4];
        let source_str = match source {
            0 => "None",
            1 => "External",
            2 => "Internal",
            _ => "Unknown",
        };
        tags.push(("FlashSource".to_string(), source_str.to_string()));

        // Offset 0x06-0x07: ExternalFlashFirmware (int8u[2])
        if data[6] == 0 && data[7] == 0 {
            tags.push(("ExternalFlashFirmware".to_string(), "n/a".to_string()));
        } else {
            tags.push((
                "ExternalFlashFirmware".to_string(),
                format!("{}.{:02}", data[6], data[7]),
            ));
        }

        // Offset 0x08: ExternalFlashFlags (int8u)
        let flags = data[8];
        if flags == 0 {
            tags.push(("ExternalFlashFlags".to_string(), "(none)".to_string()));
        } else {
            let mut flag_strs = Vec::new();
            if flags & 0x01 != 0 {
                flag_strs.push("Fired");
            }
            if flags & 0x04 != 0 {
                flag_strs.push("Bounce");
            }
            if flags & 0x10 != 0 {
                flag_strs.push("Wide Adapter");
            }
            if flags & 0x20 != 0 {
                flag_strs.push("Dome Diffuser");
            }
            tags.push(("ExternalFlashFlags".to_string(), flag_strs.join(", ")));
        }

        // Offset 0x09.2: FlashCommanderMode (bits)
        let commander_mode = data[9] & 0x7F;
        let commander_str = match commander_mode {
            0 => "Off",
            1 => "TTL",
            2 => "Auto Aperture",
            3 => "Manual",
            4 => "Repeating Flash",
            _ => "Unknown",
        };
        tags.push(("FlashCommanderMode".to_string(), commander_str.to_string()));

        // Offset 0x0F.1: FlashControlMode
        if data.len() > 0x0F {
            let control_mode = data[0x0F] & 0x7F;
            let control_str = match control_mode {
                0 => "Off",
                1 => "iTTL-BL",
                2 => "iTTL",
                3 => "Auto Aperture",
                4 => "Automatic",
                5 => "GN (distance priority)",
                6 => "Manual",
                7 => "Repeating Flash",
                _ => "Unknown",
            };
            tags.push(("FlashControlMode".to_string(), control_str.to_string()));
        }

        // Offset 0x11: FlashCompensation (int8s)
        if data.len() > 0x11 {
            let comp = data[0x11] as i8;
            let ev = comp as f64 / 6.0;
            tags.push((
                "FlashCompensation".to_string(),
                format!("{:.1}", ev).trim_end_matches(".0").to_string(),
            ));
        }

        // Offset 0x12: FlashGNDistance (int8u)
        if data.len() > 0x12 {
            let gn = data[0x12];
            tags.push(("FlashGNDistance".to_string(), gn.to_string()));
        }

        // Offset 0x13.1: FlashGroupAControlMode
        if data.len() > 0x13 {
            let group_a = data[0x13] & 0x7F;
            let group_a_str = match group_a {
                0 => "Off",
                1 => "iTTL-BL",
                2 => "iTTL",
                3 => "Auto Aperture",
                4 => "Automatic",
                5 => "GN (distance priority)",
                6 => "Manual",
                7 => "Repeating Flash",
                _ => "Unknown",
            };
            tags.push((
                "FlashGroupAControlMode".to_string(),
                group_a_str.to_string(),
            ));
        }

        // Offset 0x14.1: FlashGroupBControlMode
        if data.len() > 0x14 {
            let group_b = data[0x14] & 0x7F;
            let group_b_str = match group_b {
                0 => "Off",
                1 => "iTTL-BL",
                2 => "iTTL",
                3 => "Auto Aperture",
                4 => "Automatic",
                5 => "GN (distance priority)",
                6 => "Manual",
                7 => "Repeating Flash",
                _ => "Unknown",
            };
            tags.push((
                "FlashGroupBControlMode".to_string(),
                group_b_str.to_string(),
            ));
        }

        // Offset 0x15.1: FlashGroupCControlMode
        if data.len() > 0x15 {
            let group_c = data[0x15] & 0x7F;
            let group_c_str = match group_c {
                0 => "Off",
                1 => "iTTL-BL",
                2 => "iTTL",
                3 => "Auto Aperture",
                4 => "Automatic",
                5 => "GN (distance priority)",
                6 => "Manual",
                7 => "Repeating Flash",
                _ => "Unknown",
            };
            tags.push((
                "FlashGroupCControlMode".to_string(),
                group_c_str.to_string(),
            ));
        }

        // Offset 0x16: FlashGroupACompensation (int8s)
        if data.len() > 0x16 {
            let comp = data[0x16] as i8;
            let ev = comp as f64 / 6.0;
            tags.push((
                "FlashGroupACompensation".to_string(),
                format!("{:.1}", ev).trim_end_matches(".0").to_string(),
            ));
        }

        // Offset 0x17: FlashGroupBCompensation (int8s)
        if data.len() > 0x17 {
            let comp = data[0x17] as i8;
            let ev = comp as f64 / 6.0;
            tags.push((
                "FlashGroupBCompensation".to_string(),
                format!("{:.1}", ev).trim_end_matches(".0").to_string(),
            ));
        }

        // Offset 0x18: FlashGroupCCompensation (int8s)
        if data.len() > 0x18 {
            let comp = data[0x18] as i8;
            let ev = comp as f64 / 6.0;
            tags.push((
                "FlashGroupCCompensation".to_string(),
                format!("{:.1}", ev).trim_end_matches(".0").to_string(),
            ));
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
        "FPNR" => "Long Exposure NR",
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

    // Format the lens description
    if (min_focal - max_focal).abs() < 0.1 {
        // Prime lens
        Some(format!("{:.0}mm f/{:.1}", min_focal, min_aperture))
    } else {
        // Zoom lens
        if (min_aperture - max_aperture).abs() < 0.1 {
            // Constant aperture
            Some(format!(
                "{:.0}-{:.0}mm f/{:.1}",
                min_focal, max_focal, min_aperture
            ))
        } else {
            // Variable aperture
            Some(format!(
                "{:.0}-{:.0}mm f/{:.1}-{:.1}",
                min_focal, max_focal, min_aperture, max_aperture
            ))
        }
    }
}

/// Parse Nikon maker notes
pub fn parse_nikon_maker_notes(
    data: &[u8],
    endian: Endianness,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    // Nikon maker notes often start with "Nikon\0" header
    if data.len() < 18 {
        return Ok(tags);
    }

    let base_offset;
    let ifd_offset;
    let maker_endian;

    // Check for Nikon Type 3 header: "Nikon\0" + version + TIFF header
    if data.starts_with(b"Nikon\0") {
        // Structure: "Nikon\0" (6 bytes) + version (4 bytes) + TIFF header
        base_offset = 10; // Start of TIFF header (after "Nikon\0" + version)

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
        // No Nikon header, use the data as-is
        base_offset = 0;
        maker_endian = endian;
        ifd_offset = 0;
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
                // Value at offset (relative to TIFF header for Nikon Type 3)
                let abs_offset = base_offset + value_offset as usize;
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
                    let bytes = value_bytes[..count as usize].to_vec();
                    // Apply decoder for specific tags
                    if tag_id == NIKON_LENS_TYPE && !bytes.is_empty() {
                        ExifValue::Ascii(decode_lens_type_exiftool(bytes[0]))
                    } else if tag_id == NIKON_FLASH_MODE && !bytes.is_empty() {
                        ExifValue::Ascii(decode_flash_mode_exiftool(bytes[0]).to_string())
                    } else if tag_id == NIKON_IMAGE_AUTHENTICATION && !bytes.is_empty() {
                        ExifValue::Ascii(decode_image_authentication_exiftool(bytes[0]).to_string())
                    } else if tag_id == NIKON_SHOOTING_MODE && !bytes.is_empty() {
                        // ShootingMode can be stored as Byte on older cameras
                        ExifValue::Ascii(decode_shooting_mode_exiftool(bytes[0] as u16))
                    } else {
                        ExifValue::Byte(bytes)
                    }
                }
                2 => {
                    // ASCII
                    let s = String::from_utf8_lossy(&value_bytes[..count as usize])
                        .trim_end_matches('\0')
                        .to_string();
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
                    if tag_id == NIKON_ISO_SETTING && values.len() >= 2 {
                        let iso = if values[1] > 0 { values[1] } else { values[0] };
                        ExifValue::Short(vec![iso])
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
                            _ => None,
                        };

                        if let Some(s) = decoded {
                            ExifValue::Ascii(s)
                        } else {
                            ExifValue::Short(values)
                        }
                    } else if tag_id == NIKON_CROP_HI_SPEED && !values.is_empty() {
                        // CropHiSpeed is a 7-element array, decode the first value
                        ExifValue::Ascii(decode_crop_hi_speed_exiftool(values[0]).to_string())
                    } else if tag_id == NIKON_EXPOSURE_TUNING && !values.is_empty() {
                        // ExposureTuning is a 7-element array, output only the first value
                        ExifValue::Short(vec![values[0]])
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
                        if v == 4294965247 {
                            ExifValue::Ascii("n/a".to_string())
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

                    // Apply decoder for lens info
                    if tag_id == NIKON_LENS {
                        if let Some(formatted) = format_lens_info(&values) {
                            ExifValue::Ascii(formatted)
                        } else {
                            ExifValue::Rational(values)
                        }
                    } else {
                        ExifValue::Rational(values)
                    }
                }
                7 => {
                    // UNDEFINED - handle packed rational values specially
                    match tag_id {
                        // Signed packed rationals (a * b/c where a,b,c are signed)
                        NIKON_PROGRAM_SHIFT
                        | NIKON_EXPOSURE_DIFFERENCE
                        | NIKON_FLASH_EXPOSURE_COMP
                        | NIKON_FLASH_EXPOSURE_BRACKET_VALUE
                        | NIKON_EXPOSURE_BRACKET_VALUE => {
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
                        _ => ExifValue::Undefined(value_bytes),
                    }
                }
                _ => {
                    // Unsupported type
                    continue;
                }
            };

            // Skip tags that are better represented by their parsed sub-structure values
            // These tags contain raw/binary data that's decoded from ISOInfo structure
            if matches!(tag_id, NIKON_ISO_SETTING | NIKON_ISO_SETTING_2) {
                continue;
            }

            tags.insert(
                tag_id,
                MakerNoteTag {
                    tag_id,
                    tag_name: get_nikon_tag_name(tag_id),
                    value: value.clone(),
                },
            );

            // Parse sub-structures and insert extracted fields as separate tags
            match tag_id {
                NIKON_ISO_INFO => {
                    if let ExifValue::Undefined(ref bytes) = value {
                        for (name, val) in parse_iso_info(bytes) {
                            tags.insert(
                                0x9000 + tags.len() as u16, // Pseudo tag ID
                                MakerNoteTag {
                                    tag_id: 0x9000 + tags.len() as u16,
                                    tag_name: Some(Box::leak(name.into_boxed_str())),
                                    value: ExifValue::Ascii(val),
                                },
                            );
                        }
                    }
                }
                NIKON_VR_INFO => {
                    if let ExifValue::Undefined(ref bytes) = value {
                        for (name, val) in parse_vr_info(bytes, maker_endian) {
                            tags.insert(
                                0x9100 + tags.len() as u16, // Pseudo tag ID
                                MakerNoteTag {
                                    tag_id: 0x9100 + tags.len() as u16,
                                    tag_name: Some(Box::leak(name.into_boxed_str())),
                                    value: ExifValue::Ascii(val),
                                },
                            );
                        }
                    }
                }
                NIKON_PICTURE_CONTROL_DATA => {
                    if let ExifValue::Undefined(ref bytes) = value {
                        for (name, val) in parse_picture_control(bytes) {
                            tags.insert(
                                0x9200 + tags.len() as u16, // Pseudo tag ID
                                MakerNoteTag {
                                    tag_id: 0x9200 + tags.len() as u16,
                                    tag_name: Some(Box::leak(name.into_boxed_str())),
                                    value: ExifValue::Ascii(val),
                                },
                            );
                        }
                    }
                }
                NIKON_FLASH_INFO => {
                    if let ExifValue::Undefined(ref bytes) = value {
                        for (name, val) in parse_flash_info(bytes) {
                            tags.insert(
                                0x9300 + tags.len() as u16, // Pseudo tag ID
                                MakerNoteTag {
                                    tag_id: 0x9300 + tags.len() as u16,
                                    tag_name: Some(Box::leak(name.into_boxed_str())),
                                    value: ExifValue::Ascii(val),
                                },
                            );
                        }
                    }
                }
                _ => {}
            }
        }
    }

    Ok(tags)
}
