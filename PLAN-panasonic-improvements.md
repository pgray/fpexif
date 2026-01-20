# Panasonic EXIF Improvements Plan

## Current State (2026-01-18)
- Match rate (raws): 86.5%
- Match rate (data.lfs): 77.4% (+0.3% from FacesDetected name fix)
- Files tested: 20 RW2 (raws), 398 RW2 (data.lfs)

### Tag Name Fix (2026-01-18)
- Fixed `FaceDetected` -> `FacesDetected` (now matches ExifTool's naming)
- Matching tags: 63333 -> 63525 (+192)
- Note: Creates mismatches due to Yes/No vs 0/1 formatting difference

## Completed Work

### RW2 IFD0 Parsing (2026-01-18)
Added full parsing of Panasonic RAW IFD0 tags, which dramatically improved match rates:
- **Before**: 59.8% (raws), 54.2% (data.lfs)
- **After**: 86.5% (raws), 77.1% (data.lfs)

Implemented in `src/formats/tiff.rs`:
- `Rw2Metadata` struct for RW2-specific tags
- `extract_rw2_metadata_if_rw2()` - main extraction function
- `extract_rw2_ifd0_metadata()` - IFD0 tag parser
- `parse_rw2_wb_info()` - WBInfo/WBInfo2 subdirectory parser
- `parse_rw2_distortion_info()` - DistortionInfo subdirectory parser

Tags now extracted:
- Sensor: `SensorWidth`, `SensorHeight`, `SensorTopBorder`, `SensorLeftBorder`, `SensorBottomBorder`, `SensorRightBorder`
- Image: `CFAPattern`, `BitsPerSample`, `SamplesPerPixel`, `RowsPerStrip`, `PanasonicRawVersion`, `RawFormat`
- White Balance: `WBRedLevel`, `WBGreenLevel`, `WBBlueLevel`, `NumWBEntries`, `WBType1-7`, `WB_RGBLevels1-7`
- Black Levels: `BlackLevelRed`, `BlackLevelGreen`, `BlackLevelBlue`
- Linearity: `LinearityLimitRed`, `LinearityLimitGreen`, `LinearityLimitBlue`
- Distortion: `DistortionCorrection`, `DistortionScale`, `DistortionParam02/04/08/09/11`, `DistortionN`
- High ISO: `HighISOMultiplierRed/Green/Blue`
- Crop: `CropTop`, `CropLeft`, `CropBottom`, `CropRight`
- Computed: `RedBalance`, `BlueBalance` (from WB levels)

## Remaining Issues

### Top Missing Tags (data.lfs)
| Tag | Count | Notes |
|-----|-------|-------|
| FocalLengthIn35mmFormat | 133 | Precision mismatch |
| CircleOfConfusion | 78 | Precision mismatch |
| FacesDetected | 61 | MakerNote tag |
| NoiseReductionParams | 57 | Binary data parsing |
| PrintIMVersion | 53 | Print Image Matching |
| NumFacePositions | 52 | MakerNote tag |
| AdvancedSceneMode | 50 | MakerNote decode needed |
| InternalNDFilter | 50 | MakerNote tag |
| FacesRecognized | 50 | MakerNote tag |

### MakerNote Decode Issues
| Tag | Issue | Fix |
|-----|-------|-----|
| AdvancedSceneMode | Numeric output instead of decode | Add decode_advanced_scene_mode_exiftool |
| InternalSerialNumber | Raw bytes instead of decoded | Parse "(X15) 2014:06:12 no. 0153" format |
| ContrastMode | Wrong decode values | Fix decode_contrast_mode_exiftool |
| FlashBias | Minor formatting | Check format differences |
| PitchAngle/RollAngle | Precision formatting | Match ExifTool decimal places |
| AccessoryType | Empty vs "NO-ACCESSORY" | Add default values |
| City/Landmark | Binary data vs empty string | Parse location data |
| BabyAge | Empty vs "(not set)" | Add default text |

## Next Steps

### Phase 1: Precision Fixes
1. **FocalLengthIn35mmFormat** - Match ExifTool rounding
2. **CircleOfConfusion** - Match precision

### Phase 2: MakerNote Decodes
1. **AdvancedSceneMode** - Add lookup table from Panasonic.pm
2. **ContrastMode** - Fix decode values
3. **InternalSerialNumber** - Parse encoded format

### Phase 3: Face Detection
1. **FacesDetected** - Parse face detect info
2. **NumFacePositions** - Parse from FaceDetectInfo
3. **FacesRecognized** - Parse recognition data

## Reference Files
- `src/formats/tiff.rs` - RW2 IFD0 parsing (Rw2Metadata)
- `src/makernotes/panasonic.rs` - Panasonic MakerNote parser
- `exiftool/lib/Image/ExifTool/Panasonic.pm` - ExifTool Panasonic module
- `exiftool/lib/Image/ExifTool/PanasonicRaw.pm` - ExifTool PanasonicRaw module
- `exiv2/src/panasonicmn_int.cpp` - exiv2 Panasonic implementation

## Testing
```bash
./bin/mfr-test panasonic --save-baseline
# Make changes
./bin/mfr-test panasonic --check
./bin/mfr-test panasonic --data-lfs  # Test larger dataset
```
