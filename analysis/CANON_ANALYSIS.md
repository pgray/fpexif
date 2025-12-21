# Canon CR2 Maker Notes Analysis and Fix Proposal

## Implementation Status

The following items from this analysis have been implemented:

### Completed
- **Model database** (~65 entries) - EOS, PowerShot, etc. (was already present)
- **Lens database** (~180 entries) - EF, EF-S, RF mount lenses via `get_canon_lens_name()`

### Pending
- Core EXIF formatting fixes (UserComment null handling, GPS formatting)
- APEX value output format (string -> number for MeasuredEV, TargetAperture, etc.)
- AF area array formatting (space-separated strings)
- Categories lookup (bit flag decoding)
- DateStampMode lookup
- AEBBracketValue format fix
- FocalLength calculation fix
- Additional CanonModelID entries
- AFInfo structure parsing
- SensorInfo structure parsing
- CameraInfo structure parsing
- Computed fields (DOF, CircleOfConfusion, HyperfocalDistance)

---

## Executive Summary

The Canon maker notes implementation has extensive tag parsing but needs refinements to match ExifTool's output formatting and value interpretations. This analysis identifies the most impactful fixes needed to match ExifTool's output for 55 CR2 test files.

## Test Results Overview

- **Files Tested**: 55 CR2 files
- **Files Passing**: 0
- **Total Issues**: 13,643
- **Value Mismatches**: 1,214
- **Missing Fields**: 10,542

## Top Value Mismatches by Frequency

### Category 1: Formatting Issues (Affects Most Files)

| Field | Count | Issue | Fix Location |
|-------|-------|-------|--------------|
| UserComment | 55 | Base64 of null bytes should be empty string | `src/output.rs` |
| AEBBracketValue | 55 | `"Off"` should be `0` (numeric) | `src/makernotes/canon.rs` |
| MaxAperture | 55 | `"2.8"` (string) should be `2.8` (unquoted) | `src/makernotes/canon.rs` |
| MinAperture | 55 | `"8"` (string) should be `8` (unquoted) | `src/makernotes/canon.rs` |
| TargetAperture | 55 | `"5.6"` (string) should be `5.6` (unquoted) | `src/makernotes/canon.rs` |
| MeasuredEV | 55 | `"13.31"` (string) should be `13.31` (unquoted) | `src/makernotes/canon.rs` |
| Categories | 55 | `[8,0]` should be `"(none)"` or category name | `src/makernotes/canon.rs` |
| DateStampMode | 55 | `0` should be `"Off"` | `src/makernotes/canon.rs` |

### Category 2: AF (Auto Focus) Array Formatting

| Field | Count | Issue | Fix Location |
|-------|-------|-------|--------------|
| AFAreaHeights | 47 | Array `[1936,1288]` should be space-separated string | `src/makernotes/canon.rs` |
| AFAreaWidths | 47 | Array should be space-separated string | `src/makernotes/canon.rs` |
| AFAreaXPositions | 47 | Array should be space-separated string | `src/makernotes/canon.rs` |
| AFAreaYPositions | 47 | Array should be space-separated string | `src/makernotes/canon.rs` |
| AFAreaMode | 42 | Different value interpretation (lookup table mismatch) | `src/makernotes/canon.rs` |
| NumAFPoints | 38 | Value mismatch (parsing wrong index) | `src/makernotes/canon.rs` |

### Category 3: GPS Formatting

| Field | Count | Issue | Fix Location |
|-------|-------|-------|--------------|
| GPSLatitude | 12 | `[41.0,21.0,48.32]` should be `"41 deg 21' 48.32\" N"` | `src/output.rs` |
| GPSLongitude | 12 | Similar DMS format needed | `src/output.rs` |
| GPSTimeStamp | 12 | `[9.0,53.0,44.0]` should be `"09:53:44"` | `src/output.rs` |
| GPSAltitude | 12 | `226.6551` should be `"226.6 m Above Sea Level"` | `src/output.rs` |
| GPSAltitudeRef | 12 | `0` should be `"Above Sea Level"` | `src/output.rs` |
| GPSLatitudeRef | 12 | `"N"` should be `"North"` | `src/output.rs` |
| GPSLongitudeRef | 12 | `"E"` should be `"East"` | `src/output.rs` |

### Category 4: Canon-Specific Value Lookups

| Field | Count | Issue | Fix Location |
|-------|-------|-------|--------------|
| CanonModelID | 35 | Missing model lookup (shows number) | `src/makernotes/canon.rs` |
| ExposureMode | 30 | Wrong lookup table used | `src/makernotes/canon.rs` |
| AutoISO | 28 | Calculated value differs | `src/makernotes/canon.rs` |
| BaseISO | 25 | Calculated value differs | `src/makernotes/canon.rs` |
| FocalLength | 22 | `8557.0 mm` vs `8.6 mm` (wrong decode) | `src/makernotes/canon.rs` |
| MinFocalLength | 20 | `3 mm` vs `3.8 mm` | `src/makernotes/canon.rs` |
| MaxFocalLength | 20 | `22 mm` vs `22.5 mm` | `src/makernotes/canon.rs` |

## Detailed Fix Proposals

### Fix 1: Numeric Output for APEX Values

**File**: `src/makernotes/canon.rs`

The APEX values (MeasuredEV, TargetAperture, MaxAperture, MinAperture) are being output as quoted strings but should be unquoted numbers in JSON.

**Current**:
```rust
ExifValue::Ascii(format!("{:.2}", ev))
```

**Fix**:
Create a new `ExifValue::Float` variant or output as rational:
```rust
// For MeasuredEV, output as number not string
ExifValue::Rational(vec![(((ev * 100.0) as u32), 100)])
```

### Fix 2: GPS Coordinate Formatting

**File**: `src/output.rs`

Add GPS coordinate formatting to convert rational arrays to DMS strings:

```rust
// GPSLatitude/GPSLongitude (tag 0x0002/0x0004)
fn format_gps_coordinate(rationals: &[(u32, u32)], ref_value: &str) -> String {
    if rationals.len() >= 3 {
        let deg = rationals[0].0 as f64 / rationals[0].1 as f64;
        let min = rationals[1].0 as f64 / rationals[1].1 as f64;
        let sec = rationals[2].0 as f64 / rationals[2].1 as f64;
        format!("{} deg {}' {:.2}\" {}", deg as i32, min as i32, sec, ref_value)
    } else {
        String::new()
    }
}

// GPSTimeStamp (tag 0x0007)
fn format_gps_timestamp(rationals: &[(u32, u32)]) -> String {
    if rationals.len() >= 3 {
        let h = rationals[0].0 / rationals[0].1;
        let m = rationals[1].0 / rationals[1].1;
        let s = rationals[2].0 / rationals[2].1;
        format!("{:02}:{:02}:{:02}", h, m, s)
    } else {
        String::new()
    }
}
```

### Fix 3: GPS Reference Value Lookups

**File**: `src/output.rs` or `src/tags.rs`

```rust
pub fn get_gps_latitude_ref_description(value: &str) -> &'static str {
    match value {
        "N" => "North",
        "S" => "South",
        _ => value,
    }
}

pub fn get_gps_longitude_ref_description(value: &str) -> &'static str {
    match value {
        "E" => "East",
        "W" => "West",
        _ => value,
    }
}

pub fn get_gps_altitude_ref_description(value: u8) -> &'static str {
    match value {
        0 => "Above Sea Level",
        1 => "Below Sea Level",
        _ => "Unknown",
    }
}
```

### Fix 4: AF Area Array Formatting

**File**: `src/makernotes/canon.rs`

The AF area arrays should be formatted as space-separated strings:

```rust
// When decoding AFInfo
let af_area_heights: Vec<i16> = /* read values */;
let formatted = af_area_heights.iter()
    .map(|v| v.to_string())
    .collect::<Vec<_>>()
    .join(" ");
decoded.insert("AFAreaHeights".to_string(), ExifValue::Ascii(formatted));
```

### Fix 5: Categories Lookup

**File**: `src/makernotes/canon.rs`

```rust
fn decode_categories(value: u16) -> &'static str {
    if value == 0 {
        "(none)"
    } else {
        // Bit flags: 0x01=People, 0x02=Scenery, 0x04=Events, etc.
        let mut cats = Vec::new();
        if value & 0x01 != 0 { cats.push("People"); }
        if value & 0x02 != 0 { cats.push("Scenery"); }
        if value & 0x04 != 0 { cats.push("Events"); }
        if value & 0x08 != 0 { cats.push("Sports"); }
        if value & 0x10 != 0 { cats.push("Night Scene"); }
        // ... etc
        if cats.is_empty() { "(none)" } else { /* join */ }
    }
}
```

### Fix 6: DateStampMode Lookup

**File**: `src/makernotes/canon.rs`

```rust
fn decode_date_stamp_mode(value: u16) -> &'static str {
    match value {
        0 => "Off",
        1 => "Date",
        2 => "Date & Time",
        _ => "Unknown",
    }
}
```

### Fix 7: AEBBracketValue Format

**File**: `src/makernotes/canon.rs`

Change from string "Off" to numeric 0 for JSON output when bracketing is off:

```rust
// AEBBracketValue - output as number, not string
if bracket_value == 0 {
    ExifValue::Short(vec![0])  // Numeric 0, not "Off"
} else {
    ExifValue::Ascii(format!("{}", bracket_value))
}
```

### Fix 8: CanonModelID Lookup Enhancement

**File**: `src/makernotes/canon.rs`

The model lookup table needs more entries. Current implementation has ~90 models but ExifTool has 400+. Priority additions:

```rust
// Add missing common models
0x03740000 => "PowerShot SX60 HS",
0x03700000 => "PowerShot S90",
// ... many more needed
```

### Fix 9: FocalLength Calculation Fix

**File**: `src/makernotes/canon.rs`

There's a bug where FocalLength is being read from wrong index or wrong calculation:

```rust
// decode_focal_length - ensure correct index and units
// FocalLength should be at index 2, stored in mm already
if data.len() > 2 {
    let focal_length = data[2] as f64;  // Not data[0]!
    // ... format appropriately
}
```

## Missing Fields Analysis

Top 20 most frequently missing fields (not in fpexif but in exiftool):

| Field | Count | Category |
|-------|-------|----------|
| AFImageHeight | 55 | AFInfo structure |
| AFImageWidth | 55 | AFInfo structure |
| AFPoint | 55 | AFInfo structure |
| AFPointsInFocus | 55 | AFInfo structure |
| BlackMaskBottomBorder | 55 | SensorInfo structure |
| BlackMaskLeftBorder | 55 | SensorInfo structure |
| BlackMaskRightBorder | 55 | SensorInfo structure |
| BlackMaskTopBorder | 55 | SensorInfo structure |
| CameraTemperature | 53 | CameraInfo structure |
| CanonFlashMode | 52 | CameraSettings |
| CircleOfConfusion | 55 | Computed field |
| ColorComponents | 55 | JPEG structure |
| DOF | 55 | Computed field |
| DriveMode | 52 | Already decoded, not exported |
| FieldOfView | 55 | Computed field |
| FileSize | 55 | File metadata |
| HyperfocalDistance | 55 | Computed field |
| LensID | 45 | Lens lookup |
| LightValue | 55 | Computed field |
| ScaleFactor35efl | 55 | Computed field |

## Implementation Priority

1. **Phase 1 (High Impact - Formatting)**:
   - Fix GPS coordinate formatting
   - Fix APEX value output format (string -> number)
   - Fix UserComment null handling
   - Fix AF area array formatting

2. **Phase 2 (Medium Impact - Lookups)**:
   - Add remaining CanonModelID entries
   - Fix Categories lookup
   - Fix DateStampMode lookup
   - Fix AEBBracketValue format

3. **Phase 3 (Complex Structures)**:
   - Parse AFInfo structure for AFImageHeight/Width
   - Parse SensorInfo structure for BlackMask values
   - Parse CameraInfo structure for CameraTemperature

4. **Phase 4 (Computed Fields)**:
   - Add CircleOfConfusion calculation
   - Add DOF calculation
   - Add HyperfocalDistance calculation
   - Add LightValue calculation

## Expected Impact

After implementing Phase 1 and 2:
- **Formatting fixes**: ~450 value mismatches eliminated
- **Lookup fixes**: ~200 value mismatches eliminated
- **Total reduction**: ~650 mismatches (54% of current 1,214 mismatches)

## Testing Strategy

After each phase:
```bash
FPEXIF_TEST_FILES=/fpexif/raws cargo test --test exiftool_json_comparison test_exiftool_json_compatibility_cr2 -- --nocapture
```

Monitor the reduction in value mismatches and missing fields counts.
