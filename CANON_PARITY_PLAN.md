# Canon 50D JSON Parity Plan

## Overview
Total missing tags: **201**
Value differences: **17**

## Priority Levels

### P0: Critical Value Fixes (must fix for correctness)
These are tags we output but with wrong values:

| Tag | Expected | Actual | Fix |
|-----|----------|--------|-----|
| SerialNumber | `"0039900513"` | `39900513` | Keep leading zeros, output as string |
| MaxAperture | `5.2` | `"5.2"` | Output as number, not string |
| MinAperture | `33` | `"33.4"` | Different calculation - check formula |
| MeasuredEV/MeasuredEV2 | `6.62` | `"6.62"` | Output as number, not string |
| TargetAperture | `5.7` | `"5.7"` | Output as number, not string |
| NumAFPoints | `9` | `2` | Wrong parsing of AFInfo2 array |
| AFAreaWidths/Heights/Positions | arrays | wrong arrays | AFInfo2 parsing is broken |
| AFAreaMode | `"Single-point AF"` | `"Face AiAF"` | Wrong index or decode |
| TargetExposureTime | `"1/81"` | `"1/83"` | Rounding difference |
| StripOffsets/StripByteCounts | large values | small values | Reading wrong IFD |

### P1: High Priority Missing Tags (~30 tags)

**ProcessingInfo (tag 0x00A0) - 13 tags**
Already parsing but not extracting all fields:
- `PictureStyle`, `ToneCurve`, `Sharpness`, `SharpnessFrequency`
- `SharpnessStandard`, `SharpnessPortrait`, `SharpnessLandscape`, etc.
- Need to decode processing info sub-array fully

**ColorData (tag 0x4001) - 30 tags**
Currently skipped as "large binary blob":
- `ColorTemperature`, `ColorTempAsShot`, `ColorTempAuto`, etc.
- `WB_RGGBLevelsAsShot`, `WB_RGGBLevelsDaylight`, etc.
- Need to parse ColorData structure (version-dependent)

**Computed/Derived fields - 18 tags**
These are calculated by ExifTool, not stored in file:
- `Lens` = MinFocalLength + " - " + MaxFocalLength + " mm"
- `LensID` = same as LensType
- `ShutterSpeed` = same as ExposureTime
- `DriveMode` = composite from ContinuousDrive
- `ImageSize` = Width + "x" + Height
- `Megapixels` = (Width * Height) / 1000000
- `FOV`, `DOF`, `HyperfocalDistance`, `CircleOfConfusion` = optical calculations
- `ScaleFactor35efl`, `FocalLength35efl`, `Lens35efl` = crop factor calculations

### P2: Medium Priority (~50 tags)

**SensorInfo (tag 0x00E0) - 20 tags**
- `SensorWidth`, `SensorHeight`, `SensorLeftBorder`, etc.
- `CroppedImageWidth`, `CroppedImageHeight`, etc.
- `BlackMaskLeftBorder`, `BlackMaskRightBorder`, etc.

**AFInfo2/3 improvements - 13 tags**
Fix current parsing + add:
- `AFPointsInFocus`, `AFPointsSelected`
- `AFMicroAdjMode`, `AFMicroAdjValue`, `AFMicroadjustment`
- `AFAssistBeam`, `AFPointSelectionMethod`
- `FocusDistanceLower`, `FocusDistanceUpper`

**AspectInfo (tag 0x009A) - 5 tags**
- `AspectRatio`, `CroppedImageWidth`, `CroppedImageHeight`, etc.

### P3: Lower Priority (~100 tags)

**Custom Functions (tag 0x000F, 0x0099)**
- 50+ custom function settings
- Complex model-dependent decoding

**Other MakerNote tags**
- `VignettingCorrVersion`, `VRDOffset`
- `ContrastFaithful`, `ContrastLandscape`, etc.
- `SaturationFaithful`, `SaturationLandscape`, etc.
- Various camera-specific settings

**File metadata**
- `ThumbnailOffset`, `ThumbnailLength` (from IFD1)
- `PreviewImageStart`, `PreviewImageLength`
- `CanonImageWidth`, `CanonImageHeight`

---

## Implementation Order

### Phase 1: Fix Value Errors (P0)
1. Fix numeric vs string output for aperture/EV values
2. Fix SerialNumber to preserve leading zeros
3. Fix AFInfo2 parsing (NumAFPoints, AFArea* arrays)
4. Fix StripOffsets/StripByteCounts IFD reading

### Phase 2: Add ProcessingInfo + ColorData (P1)
1. Expand ProcessingInfo decoding with all sharpness/tone fields
2. Add ColorData parsing (ColorTemperature, WB levels)
3. Add computed fields (Lens, LensID, ImageSize, Megapixels)

### Phase 3: Add SensorInfo + AF improvements (P2)
1. Parse SensorInfo array
2. Parse AspectInfo array
3. Improve AF field decoding

### Phase 4: Custom Functions + remaining (P3)
1. Add CustomFunctions parsing
2. Add remaining MakerNote fields
3. Add IFD1 thumbnail info

---

## Quick Wins (can do immediately)

1. **LensID** - just duplicate LensType value
2. **Lens** - format from MinFocalLength/MaxFocalLength
3. **ImageSize** - format Width + "x" + Height
4. **Megapixels** - calculate from dimensions
5. **ShutterSpeed** - alias for ExposureTime
6. **Fix numeric types** - MaxAperture, MeasuredEV, TargetAperture as numbers

## Files to Modify

- `src/makernotes/canon.rs` - Main Canon parsing
- `src/output.rs` - JSON formatting, computed fields
- `src/lib.rs` or `src/parser.rs` - IFD reading for thumbnails
