# Samsung EXIF Improvements Plan

## Current State (2025-12-31)
- Match rate: 35.6%
- Files tested: 7 SRW
- Matching: 256 | Mismatched: 37 | Missing: 426 | Extra: 88

## Top Mismatches by Frequency

| Tag | Count | Issue | Fix |
|-----|-------|-------|-----|
| ImageSize/Megapixels | ~7 | Sensor vs output dimensions | Check active area |
| Compression | ~5 | Numeric vs decoded | "32769"/"32770" vs "Packed RAW"/"Samsung SRW Compressed" |
| Make | ~2 | Garbage data appended | "SAMSUNG �      �@ڳ" vs "SAMSUNG" |
| FocalLength | ~1 | "inf" vs "0.0 mm" | Handle infinity case |
| ExposureMode | ~1 | "Unknown" vs "Unknown (4)" | Format unknown values |
| ExposureCompensation | ~1 | Value mismatch | Check parsing |
| ISO | ~1 | Value mismatch | 400 vs 480 - check source |

## Implementation Steps

### Phase 1: Critical Fixes
1. **Make** - Trim garbage bytes after "SAMSUNG"
2. **Compression** - Add Samsung-specific decode:
   - 32769 => "Packed RAW"
   - 32770 => "Samsung SRW Compressed"
3. **FocalLength** - Return "0.0 mm" instead of "inf"

### Phase 2: Dimension Fixes
1. **ImageSize/Megapixels** - Use active sensor area, not full sensor
   - SensorAreas tag contains: "0 0 W H left top right bottom"

### Phase 3: Decode Fixes
1. **ExposureMode** - Format unknown values as "Unknown (N)"
2. **ExposureCompensation** - Verify calculation

## Reference Files
- `src/makernotes/samsung.rs` - Samsung MakerNote parser (TODO: Create)
- `exiftool/lib/Image/ExifTool/Samsung.pm` - ExifTool Samsung module
- `exiv2/src/samsungmn_int.cpp` - exiv2 Samsung implementation

## Samsung SRW Notes
- NX series cameras use SRW (Samsung RAW)
- Contains encrypted metadata sections
- Some tags overlap with older Samsung compact cameras

## Testing
```bash
./bin/mfr-test samsung --save-baseline
# Make changes
./bin/mfr-test samsung --check
```
