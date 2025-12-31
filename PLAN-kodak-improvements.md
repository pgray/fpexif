# Kodak EXIF Improvements Plan

## Current State (2025-12-31)
- Match rate: 28.2%
- Files tested: 5 KDC, 1 DCR
- Matching: 213 | Mismatched: 15 | Missing: 526 | Extra: 128

## Top Mismatches by Frequency

| Tag | Count | Issue | Fix |
|-----|-------|-------|-----|
| BitsPerSample | ~1 | "12 12 12" vs "12" | Use single value for uniform samples |
| ImageWidth/Height | ~1 | Wrong dimensions (DCR) | Check thumbnail vs main image |
| ImageSize/Megapixels | ~1 | Derived from wrong dimensions | Fix source dimensions |
| PhotometricInterpretation | ~1 | "YCbCr" vs "Color Filter Array" | Check CFA detection |
| Copyright | ~1 | Trailing space | "Kodak Digital Camera 50 " => trim |
| WhiteBalance | ~1 | Decode mismatch | "Auto" vs "Shade" |
| Compression | ~1 | Decode mismatch | "Uncompressed" vs "JPEG" |
| MeteringMode | ~1 | Decode mismatch | "Multi-segment" vs "Partial" |

## Implementation Steps

### Phase 1: DCR File Fixes
The Kodak DCS Pro has complex issues:
1. **ImageWidth/Height** - Detect main image vs preview correctly
2. **BitsPerSample** - Return single value when uniform
3. **PhotometricInterpretation** - Detect CFA pattern properly

### Phase 2: Formatting Fixes
1. **Copyright** - Trim trailing whitespace
2. **Megapixels** - Round consistently (0.381 vs 0.4)

### Phase 3: Decode Fixes
1. **WhiteBalance** - Verify decode values
2. **MeteringMode** - Check metering mode decode

## Reference Files
- `src/kodak.rs` - Kodak file parser (if exists)
- `exiftool/lib/Image/ExifTool/Kodak.pm` - ExifTool Kodak module

## Notes
- Kodak DCR files are complex with many proprietary tags
- KDC files are simpler consumer camera format
- Low file count means small impact on overall parity

## Testing
```bash
./bin/mfr-test kodak --save-baseline
# Make changes
./bin/mfr-test kodak --check
```
