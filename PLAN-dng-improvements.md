# DNG/Ricoh/Leica EXIF Improvements Plan

## Current State (2025-12-31)
- Match rate: 32.7%
- Files tested: 13 DNG (from various manufacturers)
- Matching: 621 | Mismatched: 168 | Missing: 1,112 | Extra: 188

**Note:** DNG/Ricoh/Leica tests use the same DNG files since these manufacturers primarily output DNG.

## Top Mismatches by Frequency

| Tag | Count | Issue | Fix |
|-----|-------|-------|-----|
| AsShotNeutral | ~10 | Float precision | Limit to ~10 decimal places |
| ActiveArea | ~8 | Array format | "[18,96,3062,4168]" vs "18 96 3062 4168" |
| BlackLevelRepeatDim | ~5 | Array format | "[1,1]" vs "1 1" |
| BlackLevel | ~5 | Array format | "[0,0,0,0]" vs "0 0 0 0" |
| StripOffsets/ByteCounts | ~5 | Large array vs binary | Show "(Binary data)" |
| GPSLatitude/Longitude | ~5 | Formatting with direction | Check GPS format |
| WhiteBalance | ~3 | Decode values | "Auto" vs "Auto or Manual" |
| ExposureTime/ShutterSpeedValue | ~3 | Division by zero | Handle infinite/undefined |
| CFAPlaneColor | ~2 | Array format | "[0,1,2]" vs "Red,Green,Blue" |
| ShadowScale | ~2 | Complex parsing | Large binary blob issue |

## Implementation Steps

### Phase 1: Array Formatting (High Impact)
All DNG array tags should use space-separated format:
1. **ActiveArea** - "18 96 3062 4168"
2. **BlackLevelRepeatDim** - "1 1"
3. **BlackLevel** - "0 0 0 0"
4. **CFAPlaneColor** - "Red,Green,Blue" (comma-separated names)

### Phase 2: Precision/Formatting
1. **AsShotNeutral** - Limit float to ~10 decimal places
2. **StripOffsets/ByteCounts** - Show as "(Binary data N bytes)" for large arrays
3. **ExposureTime** - Handle division by zero cases

### Phase 3: GPS Formatting
1. **GPSLatitude/Longitude** - Include direction suffix (N/S, E/W)
2. **GPSAltitude** - Format consistently

### Phase 4: Decode Values
1. **WhiteBalance** - Map to "Auto or Manual" when appropriate
2. **CFAPlaneColor** - Map numeric to color names

## DNG-Specific Tags Reference

Key DNG tags from Adobe DNG Specification:
| Tag | Type | Description |
|-----|------|-------------|
| 50706 | DNGVersion | DNG version number |
| 50707 | DNGBackwardVersion | Oldest compatible version |
| 50708 | UniqueCameraModel | Camera model string |
| 50711 | CFALayout | CFA pattern layout |
| 50713 | BlackLevelRepeatDim | Black level tile size |
| 50714 | BlackLevel | Black level per tile |
| 50717 | WhiteLevel | Maximum pixel value |
| 50721 | ColorMatrix1 | Color matrix for illuminant 1 |
| 50728 | AsShotNeutral | White balance neutral |
| 50829 | ActiveArea | Usable image area |

## Reference Files
- `src/dng.rs` - DNG file parser
- `exiftool/lib/Image/ExifTool/DNG.pm` - ExifTool DNG module

## Testing
```bash
./bin/mfr-test dng --save-baseline
# Make changes
./bin/mfr-test dng --check
```
