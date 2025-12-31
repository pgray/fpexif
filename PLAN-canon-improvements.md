# Canon EXIF Improvements Plan

## Current State
- Match rate: 37.9%
- Files tested: 54 CR2, 18 CRW
- Matching: 6,513 | Mismatched: 498 | Missing: 10,163 | Extra: 2,209

## Top Mismatches by Frequency

| Tag | Count | Issue | Fix |
|-----|-------|-------|-----|
| StripOffsets/ByteCounts | 35 | SubIFD prioritization | Same fix as Nikon - prefer raw data SubIFD |
| TargetExposureTime | 19 | Formatting difference | Check ExifTool format |
| Megapixels | 16 | Computed field missing | Add computation from ImageWidth/Height |
| LensInfo | 15 | Not parsed | Parse from MakerNote |
| ImageSize | 14 | Computed field | Add "WxH" format |
| ISO | 12 | Formatting | Check format differences |
| CanonImageWidth/Height | 12/11 | Wrong IFD selection | Fix IFD priority |
| MeteringMode | 11 | Decode mismatch | Check decode values |
| LensType/LensID | 8/7 | Lens lookup | Add lens database |
| ExposureCompensation | 7 | Formatting | Already fixed? Check Canon-specific |

## Implementation Steps

### Phase 1: Quick Fixes
1. **Megapixels** - Compute from ImageWidth × ImageHeight / 1,000,000
2. **ImageSize** - Format as "WxH" string
3. **ExposureCompensation** - Verify Canon format matches

### Phase 2: MakerNote Decode Improvements
1. **MeteringMode** - Verify decode_metering_mode_exiftool values
2. **ISO** - Check Canon-specific ISO formatting in MakerNote
3. **TargetExposureTime** - Add decode function

### Phase 3: Lens Data
1. **LensInfo** - Parse MinFocalLength, MaxFocalLength, MinAperture, MaxAperture
2. **LensType/LensID** - Implement Canon lens lookup table
   - Reference: `exiftool/lib/Image/ExifTool/Canon.pm` - `%canonLensTypes`

### Phase 4: SubIFD Fixes (shared with Nikon)
1. **StripOffsets/StripByteCounts** - Fix SubIFD prioritization
2. **CanonImageWidth/Height** - Ensure correct IFD is used

## Reference Files
- `src/makernotes/canon.rs` - Main Canon MakerNote parser
- `exiftool/lib/Image/ExifTool/Canon.pm` - ExifTool Canon module
- `exiv2/src/canonmn_int.cpp` - exiv2 Canon implementation

## Key ExifTool Patterns to Check

### Canon Lens Types (Canon.pm ~line 7000+)
```perl
%canonLensTypes = (
    1 => 'Canon EF 50mm f/1.8',
    2 => 'Canon EF 28mm f/2.8',
    # ... hundreds of entries
);
```

### TargetExposureTime
Look for PrintConv in Canon.pm for formatting.

## Testing
```bash
./bin/mfr-test canon --save-baseline
# Make changes
./bin/mfr-test canon --check
```
