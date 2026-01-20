# Sony EXIF Improvements Plan

## Current State (Updated 2026-01-18)
- Match rate (raws): 75.8%
- Match rate (data.lfs): 81.3%
- Files tested: 1 SRF, 1 SR2, 31 ARW (raws), 195 ARW (data.lfs)
- Mismatches: 0

### Tag9402 Parsing (2026-01-18)
Added parsing for Sony's encrypted Tag9402 subdirectory which contains:
- **AFAreaMode** (offset 0x17) - Multi, Center, Spot, Flexible Spot, Zone, Expanded Flexible Spot, Tracking, Face Tracking, etc.
- **FocusMode** (offset 0x16) - Manual, AF-S, AF-C, AF-A, DMF
- **AmbientTemperature** (offset 0x04) - Only when TempTest1 flag (0x02) equals 255
- **FocusPosition2** (offset 0x2d) - For NEX/ILCE models

Model-specific handling: SLT/ILCA models use AFAreaModeSetting (0x201C) instead of Tag9402 AFAreaMode.

## Previous State
- Match rate: 36.6% (2025-12-31)
- Significant improvements came from CameraSettings parsing and other subdirectory work

## Top Mismatches by Frequency

| Tag | Count | Issue | Fix |
|-----|-------|-------|-----|
| Sharpness/Contrast/Saturation | ~15 | Returns "Normal" instead of "0" | ExifTool outputs numeric values for some cameras |
| AFMode | ~10 | Returns "Unknown" instead of correct value | Fix AFMode decode function |
| WhiteBalance | ~10 | Wrong decode values | Fix decode_white_balance_exiftool |
| ImageSize/Megapixels | ~8 | Wrong dimensions for some cameras | Check sensor vs output dimensions |
| FocusMode | ~5 | Decode mismatch | Verify decode values |
| LensID | ~5 | Lens lookup failing | Fix lens database lookup |
| ExposureMode | ~5 | Decode values | Check decode function |

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
