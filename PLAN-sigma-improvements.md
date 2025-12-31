# Sigma EXIF Improvements Plan

## Current State (2025-12-31)
- Match rate: 36.6%
- Files tested: 8 X3F
- Matching: 278 | Mismatched: 7 | Missing: 474 | Extra: 67

## Top Mismatches by Frequency

| Tag | Count | Issue | Fix |
|-----|-------|-------|-----|
| WhiteBalance | ~2 | Decode mismatch | "Manual" vs "Overcast"/"Sunlight" |
| MeteringMode | ~1 | Decode mismatch | "Spot" vs "Center-weighted average" |
| LensID | ~1 | Numeric vs decoded | "1007" vs "Sigma 30mm F2.8" |
| ShutterSpeedValue | ~1 | Calculation issue | "1/1" vs "0.8" |
| ExposureTime | ~1 | Calculation issue | "1" vs "0.8" |

## Implementation Steps

### Phase 1: Decode Fixes
1. **WhiteBalance** - Fix white balance decode lookup
2. **MeteringMode** - Fix metering mode decode lookup
3. **LensID** - Add Sigma lens database lookup

### Phase 2: Exposure Calculation
1. **ShutterSpeedValue/ExposureTime** - Fix calculation for slow exposures

## Reference Files
- `src/x3f.rs` - X3F file parser
- `src/makernotes/sigma.rs` - Sigma MakerNote parser (TODO: Create)
- `exiftool/lib/Image/ExifTool/Sigma.pm` - ExifTool Sigma module
- `exiftool/lib/Image/ExifTool/SigmaRaw.pm` - X3F specific
- `exiv2/src/sigmamn_int.cpp` - exiv2 Sigma implementation

## X3F Format Notes
- Sigma uses Foveon X3 sensor with unique color structure
- X3F is proprietary container format
- MakerNote structure differs from TIFF-based formats

## Sigma Lens Database
Reference Sigma.pm for lens IDs:
```perl
%sigmaLensTypes = (
    '0' => 'Sigma 50mm F2.8 EX DG Macro',
    # ...
);
```

## Testing
```bash
./bin/mfr-test sigma --save-baseline
# Make changes
./bin/mfr-test sigma --check
```
