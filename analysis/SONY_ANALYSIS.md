# Sony ARW Maker Notes Analysis and Fix Proposal

## Executive Summary

The Sony maker notes implementation now correctly parses IFD structure and reads actual values (fixed in recent update), but needs value description lookups to convert numeric codes to human-readable strings. This analysis identifies the most impactful fixes needed to match ExifTool's output for 31 ARW test files.

## Test Results Overview

- **Files Tested**: 31 ARW files
- **Files Passing**: 0
- **Total Issues**: 7,688
- **Value Mismatches**: 855
- **Missing Fields**: 4,458

## Recent Fix: Value Reading

The Sony parser was recently fixed to actually read values from the IFD entries instead of returning placeholders. Now numeric values are correctly extracted (e.g., `LensID: 128` instead of `LensID: 0`).

## Top Value Mismatches by Frequency

### Category 1: Standard EXIF Formatting (Affects All Files)

| Field | Count | Issue | Fix Location |
|-------|-------|-------|--------------|
| CFAPattern | 31 | `[0,1,1,2]` should be `"[Red,Green][Green,Blue]"` | `src/output.rs` |
| CFARepeatPatternDim | 31 | `[2,2]` should be `"2 2"` | `src/output.rs` |
| PhotometricInterpretation | 31 | `32803` should be `"Color Filter Array"` | `src/output.rs` |
| PlanarConfiguration | 31 | `1` should be `"Chunky"` | `src/output.rs` |
| UserComment | 28 | Base64 of nulls should be empty string | `src/output.rs` |
| FocalLength | 25 | `"50 mm"` should be `"50.0 mm"` (with decimal) | `src/output.rs` |
| Flash | 22 | Different description wording | `src/tags.rs` |

### Category 2: Sony Maker Note Value Lookups (High Priority)

| Field | Count | Issue | Fix Location |
|-------|-------|-------|--------------|
| WhiteBalance | 31 | `80` should be `"Flash"` | `src/makernotes/sony.rs` |
| AFPointSelected | 31 | `0` should be `"Auto"` | `src/makernotes/sony.rs` |
| FocusMode | 31 | `0` should be `"Manual"` | `src/makernotes/sony.rs` |
| ExposureMode | 31 | `15` should be `"Manual"` | `src/makernotes/sony.rs` |
| DynamicRangeOptimizer | 31 | `3` should be `"Auto"` | `src/makernotes/sony.rs` |
| Quality | 31 | `65535` should be `"RAW"` | `src/makernotes/sony.rs` |
| Contrast | 31 | `0` should be `"Normal"` | `src/makernotes/sony.rs` |
| Saturation | 31 | `0` should be `"Normal"` | `src/makernotes/sony.rs` |
| Sharpness | 31 | `0` should be `"Normal"` | `src/makernotes/sony.rs` |
| SceneMode | 31 | `0` should be `"Standard"` | `src/makernotes/sony.rs` |
| ColorTemperature | 29 | `0` should be `"Auto"` | `src/makernotes/sony.rs` |
| PictureEffect | 28 | `0` should be `"Off"` | `src/makernotes/sony.rs` |
| HDR | 27 | `0` should be `"Off; Uncorrected image"` | `src/makernotes/sony.rs` |
| ReleaseMode | 27 | `0` should be `"Normal"` | `src/makernotes/sony.rs` |
| FlashLevel | 25 | `0` should be `"Normal"` | `src/makernotes/sony.rs` |
| HighISONoiseReduction | 25 | `0` should be `"Off"` | `src/makernotes/sony.rs` |
| ImageStabilization | 24 | `1` should be `"On"` | `src/makernotes/sony.rs` |
| LongExposureNoiseReduction | 23 | `1` should be `"On (unused)"` | `src/makernotes/sony.rs` |
| MultiFrameNoiseReduction | 22 | `0` should be `"Off"` | `src/makernotes/sony.rs` |
| VignettingCorrection | 22 | `2` should be `"Auto"` | `src/makernotes/sony.rs` |
| DistortionCorrection | 20 | `0` should be `"No correction params available"` | `src/makernotes/sony.rs` |
| IntelligentAuto | 18 | `0` should be `"Off"` | `src/makernotes/sony.rs` |
| Teleconverter | 15 | `0` should be `"None"` | `src/makernotes/sony.rs` |
| AntiBlur | 12 | Various values need lookup | `src/makernotes/sony.rs` |

### Category 3: Complex Lookups

| Field | Count | Issue | Fix Location |
|-------|-------|-------|--------------|
| SonyModelID | 31 | `294` should be `"SLT-A99 / SLT-A99V"` | `src/makernotes/sony.rs` |
| LensID | 28 | `128` should be `"Sigma 50mm F1.4 EX DG HSM"` | `src/makernotes/sony.rs` |
| LensSpec | 25 | Byte array should be formatted lens spec | `src/makernotes/sony.rs` |
| CreativeStyle | 20 | Empty string should be `"Standard"` | `src/makernotes/sony.rs` |
| FlashExposureComp | 15 | Wrong calculation formula | `src/makernotes/sony.rs` |

## Detailed Fix Proposals

### Fix 1: Basic Value Lookups

**File**: `src/makernotes/sony.rs`

Add these lookup functions:

```rust
/// Decode WhiteBalance value
pub fn decode_white_balance(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        1 => "Color Temperature/Color Filter",
        16 => "Daylight",
        32 => "Cloudy",
        48 => "Shade",
        64 => "Tungsten",
        80 => "Flash",
        96 => "Fluorescent",
        112 => "Custom",
        256 => "Underwater",
        _ => "Unknown",
    }
}

/// Decode FocusMode value
pub fn decode_focus_mode(value: u16) -> &'static str {
    match value {
        0 => "Manual",
        1 => "AF-S",
        2 => "AF-C",
        3 => "AF-A",
        4 => "DMF",
        _ => "Unknown",
    }
}

/// Decode ExposureMode value
pub fn decode_exposure_mode(value: u16) -> &'static str {
    match value {
        0 => "Auto",
        1 => "Portrait",
        2 => "Beach",
        3 => "Sports",
        4 => "Snow",
        5 => "Landscape",
        6 => "Program",
        7 => "Aperture Priority",
        8 => "Shutter Priority",
        9 => "Night Scene",
        15 => "Manual",
        _ => "Unknown",
    }
}

/// Decode Quality value
pub fn decode_quality(value: u16) -> &'static str {
    match value {
        0 => "Normal",
        1 => "Fine",
        2 => "Extra Fine",
        65535 => "RAW",
        _ => "Unknown",
    }
}

/// Decode DynamicRangeOptimizer value
pub fn decode_dro(value: u16) -> &'static str {
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
        _ => "Unknown",
    }
}

/// Decode Contrast/Saturation/Sharpness
pub fn decode_adjustment(value: i16) -> &'static str {
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

/// Decode SceneMode
pub fn decode_scene_mode(value: u16) -> &'static str {
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
        _ => "Unknown",
    }
}

/// Decode HDR value
pub fn decode_hdr(value: u32) -> String {
    let level = (value >> 8) & 0xFF;
    let ev = value & 0xFF;

    if level == 0 {
        if ev == 0 {
            "Off; Uncorrected image".to_string()
        } else {
            format!("Off; {}EV", ev)
        }
    } else {
        format!("HDR Lv{}; {}EV", level, ev)
    }
}

/// Decode ImageStabilization
pub fn decode_image_stabilization(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "On",
        2 => "On (Shooting Only)",
        _ => "Unknown",
    }
}

/// Decode AFPointSelected
pub fn decode_af_point_selected(value: u16) -> &'static str {
    match value {
        0 => "Auto",
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
```

### Fix 2: SonyModelID Lookup

**File**: `src/makernotes/sony.rs`

```rust
pub fn get_sony_model_name(model_id: u32) -> Option<&'static str> {
    match model_id {
        256 => Some("DSLR-A100"),
        257 => Some("DSLR-A900"),
        258 => Some("DSLR-A700"),
        259 => Some("DSLR-A200"),
        260 => Some("DSLR-A350"),
        261 => Some("DSLR-A300"),
        263 => Some("DSLR-A380"),
        264 => Some("DSLR-A330"),
        265 => Some("DSLR-A230"),
        266 => Some("DSLR-A290"),
        269 => Some("DSLR-A850"),
        270 => Some("DSLR-A550"),
        273 => Some("DSLR-A500"),
        274 => Some("DSLR-A450"),
        275 => Some("SLT-A33"),
        276 => Some("SLT-A55"),
        278 => Some("DSLR-A560"),
        279 => Some("DSLR-A580"),
        280 => Some("NEX-3"),
        281 => Some("NEX-5"),
        282 => Some("NEX-VG10"),
        283 => Some("SLT-A35"),
        284 => Some("SLT-A65"),
        285 => Some("SLT-A77"),
        286 => Some("NEX-C3"),
        287 => Some("NEX-F3"),
        288 => Some("SLT-A37"),
        289 => Some("SLT-A57"),
        290 => Some("NEX-5N"),
        291 => Some("NEX-7"),
        292 => Some("NEX-VG20"),
        293 => Some("SLT-A99"),
        294 => Some("SLT-A99 / SLT-A99V"),
        295 => Some("NEX-5R"),
        296 => Some("NEX-6"),
        297 => Some("DSC-RX100"),
        298 => Some("DSC-RX1"),
        299 => Some("NEX-VG900"),
        // ... add more as needed
        _ => None,
    }
}
```

### Fix 3: LensID Lookup (Sample)

**File**: `src/makernotes/sony.rs`

Sony lens IDs are complex and require a large lookup table. Here's a starter:

```rust
pub fn get_sony_lens_name(lens_id: u32) -> Option<&'static str> {
    match lens_id {
        0 => Some("Unknown"),
        1 => Some("Minolta AF 28mm F2.8"),
        2 => Some("Minolta AF 35mm F2"),
        // ... hundreds of entries
        128 => Some("Sigma 50mm F1.4 EX DG HSM"),
        // Sony/Minolta lens database is extensive
        _ => None,
    }
}
```

Note: The Sony lens database has 1000+ entries. Consider loading from external file or using a phf crate for compile-time hashing.

### Fix 4: Apply Lookups in Parser

**File**: `src/makernotes/sony.rs`

Modify `parse_ifd_entry` to apply lookups:

```rust
fn apply_sony_value_lookup(tag_id: u16, value: ExifValue) -> ExifValue {
    match tag_id {
        SONY_WHITE_BALANCE => {
            if let ExifValue::Short(v) = &value {
                if let Some(first) = v.first() {
                    return ExifValue::Ascii(decode_white_balance(*first).to_string());
                }
            }
            value
        }
        SONY_FOCUS_MODE => {
            if let ExifValue::Short(v) = &value {
                if let Some(first) = v.first() {
                    return ExifValue::Ascii(decode_focus_mode(*first).to_string());
                }
            }
            value
        }
        SONY_SONY_MODEL_ID => {
            if let ExifValue::Long(v) = &value {
                if let Some(first) = v.first() {
                    if let Some(name) = get_sony_model_name(*first) {
                        return ExifValue::Ascii(name.to_string());
                    }
                }
            }
            value
        }
        // ... more lookups
        _ => value,
    }
}
```

## Missing Fields Analysis

Top 20 most frequently missing fields:

| Field | Count | Category |
|-------|-------|----------|
| AFMicroAdj | 31 | AF structure |
| AFPoint | 31 | AF structure |
| AFPointAtShutterRelease | 31 | AF structure |
| AFPointsSelected | 31 | AF structure |
| AFStatusActiveSensor | 31 | AF structure |
| BitsPerSample | 31 | TIFF structure |
| BlackLevel | 31 | RAW processing |
| ColorMatrix | 31 | Color calibration |
| ColorTempKelvin | 31 | WB sub-structure |
| DefaultCropOrigin | 31 | DNG structure |
| DefaultCropSize | 31 | DNG structure |
| FNumber | 31 | Already in EXIF, duplicate |
| FileSize | 31 | File metadata |
| FocalLength35efl | 31 | Computed field |
| ISO | 31 | Already in EXIF, duplicate |
| ImageHeight | 31 | Already in EXIF |
| ImageWidth | 31 | Already in EXIF |
| LensInfo | 31 | Complex structure |
| LinearityUpperMargin | 31 | RAW calibration |
| MaxAperture | 31 | Lens structure |

## Implementation Priority

1. **Phase 1 (Quick Wins - 1-2 hours)**:
   - Add basic value lookup functions
   - Apply lookups to WhiteBalance, FocusMode, Quality, etc.
   - Expected: ~300 mismatches eliminated

2. **Phase 2 (Medium Effort - 2-3 hours)**:
   - Add SonyModelID lookup table (~50 common models)
   - Add basic LensID lookup (~100 common lenses)
   - Add CreativeStyle decoding
   - Expected: ~100 mismatches eliminated

3. **Phase 3 (Standard EXIF Fixes)**:
   - Fix CFAPattern formatting
   - Fix GPS formatting
   - Fix PhotometricInterpretation lookup
   - Expected: ~150 mismatches eliminated

4. **Phase 4 (Complex Structures)**:
   - Parse AF info structures
   - Parse color calibration data
   - Add computed fields

## Expected Impact

After implementing Phase 1 and 2:
- **Value lookup fixes**: ~400 value mismatches eliminated
- **Standard EXIF fixes**: ~150 value mismatches eliminated
- **Total reduction**: ~550 mismatches (64% of current 855 mismatches)

## Testing Strategy

After each phase:
```bash
FPEXIF_TEST_FILES=/fpexif/raws cargo test --test exiftool_json_comparison test_exiftool_json_compatibility_arw -- --nocapture
```

Monitor the reduction in value mismatches count.
