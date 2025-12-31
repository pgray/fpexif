# Sony EXIF Improvements Plan

## Current State
- Match rate: 35.2%
- Files tested: 1 SRF, 1 SR2, 31 ARW
- Matching: 2,366 | Mismatched: 344 | Missing: 4,004 | Extra: 2,110

## Top Mismatches by Frequency

| Tag | Count | Issue | Fix |
|-----|-------|-------|-----|
| LensSpec | 18 | Formatting | Format as "MinFL-MaxFL mm f/MinAp-MaxAp" |
| SoftSkinEffect | 10 | Decode values wrong | Fix decode_soft_skin_effect_exiftool |
| SequenceNumber | 8 | Formatting | Check ExifTool format |
| AFPointSelected | 8 | Decode values | Add/fix decode function |
| AutoPortraitFramed | 7 | Decode values | Add decode function |
| Sharpness | 6 | Decode mismatch | Fix decode values |
| Quality | 6 | Decode mismatch | Fix decode values |
| LensID | 6 | Lens lookup failing | Fix lens database lookup |
| HDR | 6 | Decode values | Add/fix decode function |
| FlashAction | 6 | Decode values | Add decode function |
| Saturation | 5 | Decode mismatch | Fix decode values |
| LateralChromaticAberration | 5 | Formatting | Check format |
| AFPointsUsed | 5 | Decode/format | Fix decode function |
| Contrast | 4 | Decode mismatch | Fix decode values |
| FocusMode | 3 | Decode mismatch | Verify decode values |

## Implementation Steps

### Phase 1: Decode Function Fixes
These are straightforward value mapping fixes in `src/makernotes/sony.rs`:

1. **SoftSkinEffect** - Fix decode_soft_skin_effect_exiftool (10 fixes)
2. **Sharpness** - Fix sharpness decode values (6 fixes)
3. **Quality** - Fix quality decode values (6 fixes)
4. **Saturation** - Fix saturation decode values (5 fixes)
5. **Contrast** - Fix contrast decode values (4 fixes)
6. **FocusMode** - Verify focus mode values (3 fixes)

### Phase 2: Add Missing Decode Functions
1. **AFPointSelected** - Add decode function (8 fixes)
2. **AutoPortraitFramed** - Add decode function (7 fixes)
3. **HDR** - Add decode function (6 fixes)
4. **FlashAction** - Add decode function (6 fixes)
5. **AFPointsUsed** - Add decode function (5 fixes)

### Phase 3: Formatting Fixes
1. **LensSpec** - Format as "MinFL-MaxFL mm f/MinAp-MaxAp" (18 fixes)
2. **SequenceNumber** - Check formatting (8 fixes)
3. **LateralChromaticAberration** - Check formatting (5 fixes)

### Phase 4: Lens Database
1. **LensID** - Verify Sony lens lookup is working (6 fixes)
   - Reference: `exiftool/lib/Image/ExifTool/Sony.pm` - lens tables

## Reference Files
- `src/makernotes/sony.rs` - Main Sony MakerNote parser
- `exiftool/lib/Image/ExifTool/Sony.pm` - ExifTool Sony module
- `exiv2/src/sonymn_int.cpp` - exiv2 Sony implementation

## Key Tags to Check in Sony.pm

### SoftSkinEffect (tag 0x200F)
```perl
0x200f => {
    Name => 'SoftSkinEffect',
    PrintConv => {
        0 => 'Off',
        1 => 'Low',
        2 => 'Mid',
        3 => 'High',
    },
},
```

### AFPointSelected
Look for AFPointSelected in Sony.pm PrintConv.

### Quality
Check Quality tag decode values.

## Testing
```bash
./bin/mfr-test sony --save-baseline
# Make changes
./bin/mfr-test sony --check
```

## Notes
- Sony has multiple MakerNote formats across camera generations
- Some tags have camera-specific decode values
- Lens database is large and complex (Sony A-mount, E-mount, third-party)
