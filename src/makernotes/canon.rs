// makernotes/canon.rs - Canon maker notes parsing

use crate::data_types::{Endianness, ExifValue};
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
        224 => Some("Canon EF 70-200mm f/2.8L IS II USM"),
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

/// Get the Canon model name from model ID
pub fn get_canon_model_name(model_id: u32) -> Option<&'static str> {
    match model_id {
        // EOS DSLRs
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
        0x80000271 => Some("EOS-1D Mark IV"),
        0x80000281 => Some("EOS 5D Mark III"),
        0x80000285 => Some("EOS Rebel T3i / 600D / Kiss X5"),
        0x80000286 => Some("EOS 60D"),
        0x80000287 => Some("EOS Rebel T3 / 1100D / Kiss X50"),
        0x80000288 => Some("EOS 7D Mark II"),
        0x80000289 => Some("EOS 5D Mark IV"),
        0x80000301 => Some("EOS Rebel T4i / 650D / Kiss X6i"),
        0x80000302 => Some("EOS 6D"),
        0x80000324 => Some("EOS-1D C"),
        0x80000325 => Some("EOS 70D"),
        0x80000326 => Some("EOS Rebel T5i / 700D / Kiss X7i"),
        0x80000327 => Some("EOS Rebel T5 / 1200D / Kiss X70"),
        0x80000328 => Some("EOS-1D X Mark II"),
        0x80000331 => Some("EOS M"),
        0x80000346 => Some("EOS Rebel SL1 / 100D / Kiss X7"),
        0x80000347 => Some("EOS Rebel T6s / 760D / 8000D"),
        0x80000349 => Some("EOS 5DS"),
        0x80000350 => Some("EOS 5DS R"),
        0x80000355 => Some("EOS M2"),
        0x80000382 => Some("EOS 80D"),
        0x80000393 => Some("EOS Rebel T6i / 750D / Kiss X8i"),
        0x80000401 => Some("EOS Rebel T6 / 1300D / Kiss X80"),
        0x80000404 => Some("EOS M3"),
        0x80000405 => Some("EOS M10"),
        0x80000406 => Some("EOS Rebel T7i / 800D / Kiss X9i"),
        0x80000408 => Some("EOS 77D / 9000D"),
        0x80000417 => Some("EOS Rebel SL2 / 200D / Kiss X9"),
        0x80000421 => Some("EOS 6D Mark II"),
        0x80000422 => Some("EOS Rebel T7 / 2000D / Kiss X90"),
        0x80000424 => Some("EOS M50 / Kiss M"),
        0x80000428 => Some("EOS R"),
        0x80000432 => Some("EOS RP"),
        0x80000435 => Some("EOS Rebel SL3 / 250D / Kiss X10"),
        0x80000436 => Some("EOS 90D"),
        0x80000437 => Some("EOS M6 Mark II"),
        0x80000450 => Some("EOS R5"),
        0x80000453 => Some("EOS R6"),
        0x80000464 => Some("EOS-1D X Mark III"),
        0x80000468 => Some("EOS M50 Mark II / Kiss M2"),
        // PowerShot models
        0x01140000 => Some("PowerShot S30"),
        0x01668000 => Some("PowerShot G2"),
        0x01720000 => Some("PowerShot S40"),
        0x01840000 => Some("PowerShot G3"),
        0x02230000 => Some("PowerShot G5"),
        0x02290000 => Some("PowerShot S50"),
        0x02460000 => Some("PowerShot A75"),
        0x02700000 => Some("PowerShot G6"),
        0x02720000 => Some("PowerShot S70"),
        0x02890000 => Some("PowerShot A620"),
        0x02950000 => Some("PowerShot G7"),
        0x03110000 => Some("PowerShot G9"),
        0x03190000 => Some("PowerShot G10"),
        0x03320000 => Some("PowerShot G11"),
        0x03340000 => Some("PowerShot S90"),
        0x03390000 => Some("PowerShot SX1 IS"),
        0x03540000 => Some("PowerShot G12"),
        0x03640000 => Some("PowerShot G1 X"),
        0x03950000 => Some("PowerShot S110"),
        0x04040000 => Some("PowerShot G15"),
        0x04060000 => Some("PowerShot G16"),
        0x04180000 => Some("PowerShot G1 X Mark II"),
        0x04350000 => Some("PowerShot G7 X"),
        0x04370000 => Some("PowerShot G3 X"),
        0x04380000 => Some("PowerShot G9 X"),
        0x04420000 => Some("PowerShot G5 X"),
        0x04470000 => Some("PowerShot G7 X Mark II"),
        0x04510000 => Some("PowerShot G1 X Mark III"),
        0x04560000 => Some("PowerShot G9 X Mark II"),
        0x04620000 => Some("PowerShot SX60 HS"),
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
/// that can be easily interpreted.
pub fn decode_camera_settings(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // Macro mode (index 1)
    if data.len() > 1 {
        let macro_mode = match data[1] {
            1 => "Macro",
            2 => "Normal",
            _ => "Unknown",
        };
        decoded.insert(
            "MacroMode".to_string(),
            ExifValue::Ascii(macro_mode.to_string()),
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
        let quality = match data[3] {
            1 => "Economy",
            2 => "Normal",
            3 => "Fine",
            4 => "RAW",
            5 => "Superfine",
            130 => "Normal Movie",
            _ => "Unknown",
        };
        decoded.insert("Quality".to_string(), ExifValue::Ascii(quality.to_string()));
    }

    // Flash mode (index 4)
    if data.len() > 4 {
        let flash_mode = match data[4] {
            0 => "Flash Not Fired",
            1 => "Auto",
            2 => "On",
            3 => "Red-eye reduction",
            4 => "Slow-sync",
            5 => "Auto + Red-eye reduction",
            6 => "On + Red-eye reduction",
            16 => "External flash",
            _ => "Unknown",
        };
        decoded.insert(
            "FlashMode".to_string(),
            ExifValue::Ascii(flash_mode.to_string()),
        );
    }

    // Drive mode (index 5)
    if data.len() > 5 {
        let drive_mode = match data[5] {
            0 => "Single-frame Shooting",
            1 => "Continuous Shooting",
            2 => "Movie",
            3 => "Continuous, Speed Priority",
            4 => "Continuous, Low",
            5 => "Continuous, High",
            6 => "Silent Single Shooting",
            9 => "Single-frame Shooting, Silent",
            10 => "Continuous Shooting, Silent",
            _ => "Unknown",
        };
        decoded.insert(
            "DriveMode".to_string(),
            ExifValue::Ascii(drive_mode.to_string()),
        );
    }

    // Focus mode (index 7)
    if data.len() > 7 {
        let focus_mode = match data[7] {
            0 => "One-shot AF",
            1 => "AI Servo AF",
            2 => "AI Focus AF",
            3 => "Manual Focus",
            4 => "Single",
            5 => "Continuous",
            6 => "Manual Focus",
            16 => "Pan Focus",
            256 => "Manual",
            _ => "Unknown",
        };
        decoded.insert(
            "FocusMode".to_string(),
            ExifValue::Ascii(focus_mode.to_string()),
        );
    }

    // AF assist beam (index 10)
    if data.len() > 10 {
        let af_assist = match data[10] {
            0 => "Off",
            1 => "On (Auto)",
            2 => "On",
            0xFFFF => "n/a",
            _ => "Unknown",
        };
        decoded.insert(
            "AFAssistBeam".to_string(),
            ExifValue::Ascii(af_assist.to_string()),
        );
    }

    // Metering mode (index 17)
    if data.len() > 17 {
        let metering_mode = match data[17] {
            3 => "Evaluative",
            4 => "Partial",
            5 => "Center-weighted average",
            _ => "Unknown",
        };
        decoded.insert(
            "MeteringMode".to_string(),
            ExifValue::Ascii(metering_mode.to_string()),
        );
    }

    // Focus range (index 18)
    if data.len() > 18 {
        let focus_range = match data[18] {
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
            _ => "Unknown",
        };
        decoded.insert(
            "FocusRange".to_string(),
            ExifValue::Ascii(focus_range.to_string()),
        );
    }

    // Exposure mode (index 20)
    if data.len() > 20 {
        let exposure_mode = match data[20] {
            0 => "Easy",
            1 => "Program AE",
            2 => "Shutter speed priority AE",
            3 => "Aperture-priority AE",
            4 => "Manual",
            5 => "Depth-of-field AE",
            6 => "M-Dep",
            7 => "Bulb",
            _ => "Unknown",
        };
        decoded.insert(
            "ExposureMode".to_string(),
            ExifValue::Ascii(exposure_mode.to_string()),
        );
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
    if data.len() > 26 && data[26] > 0 {
        let apex = data[26] as f64 / 32.0;
        let f_number = 2f64.powf(apex / 2.0);
        let rounded = (f_number * 10.0).round() / 10.0;
        decoded.insert(
            "MaxAperture".to_string(),
            ExifValue::Ascii(format!("{}", rounded)),
        );
    }

    // Min aperture (index 27) - Convert Canon aperture value to f-number
    if data.len() > 27 && data[27] > 0 {
        let apex = data[27] as f64 / 32.0;
        let f_number = 2f64.powf(apex / 2.0);
        let rounded = (f_number * 10.0).round() / 10.0;
        decoded.insert(
            "MinAperture".to_string(),
            ExifValue::Ascii(format!("{}", rounded)),
        );
    }

    // Camera ISO (index 16)
    // Canon uses a special encoding: if bit 0x4000 is set, the lower 14 bits are the ISO value
    // Otherwise it's a lookup table value (15=Auto, 16=50, 17=100, 18=200, 19=400, 20=800)
    if data.len() > 16 && data[16] != 0x7FFF {
        let raw_iso = data[16];
        let iso_value = if raw_iso & 0x4000 != 0 {
            // Direct ISO value in lower 14 bits
            raw_iso & 0x3FFF
        } else {
            // Lookup table value
            match raw_iso {
                0 => 0,  // n/a
                14 => 0, // Auto High (show as 0)
                15 => 0, // Auto (show as 0)
                16 => 50,
                17 => 100,
                18 => 200,
                19 => 400,
                20 => 800,
                _ => raw_iso, // Unknown, return raw value
            }
        };
        decoded.insert("CameraISO".to_string(), ExifValue::Short(vec![iso_value]));
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
            if bits & 0x0080 != 0 {
                descriptions.push("External");
            } else if bits & 0x0040 != 0 {
                descriptions.push("Internal");
            }
            if descriptions.is_empty() {
                format!("Unknown (0x{:04x})", bits)
            } else {
                descriptions.join(", ")
            }
        };
        decoded.insert("FlashBits".to_string(), ExifValue::Ascii(flash_bits_desc));
    }

    // Image stabilization (index 34)
    if data.len() > 34 {
        let is_value = data[34];
        let is_desc = match is_value {
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
            _ => "Unknown",
        };
        decoded.insert(
            "ImageStabilization".to_string(),
            ExifValue::Ascii(is_desc.to_string()),
        );
    }

    // Manual flash output (index 41)
    if data.len() > 41 {
        let mfo = data[41];
        let mfo_desc = match mfo {
            0x0000 => "n/a",
            0x0500 => "Full",
            0x0502 => "Medium",
            0x0504 => "Low",
            0x7FFF => "n/a",
            _ => "Unknown",
        };
        decoded.insert(
            "ManualFlashOutput".to_string(),
            ExifValue::Ascii(mfo_desc.to_string()),
        );
    }

    decoded
}

/// Decode Canon ShotInfo array sub-fields
///
/// This function decodes the CanonShotInfo array into individual fields
/// that can be easily interpreted.
pub fn decode_shot_info(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // Auto ISO (index 1)
    if data.len() > 1 {
        decoded.insert("AutoISO".to_string(), ExifValue::Short(vec![data[1]]));
    }

    // Base ISO (index 2)
    if data.len() > 2 {
        decoded.insert("BaseISO".to_string(), ExifValue::Short(vec![data[2]]));
    }

    // Measured EV (index 3) - Canon APEX value, stored as value/32 with +5.0 offset
    // MeasuredEV represents the metered exposure value
    // ExifTool uses: MeasuredEV = (raw / 32.0) + 5.0
    if data.len() > 3 {
        let raw = data[3] as i16; // Treat as signed for negative EV values
        let ev = (raw as f64 / 32.0) + 5.0;
        decoded.insert(
            "MeasuredEV".to_string(),
            ExifValue::Ascii(format!("{:.2}", ev)),
        );
    }

    // Target aperture (index 4) - Canon APEX aperture value
    // f-number = 2^(value/64)
    if data.len() > 4 && data[4] > 0 {
        let apex = data[4] as f64 / 64.0;
        let f_number = 2f64.powf(apex);
        let rounded = (f_number * 10.0).round() / 10.0;
        decoded.insert(
            "TargetAperture".to_string(),
            ExifValue::Ascii(format!("{}", rounded)),
        );
    }

    // Target exposure time (index 5) - Canon APEX time value
    // exposure time = 2^(-value/32), displayed as fraction
    if data.len() > 5 && data[5] > 0 {
        let apex = data[5] as f64 / 32.0;
        let time = 2f64.powf(-apex);
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
        let wb = match data[7] {
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
            _ => "Unknown",
        };
        decoded.insert("WhiteBalance".to_string(), ExifValue::Ascii(wb.to_string()));
    }

    // Sequence number (index 9)
    if data.len() > 9 {
        decoded.insert(
            "SequenceNumber".to_string(),
            ExifValue::Short(vec![data[9]]),
        );
    }

    // Flash exposure compensation (index 15)
    if data.len() > 15 {
        decoded.insert(
            "FlashExposureComp".to_string(),
            ExifValue::Short(vec![data[15]]),
        );
    }

    // Auto exposure bracketing (index 16)
    if data.len() > 16 {
        let aeb = match data[16] {
            0xFFFF | 0 => "Off",
            1 => "On",
            _ => "Unknown",
        };
        decoded.insert(
            "AEBBracketValue".to_string(),
            ExifValue::Ascii(aeb.to_string()),
        );
    }

    // Exposure compensation (index 6)
    if data.len() > 6 {
        // Canon stores exposure compensation as a signed value
        // The value needs to be divided by 32 to get the actual EV compensation
        let raw = data[6] as i16;
        let ev_comp = raw as f64 / 32.0;
        decoded.insert(
            "ExposureCompensation".to_string(),
            ExifValue::Ascii(format!("{:+.1}", ev_comp)),
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
        // Similar to exposure compensation, divide by 32
        let raw = data[15] as i16;
        let flash_comp = raw as f64 / 32.0;
        decoded.insert(
            "FlashExposureComp".to_string(),
            ExifValue::Ascii(format!("{:+.1}", flash_comp)),
        );
    }

    // Subject distance (index 19)
    if data.len() > 19 {
        decoded.insert(
            "SubjectDistance".to_string(),
            ExifValue::Short(vec![data[19]]),
        );
    }

    decoded
}

/// Decode Canon FocalLength array sub-fields
///
/// This function decodes the CanonFocalLength array into individual fields
/// that can be easily interpreted.
pub fn decode_focal_length(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // Focal type (index 0)
    if !data.is_empty() {
        let focal_type = match data[0] {
            0 => "n/a",
            1 => "Fixed",
            2 => "Zoom",
            _ => "Unknown",
        };
        decoded.insert(
            "FocalType".to_string(),
            ExifValue::Ascii(focal_type.to_string()),
        );
    }

    // Focal length (index 1)
    // Canon stores focal length multiplied by FocalUnits (typically 1)
    // Exiftool shows this as "55.0 mm" format
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

/// Decode Canon FileInfo array sub-fields
///
/// This function decodes the CanonFileInfo array into individual fields
/// that can be easily interpreted.
pub fn decode_file_info(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // File number (index 1)
    if data.len() > 1 {
        decoded.insert("FileNumber".to_string(), ExifValue::Short(vec![data[1]]));
    }

    // Bracket mode (index 3)
    if data.len() > 3 {
        let bracket_mode = match data[3] {
            0 => "Off",
            1 => "AEB",
            2 => "FEB",
            3 => "ISO",
            4 => "WB",
            _ => "Unknown",
        };
        decoded.insert(
            "BracketMode".to_string(),
            ExifValue::Ascii(bracket_mode.to_string()),
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

    decoded
}

/// Decode Canon AFInfo2 array sub-fields
///
/// This function decodes the CanonAFInfo2 array which contains AF area information.
pub fn decode_af_info2(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // AF area mode (index 0)
    if !data.is_empty() {
        let af_mode = match data[0] {
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
            // Compact camera modes
            0x0060 => "Face AiAF",
            0x1001 => "Single-point AF (Compact)",
            0x1002 => "Tracking AF (Compact)",
            0x1003 => "Face + Tracking (Compact)",
            0xFFFF => "n/a",
            _ => "Unknown",
        };
        decoded.insert(
            "AFAreaMode".to_string(),
            ExifValue::Ascii(af_mode.to_string()),
        );
    }

    // Number of AF points (index 1)
    if data.len() > 1 {
        let num_af_points = data[1];
        decoded.insert(
            "NumAFPoints".to_string(),
            ExifValue::Short(vec![num_af_points]),
        );

        // AF area widths (index 2 onwards, based on num_af_points)
        let mut widths = Vec::new();
        let mut heights = Vec::new();
        let mut x_positions = Vec::new();
        let mut y_positions = Vec::new();

        for i in 0..num_af_points as usize {
            // Widths start at index 2
            if data.len() > 2 + i {
                widths.push(data[2 + i]);
            }
            // Heights start after widths
            if data.len() > 2 + num_af_points as usize + i {
                heights.push(data[2 + num_af_points as usize + i]);
            }
            // X positions start after heights
            if data.len() > 2 + 2 * num_af_points as usize + i {
                x_positions.push(data[2 + 2 * num_af_points as usize + i]);
            }
            // Y positions start after X positions
            if data.len() > 2 + 3 * num_af_points as usize + i {
                y_positions.push(data[2 + 3 * num_af_points as usize + i]);
            }
        }

        if !widths.is_empty() {
            decoded.insert("AFAreaWidths".to_string(), ExifValue::Short(widths));
        }
        if !heights.is_empty() {
            decoded.insert("AFAreaHeights".to_string(), ExifValue::Short(heights));
        }
        if !x_positions.is_empty() {
            decoded.insert(
                "AFAreaXPositions".to_string(),
                ExifValue::Short(x_positions),
            );
        }
        if !y_positions.is_empty() {
            decoded.insert(
                "AFAreaYPositions".to_string(),
                ExifValue::Short(y_positions),
            );
        }
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
            // Skip large binary blobs to save memory (camera info, dust removal, color data)
            if matches!(
                tag_id,
                CANON_CAMERA_INFO | CANON_DUST_REMOVAL_DATA | CANON_COLOR_DATA
            ) {
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
            );

            if should_decode {
                if let ExifValue::Short(ref shorts) = value {
                    let decoded = match tag_id {
                        CANON_CAMERA_SETTINGS => decode_camera_settings(shorts),
                        CANON_SHOT_INFO => decode_shot_info(shorts),
                        CANON_FOCAL_LENGTH => decode_focal_length(shorts),
                        CANON_FILE_INFO => decode_file_info(shorts),
                        CANON_AF_INFO_2 => decode_af_info2(shorts),
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
