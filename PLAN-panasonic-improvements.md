# Panasonic EXIF Improvements Plan

## Current State (2025-12-31)
- Match rate: 41.8%
- Files tested: 20 RW2
- Matching: 1,567 | Mismatched: 77 | Missing: 2,101 | Extra: 1,257

## Top Mismatches by Frequency

| Tag | Count | Issue | Fix |
|-----|-------|-------|-----|
| AdvancedSceneMode | ~15 | Numeric output instead of decode | Add decode_advanced_scene_mode_exiftool |
| InternalSerialNumber | ~10 | Raw bytes instead of decoded | Parse "(X15) 2014:06:12 no. 0153" format |
| ContrastMode | ~8 | Wrong decode values | Fix decode_contrast_mode_exiftool |
| FlashBias | ~5 | Minor formatting | Check format differences |
| PitchAngle/RollAngle | ~5 | Precision formatting | Match ExifTool decimal places |
| AccessoryType/SerialNumber | ~5 | Empty vs "NO-ACCESSORY" | Add default values |
| City/Landmark | ~3 | Binary data vs empty string | Parse location data |
| BabyAge | ~2 | Empty vs "(not set)" | Add default text |

## Implementation Steps

### Phase 1: Decode Function Fixes
1. **AdvancedSceneMode** - Add lookup table:
   - 1 => "Off"
   - 5 => "Macro (intelligent auto)"
   - etc.
2. **ContrastMode** - Fix decode values, check Panasonic.pm
3. **InternalSerialNumber** - Parse the encoded format

### Phase 2: Formatting Fixes
1. **FlashBias** - Verify formatting
2. **PitchAngle/RollAngle** - Match precision
3. **AccessoryType** - Return "NO-ACCESSORY" for empty

### Phase 3: Location Data
1. **City** - Parse binary location data if present
2. **Landmark** - Handle empty properly

## Reference Files
- `src/makernotes/panasonic.rs` - Main Panasonic MakerNote parser
- `exiftool/lib/Image/ExifTool/Panasonic.pm` - ExifTool Panasonic module
- `exiv2/src/panasonicmn_int.cpp` - exiv2 Panasonic implementation

## Key ExifTool Patterns

### AdvancedSceneMode (tag 0x001d)
```perl
0x001d => {
    Name => 'AdvancedSceneMode',
    Writable => 'int16u',
    PrintConv => {
        1 => 'Off',
        2 => 'Outdoor/Illuminations/Grass/Snow',
        3 => 'Indoor/Architecture/Objects/Flowers',
        # ... many more
    },
},
```

### InternalSerialNumber (tag 0x0025)
Decodes to: "(MODEL) YYYY:MM:DD no. NNNN"

## Testing
```bash
./bin/mfr-test panasonic --save-baseline
# Make changes
./bin/mfr-test panasonic --check
```
