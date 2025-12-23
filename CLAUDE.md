we must always run the following before pushing our branches

`./bin/ccc`

don't allow dead code

don't run --release builds

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
