# Tag Parity Plan: RAW_CANON_50D.CR2

## Overview
- **Camera**: Canon EOS 50D
- **File**: test-data/RAW_CANON_50D.CR2
- **Reference**: test-data/RAW_CANON_50D.json (exiftool output)

## Current Status
fpexif extracts basic EXIF/TIFF tags but lacks most Canon MakerNote decoding.

---

## Missing Tags by Category

### 1. Canon CameraSettings (Tag 0x0001)
**Priority: HIGH** - Core shooting parameters

| Tag | ExifTool Value | Notes |
|-----|----------------|-------|
| MacroMode | Normal | |
| SelfTimer | Off | |
| Quality | RAW | |
| CanonFlashMode | Off | |
| ContinuousDrive | Continuous, High | |
| FocusMode | One-shot AF | |
| RecordMode | CR2 | |
| CanonImageSize | n/a | |
| EasyMode | Manual | |
| DigitalZoom | None | |
| Contrast | Normal | |
| Saturation | Normal | |
| MeteringMode | Center-weighted average | Canon-specific value |
| FocusRange | Not Known | |
| CanonExposureMode | Manual | |
| LensType | Canon EF-S 18-200mm f/3.5-5.6 IS | LensID lookup |
| MaxFocalLength | 200 mm | |
| MinFocalLength | 18 mm | |
| FocalUnits | 1/mm | |
| MaxAperture | 5.2 | |
| MinAperture | 33 | |
| FlashActivity | 0 | |
| FlashBits | (none) | |
| ZoomSourceWidth | 0 | |
| ZoomTargetWidth | 0 | |
| ManualFlashOutput | n/a | |
| ColorTone | Normal | |
| SRAWQuality | n/a | |

### 2. Canon FocalLength (Tag 0x0002)
**Priority: HIGH**

| Tag | ExifTool Value | Notes |
|-----|----------------|-------|
| FocalType | (implied) | Zoom vs Prime |
| FocalLength | 70.0 mm | Already in EXIF, redundant |

### 3. Canon ShotInfo (Tag 0x0004)
**Priority: HIGH**

| Tag | ExifTool Value | Notes |
|-----|----------------|-------|
| AutoISO | 100 | |
| BaseISO | 800 | |
| MeasuredEV | 6.62 | |
| TargetAperture | 5.7 | |
| TargetExposureTime | 1/81 | |
| ExposureCompensation | 0 | |
| WhiteBalance | Auto | Canon-specific enum |
| SlowShutter | None | |
| SequenceNumber | 0 | |
| OpticalZoomCode | n/a | |
| CameraTemperature | 26 C | |
| FlashGuideNumber | 0 | |
| FlashExposureComp | 0 | |
| AutoExposureBracketing | Off | |
| AEBBracketValue | 0 | |
| ControlMode | Camera Local Control | |
| MeasuredEV2 | 7.125 | |
| BulbDuration | 0 | |
| CameraType | EOS High-end | |
| NDFilter | n/a | |

### 4. Canon ImageType (Tag 0x0006) & FirmwareVersion (Tag 0x0007)
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| CanonImageType | Canon EOS 50D |
| CanonFirmwareVersion | Firmware Version 2.9.1 |

### 5. Canon FileInfo (Tag 0x0093)
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| FileNumber | 101-9600 |
| FileIndex | 9600 |
| DirectoryIndex | 101 |
| ShutterMode | Mechanical |
| FlashExposureLock | Off |

### 6. Canon SerialNumber/OwnerName (Tags 0x000c, 0x0009)
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| SerialNumber | 0039900513 |
| OwnerName | (empty) |

### 7. Canon ModelID (Tag 0x0010)
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| CanonModelID | EOS 50D |

### 8. Canon AFInfo (Tag 0x0012) / AFInfo2 (Tag 0x0026)
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| AFAreaMode | Single-point AF |
| NumAFPoints | 9 |
| ValidAFPoints | 9 |
| CanonImageWidth | 4752 |
| CanonImageHeight | 3168 |
| AFImageWidth | 4752 |
| AFImageHeight | 3168 |
| AFAreaWidths | 99 80 80 80 118 80 80 80 99 |
| AFAreaHeights | 79 98 98 98 120 98 98 98 79 |
| AFAreaXPositions | 0 -788 788 -1254 0 1254 -788 788 0 |
| AFAreaYPositions | 681 361 361 0 0 0 -361 -361 -681 |
| AFPointsInFocus | 4 |
| AFPointsSelected | 4 |

### 9. Canon PictureStyle (Tags 0x00a0, 0x0096)
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| ToneCurve | Standard |
| Sharpness | 3 |
| PictureStyle | Standard |
| ContrastStandard | 0 |
| SharpnessStandard | 3 |
| SaturationStandard | 0 |
| (... many more per-style values) | |
| UserDef1PictureStyle | Standard |
| UserDef2PictureStyle | Standard |
| UserDef3PictureStyle | Standard |

### 10. Canon ColorData (Tag 0x4001)
**Priority: LOW** - Large, complex structure

| Tag | ExifTool Value |
|-----|----------------|
| ColorDataVersion | 6 (50D/5DmkII) |
| WB_RGGBLevelsAsShot | 1872 1024 1024 1780 |
| ColorTempAsShot | 3948 |
| WB_RGGBLevelsAuto | 1872 1024 1024 1780 |
| (... many WB levels for different presets) | |
| AverageBlackLevel | 1026 1026 1026 1026 |
| PerChannelBlackLevel | 1022 1022 1030 1030 |
| NormalWhiteLevel | 14000 |
| SpecularWhiteLevel | 14512 |

### 11. Canon LensModel (Tag 0x0095)
**Priority: HIGH**

| Tag | ExifTool Value |
|-----|----------------|
| LensModel | EF-S18-200mm f/3.5-5.6 IS |
| InternalSerialNumber | P0001432 |

### 12. Canon CustomFunctions (Tag 0x000f, 0x0012)
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| ExposureLevelIncrements | 1/3 Stop |
| ISOSpeedIncrements | 1/3 Stop |
| ISOExpansion | On |
| LongExposureNoiseReduction | Off |
| HighISONoiseReduction | Standard |
| HighlightTonePriority | Disable |
| AutoLightingOptimizer | Standard |
| (... many more custom function settings) | |

### 13. Canon SensorInfo (Tag 0x00e0)
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| SensorWidth | 4832 |
| SensorHeight | 3228 |
| SensorLeftBorder | 72 |
| SensorTopBorder | 56 |
| SensorRightBorder | 4823 |
| SensorBottomBorder | 3223 |
| BlackMaskLeftBorder | 0 |
| BlackMaskTopBorder | 0 |
| BlackMaskRightBorder | 0 |
| BlackMaskBottomBorder | 0 |

### 14. Canon Processing Info (Tag 0x00a0)
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| SensorRedLevel | 0 |
| SensorBlueLevel | 0 |
| WhiteBalanceRed | 0 |
| WhiteBalanceBlue | 0 |
| ColorTemperature | 5200 |
| DigitalGain | 0 |
| WBShiftAB | 0 |
| WBShiftGM | 0 |

### 15. Canon VignettingCorr (Tag 0x40d0)
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| VignettingCorrVersion | 0 |
| PeripheralLighting | On |
| DistortionCorrection | Off |
| ChromaticAberrationCorr | Off |
| PeripheralLightingValue | 60 |
| DistortionCorrectionValue | 0 |

### 16. Canon AspectInfo (Tag 0x009a)
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| AspectRatio | 3:2 |
| CroppedImageWidth | 4752 |
| CroppedImageHeight | 3168 |
| CroppedImageLeft | 0 |
| CroppedImageTop | 0 |

### 17. Derived/Calculated Values
**Priority: LOW** - Calculated by ExifTool from raw values

| Tag | ExifTool Value | Notes |
|-----|----------------|-------|
| DriveMode | Continuous Shooting | From ContinuousDrive |
| Lens | 18.0 - 200.0 mm | From focal lengths |
| ShootingMode | Manual | From EasyMode |
| WB_RGGBLevels | 1872 1024 1024 1780 | Composite |
| BlueBalance | 1.738281 | Calculated |
| RedBalance | 1.828125 | Calculated |
| LensID | Canon EF-S 18-200mm f/3.5-5.6 IS | Lens lookup |
| ScaleFactor35efl | 1.6 | Crop factor |
| Lens35efl | 28.5 - 316.9 mm | Calculated |
| CircleOfConfusion | 0.019 mm | Calculated |
| DOF | 0.75 m (3.83 - 4.58 m) | Calculated |
| FOV | 18.4 deg | Calculated |
| FocalLength35efl | 110.9 mm | Calculated |
| HyperfocalDistance | 46.14 m | Calculated |
| LightValue | 8.3 | Calculated |

---

## Implementation Plan

### Phase 1: Core MakerNote Structure
1. Parse Canon MakerNote IFD structure (already done)
2. Add tag constants for all Canon MakerNote tags
3. Implement `get_canon_tag_name()` for all tags

### Phase 2: CameraSettings (0x0001) - HIGH PRIORITY
1. Parse CameraSettings sub-IFD (array of shorts)
2. Implement decoders:
   - `decode_macro_mode_exiftool()` / `decode_macro_mode_exiv2()`
   - `decode_quality_exiftool()` / `decode_quality_exiv2()`
   - `decode_flash_mode_exiftool()` / `decode_flash_mode_exiv2()`
   - `decode_continuous_drive_exiftool()` / `decode_continuous_drive_exiv2()`
   - `decode_focus_mode_exiftool()` / `decode_focus_mode_exiv2()`
   - `decode_record_mode_exiftool()` / `decode_record_mode_exiv2()`
   - `decode_image_size_exiftool()` / `decode_image_size_exiv2()`
   - `decode_easy_mode_exiftool()` / `decode_easy_mode_exiv2()`
   - `decode_digital_zoom_exiftool()` / `decode_digital_zoom_exiv2()`
   - `decode_contrast_exiftool()` / `decode_contrast_exiv2()`
   - `decode_saturation_exiftool()` / `decode_saturation_exiv2()`
   - `decode_metering_mode_exiftool()` / `decode_metering_mode_exiv2()`
   - `decode_focus_range_exiftool()` / `decode_focus_range_exiv2()`
   - `decode_exposure_mode_exiftool()` / `decode_exposure_mode_exiv2()`

### Phase 3: ShotInfo (0x0004) - HIGH PRIORITY
1. Parse ShotInfo sub-IFD (array of shorts with camera-specific offsets)
2. Implement camera temperature conversion
3. Implement ISO calculations
4. Implement EV calculations

### Phase 4: LensType/LensID - HIGH PRIORITY
1. Implement Canon lens database from `%Image::ExifTool::Canon::LensTypes`
2. Create `get_canon_lens_name(lens_id)` function

### Phase 5: Other MakerNote Tags - MEDIUM PRIORITY
1. ImageType (0x0006) - simple string
2. FirmwareVersion (0x0007) - simple string
3. OwnerName (0x0009) - simple string
4. SerialNumber (0x000c) - integer to string
5. ModelID (0x0010) - model lookup table
6. AFInfo/AFInfo2 (0x0012, 0x0026) - complex structure
7. FileInfo (0x0093) - directory/file number parsing
8. LensModel (0x0095) - string parsing

### Phase 6: ColorData/ProcessingInfo - LOW PRIORITY
1. ColorData (0x4001) - version-dependent parsing, very complex
2. ProcessingInfo (0x00a0) - picture style, tone curve
3. SensorInfo (0x00e0) - sensor dimensions
4. VignettingCorr (0x40d0) - correction parameters

### Phase 7: CustomFunctions - LOW PRIORITY
1. Parse camera-specific custom function arrays
2. Implement setting decoders

### Phase 8: Calculated Values - LOW PRIORITY
1. Implement composite tag calculations (DOF, FOV, 35mm equivalent, etc.)

---

## Reference Files
- ExifTool: `exiftool/lib/Image/ExifTool/Canon.pm`
- ExifTool: `exiftool/lib/Image/ExifTool/CanonCustom.pm`
- exiv2: `exiv2/src/canonmn_int.cpp`

## Test Command
```bash
./target/debug/fpexif exiftool test-data/RAW_CANON_50D.CR2 | sort > /tmp/fpexif.txt
exiftool -s test-data/RAW_CANON_50D.CR2 | sort > /tmp/exiftool.txt
diff /tmp/fpexif.txt /tmp/exiftool.txt
```
