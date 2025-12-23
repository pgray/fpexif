# Tag Parity Plan: RAW_FUJI_X30.RAF

## Overview
- **Camera**: Fujifilm X30
- **File**: test-data/RAW_FUJI_X30.RAF
- **Reference**: test-data/RAW_FUJI_X30.json (exiftool output)

## Current Status
fpexif extracts basic EXIF/TIFF tags but lacks Fuji MakerNote decoding and RAF-specific metadata.

---

## Missing Tags by Category

### 1. RAF File Header Tags
**Priority: HIGH** - RAF-specific structure

| Tag | ExifTool Value | Notes |
|-----|----------------|-------|
| RAFVersion | 0101 | From RAF header |
| RAFCompression | Uncompressed | From RAF header |

### 2. Basic Fuji MakerNote Tags
**Priority: HIGH**

| Tag ID | Tag Name | ExifTool Value | Notes |
|--------|----------|----------------|-------|
| 0x0000 | Version | 0130 | |
| 0x0010 | InternalSerialNumber | FC  B2595279     Y31954 2014:12:17 3EA330218443 | Complex string |
| 0x1000 | Quality | NORMAL  | Note trailing space |
| 0x1001 | Sharpness | Normal | |
| 0x1002 | WhiteBalance | Auto | |
| 0x1003 | Saturation | 0 (normal) | |
| 0x100a | WhiteBalanceFineTune | Red +0, Blue +0 | Two-part value |
| 0x100b | NoiseReduction | 0 (normal) | |
| 0x1010 | FujiFlashMode | Off | Different from standard Flash tag |
| 0x1011 | FlashExposureComp | 0 | |
| 0x1020 | Macro | Off | |
| 0x1021 | FocusMode | Auto | |
| 0x1022 | AFMode | Single Point | |
| 0x1023 | FocusPixel | 1023 768 | X Y coordinates |
| 0x1030 | SlowSync | Off | |
| 0x1031 | PictureMode | Aperture-priority AE | |
| 0x1032 | ExposureCount | 1 | |
| 0x1040 | ShadowTone | 0 (normal) | X-Trans feature |
| 0x1041 | HighlightTone | 0 (normal) | X-Trans feature |
| 0x1044 | DigitalZoom | 0 | |
| 0x1045 | LensModulationOptimizer | On | |
| 0x1050 | ShutterType | Mechanical | vs Electronic |

### 3. Bracketing and Sequence
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| AutoBracketing | Off |
| SequenceNumber | 0 |

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
**Priority: HIGH** - Key Fuji feature

| Tag | ExifTool Value |
|-----|----------------|
| FilmMode | F0/Standard (Provia) |

### 7. Image Stabilization
**Priority: MEDIUM**

| Tag | ExifTool Value | Notes |
|-----|----------------|-------|
| ImageStabilization | Optical; On (mode 2, shooting only); 0 | Complex multi-part |

### 8. Face Detection
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| FacesDetected | 0 |
| NumFaceElements | 0 |

### 9. Rating
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| Rating | 0 |
| ImageGeneration | Original Image |

### 10. RAF Raw Data Tags
**Priority: HIGH** - From RAF structure, not EXIF

| Tag | ExifTool Value | Notes |
|-----|----------------|-------|
| RawImageFullSize | 4096x3012 | |
| RawImageCropTopLeft | 6 16 | |
| RawImageCroppedSize | 4000x3000 | |
| FujiLayout | 12 12 12 12 | |
| XTransLayout | GBGGRG RGRBGB GBGGRG GRGGBG BGBRGR GRGGBG | X-Trans CFA |
| RawExposureBias | -0.6 | |
| RawImageWidth | 4032 | |
| RawImageHeight | 3012 | |
| RawImageFullWidth | 4096 | |
| RawImageFullHeight | 3012 | |
| BitsPerSample | 12 | |
| StripOffsets | 1085952 | |
| StripByteCounts | 18505728 | |
| BlackLevel | 256 256 256... (36 values) | Per X-Trans position |
| GeometricDistortionParams | (complex array) | Lens correction |
| WB_GRBLevelsStandard | 302 345 814 17 302 592 508 21 | |
| WB_GRBLevelsAuto | 302 502 712 | |
| WB_GRBLevels | 302 502 712 | |
| ChromaticAberrationParams | (complex array) | CA correction |
| VignettingParams | (complex array) | Vignette correction |

### 11. Derived/Calculated Values
**Priority: LOW**

| Tag | ExifTool Value | Notes |
|-----|----------------|-------|
| BlueBalance | 2.357616 | From WB levels |
| RedBalance | 1.662252 | From WB levels |
| ImageSize | 4000x3000 | Cropped size |
| Megapixels | 12.0 | |
| ScaleFactor35efl | 3.9 | Compact sensor |
| CircleOfConfusion | 0.008 mm | |
| FOV | 49.2 deg | |
| FocalLength35efl | 39.3 mm | |
| HyperfocalDistance | 2.34 m | |
| LightValue | 9.6 | |

---

## Implementation Plan

### Phase 1: RAF File Structure
**Priority: HIGH**

1. Parse RAF header for version and compression info
2. Extract JPEG preview offset/length
3. Parse RAF directory structure
4. Extract raw image dimensions and offsets

### Phase 2: Fuji MakerNote Core Tags
**Priority: HIGH**

1. Implement tag constants for all Fuji tags
2. Implement `get_fuji_tag_name()` function
3. Implement decoders:
   - `decode_quality_exiftool()` / `decode_quality_exiv2()`
   - `decode_sharpness_exiftool()` / `decode_sharpness_exiv2()`
   - `decode_white_balance_exiftool()` / `decode_white_balance_exiv2()`
   - `decode_saturation_exiftool()` / `decode_saturation_exiv2()`
   - `decode_noise_reduction_exiftool()` / `decode_noise_reduction_exiv2()`
   - `decode_flash_mode_exiftool()` / `decode_flash_mode_exiv2()`
   - `decode_macro_exiftool()` / `decode_macro_exiv2()`
   - `decode_focus_mode_exiftool()` / `decode_focus_mode_exiv2()`
   - `decode_af_mode_exiftool()` / `decode_af_mode_exiv2()`
   - `decode_slow_sync_exiftool()` / `decode_slow_sync_exiv2()`
   - `decode_picture_mode_exiftool()` / `decode_picture_mode_exiv2()`
   - `decode_shadow_tone_exiftool()` / `decode_shadow_tone_exiv2()`
   - `decode_highlight_tone_exiftool()` / `decode_highlight_tone_exiv2()`
   - `decode_shutter_type_exiftool()` / `decode_shutter_type_exiv2()`

### Phase 3: Film Simulation
**Priority: HIGH**

1. Implement FilmMode decoder with all film simulation types:
   - F0/Standard (Provia)
   - F1a/Studio Portrait
   - F1b/Studio Portrait Smooth
   - F1c/Studio Portrait Enhanced
   - F2/Fujichrome (Velvia)
   - F3/Pro Neg Std
   - F4/Pro Neg Hi
   - FC/Classic Chrome
   - (many more...)

### Phase 4: Dynamic Range
**Priority: HIGH**

1. Implement DynamicRange decoder
2. Implement DynamicRangeSetting decoder
3. Parse DevelopmentDynamicRange value

### Phase 5: RAF Raw Data
**Priority: MEDIUM**

1. Parse RAF raw data header
2. Extract:
   - RawImageFullSize
   - RawImageCropTopLeft
   - RawImageCroppedSize
   - FujiLayout
   - XTransLayout (for X-Trans sensors)
   - RawExposureBias
3. Parse BlackLevel array (36 values for X-Trans)
4. Parse lens correction parameters:
   - GeometricDistortionParams
   - ChromaticAberrationParams
   - VignettingParams

### Phase 6: White Balance Data
**Priority: MEDIUM**

1. Parse WB_GRBLevels (different from Canon RGGB order)
2. Calculate RedBalance, BlueBalance
3. Handle WB_GRBLevelsStandard multi-value format

### Phase 7: Other Tags
**Priority: LOW**

1. InternalSerialNumber parsing
2. WhiteBalanceFineTune parsing
3. FocusPixel coordinates
4. ImageStabilization multi-value parsing
5. Face detection data
6. Rating and ImageGeneration

### Phase 8: Warning Flags
**Priority: LOW**

1. BlurWarning decoder
2. FocusWarning decoder
3. ExposureWarning decoder

### Phase 9: Calculated Values
**Priority: LOW**

1. Implement 35mm equivalent calculations for compact sensor
2. Implement DOF, FOV calculations

---

## Fuji-Specific Notes

### RAF File Format
- Magic: "FUJIFILMCCD-RAW "
- Version in header (0101, 0110, 0200, etc.)
- JPEG preview embedded
- Multiple IFDs for different data
- X-Trans sensors have 6x6 CFA pattern

### X-Trans CFA
The X30 uses an X-Trans sensor with a unique 6x6 color filter array:
```
GBGGRG
RGRBGB
GBGGRG
GRGGBG
BGBRGR
GRGGBG
```
This requires different demosaicing than standard Bayer sensors.

### Film Simulation
Fuji's key differentiator - extensive film simulation modes based on classic film stocks.

### Tone Curves
ShadowTone and HighlightTone are X-Trans specific tone curve adjustments.

---

## Reference Files
- ExifTool: `exiftool/lib/Image/ExifTool/FujiFilm.pm`
- exiv2: `exiv2/src/fujimn_int.cpp`

## Test Command
```bash
./target/debug/fpexif exiftool test-data/RAW_FUJI_X30.RAF | sort > /tmp/fpexif.txt
exiftool -s test-data/RAW_FUJI_X30.RAF | sort > /tmp/exiftool.txt
diff /tmp/fpexif.txt /tmp/exiftool.txt
```
