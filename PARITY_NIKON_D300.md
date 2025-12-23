# Tag Parity Plan: RAW_NIKON_D300.NEF

## Overview
- **Camera**: Nikon D300
- **File**: test-data/RAW_NIKON_D300.NEF
- **Reference**: test-data/RAW_NIKON_D300.json (exiftool output)

## Current Status
fpexif extracts basic EXIF/TIFF tags but lacks Nikon MakerNote decoding.

---

## Missing Tags by Category

### 1. Basic MakerNote Tags
**Priority: HIGH**

| Tag ID | Tag Name | ExifTool Value | Notes |
|--------|----------|----------------|-------|
| 0x0001 | MakerNoteVersion | 2.10 | |
| 0x0002 | ISO | (in ISOInfo) | |
| 0x0004 | Quality | RAW | |
| 0x0005 | WhiteBalance | Auto | |
| 0x0007 | FocusMode | AF-S | |
| 0x0008 | FlashSetting | Normal | |
| 0x0009 | FlashType | (empty) | |
| 0x000c | WhiteBalanceFineTune | 0 0 | Two values |
| 0x000e | WB_RBLevels | 1.359375 1.41796875 1 1 | |
| 0x0010 | ProgramShift | 0 | |
| 0x0011 | ExposureDifference | 0 | |
| 0x0013 | PreviewImageStart | 9800 | |
| 0x0014 | PreviewImageLength | 107382 | |
| 0x0017 | FlashExposureComp | 0 | |
| 0x0018 | ISOSetting | (empty) | |
| 0x0019 | ExternalFlashExposureComp | 0 | |
| 0x001a | FlashExposureBracketValue | 0.0 | |
| 0x001b | ExposureBracketValue | 0 | |
| 0x001d | SerialNumber | 4000025 | |
| 0x001e | ColorSpace | sRGB | |
| 0x001f | VRInfoVersion | 0100 | |
| 0x0022 | ImageAuthentication | Off | |

### 2. CropHiSpeed (Tag 0x001b)
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| CropHiSpeed | Off (4352x2868 cropped to 4352x2868 at pixel 0,0) |
| ExposureTuning | 0 |

### 3. VRInfo (Tag 0x001f)
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| VRInfoVersion | 0100 |
| VibrationReduction | Off |
| VRMode | Normal |

### 4. Active D-Lighting (Tag 0x0022)
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| ActiveD-Lighting | Normal |

### 5. PictureControlData (Tag 0x0023)
**Priority: HIGH**

| Tag | ExifTool Value |
|-----|----------------|
| PictureControlVersion | 0100 |
| PictureControlName | Standard |
| PictureControlBase | Standard |
| PictureControlAdjust | Full Control |
| PictureControlQuickAdjust | Normal |
| Brightness | Normal |
| HueAdjustment | None |
| FilterEffect | n/a |
| ToningEffect | n/a |
| ToningSaturation | n/a |

### 6. WorldTime (Tag 0x0024)
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| TimeZone | +01:00 |
| DaylightSavings | No |
| DateDisplayFormat | Y/M/D |

### 7. ISOInfo (Tag 0x0025)
**Priority: HIGH**

| Tag | ExifTool Value |
|-----|----------------|
| ISO | 100 |
| ISOExpansion | Lo 1.0 |
| ISOExpansion2 | Lo 1.0 |

### 8. LensType (Tag 0x0083)
**Priority: HIGH**

| Tag | ExifTool Value | Notes |
|-----|----------------|-------|
| LensType | G | G-type lens flags |

### 9. Lens (Tag 0x0084)
**Priority: HIGH**

| Tag | ExifTool Value |
|-----|----------------|
| Lens | 17-55mm f/2.8 |

### 10. FlashMode (Tag 0x0087)
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| FlashMode | Did Not Fire |

### 11. ShootingMode (Tag 0x0089)
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| ShootingMode | Continuous |

### 12. ShotInfo (Tag 0x0091)
**Priority: HIGH** - Complex versioned structure

| Tag | ExifTool Value |
|-----|----------------|
| ShotInfoVersion | 0210 |
| ISO2 | 100 |
| AFFineTuneAdj | 0 |

### 13. CustomSettingsD300 (Tag 0x0091 subset)
**Priority: MEDIUM** - Many custom function settings

| Tag | ExifTool Value |
|-----|----------------|
| CustomSettingsBank | A |
| CustomSettingsAllDefault | No |
| AF-CPrioritySelection | Release |
| AF-SPrioritySelection | Focus |
| AFPointSelection | 51 Points |
| DynamicAFArea | 51 Points (3D-tracking) |
| FocusTrackingLockOn | Off |
| AFActivation | Shutter/AF-On |
| FocusPointWrap | Wrap |
| AFPointIllumination | On |
| AFAssist | On |
| AF-OnForMB-D10 | AF-On |
| ISOStepSize | 1/3 EV |
| ExposureControlStepSize | 1/3 EV |
| ExposureCompStepSize | 1/3 EV |
| EasyExposureCompensation | Off |
| CenterWeightedAreaSize | 8 mm |
| FineTuneOptCenterWeighted | 0 |
| FineTuneOptMatrixMetering | 0 |
| FineTuneOptSpotMetering | 0 |
| MultiSelectorShootMode | Select Center Focus Point |
| MultiSelectorPlaybackMode | Zoom On/Off |
| InitialZoomSetting | Medium Magnification |
| MultiSelector | Do Nothing |
| ExposureDelayMode | Off |
| CLModeShootingSpeed | 3 fps |
| MaxContinuousRelease | 100 |
| ReverseIndicators | + 0 - |
| FileNumberSequence | On |
| BatteryOrder | MB-D10 First |
| MB-D10Batteries | LR6 (AA alkaline) |
| Beep | Off |
| ShootingInfoDisplay | Auto |
| GridDisplay | On |
| ViewfinderWarning | On |
| FuncButton | Bracketing Burst |
| FuncButtonPlusDials | Auto Bracketing |
| PreviewButton | Preview |
| PreviewButtonPlusDials | None |
| AELockButton | AE/AF Lock |
| AELockButtonPlusDials | None |
| CommandDialsReverseRotation | No |
| CommandDialsChangeMainSub | Off |
| CommandDialsApertureSetting | Sub-command Dial |
| CommandDialsMenuAndPlayback | Off |
| LCDIllumination | Off |
| PhotoInfoPlayback | Info Up-down, Playback Left-right |
| ShutterReleaseButtonAE-L | Off |
| ReleaseButtonToUseDial | No |
| SelfTimerTime | 10 s |
| MonitorOffTime | 20 s |
| FlashSyncSpeed | 1/250 s |
| FlashShutterSpeed | 1/60 s |
| AutoBracketSet | AE & Flash |
| AutoBracketModeM | Flash/Speed |
| AutoBracketOrder | 0,-,+ |
| ModelingFlash | Off |
| NoMemoryCard | Enable Release |
| MeteringTime | 6 s |
| InternalFlash | TTL |

### 14. NEFCompression (Tag 0x0093)
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| NEFCompression | Lossless |

### 15. NoiseReduction (Tag 0x0095)
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| NoiseReduction | Off |

### 16. ColorBalance (Tag 0x0097)
**Priority: HIGH**

| Tag | ExifTool Value |
|-----|----------------|
| WB_GRBGLevels | 256 348 363 256 |

### 17. LensData (Tag 0x0098)
**Priority: HIGH**

| Tag | ExifTool Value |
|-----|----------------|
| LensDataVersion | 0203 |
| ExitPupilPosition | 157.5 mm |
| AFAperture | 2.8 |
| FocusPosition | 0xd1 |
| FocusDistance | 3.76 m |
| LensIDNumber | 125 |
| LensFStops | 6.00 |
| MinFocalLength | 17.3 mm |
| MaxFocalLength | 55.0 mm |
| MaxApertureAtMinFocal | 2.8 |
| MaxApertureAtMaxFocal | 2.8 |
| MCUVersion | 130 |
| EffectiveMaxAperture | 2.8 |

### 18. RawImageCenter (Tag 0x009d)
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| RawImageCenter | 2176 1434 |

### 19. RetouchHistory (Tag 0x009e)
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| RetouchHistory | None |

### 20. ShutterCount (Tag 0x00a7)
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| ShutterCount | 817 |

### 21. FlashInfo (Tag 0x00a8)
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| FlashInfoVersion | 0102 |
| FlashSource | None |
| ExternalFlashFirmware | n/a |
| ExternalFlashFlags | (none) |
| FlashCommanderMode | Off |
| FlashControlMode | Off |
| FlashCompensation | 0 |
| FlashGNDistance | 0 |
| FlashGroupAControlMode | Off |
| FlashGroupBControlMode | Off |
| FlashGroupCControlMode | Off |
| FlashGroupACompensation | 0 |
| FlashGroupBCompensation | 0 |
| FlashGroupCCompensation | 0 |

### 22. MultiExposure (Tag 0x00b0)
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| MultiExposureVersion | 0100 |
| MultiExposureMode | Off |
| MultiExposureShots | 0 |
| MultiExposureAutoGain | Off |

### 23. HighISONoiseReduction (Tag 0x00b1)
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| HighISONoiseReduction | Off |

### 24. PowerUpTime (Tag 0x00b6)
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| PowerUpTime | 2007:09:22 11:43:32 |

### 25. AFInfo2 (Tag 0x00b7)
**Priority: HIGH**

| Tag | ExifTool Value |
|-----|----------------|
| AFInfo2Version | 0100 |
| AFDetectionMethod | Phase Detect |
| AFAreaMode | Single Area |
| FocusPointSchema | 51-point |
| PrimaryAFPoint | C6 (Center) |
| AFPointsUsed | C6 |

### 26. FileInfo (Tag 0x00b8)
**Priority: MEDIUM**

| Tag | ExifTool Value |
|-----|----------------|
| FileInfoVersion | 0100 |
| MemoryCardNumber | 0 |
| DirectoryNumber | 101 |
| FileNumber | 0087 |

### 27. AFFineTune (Tag 0x00b9)
**Priority: LOW**

| Tag | ExifTool Value |
|-----|----------------|
| AFFineTune | Off |
| AFFineTuneIndex | n/a |
| AFFineTuneAdjTele | 0 |

### 28. Derived/Calculated Values
**Priority: LOW**

| Tag | ExifTool Value | Notes |
|-----|----------------|-------|
| BlueBalance | 1.417969 | From WB levels |
| RedBalance | 1.359375 | From WB levels |
| AutoFocus | On | |
| ContrastDetectAF | Off | |
| LensID | AF-S DX Zoom-Nikkor 17-55mm f/2.8G IF-ED | Lens database |
| LensSpec | 17-55mm f/2.8 G | |
| PhaseDetectAF | On (51-point) | |
| ScaleFactor35efl | 1.5 | DX crop factor |
| CircleOfConfusion | 0.020 mm | |
| DOF | 8.38 m (2.28 - 10.66 m) | |
| FOV | 67.1 deg (4.99 m) | |
| FocalLength35efl | 27.0 mm | |
| HyperfocalDistance | 5.78 m | |
| LightValue | 10.3 | |

---

## Implementation Plan

### Phase 1: Core MakerNote Tags
**Priority: HIGH**

1. Parse Nikon MakerNote IFD structure
2. Implement basic tags:
   - MakerNoteVersion (0x0001)
   - Quality (0x0004) with decoder
   - WhiteBalance (0x0005) with decoder
   - FocusMode (0x0007) with decoder
   - FlashSetting (0x0008)
   - FlashType (0x0009)
   - WB_RBLevels (0x000e)
   - SerialNumber (0x001d)
   - ColorSpace (0x001e)

### Phase 2: ISOInfo and Lens Tags
**Priority: HIGH**

1. ISOInfo (0x0025) - Parse ISO data structure
2. LensType (0x0083) - Parse lens flag bits
3. Lens (0x0084) - Parse focal length/aperture string
4. LensData (0x0098) - Complex versioned structure

### Phase 3: Lens Database
**Priority: HIGH**

1. Implement Nikon lens ID database from `%Image::ExifTool::Nikon::LensTypes`
2. Create `get_nikon_lens_name(lens_id, lens_data)` function

### Phase 4: PictureControl
**Priority: HIGH**

1. Parse PictureControlData (0x0023)
2. Implement PictureControlName decoder
3. Implement Brightness/Contrast/etc decoders

### Phase 5: AF System
**Priority: MEDIUM**

1. AFInfo2 (0x00b7) - Parse AF point data
2. Implement D300 51-point AF point naming
3. Parse AFPointsUsed bitmask

### Phase 6: Custom Settings
**Priority: MEDIUM**

1. Parse CustomSettingsD300 from ShotInfo (0x0091)
2. Implement all custom function decoders
3. Handle camera-specific custom setting layouts

### Phase 7: ShotInfo
**Priority: MEDIUM**

1. Parse versioned ShotInfo structure (0x0091)
2. Extract ISO2, AFFineTuneAdj, etc.

### Phase 8: VRInfo
**Priority: MEDIUM**

1. Parse VRInfo (0x001f)
2. Implement VibrationReduction decoder
3. Implement VRMode decoder

### Phase 9: Flash and Other Tags
**Priority: MEDIUM**

1. FlashInfo (0x00a8) - Parse flash data structure
2. FlashMode (0x0087) - Simple decoder
3. ShootingMode (0x0089) - Simple decoder
4. NEFCompression (0x0093)
5. NoiseReduction (0x0095)
6. ColorBalance (0x0097)
7. ShutterCount (0x00a7)
8. FileInfo (0x00b8)

### Phase 10: Low Priority Tags
**Priority: LOW**

1. WorldTime (0x0024)
2. RawImageCenter (0x009d)
3. RetouchHistory (0x009e)
4. MultiExposure (0x00b0)
5. HighISONoiseReduction (0x00b1)
6. PowerUpTime (0x00b6)
7. AFFineTune (0x00b9)
8. ActiveD-Lighting (0x0022)
9. CropHiSpeed (0x001b)

### Phase 11: Calculated Values
**Priority: LOW**

1. Implement White Balance calculations
2. Implement Lens35efl, DOF, FOV calculations

---

## Nikon MakerNote Specifics

### Versioned Structures
Many Nikon tags have internal version numbers that change the structure:
- LensData: 0200, 0201, 0203, 0204, 0800, etc.
- ShotInfo: 0200, 0210, 0213, 0300, etc.
- FlashInfo: 0100, 0102, 0103, etc.
- PictureControl: 0100, 0200, etc.

### Encrypted Data
Some Nikon data is encrypted/scrambled:
- Serial numbers may be encoded
- Some ShotInfo fields are camera-key encrypted

### D300 Specific
- 51-point AF system with named points (A1-D9, C6=center)
- DX format (1.5x crop)
- ShotInfo version 0210

---

## Reference Files
- ExifTool: `exiftool/lib/Image/ExifTool/Nikon.pm`
- ExifTool: `exiftool/lib/Image/ExifTool/NikonCustom.pm`
- exiv2: `exiv2/src/nikonmn_int.cpp`

## Test Command
```bash
./target/debug/fpexif exiftool test-data/RAW_NIKON_D300.NEF | sort > /tmp/fpexif.txt
exiftool -s test-data/RAW_NIKON_D300.NEF | sort > /tmp/exiftool.txt
diff /tmp/fpexif.txt /tmp/exiftool.txt
```
