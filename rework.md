# Makernote Decode Function Rework Plan

## Goal

Refactor all decode functions in makernote modules to support dual output formats:
- `decode_*_exiftool()` - values from ExifTool PrintConv
- `decode_*_exiv2()` - values from exiv2 TagDetails

## Current State

| Module | Decode Functions | Match Arms | Status |
|--------|-----------------|------------|--------|
| canon.rs | 5 | 440 | ✅ DONE |
| fuji.rs | 15 | 109 | needs rework |
| nikon.rs | 7 | 38 | needs rework |
| panasonic.rs | 16 | 135 | needs rework |
| sony.rs | 22 | 467 | needs rework |
| olympus.rs | 0 | - | needs both versions |

**Total**: 65 functions, ~1189 match arms, ~2000-2500 new lines

## Rework Steps Per Module

### 1. Rename existing functions
```rust
// Before
fn decode_focus_mode(value: u16) -> &'static str

// After
pub fn decode_focus_mode_exiftool(value: u16) -> &'static str
```

### 2. Create exiv2 versions
Look up values in `exiv2/src/*mn_int.cpp`:
```rust
pub fn decode_focus_mode_exiv2(value: u16) -> &'static str {
    match value {
        // Values from constexpr TagDetails in exiv2
    }
}
```

### 3. Update callers
Update `parse_*_maker_notes()` to use the appropriate version (default to exiftool for now).

## Module-Specific References

### Canon ✅ DONE
- ExifTool: `exiftool/lib/Image/ExifTool/Canon.pm`, `CanonCustom.pm`
- exiv2: `exiv2/src/canonmn_int.cpp`
- **Completed**: 15 individual decode functions with `_exiftool` suffix, 11 with `_exiv2` versions
- **Pattern used**:
  - Created individual `decode_*_exiftool()` functions for each field
  - Only created `_exiv2` versions where values actually differ
  - Container functions (`decode_camera_settings`, etc.) delegate to `_exiftool` version by default
  - Added `_exiv2` container functions that call the `_exiv2` decode functions
- **Fields with different exiv2 values**: MacroMode, Quality, FlashMode, DriveMode, FocusMode,
  MeteringMode, FocusRange, ExposureMode, WhiteBalance, AFAreaMode
- **Fields with identical values (no _exiv2 needed)**: ImageStabilization, ManualFlashOutput,
  FocalType, BracketMode, DateStampMode

### Fuji
- ExifTool: `exiftool/lib/Image/ExifTool/FujiFilm.pm`
- exiv2: `exiv2/src/fujimn_int.cpp`
- Functions to rework:
  - `decode_film_mode`
  - `decode_dynamic_range`
  - `decode_white_balance`
  - `decode_sharpness`
  - `decode_saturation`
  - `decode_macro`
  - `decode_focus_mode`
  - `decode_af_mode`
  - `decode_slow_sync`
  - `decode_auto_bracketing`
  - `decode_blur_warning`
  - `decode_focus_warning`
  - `decode_exposure_warning`
  - `decode_picture_mode`
  - `decode_dynamic_range_setting`

### Nikon
- ExifTool: `exiftool/lib/Image/ExifTool/Nikon.pm`, `NikonCustom.pm`
- exiv2: `exiv2/src/nikonmn_int.cpp`
- Functions to rework:
  - `decode_nikon_ascii_value`
  - `decode_active_d_lighting`
  - `decode_color_space`
  - `decode_vignette_control`
  - `decode_high_iso_noise_reduction`
  - `decode_date_stamp_mode`
  - `decode_lens_type`

### Panasonic
- ExifTool: `exiftool/lib/Image/ExifTool/Panasonic.pm`
- exiv2: `exiv2/src/panasonicmn_int.cpp`
- Functions to rework:
  - `decode_image_quality`
  - `decode_white_balance`
  - `decode_focus_mode`
  - `decode_af_area_mode`
  - `decode_image_stabilization`
  - `decode_macro_mode`
  - `decode_shooting_mode`
  - `decode_photo_style`
  - `decode_shutter_type`
  - `decode_contrast_mode`
  - `decode_burst_mode`
  - `decode_intelligent_resolution`
  - `decode_clear_retouch`
  - `decode_touch_ae`
  - `decode_flash_curtain`
  - `decode_hdr_shot`

### Sony
- ExifTool: `exiftool/lib/Image/ExifTool/Sony.pm`
- exiv2: `exiv2/src/sonymn_int.cpp`
- Functions to rework:
  - `decode_creative_style`
  - `decode_exposure_mode`
  - `decode_af_mode`
  - `decode_dynamic_range_optimizer`
  - `decode_focus_mode`
  - `decode_image_stabilization`
  - `decode_image_quality`
  - `decode_white_balance`
  - `decode_long_exposure_noise_reduction`
  - `decode_high_iso_noise_reduction`
  - `decode_scene_mode`
  - `decode_adjustment`
  - `decode_color_temperature`
  - `decode_teleconverter`
  - `decode_picture_effect`
  - `decode_vignetting_correction`
  - `decode_distortion_correction`
  - `decode_release_mode`
  - `decode_multi_frame_noise_reduction`
  - `decode_intelligent_auto`
  - `decode_hdr`
  - `decode_quality`

### Olympus
- ExifTool: `exiftool/lib/Image/ExifTool/Olympus.pm`
- exiv2: `exiv2/src/olympusmn_int.cpp`
- Currently has no decode functions - needs full implementation

## Execution Options

### Option A: All at once (parallel sub-agents)
- Launch 6 sub-agents simultaneously (one per manufacturer)
- Fastest completion
- Higher risk of merge conflicts in shared code

### Option B: Incremental
- Rework one manufacturer at a time
- Lower risk
- Can validate each before moving on

### Option C: On-demand
- Only add `_exiv2` versions when output.rs needs them
- Minimal initial work
- Technical debt accumulates

## Output Module Changes

After rework, `src/output.rs` needs:

```rust
pub enum OutputFormat {
    ExifTool,
    Exiv2,
}

// Then in formatting functions, select appropriate decoder:
match format {
    OutputFormat::ExifTool => decode_focus_mode_exiftool(value),
    OutputFormat::Exiv2 => decode_focus_mode_exiv2(value),
}
```

## Validation

After each module rework:
1. `cargo check` - compiles
2. `cargo test` - tests pass
3. `./bin/ccc` - clippy/fmt clean
4. Compare output with actual exiftool/exiv2 on sample images

## Lessons Learned (from Canon rework)

### Pattern for container functions
```rust
// Default function delegates to exiftool
pub fn decode_camera_settings(data: &[u16]) -> HashMap<String, ExifValue> {
    decode_camera_settings_exiftool(data)
}

// ExifTool version calls individual decode functions
pub fn decode_camera_settings_exiftool(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();
    if data.len() > 1 {
        decoded.insert(
            "MacroMode".to_string(),
            ExifValue::Ascii(decode_macro_mode_exiftool(data[1]).to_string()),
        );
    }
    // ... more fields
    decoded
}

// exiv2 version - only create if at least one field differs
pub fn decode_camera_settings_exiv2(data: &[u16]) -> HashMap<String, ExifValue> {
    // Same structure, calls _exiv2 decode functions where they exist
    // Falls back to _exiftool for fields that are identical
}
```

### When NOT to create _exiv2 version
- If all values in the match are identical between ExifTool and exiv2
- Add a comment: `// decode_foo_exiv2 - same as exiftool, no separate function needed`

### Finding exiv2 values
- Search for `constexpr TagDetails canon*` in `exiv2/src/canonmn_int.cpp`
- Values are in format: `{0, N_("Off")}, {1, N_("On")}`
- `N_()` is a gettext macro, ignore it - just use the string

### Common differences between ExifTool and exiv2
- Capitalization: "One-shot AF" vs "One shot AF"
- Wording: "Flash Not Fired" vs "Off"
- Additional values in one or the other
- Value numbering shifts (especially in "focus type" style fields)
