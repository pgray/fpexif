# Nikon LensData Decryption Implementation Plan

## Overview

Implement decryption for Nikon LensData version 02xx (0201, 0202, 0203, 0204) to enable proper LensID composite lookup. This will fix ~33 LensID mismatches across test files.

## Current State (Updated 2025-12-31)
- Match rate: 50.0%
- Files tested: 4 NRW, 47 NEF
- Matching: 4,820 | Mismatched: 447 | Missing: 4,376 | Extra: 1,723

### Key Mismatches
| Tag | Count | Issue |
|-----|-------|-------|
| AFAreaMode | ~20 | Returns "Unknown" instead of correct value |
| RedBalance/BlueBalance | ~30 | Wrong color balance values (encryption issue?) |
| MakerNoteVersion | ~10 | Minor formatting differences |
| Lens | ~10 | Focal length rounding differences |
| LensID | ~10 | Missing decryption for 02xx versions |

## Implementation Status

- LensData parsing exists for unencrypted versions (0100, 0101) in `src/makernotes/nikon.rs`
- Decryption function `nikon_decrypt()` already exists (lines 1318-1342)
- XLAT lookup tables already exist (lines 1279-1316)
- LensID composite lookup database exists with ~100+ lenses

## What's Missing

The encrypted LensData versions (02xx) are not decrypted before parsing. The TODO is at line 2022-2023:
```rust
// Version 02xx: Encrypted - requires decryption
// TODO: Implement decryption for 0201, 0202, 0203, 0204
```

## Encryption Details

### Algorithm
Nikon uses XOR-based stream cipher with two 256-byte substitution tables (XLAT_0, XLAT_1).

### Keys Required
1. **SerialNumber** (tag 0x001D) - Last byte used as index into XLAT_0
2. **ShutterCount** (tag 0x00A7) - All 4 bytes XOR'd together, used as index into XLAT_1

### Decrypt Start
For ALL version 02xx: decryption starts at byte 4 (version string is unencrypted)

## Implementation Steps

### Step 1: Modify `parse_lens_data` function signature

Change from:
```rust
fn parse_lens_data(data: &[u8]) -> (Vec<(String, String)>, Option<[u8; 7]>)
```

To:
```rust
fn parse_lens_data(
    data: &[u8],
    serial: Option<u32>,
    shutter_count: Option<u32>
) -> (Vec<(String, String)>, Option<[u8; 7]>)
```

### Step 2: Add version 02xx handling with decryption

```rust
else if version.starts_with("02") && data.len() >= 18 {
    if let (Some(serial), Some(count)) = (serial, shutter_count) {
        // Make mutable copy for decryption
        let mut decrypted = data.to_vec();

        // Decrypt starting at byte 4
        nikon_decrypt(serial, count, &mut decrypted, 4);

        // Parse using same offsets as 0101
        let lens_id_number = decrypted[11];
        let lens_f_stops_raw = decrypted[12];
        let min_focal_raw = decrypted[13];
        let max_focal_raw = decrypted[14];
        let max_ap_min_raw = decrypted[15];
        let max_ap_max_raw = decrypted[16];
        let mcu_version = decrypted[17];

        // ... add tags and lens_id_bytes
    }
}
```

### Step 3: Collect SerialNumber and ShutterCount during MakerNote parsing

In `parse_nikon_maker_notes()`, add tracking variables:
```rust
let mut serial_number: Option<u32> = None;
let mut shutter_count: Option<u32> = None;
```

When parsing NIKON_SERIAL_NUMBER (0x001D):
```rust
if let ExifValue::Ascii(s) = &value {
    serial_number = s.parse::<u32>().ok().or_else(|| {
        // Fallback for non-numeric serials
        if model.contains("D50") { Some(0x22) }
        else { Some(0x60) }
    });
}
```

When parsing NIKON_SHUTTER_COUNT (0x00A7):
```rust
if let ExifValue::Long(v) = &value {
    if !v.is_empty() {
        shutter_count = Some(v[0]);
    }
}
```

### Step 4: Pass keys to parse_lens_data

Change the call site:
```rust
NIKON_LENS_DATA => {
    if let ExifValue::Undefined(ref data_bytes) = value {
        let (lens_data_tags, raw_bytes) = parse_lens_data(
            data_bytes,
            serial_number,
            shutter_count
        );
        // ...
    }
}
```

### Step 5: Handle version 0800/0801 (Z-series)

These newer versions have different structure. Check ExifTool's `LensData0800` table for offsets.

## Byte Offsets (after decryption)

### Version 0201/0202/0203/0204

| Offset | Field | Conversion |
|--------|-------|------------|
| 0x00-0x03 | Version | ASCII string (unencrypted) |
| 0x04 | ExitPupilPosition | 2048 / value |
| 0x05 | AFAperture | 2^(value/24) |
| 0x08 | FocusPosition | hex value |
| 0x09 | FocusDistance | 0.01 * 10^(value/40) m |
| 0x0A | FocalLength | raw byte |
| 0x0B | LensIDNumber | raw byte |
| 0x0C | LensFStops | value / 12 |
| 0x0D | MinFocalLength | 5 * 2^(value/24) mm |
| 0x0E | MaxFocalLength | 5 * 2^(value/24) mm |
| 0x0F | MaxApertureAtMinFocal | 2^(value/24) |
| 0x10 | MaxApertureAtMaxFocal | 2^(value/24) |
| 0x11 | MCUVersion | raw byte |
| 0x12 | EffectiveMaxAperture | 2^(value/24) |

## Files to Modify

1. `src/makernotes/nikon.rs`:
   - `parse_lens_data()` - Add parameters and 02xx handling
   - `parse_nikon_maker_notes()` - Collect serial/count, pass to parse_lens_data

## Testing

1. Save baseline: `./bin/mfr-test nikon --save-baseline`
2. Implement changes
3. Check: `./bin/mfr-test nikon --check`
4. Verify LensID now shows full lens names instead of hex IDs

## Expected Impact

- Fix ~33 LensID mismatches
- Enable proper LensIDNumber, MinFocalLength, MaxFocalLength parsing
- Enable FocusDistance, AFAperture decoding for modern cameras

## Reference

- ExifTool Nikon.pm lines 5493-5663 (LensData tables)
- ExifTool Nikon.pm lines 13556-13590 (Decrypt function)
