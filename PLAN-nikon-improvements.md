# Nikon EXIF Improvements Plan

## Current State (2026-01-20)
- Match rate (raws): 87.6%
- Match rate (data.lfs): 66.8%
- Files tested: 4 NRW, 47 NEF (raws), 12 NRW, 276 NEF (data.lfs)
- Mismatches: 2 (raws), 28 (data.lfs)

## Recent Improvements

### 2026-01-20: FlashInfo Tags
Added parsing for three missing FlashInfo subdirectory tags:
- **ExternalFlashZoomOverride** - Byte 8, bit 7 (mask 0x80)
- **ExternalFlashStatus** - Byte 8, bit 0 (mask 0x01)
- **ExternalFlashReadyState** - Byte 9, bits 0-2 (mask 0x07)

Results:
- data.lfs: 65.9% → 66.3% (+0.4%)
- Matching tags: +228

### 2026-01-20: ISO Hi Prefix, SensingMethod, FlashExposureComp
- Fixed ISO "Hi" prefix not appearing in JSON (ISOInfo was overwriting)
- Fixed SensingMethod to prefer EXIF IFD (0xA217) over Main IFD (0x9217)
- Fixed FlashExposureComp to use improper fractions ("-5/3" instead of "-1 2/3")

## Remaining Issues

### Top Missing Tags (data.lfs)
| Tag | Count | Notes |
|-----|-------|-------|
| ImageSizeRAW | 26 | Composite/derived tag |
| CreatorTool | 24 | XMP metadata |
| FirmwareVersion | 20 | Encrypted (see PLAN-nikon-lensdata-decryption.md) |
| ShutterReleaseButtonAE-L | 13 | Menu settings |
| ModelingFlash | 11 | FlashInfo extended |
| FocusPointWrap | 11 | Menu settings |
| FileNumberSequence | 11 | Menu settings |
| MaxContinuousRelease | 10 | Menu settings |
| InitialZoomSetting | 10 | Menu settings |
| AF-SPrioritySelection | 10 | Menu settings |

### Remaining Mismatches
| Tag | Count | Issue |
|-----|-------|-------|
| FirmwareVersion | 14 | Encrypted - requires serial+shutter XOR decryption |
| RedBalance/BlueBalance | 4 | Minor rounding differences |
| ShutterSpeed/ExposureTime | 2 | Minor rounding (1/177 vs 1/176) |
| SerialNumber | 1 | Whitespace/case difference |
| ImageWidth/RowsPerStrip | 1 | Different IFD priorities |
| Megapixels | 2 | Calculation differences |

## Next Steps

### Phase 1: Menu Settings Tags (High Impact)
Many missing tags come from NikonSettings/MenuSettings subdirectory. Implementing these would add ~50+ tags.

1. **ShutterReleaseButtonAE-L** (13 files)
2. **FocusPointWrap** (11 files)
3. **FileNumberSequence** (11 files)
4. **MaxContinuousRelease** (10 files)
5. **InitialZoomSetting** (10 files)

### Phase 2: Extended FlashInfo Tags
Additional tags from FlashInfo not yet implemented:
- ModelingFlash
- FlashShutterSpeed
- CommanderGroupA/B settings

### Phase 3: LensData Decryption
See `PLAN-nikon-lensdata-decryption.md` for detailed plan. Would fix:
- FirmwareVersion (14 files)
- LensID lookups for modern cameras

## Reference Files
- `src/makernotes/nikon.rs` - Main Nikon MakerNote parser
- `exiftool/lib/Image/ExifTool/Nikon.pm` - ExifTool Nikon module
- `exiftool/lib/Image/ExifTool/NikonCustom.pm` - Custom settings
- `exiv2/src/nikonmn_int.cpp` - exiv2 Nikon implementation

## Testing
```bash
./bin/mfr-test nikon --save-baseline
# Make changes
./bin/mfr-test nikon --check
./bin/mfr-test nikon --data-lfs  # Test larger dataset
```
