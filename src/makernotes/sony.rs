// makernotes/sony.rs - Sony maker notes parsing

use crate::data_types::{Endianness, ExifValue};
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use std::collections::HashMap;

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
pub const SONY_LONG_EXPOSURE_NOISE_REDUCTION_2: u16 = 0xB04E;
pub const SONY_DYNAMIC_RANGE_OPTIMIZER_2: u16 = 0xB04F;
pub const SONY_HIGH_ISO_NOISE_REDUCTION_2: u16 = 0xB050;
pub const SONY_INTELLIGENT_AUTO: u16 = 0xB052;
pub const SONY_WHITE_BALANCE_2: u16 = 0xB054;

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
        SONY_DISTORTION_CORRECTION => Some("DistortionCorrection"),
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
        SONY_QUALITY_2 => Some("Quality"),
        SONY_FLASH_LEVEL => Some("FlashLevel"),
        SONY_RELEASE_MODE => Some("ReleaseMode"),
        SONY_ANTI_BLUR => Some("AntiBlur"),
        SONY_INTELLIGENT_AUTO => Some("IntelligentAuto"),
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
        281 => Some("SLT-A55"),
        285 => Some("SLT-A35"),
        286 => Some("SLT-A65"),
        287 => Some("SLT-A77"),
        291 => Some("SLT-A37"),
        292 => Some("SLT-A57"),
        294 => Some("SLT-A99"),
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
        45 => Some("Carl Zeiss Planar T* 85mm F1.4 ZA"),
        46 => Some("Carl Zeiss Vario-Sonnar T* DT 16-80mm F3.5-4.5 ZA"),
        47 => Some("Carl Zeiss Sonnar T* 135mm F1.8 ZA"),
        48 => Some("Carl Zeiss Vario-Sonnar T* 24-70mm F2.8 ZA SSM"),
        49 => Some("Sony DT 55-200mm F4-5.6 (SAL55200)"),
        50 => Some("Sony DT 18-250mm F3.5-6.3 (SAL18250)"),
        51 => Some("Sony DT 16-105mm F3.5-5.6 (SAL16105)"),
        52 => Some("Sony 70-300mm F4.5-5.6 G SSM (SAL70300G)"),
        53 => Some("Sony 70-400mm F4-5.6 G SSM (SAL70400G)"),
        54 => Some("Carl Zeiss Vario-Sonnar T* 16-35mm F2.8 ZA SSM"),
        55 => Some("Sony DT 18-55mm F3.5-5.6 SAM (SAL1855)"),
        56 => Some("Sony DT 55-200mm F4-5.6 SAM (SAL55200-2)"),
        57 => Some("Sony DT 50mm F1.8 SAM (SAL50F18)"),
        58 => Some("Sony DT 30mm F2.8 Macro SAM (SAL30M28)"),
        59 => Some("Sony 28-75mm F2.8 SAM (SAL2875)"),
        60 => Some("Carl Zeiss Distagon T* 24mm F2 ZA SSM"),
        61 => Some("Sony 85mm F2.8 SAM (SAL85F28)"),
        62 => Some("Sony DT 35mm F1.8 SAM (SAL35F18)"),
        63 => Some("Sony DT 16-50mm F2.8 SSM (SAL1650)"),
        64 => Some("Sony 500mm F4 G SSM (SAL500F40G)"),
        65 => Some("Sony DT 18-135mm F3.5-5.6 SAM (SAL18135)"),
        66 => Some("Sony 300mm F2.8 G SSM II (SAL300F28G2)"),
        67 => Some("Sony 70-200mm F2.8 G SSM II (SAL70200G2)"),
        68 => Some("Sony DT 55-300mm F4.5-5.6 SAM (SAL55300)"),
        69 => Some("Sony 70-400mm F4-5.6 G SSM II (SAL70400G2)"),
        70 => Some("Carl Zeiss Planar T* 50mm F1.4 ZA SSM"),
        // Sony E-mount lenses (use sonyLensTypes2 format with high IDs)
        32784 => Some("Sony E 16mm F2.8 (SEL16F28)"),
        32785 => Some("Sony E 18-55mm F3.5-5.6 OSS (SEL1855)"),
        32786 => Some("Sony E 55-210mm F4.5-6.3 OSS (SEL55210)"),
        32787 => Some("Sony E 18-200mm F3.5-6.3 OSS (SEL18200)"),
        32788 => Some("Sony E 30mm F3.5 Macro (SEL30M35)"),
        32789 => Some("Sony E 24mm F1.8 ZA (SEL24F18Z)"),
        32790 => Some("Sony E 50mm F1.8 OSS (SEL50F18)"),
        32791 => Some("Sony E 16-50mm F3.5-5.6 PZ OSS (SELP1650)"),
        32792 => Some("Sony E 10-18mm F4 OSS (SEL1018)"),
        32793 => Some("Sony E PZ 18-105mm F4 G OSS (SELP18105G)"),
        32794 => Some("Sony E 20mm F2.8 (SEL20F28)"),
        32795 => Some("Sony E 35mm F1.8 OSS (SEL35F18)"),
        32796 => Some("Sony E PZ 18-200mm F3.5-6.3 OSS (SELP18200)"),
        32797 => Some("Sony FE 35mm F2.8 ZA (SEL35F28Z)"),
        32798 => Some("Sony FE 24-70mm F4 ZA OSS (SEL2470Z)"),
        32799 => Some("Sony FE 55mm F1.8 ZA (SEL55F18Z)"),
        32800 => Some("Sony FE 70-200mm F4 G OSS (SEL70200G)"),
        32801 => Some("Sony FE 28-70mm F3.5-5.6 OSS (SEL2870)"),
        32802 => Some("Sony FE 16-35mm F4 ZA OSS (SEL1635Z)"),
        32803 => Some("Sony FE 90mm F2.8 Macro G OSS (SEL90M28G)"),
        32807 => Some("Sony E 18-200mm F3.5-6.3 OSS LE (SEL18200LE)"),
        32808 => Some("Sony E 50mm F1.8 OSS (SEL50F18)"),
        32813 => Some("Sony FE 28mm F2 (SEL28F20)"),
        32814 => Some("Sony FE 35mm F1.4 ZA (SEL35F14Z)"),
        32815 => Some("Sony FE 24-240mm F3.5-6.3 OSS (SEL24240)"),
        32816 => Some("Sony FE 28-135mm F4 G PZ OSS (SELP28135G)"),
        32817 => Some("Sony FE PZ 28-135mm F4 G OSS (SELP28135G)"),
        32820 => Some("Sony FE 21mm F2.8 (SEL21F28)"),
        32821 => Some("Sony FE 16mm F3.5 Fisheye (SEL16F35)"),
        32826 => Some("Sony FE 85mm F1.4 GM (SEL85F14GM)"),
        32827 => Some("Sony FE 50mm F1.4 ZA (SEL50F14Z)"),
        32829 => Some("Sony FE 70-300mm F4.5-5.6 G OSS (SEL70300G)"),
        32830 => Some("Sony FE 100mm F2.8 STF GM OSS (SEL100F28GM)"),
        32831 => Some("Sony FE 50mm F2.8 Macro (SEL50M28)"),
        32832 => Some("Sony FE 85mm F1.8 (SEL85F18)"),
        33072 => Some("Sony FE 70-200mm F2.8 GM OSS (SEL70200GM)"),
        33073 => Some("Sony FE 24-70mm F2.8 GM (SEL2470GM)"),
        33076 => Some("Sony FE 100-400mm F4.5-5.6 GM OSS (SEL100400GM)"),
        33077 => Some("Sony FE 12-24mm F4 G (SEL1224G)"),
        33079 => Some("Sony FE 16-35mm F2.8 GM (SEL1635GM)"),
        33080 => Some("Sony FE 400mm F2.8 GM OSS (SEL400F28GM)"),
        33081 => Some("Sony FE 24mm F1.4 GM (SEL24F14GM)"),
        33082 => Some("Sony FE 135mm F1.8 GM (SEL135F18GM)"),
        33083 => Some("Sony FE 200-600mm F5.6-6.3 G OSS (SEL200600G)"),
        33084 => Some("Sony FE 600mm F4 GM OSS (SEL600F40GM)"),
        33085 => Some("Sony FE 20mm F1.8 G (SEL20F18G)"),
        33086 => Some("Sony FE 35mm F1.8 (SEL35F18F)"),
        33088 => Some("Sony FE 12-24mm F2.8 GM (SEL1224GM)"),
        33089 => Some("Sony FE 50mm F1.2 GM (SEL50F12GM)"),
        33090 => Some("Sony FE 14mm F1.8 GM (SEL14F18GM)"),
        33091 => Some("Sony FE 35mm F1.4 GM (SEL35F14GM)"),
        33092 => Some("Sony FE 24mm F2.8 G (SEL24F28G)"),
        33093 => Some("Sony FE 40mm F2.5 G (SEL40F25G)"),
        33094 => Some("Sony FE 50mm F2.5 G (SEL50F25G)"),
        33095 => Some("Sony FE 70-200mm F2.8 GM OSS II (SEL70200GM2)"),
        33096 => Some("Sony FE 24-70mm F2.8 GM II (SEL2470GM2)"),
        33097 => Some("Sony FE 16-35mm F2.8 GM II (SEL1635GM2)"),
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

/// Decode ExposureMode value (tag 0xB041) - ExifTool format
pub fn decode_exposure_mode_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        1 => "Portrait",
        2 => "Landscape",
        3 => "Macro",
        4 => "Sports",
        5 => "Sunset",
        6 => "Night Scene",
        7 => "Beach/Snow",
        8 => "Fireworks",
        9 => "Anti Motion Blur",
        10 => "Pet",
        11 => "Backlight Correction HDR",
        12 => "Superior Auto",
        13 => "High Sensitivity",
        14 => "Night Portrait",
        15 => "Hand-held Twilight",
        16 => "Sweep Panorama",
        17 => "3D Sweep Panorama",
        18 => "Background Defocus",
        19 => "Soft Skin",
        20 => "Portrait (with Smile Shutter)",
        _ => "Unknown",
    }
}

/// Decode ExposureMode value (tag 0xB041) - exiv2 format
/// Note: exiv2 has completely different value mappings
pub fn decode_exposure_mode_exiv2(value: u16) -> &'static str {
    match value {
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
        _ => "Unknown",
    }
}

/// Decode AFMode value (tag 0xB043) - ExifTool format
pub fn decode_af_mode_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Default",
        1 => "Multi",
        2 => "Center",
        3 => "Spot",
        4 => "Flexible Spot",
        5 => "Touch",
        6 => "Continuous",
        7 => "Face Detected",
        8 => "Tracking",
        9 => "Zone",
        10 => "Expand Flexible Spot",
        11 => "Flexible Spot (S)",
        12 => "Flexible Spot (M)",
        13 => "Flexible Spot (L)",
        14 => "Eye AF",
        _ => "Unknown",
    }
}

/// Decode AFMode value (tag 0xB043) - exiv2 format (Set1)
pub fn decode_af_mode_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Default",
        1 => "Multi",
        2 => "Center",
        3 => "Spot",
        4 => "Flexible Spot",
        6 => "Touch",
        14 => "Tracking",
        15 => "Face Detected",
        0xffff => "n/a",
        _ => "Unknown",
    }
}

/// Decode DynamicRangeOptimizer value (tag 0xB025) - ExifTool format
/// Note: This tag can be either SHORT or LONG depending on camera model
pub fn decode_dynamic_range_optimizer_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "Standard",
        2 => "Advanced Auto",
        3 => "Auto",
        4 => "Advanced Lv1",
        5 => "Advanced Lv2",
        6 => "Advanced Lv3",
        7 => "Advanced Lv4",
        8 => "Advanced Lv5",
        9 => "Lv1",
        10 => "Lv2",
        11 => "Lv3",
        12 => "Lv4",
        13 => "Lv5",
        _ => "Unknown",
    }
}

/// Decode DynamicRangeOptimizer value (tag 0xB025) - exiv2 format
/// Note: exiv2 uses different indices for Advanced levels (8-11 vs 4-7)
pub fn decode_dynamic_range_optimizer_exiv2(value: u16) -> &'static str {
    match value {
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
        _ => "Unknown",
    }
}

/// Decode FocusMode value (tag 0x201B) - ExifTool format
/// Note: This tag can be BYTE or SHORT depending on camera model
pub fn decode_focus_mode_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Manual",
        1 => "AF-S",
        2 => "AF-C",
        3 => "AF-A",
        4 => "DMF",
        5 => "AF-D",
        6 => "Single-shot AF",
        7 => "Automatic AF",
        _ => "Unknown",
    }
}

/// Decode FocusMode value (tag 0x201B) - exiv2 format (sonyFocusMode2)
/// Note: exiv2 has gaps in the sequence (no 1, 5)
pub fn decode_focus_mode_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Manual",
        2 => "AF-S",
        3 => "AF-C",
        4 => "AF-A",
        6 => "DMF",
        7 => "AF-D",
        _ => "Unknown",
    }
}

/// Decode ImageStabilization value (tag 0xB026) - ExifTool format
/// Note: This tag can be either SHORT or LONG depending on camera model
pub fn decode_image_stabilization_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        2 => "On (2)",
        3 => "On (Shooting)",
        _ => "Unknown",
    }
}

/// Decode ImageStabilization value (tag 0xB026) - exiv2 format
/// Note: exiv2 only has Off, On, n/a
pub fn decode_image_stabilization_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        0xffff => "n/a",
        _ => "Unknown",
    }
}

/// Decode ImageQuality value (tag 0x0102) - ExifTool format
/// Note: exiv2 uses identical values - no separate version needed
pub fn decode_image_quality_exiftool(value: u16) -> &'static str {
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
        _ => "Unknown",
    }
}
// decode_image_quality_exiv2 - same as exiftool, no separate function needed

/// Decode WhiteBalance value (tag 0x0115) - ExifTool format
/// Note: exiv2 uses identical values - no separate version needed
pub fn decode_white_balance_exiftool(value: u16) -> &'static str {
    match value {
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
        _ => "Unknown",
    }
}
// decode_white_balance_exiv2 - same as exiftool, no separate function needed

/// Decode LongExposureNoiseReduction value (tag 0x2008) - ExifTool format
/// Note: exiv2 uses identical values - no separate version needed
pub fn decode_long_exposure_noise_reduction_exiftool(value: u32) -> &'static str {
    match value {
        0 => "Off",
        1 => "On (unused)",
        0x10001 => "On (dark subtracted)",
        0xffff0000 => "Off (65535)",
        0xffff0001 => "On (65535)",
        0xffffffff => "n/a",
        _ => "Unknown",
    }
}
// decode_long_exposure_noise_reduction_exiv2 - same as exiftool, no separate function needed

/// Decode HighISONoiseReduction value (tag 0x2009) - ExifTool format
/// Note: exiv2 uses identical values - no separate version needed
pub fn decode_high_iso_noise_reduction_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "Low",
        2 => "Normal",
        3 => "High",
        256 => "Auto",
        65535 => "n/a",
        _ => "Unknown",
    }
}
// decode_high_iso_noise_reduction_exiv2 - same as exiftool, no separate function needed

/// Decode SceneMode value (tag 0xB023) - ExifTool format
/// Note: exiv2 uses identical values - no separate version needed
pub fn decode_scene_mode_exiftool(value: u16) -> &'static str {
    match value {
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
        _ => "Unknown",
    }
}
// decode_scene_mode_exiv2 - same as exiftool, no separate function needed

/// Decode Contrast/Saturation/Sharpness adjustment values - ExifTool format
/// (tags 0x2004/0x2005/0x2006)
/// Note: exiv2 uses identical values - no separate version needed
pub fn decode_adjustment_exiftool(value: i32) -> &'static str {
    match value {
        -3 => "-3",
        -2 => "-2",
        -1 => "-1",
        0 => "Normal",
        1 => "+1",
        2 => "+2",
        3 => "+3",
        _ => "Unknown",
    }
}
// decode_adjustment_exiv2 - same as exiftool, no separate function needed

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

/// Decode Teleconverter value (tag 0x0105) - ExifTool format
/// Based on ExifTool's minoltaTeleconverters
/// Note: exiv2 uses identical values - no separate version needed
pub fn decode_teleconverter_exiftool(value: u32) -> &'static str {
    match value {
        0x00 => "None",
        0x04 => "Minolta/Sony AF 1.4x APO (D) (0x04)",
        0x05 => "Minolta/Sony AF 2x APO (D) (0x05)",
        0x48 => "Minolta/Sony AF 2x APO (D)",
        0x50 => "Minolta AF 2x APO II",
        0x60 => "Minolta AF 2x APO",
        0x88 => "Minolta/Sony AF 1.4x APO (D)",
        0x90 => "Minolta AF 1.4x APO II",
        0xa0 => "Minolta AF 1.4x APO",
        _ => "Unknown",
    }
}
// decode_teleconverter_exiv2 - same as exiftool, no separate function needed

/// Decode PictureEffect value (tag 0x200E) - ExifTool format
pub fn decode_picture_effect_exiftool(value: u16) -> &'static str {
    match value {
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
        _ => "Unknown",
    }
}

/// Decode PictureEffect value (tag 0x200E) - exiv2 format
/// Note: exiv2 uses offset values for Miniature modes (48-54 vs 36-42)
pub fn decode_picture_effect_exiv2(value: u16) -> &'static str {
    match value {
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
        _ => "Unknown",
    }
}

/// Decode VignettingCorrection value (tag 0x2011) - ExifTool format
/// Note: exiv2 uses identical values - no separate version needed
pub fn decode_vignetting_correction_exiftool(value: u32) -> &'static str {
    match value {
        0 => "Off",
        2 => "Auto",
        0xffffffff => "n/a",
        _ => "Unknown",
    }
}
// decode_vignetting_correction_exiv2 - same as exiftool, no separate function needed

/// Decode DistortionCorrection value (tag 0x2013) - ExifTool format
/// Note: exiv2 uses identical values - no separate version needed
pub fn decode_distortion_correction_exiftool(value: u32) -> &'static str {
    match value {
        0 => "Off",
        2 => "Auto",
        0xffffffff => "n/a",
        _ => "Unknown",
    }
}
// decode_distortion_correction_exiv2 - same as exiftool, no separate function needed

/// Decode ReleaseMode value (tag 0xB049) - ExifTool format
/// Note: exiv2 uses identical values - no separate version needed
pub fn decode_release_mode_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Normal",
        2 => "Continuous",
        5 => "Exposure Bracketing",
        6 => "White Balance Bracketing",
        8 => "DRO Bracketing",
        65535 => "n/a",
        _ => "Unknown",
    }
}
// decode_release_mode_exiv2 - same as exiftool, no separate function needed

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

/// Decode IntelligentAuto value (tag 0xB052) - ExifTool format
/// Note: exiv2 uses identical values - no separate version needed
pub fn decode_intelligent_auto_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        2 => "Advanced",
        _ => "Unknown",
    }
}
// decode_intelligent_auto_exiv2 - same as exiftool, no separate function needed

/// Decode HDR value (tag 0x200A) - ExifTool format
/// Returns the first element of the HDR pair (level)
/// Note: exiv2 uses identical values - no separate version needed
pub fn decode_hdr_exiftool(value: u16) -> &'static str {
    match value {
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
        _ => "Unknown",
    }
}
// decode_hdr_exiv2 - same as exiftool, no separate function needed

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

/// Format LensSpec byte array as a readable string
/// LensSpec is 8 bytes: [flags1, short_focal, long_focal, max_ap_short, max_ap_long, flags2, ?, ?]
pub fn format_lens_spec(bytes: &[u8]) -> String {
    if bytes.len() < 8 {
        return format!("{:?}", bytes);
    }

    // Simple formatting: just show the focal length and aperture info
    let short_focal = bytes[1];
    let long_focal = bytes[2];
    let max_ap_short = bytes[3];
    let max_ap_long = bytes[4];

    if short_focal == long_focal {
        format!("{}mm F{}", short_focal, max_ap_short as f32 / 10.0)
    } else {
        format!(
            "{}-{}mm F{}-{}",
            short_focal,
            long_focal,
            max_ap_short as f32 / 10.0,
            max_ap_long as f32 / 10.0
        )
    }
}

/// Parse a single IFD entry from Sony maker notes
fn parse_ifd_entry(
    data: &[u8],
    entry_offset: usize,
    endian: Endianness,
    base_offset: usize,
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
    // Sony maker notes use offsets relative to the maker note start
    let value_data: &[u8] = if total_size <= 4 {
        value_offset_bytes
    } else {
        let offset = read_u32(value_offset_bytes, endian) as usize;
        // Try maker note-relative offset
        let abs_offset = if offset >= base_offset {
            offset - base_offset
        } else {
            offset
        };
        if abs_offset + total_size <= data.len() {
            &data[abs_offset..abs_offset + total_size]
        } else if offset + total_size <= data.len() {
            &data[offset..offset + total_size]
        } else {
            return None;
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
                    SONY_FOCUS_MODE => Some(decode_focus_mode_exiftool(v).to_string()),
                    _ => None,
                };

                if let Some(s) = decoded {
                    ExifValue::Ascii(s)
                } else {
                    ExifValue::Byte(bytes)
                }
            } else if tag_id == SONY_LENS_SPEC && bytes.len() >= 8 {
                // Format LensSpec byte array
                ExifValue::Ascii(format_lens_spec(&bytes))
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
            ExifValue::Ascii(s)
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
                    SONY_EXPOSURE_MODE => Some(decode_exposure_mode_exiftool(v).to_string()),
                    SONY_AF_MODE => Some(decode_af_mode_exiftool(v).to_string()),
                    SONY_DYNAMIC_RANGE_OPTIMIZER => {
                        Some(decode_dynamic_range_optimizer_exiftool(v).to_string())
                    }
                    SONY_FOCUS_MODE => Some(decode_focus_mode_exiftool(v).to_string()),
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
                    SONY_QUALITY_2 => Some(decode_quality_exiftool(v as u32).to_string()),
                    _ => None,
                };

                if let Some(s) = decoded {
                    ExifValue::Ascii(s)
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
        5 | 10 => {
            // RATIONAL or SRATIONAL
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
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    // Sony maker notes can start with "SONY DSC " or be in standard TIFF IFD format
    let mut offset = 0;
    let base_offset = 0usize;

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

        if let Some((tag_id, value)) = parse_ifd_entry(data, entry_offset, endian, base_offset) {
            tags.insert(
                tag_id,
                MakerNoteTag {
                    tag_id,
                    tag_name: get_sony_tag_name(tag_id),
                    value,
                },
            );
        }
    }

    Ok(tags)
}
