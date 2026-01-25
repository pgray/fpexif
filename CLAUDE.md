we must always run the following before pushing our branches

`./bin/ccc`

don't allow dead code

don't run --release builds

## Audio Notifications

Use `./bin/beep` to notify when tasks complete:

- `./bin/beep success` - Play ascending tones when a task completes successfully
- `./bin/beep failure` - Play descending tones when you need user input or something failed

Always beep when finishing a significant task or series of tasks.

## Reference Implementations

The following submodules contain reference implementations for EXIF parsing:

- `exiftool/` - ExifTool (Perl) - comprehensive metadata reader/writer
- `exiv2/` - Exiv2 (C++) - EXIF, IPTC, XMP metadata library

Use these as references for tag definitions, maker note structures, and parsing logic.

## MakerNote Sub-Agent Guide

When adding tags to manufacturer makernote modules (`src/makernotes/*.rs`), follow this pattern:

### Multiple Output Format Support

fpexif supports multiple output formats (`exiftool`, `exiv2`, etc.) with different value mappings.
When adding decode functions, create separate versions for each format:

```rust
// ExifTool mapping (from PrintConv in Canon.pm)
pub fn decode_focus_mode_exiftool(value: u16) -> &'static str {
    match value {
        0 => "One-Shot AF",
        1 => "AI Servo AF",
        // ...
    }
}

// exiv2 mapping (from canonFocusMode in canonmn_int.cpp)
pub fn decode_focus_mode_exiv2(value: u16) -> &'static str {
    match value {
        0 => "One-shot",
        1 => "AI Servo",
        // ...
    }
}
```

### Reference Locations

#### Implemented Manufacturers

| Manufacturer | fpexif Module                 | ExifTool                     | exiv2                 |
| ------------ | ----------------------------- | ---------------------------- | --------------------- |
| Canon        | `src/makernotes/canon.rs`     | `Canon.pm`, `CanonCustom.pm` | `canonmn_int.cpp`     |
| Nikon        | `src/makernotes/nikon.rs`     | `Nikon.pm`, `NikonCustom.pm` | `nikonmn_int.cpp`     |
| Sony         | `src/makernotes/sony.rs`      | `Sony.pm`                    | `sonymn_int.cpp`      |
| Fuji         | `src/makernotes/fuji.rs`      | `FujiFilm.pm`                | `fujimn_int.cpp`      |
| Panasonic    | `src/makernotes/panasonic.rs` | `Panasonic.pm`               | `panasonicmn_int.cpp` |
| Olympus      | `src/makernotes/olympus.rs`   | `Olympus.pm`                 | `olympusmn_int.cpp`   |

#### TODO Manufacturers

| Manufacturer | ExifTool                  | exiv2                 |
| ------------ | ------------------------- | --------------------- |
| Pentax       | `Pentax.pm`               | `pentaxmn_int.cpp`    |
| Samsung      | `Samsung.pm`              | `samsungmn_int.cpp`   |
| Sigma        | `Sigma.pm`, `SigmaRaw.pm` | `sigmamn_int.cpp`     |
| Minolta      | `Minolta.pm`              | `minoltamn_int.cpp`   |
| Casio        | `Casio.pm`                | `casiomn_int.cpp`     |
| Kodak        | `Kodak.pm`                | -                     |
| Leica        | `Panasonic.pm` (shared)   | `panasonicmn_int.cpp` |
| Ricoh        | `Ricoh.pm`                | -                     |
| Phase One    | `PhaseOne.pm`             | -                     |
| Hasselblad   | `Hasselblad.pm`           | -                     |
| Leaf         | `Leaf.pm`                 | -                     |

All ExifTool paths are under `exiftool/lib/Image/ExifTool/`.
All exiv2 paths are under `exiv2/src/`.

### Code Structure Pattern

Each makernote module follows this structure:

1. **Tag constants** - `pub const MANUFACTURER_TAG_NAME: u16 = 0xNNNN;`
2. **`get_*_tag_name(tag_id)`** - Returns human-readable tag name
3. **`decode_*(value)`** - Decodes enum values to strings (e.g., `decode_focus_mode`)
4. **`parse_*_maker_notes()`** - Main parser function

### Adding New Tags

1. Find tag in ExifTool (look for `%Image::ExifTool::Manufacturer::Main` and `PrintConv =>`)
2. Find same tag in exiv2 (look for `constexpr TagDetails`)
3. Add the tag constant at the top of the file
4. Add the tag name to `get_*_tag_name()` match arm
5. Create dual decode functions: `decode_*_exiftool()` and `decode_*_exiv2()`
6. Wire up decoding in the parse function

### How to Read ExifTool PrintConv

```perl
# In Canon.pm, look for patterns like:
0x7 => {
    Name => 'FocusMode',
    PrintConv => {
        0 => 'One-Shot AF',
        1 => 'AI Servo AF',
        2 => 'AI Focus AF',
    },
},
```

### How to Read exiv2 TagDetails

```cpp
// In canonmn_int.cpp, look for:
constexpr TagDetails canonFocusMode[] = {
    {0, N_("One-shot")},
    {1, N_("AI Servo")},
    {2, N_("AI Focus")},
};
```

### Macros for Tag Decoding (`src/macros.rs`)

Use these macros to reduce boilerplate when adding new tag decoders:

#### `define_tag_decoder!` - Generate dual exiftool/exiv2 decode functions

```rust
// When exiftool and exiv2 have DIFFERENT mappings:
define_tag_decoder! {
    white_balance,
    exiftool: {
        1 => "Auto",
        2 => "Daylight",
        4 => "Incandescent",
    },
    exiv2: {
        1 => "Auto",
        2 => "Daylight",
        4 => "Halogen",  // Different name in exiv2
    }
}

// When both formats use the SAME mappings:
define_tag_decoder! {
    focus_mode,
    both: {
        1 => "Auto",
        2 => "Manual",
        4 => "Auto, Focus button",
    }
}

// With explicit type (default is u16):
define_tag_decoder! {
    adjustment,
    type: i32,
    both: {
        -2 => "-2",
        -1 => "-1",
        0 => "0",
        1 => "+1",
    }
}
```

This generates `decode_white_balance_exiftool()` and `decode_white_balance_exiv2()` functions.

#### `decode_field!` - Extract fields from binary arrays

Use in decode functions that process `&[u16]` arrays (like CameraSettings, ShotInfo):

```rust
pub fn decode_camera_settings(data: &[u16]) -> HashMap<String, ExifValue> {
    let mut decoded = HashMap::new();

    // Simple decode - call decoder function on data[index]
    decode_field!(decoded, data, 7, "FocusMode", decode_focus_mode_exiftool);

    // With skip condition - skip if value equals skip_value
    decode_field!(decoded, data, 19, "AFPoint", decode_af_point_exiftool, skip_if: 0);

    // Raw numeric output (no decoder function)
    decode_field!(decoded, data, 5, "RawValue", raw_u16);
    decode_field!(decoded, data, 6, "SignedValue", raw_i16);

    // With i16 cast before decoding (for signed values stored as u16)
    decode_field!(decoded, data, 9, "Contrast", decode_contrast_exiftool, cast: i16);

    decoded
}
```

#### `decode_picture_styles!` - Canon PictureStyle fields

For Canon ColorData PictureStyleInfo sections (9 styles × 4 fields each):

```rust
decode_picture_styles!(decoded, data,
    0x00 => "Standard",
    0x06 => "Portrait",
    0x0c => "Landscape",
    // ... generates ContrastStandard, SharpnessStandard, SaturationStandard, ColorToneStandard, etc.
);
```

### Decode Function Naming Convention

All value-to-string decode functions MUST have format suffixes:

```rust
// In src/makernotes/sony.rs

/// ExifTool mapping - from Sony.pm PrintConv
pub fn decode_focus_mode_exiftool(value: u16) -> &'static str {
    match value {
        0 => "Manual",
        1 => "AF-S",
        2 => "AF-C",
        3 => "AF-A",
        _ => "Unknown",
    }
}

/// exiv2 mapping - from sonymn_int.cpp sonyFocusMode2
pub fn decode_focus_mode_exiv2(value: u16) -> &'static str {
    match value {
        0 => "Manual",
        2 => "AF-S",
        3 => "AF-C",
        4 => "AF-A",
        6 => "DMF",
        _ => "Unknown",
    }
}
```

### What to Look For in Each Reference

#### ExifTool (.pm files)

| Pattern                         | Maps to                          |
| ------------------------------- | -------------------------------- |
| `PrintConv => { 0 => 'Value' }` | `decode_*_exiftool()` match arms |
| `%manufacturerLensTypes`        | `get_*_lens_name()`              |
| `%manufacturerModelID`          | `get_*_model_name()`             |
| `Name => 'TagName'`             | Tag constant name                |

#### exiv2 (\*mn_int.cpp files)

| Pattern                        | Maps to                                   |
| ------------------------------ | ----------------------------------------- |
| `constexpr TagDetails name[]`  | `decode_*_exiv2()` match arms             |
| `TagInfo(0xNNNN, "Name", ...)` | Tag constant and name                     |
| Lens/model arrays              | `get_*_lens_name()`, `get_*_model_name()` |

### Tags to Implement for Each Manufacturer

These are the common tags every manufacturer module should decode:

| Tag             | ExifTool pattern                   | exiv2 pattern     |
| --------------- | ---------------------------------- | ----------------- |
| FocusMode       | `PrintConv => { 0 => 'One-Shot' }` | `*FocusMode[]`    |
| WhiteBalance    | `PrintConv => { 0 => 'Auto' }`     | `*WhiteBalance[]` |
| ExposureMode    | `PrintConv`                        | `*ExposureMode[]` |
| MeteringMode    | `PrintConv`                        | `*MeteringMode[]` |
| ImageQuality    | `PrintConv`                        | `*Quality[]`      |
| Sharpness       | `PrintConv`                        | `*Sharpness[]`    |
| Saturation      | `PrintConv`                        | `*Saturation[]`   |
| Contrast        | `PrintConv`                        | `*Contrast[]`     |
| LensType/LensID | `%*LensTypes`                      | `*LensType[]`     |
| ModelID         | `%*ModelID`                        | `*ModelId[]`      |

json files in ./test-data are `exiftool -j` output saved with Binary fields removed

## Regression Prevention

Use `./bin/mfr-test` to track progress and prevent regressions when working on MakerNote parsing:

```bash
# Before starting work on a manufacturer, save baseline
./bin/mfr-test <manufacturer> --save-baseline

# After making changes, check for regressions
./bin/mfr-test <manufacturer> --check

# View current match rate without baseline comparison
./bin/mfr-test <manufacturer>

# List all saved baselines
./bin/mfr-test --list-baselines
```

Supported manufacturers: `canon`, `nikon`, `sony`, `fujifilm`, `panasonic`, `olympus`, `pentax`, `minolta`, `kodak`, `sigma`, `samsung`, `ricoh`, `leica`, `dng`

### Test Data Directories

There are two test data directories:

| Directory | Flag | Description |
|-----------|------|-------------|
| `/fpexif/raws/` | (default) | Small curated set of RAW files for quick testing |
| `/fpexif/data.lfs/` | `--data-lfs` | Large dataset with more camera models and edge cases |

```bash
# Test against default /fpexif/raws directory
./bin/mfr-test fujifilm

# Test against larger /fpexif/data.lfs directory
./bin/mfr-test fujifilm --data-lfs

# Verbose output shows per-file details and specific mismatches
./bin/mfr-test fujifilm --data-lfs --verbose
```

Always test against both directories when making significant changes. The `data.lfs` directory often reveals edge cases not present in the smaller `raws` set.

The `--check` flag shows:
- **Matching/Mismatched/Missing/Extra** counts with delta from baseline
- **IMPROVEMENTS** - tags that now match that didn't before
- **REGRESSIONS** - tags that stopped matching (investigate before committing)

**Workflow:**
1. Save baseline before starting: `./bin/mfr-test olympus --save-baseline`
2. Make changes to the makernote parser
3. Check for regressions: `./bin/mfr-test olympus --check`
4. Test against larger dataset: `./bin/mfr-test olympus --data-lfs`
5. If regressions appear, investigate and fix before committing
6. Run `./bin/ccc` before pushing
