# Tag Parity Plan: DSCF0062.RAF (Fuji X70)

## Overview
- **Camera**: Fujifilm X70 (APS-C compact)
- **File**: test-data/DSCF0062.RAF
- **Reference**: test-data/DSCF0062.json (exiftool output)

## Current Status
fpexif extracts basic EXIF/TIFF tags but lacks Fuji MakerNote decoding and RAF-specific metadata.

---

## Comparison with X30

The X70 shares most MakerNote structure with the X30, but has key differences:
- **Sensor**: APS-C (1.5x crop) vs 2/3" (3.9x crop)
- **RAF Version**: 0110 vs 0101
- **X-Trans Layout**: Different 6x6 pattern
- **BitsPerSample**: 14-bit vs 12-bit
- **Fixed lens**: 18.5mm f/2.8 (28mm equiv) vs 7.1-28.4mm zoom

---

## Missing Tags by Category

### 1. RAF File Header Tags
**Priority: HIGH**

| Tag | ExifTool Value | Notes |
|-----|----------------|-------|
| RAFVersion | 0110 | Newer than X30's 0101 |
| RAFCompression | Uncompressed | |

### 2. Basic Fuji MakerNote Tags
**Priority: HIGH**

| Tag ID | Tag Name | ExifTool Value | Notes |
|--------|----------|----------------|-------|
| 0x0000 | Version | 0130 | Same as X30 |
| 0x0010 | InternalSerialNumber | FC  B2923690     Y31254 2016:04:13 5CD330116219 | |
| 0x1000 | Quality | NORMAL  | |
| 0x1001 | Sharpness | Normal | |
| 0x1002 | WhiteBalance | Auto | |
| 0x1003 | Saturation | 0 (normal) | |
| 0x100a | WhiteBalanceFineTune | Red +0, Blue +0 | |
| 0x100b | NoiseReduction | +1 (medium strong) | Different from X30 |
| 0x1010 | FujiFlashMode | Off | |
| 0x1011 | FlashExposureComp | 0 | |
| 0x1021 | FocusMode | Auto | |
| 0x1022 | AFMode | Zone | Different from X30's Single Point |
| 0x1023 | FocusPixel | 960 640 | |
| 0x1030 | SlowSync | Off | |
| 0x1031 | PictureMode | Aperture-priority AE | |
| 0x1032 | ExposureCount | 1 | |
| 0x1040 | ShadowTone | +2 (hard) | Different from X30 |
| 0x1041 | HighlightTone | +1 (medium hard) | Different from X30 |
| 0x1044 | DigitalZoom | 0 | |
| 0x1045 | LensModulationOptimizer | On | |
| 0x1050 | ShutterType | Mechanical | |

**Note**: X70 doesn't have Macro tag (fixed lens camera with close focus built-in)

### 3. Bracketing and Sequence
**Priority: MEDIUM**

| Tag | ExifTool Value | Notes |
|-----|----------------|-------|
| AutoBracketing | On | Different from X30 |
| SequenceNumber | 3 | Third shot in bracket |

### 4. Warning Flags
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| BlurWarning | Blur Warning |
| FocusWarning | Good |
| ExposureWarning | Good |

### 5. Dynamic Range
**Priority: HIGH**

| Tag | ExifTool Value |
|-----|----------------|
| DynamicRange | Standard |
| DynamicRangeSetting | Manual |
| DevelopmentDynamicRange | 100 |

### 6. Film Simulation
**Priority: HIGH**

| Tag | ExifTool Value | Notes |
|-----|----------------|-------|
| FilmMode | F2/Fujichrome (Velvia) | Different from X30's Provia |

### 7. Face Detection
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| FacesDetected | 0 |
| NumFaceElements | 0 |

### 8. Rating
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| Rating | 0 |
| ImageGeneration | Original Image |

### 9. RAF Raw Data Tags
**Priority: HIGH**

| Tag | ExifTool Value | Notes |
|-----|----------------|-------|
| RawImageFullSize | 4992x3296 | Larger than X30 |
| RawImageCropTopLeft | 16 24 | |
| RawImageCroppedSize | 4896x3264 | 16MP APS-C |
| FujiLayout | 12 12 12 12 | |
| XTransLayout | BGGRGG RGGBGG GBRGRB RGGBGG BGGRGG GRBGBR | Different pattern from X30 |
| RawExposureBias | -1.4 | |
| RawImageWidth | 4936 | |
| RawImageHeight | 3296 | |
| RawImageFullWidth | 4992 | |
| RawImageFullHeight | 3296 | |
| BitsPerSample | 14 | Higher than X30's 12-bit |
| StripOffsets | 904704 | |
| StripByteCounts | 32907264 | |
| BlackLevel | 1026 1026 1026... (36 values) | Higher than X30's 256 |
| GeometricDistortionParams | (complex array) | |
| WB_GRBLevelsStandard | 302 372 684 17 302 655 399 21 | |
| WB_GRBLevelsAuto | 302 625 440 | |
| WB_GRBLevels | 302 625 440 | |
| ChromaticAberrationParams | (complex array) | |
| VignettingParams | (complex array) | |

### 10. Derived/Calculated Values
**Priority: LOW**

| Tag | ExifTool Value | Notes |
|-----|----------------|-------|
| BlueBalance | 1.456954 | |
| RedBalance | 2.069536 | |
| ImageSize | 4896x3264 | |
| Megapixels | 16.0 | |
| ScaleFactor35efl | 1.5 | APS-C crop factor |
| CircleOfConfusion | 0.020 mm | |
| FOV | 64.7 deg | |
| FocalLength35efl | 28.4 mm | |
| HyperfocalDistance | 6.25 m | |
| LightValue | 1.9 | Low light shot |

---

## X70-Specific Differences from X30

### Sensor
- APS-C sensor (23.6 x 15.6 mm) vs 2/3" (8.8 x 6.6 mm)
- 16MP vs 12MP
- 14-bit depth vs 12-bit
- 1.5x crop factor vs 3.9x

### X-Trans Pattern
The X70 uses a different X-Trans II layout:
```
BGGRGG
RGGBGG
GBRGRB
RGGBGG
BGGRGG
GRBGBR
```

### BlackLevel
Higher black level (1026 vs 256) due to 14-bit data

### No Macro Tag
Fixed lens camera - close focus is built into the lens design

### AF Modes
Supports Zone AF mode in addition to Single Point

---

## Implementation Plan

Most implementation is shared with X30 plan. X70-specific items:

### Phase 1: Handle RAF Version 0110
1. Parse newer RAF header format
2. Handle differences in raw data structure

### Phase 2: 14-bit Data Support
1. Adjust black level parsing for higher values
2. Handle 14-bit strip data

### Phase 3: X-Trans II Layout
1. Detect X-Trans II vs X-Trans I
2. Parse different 6x6 CFA pattern

### Phase 4: AFMode Zone
1. Add Zone AF mode to decoder

### Phase 5: Extended Tone Values
1. Handle +2 (hard), +1 (medium hard) values for ShadowTone/HighlightTone

### Phase 6: Bracketing Info
1. Ensure AutoBracketing On/Off decoder
2. Handle SequenceNumber for bracket sequences

---

## Shared Implementation with X30

The following can be directly reused:
- All MakerNote tag constants
- All decode functions (Quality, WhiteBalance, FocusMode, etc.)
- FilmMode decoder (same film simulations available)
- DynamicRange decoders
- Warning flag decoders
- Face detection parsing
- WB level calculations

---

## Reference Files
- ExifTool: `exiftool/lib/Image/ExifTool/FujiFilm.pm`
- exiv2: `exiv2/src/fujimn_int.cpp`

## Test Command
```bash
./target/debug/fpexif exiftool test-data/DSCF0062.RAF | sort > /tmp/fpexif.txt
exiftool -s test-data/DSCF0062.RAF | sort > /tmp/exiftool.txt
diff /tmp/fpexif.txt /tmp/exiftool.txt
```
