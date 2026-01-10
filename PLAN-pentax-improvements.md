# Pentax EXIF Improvements Plan

## Current State (2025-12-31)
- Match rate: 26.4%
- Files tested: 13 DNG, 17 PEF
- Matching: 1,389 | Mismatched: 505 | Missing: 3,370 | Extra: 1,070

**Note:** Test includes non-Pentax DNG files (Leica, Nokia, etc.)

## Top Mismatches by Frequency

| Tag | Count | Issue | Fix |
|-----|-------|-------|-----|
| ActiveArea | ~10 | Array formatting | "[18,96,3062,4168]" vs "18 96 3062 4168" |
| AsShotNeutral | ~10 | Float precision | Limit to ~10 decimal places |
| FocalLength | ~10 | Precision | "38.02 mm" vs "38.0 mm" |
| GPSLatitude/Longitude | ~8 | Format with seconds | Check GPS formatting |
| LinearizationTable | ~5 | Array vs binary | Show as "(Binary data)" for large arrays |
| WhiteBalance | ~5 | Decode values | "Auto" vs "Auto or Manual" |
| ExposureTime | ~3 | Division by zero | "1/0" should be handled |

## Implementation Steps

### Phase 1: Array Formatting
1. **ActiveArea** - Use space-separated format instead of JSON array
2. **AsShotNeutral** - Limit float precision
3. **LinearizationTable** - Show as binary for large tables

### Phase 2: Precision Fixes
1. **FocalLength** - Round to 1 decimal place
2. **ExposureTime** - Handle edge cases (infinite exposure)

### Phase 3: GPS Formatting
1. **GPSLatitude/Longitude** - Match ExifTool format with direction suffix

## Reference Files
- `src/makernotes/pentax.rs` - Main Pentax MakerNote parser (TODO: Create)
- `exiftool/lib/Image/ExifTool/Pentax.pm` - ExifTool Pentax module
- `exiv2/src/pentaxmn_int.cpp` - exiv2 Pentax implementation

## Notes
- Pentax PEF files need dedicated MakerNote parser
- Many test files are DNG from other manufacturers
- Focus on DNG tag formatting first

## Testing
```bash
./bin/mfr-test pentax --save-baseline
# Make changes
./bin/mfr-test pentax --check
```
