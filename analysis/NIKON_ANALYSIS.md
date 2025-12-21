# Nikon NEF Maker Notes Analysis and Fix Proposal

## Implementation Status

The following items from this analysis have been implemented:

### Completed
- **ColorSpace decoder** (0x001E) - Decodes 1=sRGB, 2=Adobe RGB
- **VignetteControl decoder** (0x002A) - Decodes 0=Off, 1=Low, 2=Normal, 3=High
- **DateStampMode decoder** (0x009D) - Decodes 0=Off, 1=Date & Time, 2=Date, 3=Date Counter
- **SerialNumber parsing** (0x001D) - Reads serial from ASCII string
- **ShutterCount parsing** (0x00A7) - Via encrypted ShotInfo (0x0091) structure
- **ShotInfo decryption** - Nikon cipher with XLAT lookup tables implemented
- **Lens database** (~90 entries) - Composite 8-byte hex key lookup
- **New tag constants**: NIKON_DATE_STAMP_MODE, NIKON_SERIAL_NUMBER_2, NIKON_IMAGE_DATA_SIZE, NIKON_IMAGE_COUNT, NIKON_DELETED_IMAGE_COUNT, NIKON_SHUTTER_COUNT

### Pending
- Core EXIF formatting fixes (CFAPattern, PhotometricInterpretation, etc.)
- LensData (0x0083) - encrypted structure for detailed lens info
- FlashInfo (0x00A8) - flash-related metadata
- Complex nested structures with version-specific parsing

---

## Executive Summary

The current Nikon maker notes implementation is a stub that doesn't parse actual tag values. This analysis identifies the most impactful fixes needed to match ExifTool's output for 47 NEF test files.

## Test Results Overview

- **Files Tested**: 47 NEF files
- **Files Passing**: 0
- **Total Issues**: 359,199
- **Value Mismatches**: 898
- **Missing Fields**: 6,559

## Top Value Mismatches by Frequency

### Category 1: Standard EXIF Formatting Issues (Non-Nikon)

These affect all 47 files and are in the core EXIF parsing, not Nikon maker notes:

| Field | Count | Issue | Fix Location |
|-------|-------|-------|--------------|
| CFAPattern | 47 | Array `[0,1,1,2]` should be `"[Red,Green][Green,Blue]"` | `src/output.rs` |
| CFARepeatPatternDim | 47 | Array `[2,2]` should be `"2 2"` | `src/output.rs` |
| PhotometricInterpretation | 47 | Number `32803` should be `"Color Filter Array"` | `src/tags.rs` + `src/output.rs` |
| PlanarConfiguration | 47 | Number `1` should be `"Chunky"` | `src/tags.rs` + `src/output.rs` |
| ReferenceBlackWhite | 47 | Array should be space-separated string | `src/output.rs` |
| GainControl | 44 | Number `1` should be `"Low gain up"` | Already has function, check usage |
| FocalLength | 44 | `"10.0 mm"` should be `"10 mm"` (no decimal for integers) | `src/output.rs` line 96 |
| UserComment | 42 | Base64 of null bytes should be empty string | `src/output.rs` format_undefined_value |
| MeteringMode | 37 | `"Evaluative"` should be `"Multi-segment"` for Nikon | `src/tags.rs` get_metering_mode_description |
| SubSecTime/Digitized/Original | 34 | `"24"` should be `24` (remove quotes) | `src/output.rs` |
| SensingMethod | 28 | Number `2` should be `"One-chip color area"` | Already has function, check usage |
| Flash | 27 | `"Flash did not fire"` vs `"No Flash"` (synonym issue) | `src/tags.rs` |
| LightSource | 22 | `"Unknown"` should be `"Natural"` for some values | `src/tags.rs` |
| Compression | 21 | Should recognize Nikon NEF compression | `src/tags.rs` |

### Category 2: Nikon Maker Note Missing Value Parsing

These require actual implementation in `src/makernotes/nikon.rs`:

| Tag ID | Name | Count | Expected Value Example | Current Value |
|--------|------|-------|----------------------|---------------|
| 0x0002 | ISO | 36 | `"720"` | `""` (empty) |
| 0x0004 | Quality | 36 | `"RAW"` | `""` (empty) |
| 0x0005 | WhiteBalance | 41 | `"Auto"` | `""` (empty) |
| 0x0007 | FocusMode | 31 | `"AF-C"`, `"AF-S"` | `""` (empty) |
| 0x0006 | Sharpness | 25 | `"Soft"`, `"Normal"` | `""` (empty) |
| 0x0008 | FlashSetting | 8 | `"Normal"` | `""` (empty) |
| 0x001D | SerialNumber | 15 | `"51001710"` | `""` (empty) |
| 0x001E | ColorSpace | 20 | `"Adobe RGB"`, `"sRGB"` | `""` (empty) |
| 0x002A | VignetteControl | 20 | `"Normal"`, `"Off"` | `""` (empty) |
| 0x0098 | LensType | 3 | `"G VR"` | `""` (empty) |
| 0x0099 | Lens | 8 | `"11-27.5mm f/3.5-5.6"` | `""` (empty) |

## Detailed Fix Proposals

### Fix 1: Core EXIF Output Formatting (Highest Impact)

**File**: `src/output.rs`

**Changes**:

1. **CFAPattern** - Add special formatting for tag 0x828E:
```rust
// In format_exif_value_for_json, handle CFAPattern specially
0x828E => {
    // CFAPattern - format as "[Red,Green][Green,Blue]" style
    if let ExifValue::Byte(v) = value {
        if v.len() == 4 {
            let color_names = |c: u8| match c {
                0 => "Red",
                1 => "Green",
                2 => "Blue",
                3 => "Cyan",
                4 => "Magenta",
                5 => "Yellow",
                6 => "White",
                _ => "Unknown",
            };
            return Value::String(format!(
                "[{},{}][{},{}]",
                color_names(v[0]),
                color_names(v[1]),
                color_names(v[2]),
                color_names(v[3])
            ));
        }
    }
}
```

2. **CFARepeatPatternDim** - Add to `should_format_as_space_separated`:
```rust
fn should_format_as_space_separated(tag_id: u16) -> bool {
    matches!(
        tag_id,
        0x0102 // BitsPerSample
        | 0x0013 // ThumbnailImageValidArea
        | 0x828D // CFARepeatPatternDim - NEW
        | 0x0214 // ReferenceBlackWhite - NEW
    )
}
```

3. **PhotometricInterpretation** - Add description function in `src/tags.rs`:
```rust
pub fn get_photometric_interpretation_description(value: u16) -> &'static str {
    match value {
        0 => "WhiteIsZero",
        1 => "BlackIsZero",
        2 => "RGB",
        3 => "RGB Palette",
        4 => "Transparency Mask",
        5 => "CMYK",
        6 => "YCbCr",
        8 => "CIELab",
        9 => "ICCLab",
        10 => "ITULab",
        32803 => "Color Filter Array",
        34892 => "Linear Raw",
        _ => "Unknown",
    }
}
```

Then use in `src/output.rs` format_short_value:
```rust
0x0106 => Value::String(crate::tags::get_photometric_interpretation_description(value).to_string()),
```

4. **PlanarConfiguration** - Add description function:
```rust
pub fn get_planar_configuration_description(value: u16) -> &'static str {
    match value {
        1 => "Chunky",
        2 => "Planar",
        _ => "Unknown",
    }
}
```

5. **FocalLength** - Fix decimal formatting (line 96):
```rust
0x920A => {
    // FocalLength - add mm unit, no decimal if integer
    let focal_length = num as f64 / den as f64;
    if focal_length.fract() == 0.0 {
        Value::String(format!("{} mm", focal_length as i64))
    } else {
        Value::String(format!("{} mm", focal_length))
    }
}
```

6. **UserComment** - Fix format_undefined_value to detect ASCII null padding:
```rust
fn format_undefined_value(data: &[u8], tag_id: u16) -> Value {
    match tag_id {
        0x9286 => {
            // UserComment - check for ASCII charset marker
            if data.len() >= 8 && &data[0..8] == b"ASCII\0\0\0" {
                // ASCII charset - skip header and check if rest is just nulls/spaces
                let content = &data[8..];
                let cleaned = String::from_utf8_lossy(content)
                    .trim_end_matches('\0')
                    .trim()
                    .to_string();
                if cleaned.is_empty() {
                    return Value::String(String::new());
                } else {
                    return Value::String(cleaned);
                }
            }
            // Otherwise use base64 as before
            Value::String(base64_encode(data))
        }
        // ... rest of function
    }
}
```

7. **SubSecTime** - These are being quoted because they're stored as ASCII. Fix in format_exif_value_for_json:
```rust
ExifValue::Ascii(s) => {
    let cleaned = s.trim_end_matches('\0').trim();
    // SubSecTime tags should be plain numbers, not quoted
    if matches!(tag_id, 0x9290 | 0x9291 | 0x9292) {
        // Try to parse as number
        if let Ok(num) = cleaned.parse::<u32>() {
            return Value::Number(num.into());
        }
    }
    Value::String(cleaned.to_string())
}
```

### Fix 2: Nikon Maker Notes Value Parsing (High Impact)

**File**: `src/makernotes/nikon.rs`

**Current Problem**: The parser reads the IFD structure but doesn't actually extract the value data. It creates placeholder values.

**Solution**: Parse actual values from the maker note data. Here's the complete rewrite:

```rust
use crate::data_types::{Endianness, ExifValue};
use crate::errors::ExifError;
use crate::makernotes::MakerNoteTag;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::collections::HashMap;
use std::io::{Cursor, Seek, SeekFrom};

// ... keep all the tag constants ...

/// Get human-readable value for Nikon Quality tag (0x0004)
fn decode_quality(data: &str) -> &'static str {
    match data.trim() {
        "FINE" => "Fine",
        "NORMAL" => "Normal",
        "BASIC" => "Basic",
        "RAW" => "RAW",
        "RAW+FINE" => "RAW + Fine",
        "RAW+NORMAL" => "RAW + Normal",
        "RAW+BASIC" => "RAW + Basic",
        s if s.contains("Raw") => s,
        _ => data.trim(),
    }
}

/// Get human-readable value for Nikon WhiteBalance tag (0x0005)
fn decode_white_balance(data: &str) -> &'static str {
    match data.trim() {
        "AUTO" => "Auto",
        "SUNNY" | "DIRECT SUNLIGHT" => "Daylight",
        "SHADE" => "Shade",
        "CLOUDY" => "Cloudy",
        "TUNGSTEN" | "INCANDESCENT" => "Tungsten",
        "FLUORESCENT" => "Fluorescent",
        "FLASH" => "Flash",
        "CLOUDY2" => "Cloudy",
        "PRESET" | "MANUAL" => "Preset",
        _ => data.trim(),
    }
}

/// Get human-readable value for Nikon FocusMode tag (0x0007)
fn decode_focus_mode(data: &str) -> &'static str {
    match data.trim() {
        "AF-S" => "AF-S",
        "AF-C" => "AF-C",
        "AF-A" => "AF-A",
        "MF" => "Manual",
        _ => data.trim(),
    }
}

/// Get human-readable value for Nikon Sharpness tag (0x0006)
fn decode_sharpness(data: &str) -> &'static str {
    match data.trim() {
        "AUTO" => "Auto",
        "NORMAL" => "Normal",
        "LOW" => "Soft",
        "MED.L" => "Medium Soft",
        "MED.H" => "Medium Hard",
        "HIGH" => "Hard",
        "NONE" => "None",
        "" => "",
        _ => data.trim(),
    }
}

/// Get human-readable value for Nikon FlashSetting tag (0x0008)
fn decode_flash_setting(data: &str) -> &'static str {
    match data.trim() {
        "NORMAL" => "Normal",
        "RED-EYE" => "Red-eye Reduction",
        "SLOW" => "Slow",
        "REAR" => "Rear Curtain",
        "RED-EYE+SLOW" => "Red-eye + Slow",
        "" => "",
        _ => data.trim(),
    }
}

/// Get human-readable value for Nikon ColorMode tag (0x0003)
fn decode_color_mode(data: &str) -> &'static str {
    match data.trim() {
        "COLOR" => "Color",
        "B&W" | "BLACK & WHITE" => "Black & White",
        "" => "",
        _ => data.trim(),
    }
}

/// Helper to read value data from maker note
fn read_value_data<'a>(
    data: &'a [u8],
    offset: usize,
    tag_type: u16,
    count: u32,
    value_offset_field: u32,
    endian: Endianness,
    tiff_base: usize,
) -> Option<&'a [u8]> {
    // Calculate size of value
    let type_size = match tag_type {
        1 | 2 | 6 | 7 => 1,  // BYTE, ASCII, SBYTE, UNDEFINED
        3 | 8 => 2,           // SHORT, SSHORT
        4 | 9 => 4,           // LONG, SLONG
        5 | 10 => 8,          // RATIONAL, SRATIONAL
        _ => return None,
    };

    let value_size = type_size * count as usize;

    if value_size <= 4 {
        // Value is stored inline in the value_offset field
        // We need to extract it from the bytes at current cursor position - 4
        // This is already in value_offset_field as raw bytes
        None // Will handle inline below
    } else {
        // Value is stored at offset
        let val_offset = value_offset_field as usize;
        // Offset is relative to TIFF base (start of maker note TIFF header)
        if tiff_base + val_offset + value_size <= data.len() {
            Some(&data[tiff_base + val_offset..tiff_base + val_offset + value_size])
        } else {
            None
        }
    }
}

/// Parse Nikon maker notes
pub fn parse_nikon_maker_notes(
    data: &[u8],
    endian: Endianness,
) -> Result<HashMap<u16, MakerNoteTag>, ExifError> {
    let mut tags = HashMap::new();

    if data.len() < 10 {
        return Ok(tags);
    }

    let mut tiff_base = 0;

    // Check for Nikon header
    if data.starts_with(b"Nikon\0") {
        tiff_base = 10; // Skip "Nikon\0" + TIFF header (II or MM + 0x2A00)
    }

    if tiff_base >= data.len() {
        return Ok(tags);
    }

    let mut cursor = Cursor::new(&data[tiff_base..]);

    // Read number of entries
    let num_entries = match endian {
        Endianness::Little => cursor.read_u16::<LittleEndian>(),
        Endianness::Big => cursor.read_u16::<BigEndian>(),
    }
    .map_err(|_| ExifError::Format("Failed to read Nikon maker note count".to_string()))?;

    // Parse IFD entries
    for _ in 0..num_entries {
        let entry_pos = cursor.position() as usize;

        if entry_pos + 12 > data[tiff_base..].len() {
            break;
        }

        let tag_id = match endian {
            Endianness::Little => cursor.read_u16::<LittleEndian>(),
            Endianness::Big => cursor.read_u16::<BigEndian>(),
        }
        .ok();

        let tag_type = match endian {
            Endianness::Little => cursor.read_u16::<LittleEndian>(),
            Endianness::Big => cursor.read_u16::<BigEndian>(),
        }
        .ok();

        let count = match endian {
            Endianness::Little => cursor.read_u32::<LittleEndian>(),
            Endianness::Big => cursor.read_u32::<BigEndian>(),
        }
        .ok();

        // Read the value/offset field (4 bytes)
        let value_offset_bytes_pos = cursor.position();
        let value_offset = match endian {
            Endianness::Little => cursor.read_u32::<LittleEndian>(),
            Endianness::Big => cursor.read_u32::<BigEndian>(),
        }
        .ok();

        if let (Some(tag_id), Some(tag_type), Some(count), Some(value_offset)) =
            (tag_id, tag_type, count, value_offset)
        {
            // Determine value size
            let type_size = match tag_type {
                1 | 2 | 6 | 7 => 1,
                3 | 8 => 2,
                4 | 9 => 4,
                5 | 10 => 8,
                _ => 1,
            };
            let value_size = type_size * count as usize;

            // Get the actual value bytes
            let value_bytes = if value_size <= 4 {
                // Inline value - get from the value_offset field itself
                let start = tiff_base + value_offset_bytes_pos as usize;
                &data[start..start + 4]
            } else {
                // Offset to value
                let offset = value_offset as usize;
                if tiff_base + offset + value_size <= data.len() {
                    &data[tiff_base + offset..tiff_base + offset + value_size]
                } else {
                    continue;
                }
            };

            // Parse the value based on tag ID and type
            let value = match tag_id {
                NIKON_ISO_SETTING if tag_type == 3 && count >= 2 => {
                    // ISO - read as two shorts
                    let iso1 = match endian {
                        Endianness::Little => u16::from_le_bytes([value_bytes[0], value_bytes[1]]),
                        Endianness::Big => u16::from_be_bytes([value_bytes[0], value_bytes[1]]),
                    };
                    let iso2 = match endian {
                        Endianness::Little => u16::from_le_bytes([value_bytes[2], value_bytes[3]]),
                        Endianness::Big => u16::from_be_bytes([value_bytes[2], value_bytes[3]]),
                    };
                    // Use second value if non-zero, else first
                    let iso = if iso2 > 0 { iso2 } else { iso1 };
                    if iso > 0 {
                        ExifValue::Ascii(iso.to_string())
                    } else {
                        ExifValue::Ascii(String::from("0"))
                    }
                }
                NIKON_COLOR_MODE | NIKON_QUALITY | NIKON_WHITE_BALANCE | NIKON_SHARPNESS
                | NIKON_FOCUS_MODE | NIKON_FLASH_SETTING | NIKON_FLASH_TYPE
                    if tag_type == 2 =>
                {
                    // ASCII string value
                    let s = std::str::from_utf8(&value_bytes[..count as usize])
                        .unwrap_or("")
                        .trim_end_matches('\0')
                        .to_string();

                    let decoded = match tag_id {
                        NIKON_QUALITY => decode_quality(&s),
                        NIKON_WHITE_BALANCE => decode_white_balance(&s),
                        NIKON_FOCUS_MODE => decode_focus_mode(&s),
                        NIKON_SHARPNESS => decode_sharpness(&s),
                        NIKON_FLASH_SETTING => decode_flash_setting(&s),
                        NIKON_COLOR_MODE => decode_color_mode(&s),
                        _ => &s,
                    };
                    ExifValue::Ascii(decoded.to_string())
                }
                NIKON_SERIAL_NUMBER if tag_type == 2 => {
                    // Serial number - ASCII string
                    let s = std::str::from_utf8(&value_bytes[..count as usize])
                        .unwrap_or("")
                        .trim_end_matches('\0')
                        .trim()
                        .to_string();
                    ExifValue::Ascii(s)
                }
                NIKON_COLOR_SPACE if tag_type == 3 => {
                    // ColorSpace - SHORT
                    let val = match endian {
                        Endianness::Little => u16::from_le_bytes([value_bytes[0], value_bytes[1]]),
                        Endianness::Big => u16::from_be_bytes([value_bytes[0], value_bytes[1]]),
                    };
                    let color_space = match val {
                        1 => "sRGB",
                        2 => "Adobe RGB",
                        _ => "",
                    };
                    ExifValue::Ascii(color_space.to_string())
                }
                NIKON_VIGNETTE_CONTROL if tag_type == 3 => {
                    // VignetteControl - SHORT
                    let val = match endian {
                        Endianness::Little => u16::from_le_bytes([value_bytes[0], value_bytes[1]]),
                        Endianness::Big => u16::from_be_bytes([value_bytes[0], value_bytes[1]]),
                    };
                    let vignette = match val {
                        0 => "Off",
                        1 => "Low",
                        2 => "Normal",
                        3 => "High",
                        _ => "",
                    };
                    ExifValue::Ascii(vignette.to_string())
                }
                NIKON_LENS if tag_type == 3 && count >= 2 => {
                    // Lens - two shorts: min focal length, max focal length in mm
                    let min_fl = match endian {
                        Endianness::Little => u16::from_le_bytes([value_bytes[0], value_bytes[1]]),
                        Endianness::Big => u16::from_be_bytes([value_bytes[0], value_bytes[1]]),
                    };
                    let max_fl = match endian {
                        Endianness::Little => u16::from_le_bytes([value_bytes[2], value_bytes[3]]),
                        Endianness::Big => u16::from_be_bytes([value_bytes[2], value_bytes[3]]),
                    };
                    // This is a simplified version - actual lens parsing is more complex
                    if min_fl > 0 && max_fl > 0 {
                        if min_fl == max_fl {
                            ExifValue::Ascii(format!("{}mm", min_fl))
                        } else {
                            ExifValue::Ascii(format!("{}-{}mm", min_fl, max_fl))
                        }
                    } else {
                        ExifValue::Ascii(String::new())
                    }
                }
                NIKON_LENS_TYPE if tag_type == 1 => {
                    // LensType - BYTE
                    let val = value_bytes[0];
                    // Simplified - actual decoding is complex
                    let lens_type = match val & 0x0F {
                        1 => "MF",
                        2 => "D",
                        4 => "G",
                        6 => "G VR",
                        8 => "G ED",
                        10 => "G VR ED",
                        _ => "",
                    };
                    ExifValue::Ascii(lens_type.to_string())
                }
                _ => {
                    // For other tags, store as generic type
                    match tag_type {
                        2 => {
                            // ASCII
                            let s = std::str::from_utf8(&value_bytes[..count as usize])
                                .unwrap_or("")
                                .trim_end_matches('\0')
                                .to_string();
                            ExifValue::Ascii(s)
                        }
                        3 => {
                            // SHORT
                            let val = match endian {
                                Endianness::Little => {
                                    u16::from_le_bytes([value_bytes[0], value_bytes[1]])
                                }
                                Endianness::Big => {
                                    u16::from_be_bytes([value_bytes[0], value_bytes[1]])
                                }
                            };
                            ExifValue::Short(vec![val])
                        }
                        4 => {
                            // LONG
                            let val = match endian {
                                Endianness::Little => u32::from_le_bytes([
                                    value_bytes[0],
                                    value_bytes[1],
                                    value_bytes[2],
                                    value_bytes[3],
                                ]),
                                Endianness::Big => u32::from_be_bytes([
                                    value_bytes[0],
                                    value_bytes[1],
                                    value_bytes[2],
                                    value_bytes[3],
                                ]),
                            };
                            ExifValue::Long(vec![val])
                        }
                        7 => {
                            // UNDEFINED
                            ExifValue::Undefined(value_bytes[..value_size.min(value_bytes.len())].to_vec())
                        }
                        _ => ExifValue::Undefined(vec![]),
                    }
                }
            };

            tags.insert(
                tag_id,
                MakerNoteTag {
                    tag_id,
                    tag_name: get_nikon_tag_name(tag_id),
                    value,
                },
            );
        }
    }

    Ok(tags)
}
```

### Fix 3: Additional Enum Description Functions

**File**: `src/tags.rs`

Add these missing description functions:

```rust
pub fn get_photometric_interpretation_description(value: u16) -> &'static str {
    match value {
        0 => "WhiteIsZero",
        1 => "BlackIsZero",
        2 => "RGB",
        3 => "RGB Palette",
        4 => "Transparency Mask",
        5 => "CMYK",
        6 => "YCbCr",
        8 => "CIELab",
        9 => "ICCLab",
        10 => "ITULab",
        32803 => "Color Filter Array",
        34892 => "Linear Raw",
        _ => "Unknown",
    }
}

pub fn get_planar_configuration_description(value: u16) -> &'static str {
    match value {
        1 => "Chunky",
        2 => "Planar",
        _ => "Unknown",
    }
}
```

**Fix get_metering_mode_description** to use Nikon's preferred names:

```rust
pub fn get_metering_mode_description(value: u16) -> &'static str {
    match value {
        0 => "Unknown",
        1 => "Average",
        2 => "Center-weighted average",
        3 => "Spot",
        4 => "Multi-spot",
        5 => "Multi-segment",  // Changed from "Pattern" - this is what Nikon uses
        6 => "Partial",
        255 => "Other",
        _ => "Unknown",
    }
}
```

**Fix get_compression_description** to recognize Nikon NEF:

```rust
pub fn get_compression_description(value: u16) -> &'static str {
    match value {
        1 => "Uncompressed",
        2 => "CCITT 1D",
        3 => "T4/Group 3 Fax",
        4 => "T6/Group 4 Fax",
        5 => "LZW",
        6 => "JPEG (old-style)",
        7 => "JPEG",
        8 => "Adobe Deflate",
        9 => "JBIG B&W",
        10 => "JBIG Color",
        99 => "JPEG",
        262 => "Kodak 262",
        32766 => "Next",
        32767 => "Sony ARW Compressed",
        32769 => "Packed RAW",
        32770 => "Samsung SRW Compressed",
        32771 => "CCIRLEW",
        32772 => "Samsung SRW Compressed 2",
        32773 => "PackBits",
        32809 => "Thunderscan",
        32867 => "Kodak KDC Compressed",
        32895 => "IT8CTPAD",
        32896 => "IT8LW",
        32897 => "IT8MP",
        32898 => "IT8BL",
        32908 => "PixarFilm",
        32909 => "PixarLog",
        32946 => "Deflate",
        32947 => "DCS",
        34661 => "JBIG",
        34676 => "SGILog",
        34677 => "SGILog24",
        34712 => "JPEG 2000",
        34713 => "Nikon NEF Compressed",
        34892 => "Lossy JPEG",
        65000 => "Kodak DCR Compressed",
        65535 => "Pentax PEF Compressed",
        _ => "Unknown",
    }
}
```

## Implementation Priority

1. **Phase 1 (Immediate - ~2 hours)**:
   - Fix standard EXIF formatting issues (CFAPattern, PhotometricInterpretation, etc.)
   - These affect all 47 files and are in `src/output.rs` and `src/tags.rs`

2. **Phase 2 (High Priority - ~4 hours)**:
   - Rewrite `src/makernotes/nikon.rs` to actually parse values
   - Focus on the top tags: ISO, Quality, WhiteBalance, FocusMode, SerialNumber

3. **Phase 3 (Medium Priority - ~2 hours)**:
   - Add ColorSpace, VignetteControl, Sharpness parsing
   - Add Lens and LensType decoding (simplified version)

4. **Phase 4 (Future)**:
   - Parse complex nested structures (LensData, ShotInfo, FlashInfo)
   - These require understanding version-specific binary structures

## Expected Impact

After implementing Phase 1 and 2:
- **Standard EXIF fixes**: ~400 value mismatches eliminated across all formats
- **Nikon maker note fixes**: ~250 value mismatches eliminated for NEF files
- **Total reduction**: ~650 mismatches (73% of current 898 mismatches)

## Testing Strategy

After each phase:
```bash
cargo test --test exiftool_json_comparison test_exiftool_json_compatibility_nef -- --nocapture
```

Monitor the reduction in:
1. Value mismatches count
2. Missing fields count (will reduce as we parse more tags)
3. Number of files passing (target: >40 of 47 files)

## Notes

- The Nikon maker note format has multiple versions (V1, V2, V3) with different structures
- Some tags like LensData (0x0083), ShotInfo (0x0091), and FlashInfo (0x00A8) contain complex encrypted or versioned binary data
- For maximum compatibility, we should detect the camera model and parse accordingly
- ExifTool has extensive Nikon tag databases - we should consult their code for complex tags
