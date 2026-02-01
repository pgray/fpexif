# Tag Decoder Macro Usage Guide

The `define_tag_decoder!` macro simplifies creating tag decoder functions for both ExifTool and exiv2 formats.

## Basic Usage

### Before (Manual approach)

```rust
/// Decode WhiteBalance value (tag 0x0003) - ExifTool format
pub fn decode_white_balance_exiftool(value: u16) -> &'static str {
    match value {
        1 => "Auto",
        2 => "Daylight",
        3 => "Cloudy",
        4 => "Incandescent",
        5 => "Manual",
        8 => "Flash",
        10 => "Black & White",
        11 => "Shade",
        12 => "Kelvin",
        _ => "Unknown",
    }
}

/// Decode WhiteBalance value (tag 0x0003) - exiv2 format
pub fn decode_white_balance_exiv2(value: u16) -> &'static str {
    match value {
        1 => "Auto",
        2 => "Daylight",
        3 => "Cloudy",
        4 => "Halogen",
        5 => "Manual",
        8 => "Flash",
        10 => "Black and white",
        11 => "Manual",
        12 => "Shade",
        13 => "Kelvin",
        _ => "Unknown",
    }
}
```

### After (Using macro)

```rust
use crate::define_tag_decoder;

define_tag_decoder! {
    white_balance,
    exiftool: {
        1 => "Auto",
        2 => "Daylight",
        3 => "Cloudy",
        4 => "Incandescent",
        5 => "Manual",
        8 => "Flash",
        10 => "Black & White",
        11 => "Shade",
        12 => "Kelvin",
    },
    exiv2: {
        1 => "Auto",
        2 => "Daylight",
        3 => "Cloudy",
        4 => "Halogen",
        5 => "Manual",
        8 => "Flash",
        10 => "Black and white",
        11 => "Manual",
        12 => "Shade",
        13 => "Kelvin",
    }
}
```

## When formats are identical

For tags where ExifTool and exiv2 use the same mappings:

### Before

```rust
/// Decode FocusMode value (tag 0x0007)
/// Identical between ExifTool and exiv2
pub fn decode_focus_mode_exiftool(value: u16) -> &'static str {
    match value {
        1 => "Auto",
        2 => "Manual",
        4 => "Auto, Focus button",
        5 => "Auto, Continuous",
        6 => "AF-S",
        7 => "AF-C",
        8 => "AF-F",
        _ => "Unknown",
    }
}

pub fn decode_focus_mode_exiv2(value: u16) -> &'static str {
    match value {
        1 => "Auto",
        2 => "Manual",
        4 => "Auto, Focus button",
        5 => "Auto, Continuous",
        6 => "AF-S",
        7 => "AF-C",
        8 => "AF-F",
        _ => "Unknown",
    }
}
```

### After

```rust
use crate::define_tag_decoder;

define_tag_decoder! {
    focus_mode,
    both: {
        1 => "Auto",
        2 => "Manual",
        4 => "Auto, Focus button",
        5 => "Auto, Continuous",
        6 => "AF-S",
        7 => "AF-C",
        8 => "AF-F",
    }
}
```

## Benefits

1. **Reduces boilerplate**: ~30 lines → ~15 lines
2. **Ensures consistency**: Both functions are generated from the same definition
3. **Prevents errors**: No risk of forgetting to add a case to one function
4. **Better readability**: The mapping tables are more visually scannable
5. **Automatic documentation**: Doc comments are generated for both functions

## Migration Checklist

When converting existing decode functions to use the macro:

1. ✅ Import the macro: `use crate::define_tag_decoder;`
2. ✅ Compare the exiftool and exiv2 mappings
3. ✅ Use `both:` if they're identical, otherwise use `exiftool:` and `exiv2:`
4. ✅ Remove the old function definitions
5. ✅ Run tests to ensure behavior is preserved
6. ✅ Verify the generated functions work with existing code

## Real Example

Here's a complete example for Canon's FocusMode (CameraSettings array index 1):

```rust
// From ExifTool's Canon.pm:
// 0 => 'One-shot AF',
// 1 => 'AI Servo AF',
// 2 => 'AI Focus AF',
// etc.

// From exiv2's canonmn_int.cpp:
// {0, N_("One-shot")},
// {1, N_("AI Servo")},
// {2, N_("AI Focus")},
// etc.

define_tag_decoder! {
    canon_focus_mode,
    exiftool: {
        0 => "One-shot AF",
        1 => "AI Servo AF",
        2 => "AI Focus AF",
        3 => "Manual Focus",
        4 => "Single",
        5 => "Continuous",
        6 => "Manual Focus",
    },
    exiv2: {
        0 => "One-shot",
        1 => "AI Servo",
        2 => "AI Focus",
        3 => "Manual",
        4 => "Single",
        5 => "Continuous",
        6 => "Manual",
    }
}

// This generates:
// - decode_canon_focus_mode_exiftool(value: u16) -> &'static str
// - decode_canon_focus_mode_exiv2(value: u16) -> &'static str
```

## Notes

- The macro automatically adds an `_ => "Unknown"` case
- Function names follow the pattern: `decode_{name}_exiftool` and `decode_{name}_exiv2`
- The value type is always `u16` (can be extended if needed)
- The return type is always `&'static str`
