# Tag Parity Plan: RAW_CANON_S90.CR2

## Overview
- **Camera**: Canon PowerShot S90 (Compact camera)
- **File**: test-data/RAW_CANON_S90.CR2
- **Reference**: test-data/RAW_CANON_S90.json (exiftool output)

## Current Status
fpexif extracts basic EXIF/TIFF tags but lacks Canon MakerNote decoding for PowerShot models.

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
| ContinuousDrive | Continuous | |
| FocusMode | Single | Different from DSLR |
| RecordMode | CR2+JPEG | |
| CanonImageSize | n/a | |
| EasyMode | Manual | |
| DigitalZoom | None | |
| Contrast | Normal | |
| Saturation | Normal | |
| Sharpness | 0 | Different from 50D |
| CameraISO | 80 | PowerShot-specific |
| MeteringMode | Evaluative | |
| FocusRange | Auto | |
| AFPoint | Manual AF point selection | PowerShot-specific |
| CanonExposureMode | Aperture-priority AE | |
| LensType | n/a | Built-in lens |
| MaxFocalLength | 22.5 mm | |
| MinFocalLength | 6 mm | |
| FocalUnits | 1000/mm | Higher precision |
| MaxAperture | 2.8 | |
| MinAperture | 8 | |
| FlashBits | (none) | |
| FocusContinuous | Single | |
| AESetting | Normal AE | PowerShot-specific |
| ImageStabilization | Shoot Only | |
| ZoomSourceWidth | 3648 | |
| ZoomTargetWidth | 3648 | |
| SpotMeteringMode | Center | PowerShot-specific |
| ManualFlashOutput | n/a | |
| SRAWQuality | n/a | |

### 2. Canon FocalLength (Tag 0x0002)
**Priority: HIGH**

| Tag | ExifTool Value | Notes |
|-----|----------------|-------|
| FocalType | Zoom | |
| FocalPlaneXSize | 7.59 mm | PowerShot-specific |
| FocalPlaneYSize | 5.69 mm | PowerShot-specific |

### 3. Canon ShotInfo (Tag 0x0004)
**Priority: HIGH**

| Tag | ExifTool Value | Notes |
|-----|----------------|-------|
| AutoISO | 100 | |
| BaseISO | 79 | |
| MeasuredEV | 13.31 | |
| TargetAperture | 5.6 | |
| TargetExposureTime | 1/125 | |
| ExposureCompensation | +1/3 | |
| WhiteBalance | Auto | |
| SlowShutter | Off | |
| SequenceNumber | 1 | |
| OpticalZoomCode | 3 | PowerShot-specific |
| FlashGuideNumber | 0 | |
| FlashExposureComp | 0 | |
| AutoExposureBracketing | Off | |
| AEBBracketValue | 0 | |
| ControlMode | Camera Local Control | |
| FocusDistanceUpper | 6.12 m | |
| FocusDistanceLower | 0 m | |
| BulbDuration | 0 | |
| CameraType | Compact | Different from 50D |
| AutoRotate | None | PowerShot-specific |
| NDFilter | Off | PowerShot-specific |
| SelfTimer2 | 0 | PowerShot-specific |
| FlashOutput | 0 | |

### 4. Canon ImageType (Tag 0x0006) & FirmwareVersion (Tag 0x0007)
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| CanonImageType | IMG:High definition image |
| CanonFirmwareVersion | Firmware Version 1.01 |

### 5. Canon FileNumber (Tag 0x0008)
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| FileNumber | 106-0023 |

### 6. Canon OwnerName (Tag 0x0009)
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| OwnerName | (empty) |

### 7. Canon ModelID (Tag 0x0010)
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| CanonModelID | PowerShot S90 |

### 8. Canon AFInfo (Tag 0x0012)
**Priority: MEDIUM**

| Tag | ExifTool Value | Notes |
|-----|----------------|-------|
| AFAreaMode | Single-point AF | |
| NumAFPoints | 9 | |
| ValidAFPoints | 1 | Only center used |
| CanonImageWidth | 1600 | |
| CanonImageHeight | 1200 | |
| AFImageWidth | 100 | Different from DSLR |
| AFImageHeight | 100 | Different from DSLR |
| AFAreaWidths | 18 0 0 0 0 0 0 0 0 | Only first populated |
| AFAreaHeights | 18 0 0 0 0 0 0 0 0 | |
| AFAreaXPositions | 0 0 0 0 0 0 0 0 0 | |
| AFAreaYPositions | 0 0 0 0 0 0 0 0 0 | |
| AFPointsInFocus | 0 | |
| PrimaryAFPoint | 0 | |

### 9. Canon ThumbnailImageValidArea (Tag 0x001d)
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| ThumbnailImageValidArea | 0 0 0 0 |

### 10. Canon DateStampMode (Tag 0x001c)
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| DateStampMode | Off |

### 11. Canon MyColors (Tag 0x001d)
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| MyColorMode | Off |

### 12. Canon FirmwareRevision
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| FirmwareRevision | 1.01 rev 3.00 |

### 13. Canon Categories
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| Categories | (none) |

### 14. Canon IntelligentContrast
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| IntelligentContrast | Off |

### 15. Canon ImageUniqueID
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| ImageUniqueID | 74fcebdd64dead9f86bee58d14806187 |

### 16. Canon Rotation (Tag 0x0024)
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| Rotation | 0 |

### 17. Canon CameraTemperature
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| CameraTemperature | 34 C |

### 18. Canon SensorInfo (Tag 0x00e0)
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| SensorWidth | 3744 |
| SensorHeight | 2784 |
| SensorLeftBorder | 24 |
| SensorTopBorder | 28 |
| SensorRightBorder | 3671 |
| SensorBottomBorder | 2763 |
| BlackMaskLeftBorder | 3732 |
| BlackMaskTopBorder | 28 |
| BlackMaskRightBorder | 3735 |
| BlackMaskBottomBorder | 2763 |

### 19. Canon ColorData (Tag 0x4001)
**Priority: LOW**

| Tag | ExifTool Value | Notes |
|-----|----------------|-------|
| ColorDataVersion | -3 (M10/M3) | Negative version = older format |
| WB_RGGBLevelsAsShot | 1515 882 882 1840 | |
| ColorTempAsShot | 6120 | |
| WB_RGGBLevelsAuto | 1515 882 882 1840 | |
| (... many WB levels) | | |
| PerChannelBlackLevel | 128 128 128 128 | |
| SpecularWhiteLevel | 11572 | |

### 20. Canon VignettingCorr (Tag 0x40d0)
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| VignettingCorrVersion | 1 |

### 21. Derived/Calculated Values
**Priority: LOW**

| Tag | ExifTool Value | Notes |
|-----|----------------|-------|
| DriveMode | Continuous Shooting | |
| Lens | 6.0 - 22.5 mm | Built-in lens |
| ShootingMode | Aperture-priority AE | |
| WB_RGGBLevels | 1515 882 882 1840 | |
| BlueBalance | 2.086168 | |
| RedBalance | 1.717687 | |
| LensID | Unknown 6-22mm | No lens database entry |
| ScaleFactor35efl | 4.7 | High crop factor |
| Lens35efl | 28.0 - 105.0 mm | |
| CircleOfConfusion | 0.006 mm | |
| DOF | inf (1.22 m - inf) | |
| FOV | 48.5 deg | |
| FocalLength35efl | 39.9 mm | |
| HyperfocalDistance | 2.03 m | |
| LightValue | 12.3 | |

---

## Implementation Notes

### PowerShot vs DSLR Differences
1. **CameraType**: Returns "Compact" instead of "EOS High-end"
2. **FocalUnits**: 1000/mm instead of 1/mm (higher precision)
3. **AFInfo**: Simpler structure, fewer points
4. **No LensType**: Built-in lens, no separate lens database
5. **Additional tags**: DateStampMode, MyColorMode, Categories, IntelligentContrast
6. **Different ColorData version**: Negative version number indicates older format

### Shared with 50D
Most CameraSettings parsing will be shared. Only enum values and optional fields differ.

---

## Implementation Plan

### Phase 1: Leverage 50D Work
Most Canon MakerNote infrastructure from 50D work applies here.

### Phase 2: PowerShot-Specific Tags
1. Handle different FocalUnits precision (1000/mm)
2. Add PowerShot-specific CameraSettings fields:
   - AFPoint
   - SpotMeteringMode
   - AESetting
   - ImageStabilization (different from DSLR)
3. Add DateStampMode decoder
4. Add MyColorMode decoder
5. Add Categories decoder
6. Add IntelligentContrast decoder

### Phase 3: ColorData Version -3
1. Parse older ColorData format (negative version)
2. Map to correct WB level positions

### Phase 4: Compact Camera Detection
1. Detect CameraType = Compact from ShotInfo
2. Apply PowerShot-specific field positions where needed

---

## Reference Files
- ExifTool: `exiftool/lib/Image/ExifTool/Canon.pm` (look for PowerShot-specific conditionals)
- exiv2: `exiv2/src/canonmn_int.cpp`

## Test Command
```bash
./target/debug/fpexif exiftool test-data/RAW_CANON_S90.CR2 | sort > /tmp/fpexif.txt
exiftool -s test-data/RAW_CANON_S90.CR2 | sort > /tmp/exiftool.txt
diff /tmp/fpexif.txt /tmp/exiftool.txt
```
