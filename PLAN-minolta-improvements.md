# Minolta EXIF Improvements Plan

## Current State (2025-12-31)
- Match rate: 37.1%
- Files tested: 8 MRW
- Matching: 326 | Mismatched: 137 | Missing: 415 | Extra: 271

## Top Mismatches by Frequency

| Tag | Count | Issue | Fix |
|-----|-------|-------|-----|
| SubjectArea | ~8 | Array formatting | "[1504,1000,256,304]" vs "1504 1000 256 304" |
| MakerNoteVersion | ~8 | Hex vs ASCII | "4d 4c 54 30" vs "MLT0" |
| LensID | ~8 | Missing lens database | Need Minolta/Sony A-mount lens lookup |
| Teleconverter | ~8 | Numeric vs decoded | "0" vs "None" |
| RawAndJpgRecording | ~5 | Numeric vs decoded | "0" vs "Off" |
| FocusMode | ~5 | Different decode strings | "Auto Focus" vs "AF" |
| FocusDistance | ~5 | Formatting | "0" vs "0.5 m" |
| FocusArea | ~5 | Decode mismatch | "Wide Focus (normal)" vs "Spot Focus" |
| FocalLength | ~5 | Precision | "7.21484375 mm" vs "7.2 mm" |
| DriveMode | ~5 | Decode strings | "Single Frame" vs "Single" |
| MinoltaModelID | ~3 | Wrong model | Model ID lookup issue |

## Implementation Steps

### Phase 1: Formatting Fixes
1. **SubjectArea** - Use space-separated format
2. **MakerNoteVersion** - Convert hex to ASCII string
3. **FocalLength** - Round to 1 decimal place
4. **FocusDistance** - Add "m" unit, handle zero case

### Phase 2: Decode Function Fixes
1. **Teleconverter** - Add decode: 0 => "None"
2. **RawAndJpgRecording** - Add decode: 0 => "Off"
3. **FocusMode** - Use short form: "Auto Focus" => "AF", "Manual Focus" => "MF"
4. **DriveMode** - Use short form: "Single Frame" => "Single"

### Phase 3: Lens Database
1. **LensID** - Implement Minolta/Sony A-mount lens lookup
   - Reference: `exiftool/lib/Image/ExifTool/Minolta.pm` - %minoltaLensTypes

### Phase 4: Model ID
1. **MinoltaModelID** - Fix model ID lookup table

## Reference Files
- `src/makernotes/minolta.rs` - Main Minolta MakerNote parser (TODO: Create or extend)
- `exiftool/lib/Image/ExifTool/Minolta.pm` - ExifTool Minolta module
- `exiv2/src/minoltamn_int.cpp` - exiv2 Minolta implementation

## Notes
- Minolta was acquired by Sony; A-mount lens database shared
- Some tags have camera-generation-specific values
- DiMAGE series vs Dynax/Maxxum series have different structures

## Testing
```bash
./bin/mfr-test minolta --save-baseline
# Make changes
./bin/mfr-test minolta --check
```
