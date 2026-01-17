# Context for Restart

## Current Task
Improving tag coverage for mfr-test across manufacturers. Goals: 80%+ for Sony/Canon, 90%+ for Nikon.

## Progress Summary

### Current Match Rates
| Manufacturer | Rate (raws) | Rate (data.lfs) | Target | Status |
|--------------|-------------|-----------------|--------|--------|
| Fujifilm | 94.2% | - | 90%+ | Done |
| Nikon | 87.5% | 65.8% | 90% | +2.5% needed |
| Canon | 79.9% | - | 80% | Done |
| Sony | 80.2% (data.lfs) / 75.3% (raws) | - | 80% | DONE |

### Changes Made This Session

**data.lfs mismatches reduced from 138 to 98 (-40 fixed)**

1. **FlashCompensation fraction formatting** (`src/makernotes/nikon.rs`)
   - Added `format_flash_compensation()` function using PrintFraction logic
   - Outputs "-2/3", "-1/3", "+1/2" instead of decimals

2. **Sharpness value fix** (`src/makernotes/nikon.rs`)
   - Changed "HIGH" => "High" (not "Hard")
   - ExifTool FormatString title-cases words with vowels

3. **MakerNoteVersion format** (`src/makernotes/nikon.rs`)
   - Changed format from "X.Y" to "X.YZ" to preserve leading zeros
   - "0101" now becomes "1.01" not "1.1"

4. **FlashSetting Red-Eye case** (`src/makernotes/nikon.rs`)
   - Fixed "Red-eye" => "Red-Eye", "Red-eye Slow" => "Red-Eye Slow"

5. **FlashType NEW_TTL case** (`src/makernotes/nikon.rs`)
   - Added "NEW_TTL" => "New_TTL" mapping

6. **153-point AF lookup table** (`src/makernotes/nikon.rs`)
   - Corrected entire lookup table from ExifTool %afPoints153
   - Point 25 now correctly maps to "G11" not "G8"

7. **ToneComp case normalization** (`src/makernotes/nikon.rs`)
   - Added `decode_tone_comp_exiftool()` function
   - "MED.H" => "Med.H", "MED.L" => "Med.L"

8. **PictureControlName formatting** (`src/makernotes/nikon.rs`)
   - Updated `format_picture_control_string()` to match ExifTool FormatString
   - Words without vowels stay uppercase ("KR", "JP")
   - Words with vowels get title-cased ("Vivid", "Standard")

9. **FirmwareVersion for D6/Z cameras** (`src/makernotes/nikon.rs`)
   - Version 0246+ uses 8-byte FirmwareVersion (not 5)
   - "01.11.d0" now parsed correctly instead of "01.11"

### Test Results
```
./bin/mfr-test nikon
  Match rate: 87.5%, 2 mismatches, 1021 missing

./bin/mfr-test nikon --data-lfs
  Match rate: 65.8%, 98 mismatches, 21921 missing
```

### Key Files Modified
- `src/makernotes/nikon.rs` - Multiple formatting fixes
- `context.md` - This file

### Pre-push Checks
All passing: `./bin/ccc`

### Commands to Resume
```bash
# Check current status
./bin/mfr-test nikon && ./bin/mfr-test nikon --data-lfs

# Verbose output with mismatches
./bin/mfr-test nikon --data-lfs --verbose 2>&1 | grep "fpexif=.*exiftool=" | sort | uniq -c | sort -rn | head -30

# Pre-push checks
./bin/ccc
```

## Remaining Issues (98 mismatches in data.lfs)

### By Count
| Issue | Count | Notes |
|-------|-------|-------|
| FlashControlMode | 18 | "Off" vs "TTL" - structure mapping issue |
| TimeZone | 18 | Missing location suffix for Z cameras |
| PrimaryAFPoint | 12+ | Schema detection issues |
| FlashMode | 7 | "Unknown" vs "Unknown (4)" format |
| PhaseDetectAF | 5 | Wrong detection values |
| AFAreaMode | 2 | Wrong values |
| Various | ~10 | Single-occurrence issues |

### Known Difficult Fixes
- **FlashControlMode**: Different structure mappings in FlashInfo vs ShotInfo
- **TimeZone location names**: Z9-era cameras get location from ShotInfo, not WorldTime tag
- **PhaseDetectAF/AFAreaMode**: May require camera-model-specific handling

### Known Mismatches (2 total on raws - P6000_GPS.NRW)
- ShutterSpeed/ExposureTime: Rounding difference (1/177 vs 1/176)

## Technical Notes
- NikonCustom settings require camera-model-specific offset tables (D800, D7000, etc.)
- ShotInfo versions 02xx are encrypted, need serial+shutter_count for decryption
- Version 0246+ (D6, Z cameras) use 8-byte FirmwareVersion
- FormatString: words without vowels stay uppercase (KR, JP, AF)
- 153-point AF (D5/D500/D850): spiral outward from center E9

## Git Status
Branch: try-ralph
Files modified: nikon.rs, context.md
