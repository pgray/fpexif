# Fujifilm RAF Maker Notes Analysis and Fix Proposal

## Implementation Status

The following items from this analysis have been implemented:

### Completed
- **Basic IFD parsing** - Core maker note structure parsing is working
- **Tag definitions** - Comprehensive tag constants defined (37 value mismatches = lowest of all formats)
- **Film Simulation decoding** - FilmMode values decoded to human-readable names
- **WhiteBalance decoder** - Auto, Daylight, Cloudy, etc.
- **Sharpness decoder** - Softest to Hardest scale
- **Dynamic Range decoder** - Standard, Wide, Auto
- **Warning flags** - BlurWarning, FocusWarning, ExposureWarning decoders

### Pending
- FocalLength formatting (always show .0 for whole numbers)
- AutoDynamicRange tag (0x140B)
- Grain Effect / Color Chrome Effect tags
- Computed fields (DOF, CircleOfConfusion, HyperfocalDistance)

---

## Executive Summary

The Fujifilm maker notes implementation is relatively mature with proper IFD parsing and many tag definitions. The test results show excellent value matching with only **37 value mismatches** across 30 RAF files - the lowest mismatch count among all camera brands tested. The main areas for improvement are missing fields (computed fields and sub-structures) and minor formatting differences.

## Test Results Overview

- **Files Tested**: 30 RAF files
- **Files Passing**: 0
- **Total Issues**: 2,986
- **Value Mismatches**: 37 (Lowest of all formats!)
- **Missing Fields**: 2,333

## Value Mismatch Summary

With only 37 total value mismatches across 30 files, Fujifilm has the best match rate:

| Issue | Count | Notes |
|-------|-------|-------|
| FocalLength format | ~30 | `"18 mm"` vs `"18.0 mm"` |
| Other minor issues | ~7 | Various edge cases |

## Detailed Value Mismatches

### FocalLength Formatting (Primary Issue)

**Current**: `"18 mm"` (no decimal for integer values)
**Expected**: `"18.0 mm"` (always show .0 for whole numbers)

**File**: `src/output.rs`

```rust
// FocalLength formatting - always show one decimal place
0x920A => {
    let focal_length = num as f64 / den as f64;
    Value::String(format!("{:.1} mm", focal_length))
}
```

Note: This conflicts with Nikon's preference for no decimal. Consider making this camera-specific.

## Missing Fields Analysis

Top 20 most frequently missing fields:

| Field | Count | Category | Difficulty |
|-------|-------|----------|------------|
| AFMode | 30 | Already defined, not parsed | Easy |
| AutoBracketing | 30 | Already defined, not parsed | Easy |
| AutoDynamicRange | 28 | New tag needed | Medium |
| BitsPerSample | 30 | TIFF standard | Easy |
| BlackLevel | 30 | RAW processing | Medium |
| BlueBalance | 30 | WB structure | Medium |
| BlurWarning | 30 | Already defined | Easy |
| ChromaticAberrationParams | 30 | Complex structure | Hard |
| CircleOfConfusion | 30 | Computed field | Medium |
| ColorComponents | 30 | JPEG structure | Easy |
| ColorTempKelvin | 30 | WB structure | Medium |
| Compression | 30 | Already standard | Easy |
| DOF | 30 | Computed field | Medium |
| DynamicRange | 30 | Already defined | Easy |
| ExposureWarning | 30 | Already defined | Easy |
| FilmMode | 30 | Already defined | Easy |
| FocusWarning | 30 | Already defined | Easy |
| FujiFlashMode | 28 | Already defined | Easy |
| HyperfocalDistance | 30 | Computed field | Medium |
| ISO | 30 | Standard EXIF | Easy |

### Key Insight

Many "missing" fields are already defined in `src/makernotes/fuji.rs` but may not be exported correctly or the tag names don't match ExifTool's naming convention.

## Quick Wins

### 1. Export Already-Defined Tags

These tags are defined but may not appear in output:

```rust
// Tags already defined in fuji.rs
FUJI_BLUR_WARNING       // 0x1300 - BlurWarning
FUJI_FOCUS_WARNING      // 0x1301 - FocusWarning
FUJI_EXPOSURE_WARNING   // 0x1302 - ExposureWarning
FUJI_DYNAMIC_RANGE      // 0x1400 - DynamicRange
FUJI_FILM_MODE          // 0x1401 - FilmMode
FUJI_AUTO_BRACKETING    // 0x1100 - AutoBracketing
```

Verify these are being parsed and output correctly.

### 2. Add Value Description Lookups

**File**: `src/makernotes/fuji.rs`

```rust
/// Decode BlurWarning value
pub fn decode_blur_warning(value: u16) -> &'static str {
    match value {
        0 => "None",
        1 => "Blur Warning",
        _ => "Unknown",
    }
}

/// Decode FocusWarning value
pub fn decode_focus_warning(value: u16) -> &'static str {
    match value {
        0 => "Good",
        1 => "Out of Focus",
        _ => "Unknown",
    }
}

/// Decode ExposureWarning value
pub fn decode_exposure_warning(value: u16) -> &'static str {
    match value {
        0 => "Good",
        1 => "Over-exposed",
        _ => "Unknown",
    }
}

/// Decode DynamicRange value
pub fn decode_dynamic_range(value: u16) -> &'static str {
    match value {
        1 => "Standard",
        3 => "Wide",
        _ => "Unknown",
    }
}

/// Decode FilmMode value (Film Simulation)
pub fn decode_film_mode(value: u16) -> &'static str {
    match value {
        0 => "F0/Standard (Provia)",
        256 => "F1/Studio Portrait",
        272 => "F1a/Studio Portrait Enhanced Saturation",
        288 => "F1b/Studio Portrait Smooth Skin Tone",
        304 => "F1c/Studio Portrait Increased Sharpness",
        512 => "F2/Fujichrome (Velvia)",
        768 => "F3/Studio Portrait Ex",
        1024 => "F4/Velvia",
        1280 => "Pro Neg. Std",
        1281 => "Pro Neg. Hi",
        1536 => "Classic Chrome",
        1792 => "Eterna",
        2048 => "Classic Negative",
        _ => "Unknown",
    }
}

/// Decode Sharpness value
pub fn decode_sharpness(value: i16) -> &'static str {
    match value {
        -4 => "Softest",
        -3 => "Very Soft",
        -2 => "Soft",
        -1 => "Medium Soft",
        0 => "Normal",
        1 => "Medium Hard",
        2 => "Hard",
        3 => "Very Hard",
        4 => "Hardest",
        _ => "Unknown",
    }
}

/// Decode WhiteBalance value
pub fn decode_white_balance(value: u16) -> &'static str {
    match value {
        0 => "Auto",
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
        _ => "Unknown",
    }
}

/// Decode FlashMode value
pub fn decode_flash_mode(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        1 => "On",
        2 => "Off",
        3 => "Red-eye Reduction",
        4 => "External",
        _ => "Unknown",
    }
}

/// Decode Macro value
pub fn decode_macro(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        _ => "Unknown",
    }
}

/// Decode FocusMode value
pub fn decode_focus_mode(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        1 => "Manual",
        _ => "Unknown",
    }
}

/// Decode SlowSync value
pub fn decode_slow_sync(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        _ => "Unknown",
    }
}

/// Decode PictureMode value
pub fn decode_picture_mode(value: u16) -> &'static str {
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
        27 => "Dog",
        28 => "Cat",
        64 => "Aperture Priority AE",
        128 => "Shutter Priority AE",
        256 => "Manual",
        _ => "Unknown",
    }
}
```

### 3. Add New Tag Definitions

**File**: `src/makernotes/fuji.rs`

Add tags identified as missing:

```rust
// Additional Fujifilm tags
pub const FUJI_AUTO_DYNAMIC_RANGE: u16 = 0x140B;
pub const FUJI_IMAGE_GENERATION: u16 = 0x1425;
pub const FUJI_NOISE_REDUCTION: u16 = 0x100B;
pub const FUJI_HIGH_ISO_NOISE_REDUCTION: u16 = 0x100E;
pub const FUJI_CLARITY: u16 = 0x100F;
pub const FUJI_GRAIN_EFFECT: u16 = 0x1047;
pub const FUJI_COLOR_CHROME_EFFECT: u16 = 0x1048;
pub const FUJI_COLOR_CHROME_FX_BLUE: u16 = 0x104D;
pub const FUJI_WHITE_BALANCE_FINE_TUNE: u16 = 0x100A;

// And add to get_fuji_tag_name:
FUJI_AUTO_DYNAMIC_RANGE => Some("AutoDynamicRange"),
FUJI_IMAGE_GENERATION => Some("ImageGeneration"),
FUJI_NOISE_REDUCTION => Some("NoiseReduction"),
FUJI_HIGH_ISO_NOISE_REDUCTION => Some("HighISONoiseReduction"),
FUJI_CLARITY => Some("Clarity"),
FUJI_GRAIN_EFFECT => Some("GrainEffect"),
FUJI_COLOR_CHROME_EFFECT => Some("ColorChromeEffect"),
FUJI_COLOR_CHROME_FX_BLUE => Some("ColorChromeFXBlue"),
FUJI_WHITE_BALANCE_FINE_TUNE => Some("WhiteBalanceFineTune"),
```

## Computed Fields

These fields are calculated from other metadata rather than read directly:

| Field | Calculation |
|-------|-------------|
| CircleOfConfusion | Sensor size / 1500 |
| DOF | f(focal length, aperture, focus distance) |
| FieldOfView | 2 * atan(sensor_size / (2 * focal_length)) |
| HyperfocalDistance | focal_length^2 / (aperture * CoC) |
| LightValue | log2(f^2 / exposure_time) |
| ScaleFactor35efl | 43.27 / sensor_diagonal |

These require additional sensor size database and computational logic.

## Implementation Priority

1. **Phase 1 (Immediate - 30 minutes)**:
   - Fix FocalLength decimal formatting (if needed)
   - Verify existing tags are being exported
   - Expected: ~30 mismatches eliminated

2. **Phase 2 (Low Priority - 1 hour)**:
   - Add value description lookup functions
   - Apply lookups to parsed values
   - Expected: Minor quality improvement

3. **Phase 3 (Medium Priority - 2 hours)**:
   - Add new tag definitions
   - Improve parsing coverage
   - Expected: ~200 missing fields addressed

4. **Phase 4 (Future)**:
   - Add computed field calculations
   - Parse complex sub-structures

## Expected Impact

Fujifilm is already in excellent shape:

- **Current value mismatches**: 37
- **After Phase 1**: ~7 mismatches (81% reduction)
- **After Phase 2-3**: Minimal additional reduction, but more complete output

## Testing Strategy

After each phase:
```bash
FPEXIF_TEST_FILES=/fpexif/raws cargo test --test exiftool_json_comparison test_exiftool_json_compatibility_raf -- --nocapture
```

## Summary

Fujifilm is the best-performing format in terms of value accuracy. The focus should be on:

1. Exporting more fields that are already parsed
2. Adding value lookups for human-readable output
3. Adding new tag definitions for complete coverage

The 37 value mismatches are primarily formatting differences (FocalLength), not parsing errors.
