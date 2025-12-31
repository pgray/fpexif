# Fujifilm EXIF Improvements Plan

## Current State (2025-12-31)
- Match rate: 74.0% (Highest among all manufacturers!)
- Files tested: 30 RAF
- Matching: 2,657 | Mismatched: 62 | Missing: 870 | Extra: 755

## Top Mismatches by Frequency

| Tag | Count | Issue | Fix |
|-----|-------|-------|-----|
| ChromaticAberrationParams | ~10 | Scientific notation formatting | Use "1.6e-05" instead of "0.000016" |
| LensID | ~5 | Minor spacing in lens names | "XF55-200mmF3.5-4.8" vs "XF55-200mm F3.5-4.8" |
| RawExposureBias | ~5 | Floating point precision | Values match but format differs |
| MaxApertureAtMinFocal/MaxFocal | ~5 | Values match, format issue | Check precision formatting |
| ExposureTime | ~3 | Fraction vs decimal | "1/3" vs "0.3" |
| Copyright | ~2 | Whitespace handling | " " vs "" for empty |

## Implementation Steps

### Phase 1: Formatting Fixes (High Impact)
1. **ChromaticAberrationParams** - Use scientific notation for small values
2. **LensID** - Add space between focal length and aperture in lens names
3. **ExposureTime** - Use decimal format for slow shutter speeds

### Phase 2: Minor Cleanup
1. **Copyright** - Trim whitespace from empty strings
2. **RawExposureBias** - Verify precision formatting

## Reference Files
- `src/makernotes/fuji.rs` - Main Fujifilm MakerNote parser
- `exiftool/lib/Image/ExifTool/FujiFilm.pm` - ExifTool Fujifilm module
- `exiv2/src/fujimn_int.cpp` - exiv2 Fujifilm implementation

## Notes
- Fujifilm has the highest match rate - focus on maintaining quality
- Most mismatches are formatting, not decode issues
- Consider adding computed fields (FocalLength35efl, FOV, etc.)

## Testing
```bash
./bin/mfr-test fujifilm --save-baseline
# Make changes
./bin/mfr-test fujifilm --check
```
