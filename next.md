# fpexif - Next Steps for Tag Support

## Current State (Dec 2024)

Overall exiftool match rate varies significantly by format. Formats with dedicated makernotes modules perform better, but even those have significant gaps.

### Existing Makernotes Modules
- `src/makernotes/canon.rs` (86KB)
- `src/makernotes/fuji.rs` (26KB)
- `src/makernotes/nikon.rs` (38KB)
- `src/makernotes/olympus.rs` (44KB)
- `src/makernotes/panasonic.rs` (39KB)
- `src/makernotes/sony.rs` (49KB)

---

## Priority 1: Missing Makernotes Modules

### 1.1 Pentax Makernotes
- **Impact**: 17 PEF files, currently 16.6% match rate
- **Reference**: `exiftool/lib/Image/ExifTool/Pentax.pm`, `exiv2/src/pentaxmn_int.cpp`
- **Key tags to implement**:
  - PentaxModelID
  - LensType / LensInfo
  - FocusMode, AFPointSelected
  - WhiteBalance, WhiteBalanceMode
  - ImageTone, Saturation, Sharpness, Contrast
  - DriveMode, MeteringMode
  - Quality, PictureMode
  - ShakeReduction
  - Temperature
- **Estimated effort**: 8-12 hours

### 1.2 Samsung Makernotes
- **Impact**: 7 SRW files, currently 28.5% match rate
- **Reference**: `exiftool/lib/Image/ExifTool/Samsung.pm`, `exiv2/src/samsungmn_int.cpp`
- **Key tags to implement**:
  - ModelID
  - LensType
  - FocusMode, AFPointSelected
  - WhiteBalance
  - ColorSpace
  - SmartRange
  - ExposureProgram
  - FaceDetect
- **Estimated effort**: 4-6 hours

---

## Priority 2: Parsing Failures

### 2.1 MRW (Minolta RAW) - Many Files Failing
- **Issue**: "No EXIF data found in MRW file" on 9 files
- **Root cause**: PRD/TTW block parsing not finding TIFF data in all variants
- **Files failing**:
  - RAW_MINOLTA_5D.MRW
  - RAW_MINOLTA_7D_SRGB.MRW
  - RAW_MINOLTA_A1.MRW
  - RAW_MINOLTA_A2.MRW
  - RAW_MINOLTA_DIMAGE_7HI.MRW
  - RAW_MINOLTA_DIMAGE_7I.MRW
  - RAW_MINOLTA_DIMAGE7.MRW
  - RAW_MINOLTA_DIMAGE_A200.MRW
- **Investigation needed**: Compare file structures, check if EXIF is in different blocks
- **Reference**: ExifTool MinoltaRaw.pm
- **Estimated effort**: 4-6 hours

### 2.2 X3F (Sigma/Foveon) - Directory Issues
- **Issue**: Directory structure parsing incomplete
- **Current state**: Signature detected, but directory/property parsing fails
- **Reference**: ExifTool SigmaRaw.pm
- **Estimated effort**: 4-6 hours

### 2.3 MDC (Minolta RD175) - Unsupported Format
- **Issue**: "Unsupported image format"
- **Notes**: Very rare legacy format, low priority
- **Estimated effort**: 2-4 hours

---

## Priority 3: Expand Existing Makernotes

### 3.1 Canon (CR2) - 23.3% match, 10,198 missing tags
High-value tags to add:
- CanonCameraSettings (many sub-fields)
- CanonShotInfo (many sub-fields)
- CanonFileInfo
- CanonProcessingInfo
- VignettingCorrection
- LensModel, LensSerialNumber
- DustRemovalData
- ColorData (complex structure)
- AFInfo, AFInfo2

### 3.2 Nikon (NEF) - 24.1% match, 6,274 missing tags
High-value tags to add:
- NikonShotInfo (many sub-fields)
- NikonAFInfo, NikonAFInfo2
- NikonLensData
- NikonColorBalance
- NikonPictureControl
- NikonWorldTime
- NikonVRInfo
- NikonMeteringInfo
- NikonFlashInfo

### 3.3 Olympus (ORF) - 20.5% match, 5,081 missing tags
High-value tags to add:
- OlympusCameraSettings
- OlympusEquipment
- OlympusFocusInfo
- OlympusImageProcessing
- OlympusRawDevelopment

### 3.4 Fuji (RAF) - 46.1% match, 1,836 missing tags
Additional tags to add:
- FilmMode details
- DynamicRange settings
- FocusSettings sub-fields
- FlashMode details

### 3.5 Sony (ARW) - 28.6% match, 4,458 missing tags
High-value tags to add:
- SonyTag9050 (many camera settings)
- SonyAFInfo
- SonyExposureData
- SonyFocusPosition

### 3.6 Panasonic (RW2) - 31.3% match, 2,327 missing tags
Additional tags to add:
- Advanced scene modes
- Face detection data
- Lens-specific data

---

## Priority 4: Standard EXIF Tag Gaps

Some standard EXIF tags may not be fully decoded:
- SubSecTime variants
- OffsetTime variants (timezone)
- Composite tags (calculated values)
- XMP embedded data
- IPTC embedded data

---

## Implementation Order

### Phase 1: Quick Wins
1. Pentax makernotes module (biggest gap with no module)
2. Samsung makernotes module

### Phase 2: Fix Parsing Issues
3. MRW file variant support
4. X3F directory parsing

### Phase 3: Expand Coverage
5. Canon makernotes expansion
6. Nikon makernotes expansion
7. Olympus makernotes expansion

### Phase 4: Polish
8. Sony, Fuji, Panasonic expansions
9. Standard EXIF tag gaps
10. MDC support (if requested)

---

## Testing Strategy

For each new module/expansion:
1. Run `cargo test` for unit tests
2. Run exiftool comparison tests
3. Check for regressions in match rates
4. Verify no new critical issues

---

## Reference Files

| Manufacturer | ExifTool | exiv2 |
|-------------|----------|-------|
| Pentax | `Pentax.pm` | `pentaxmn_int.cpp` |
| Samsung | `Samsung.pm` | `samsungmn_int.cpp` |
| Minolta | `Minolta.pm`, `MinoltaRaw.pm` | `minoltamn_int.cpp` |
| Sigma | `Sigma.pm`, `SigmaRaw.pm` | `sigmamn_int.cpp` |

All ExifTool paths: `exiftool/lib/Image/ExifTool/`
All exiv2 paths: `exiv2/src/`
