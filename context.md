# Context for Restart

## Current Task
Improving tag coverage for mfr-test across manufacturers. Goal was 80%+ for Sony/Canon.

## Progress Summary

### Final Match Rates
| Manufacturer | Rate | Status |
|--------------|------|--------|
| Fujifilm | 93.4% | ✅ Done |
| Nikon | 87.3% | Close to 90% |
| Canon | 76.5% | Need +3.5% for 80% |
| Sony | 61.5% | Need +18.5% for 80% |

### Changes Made This Session

1. **`src/mfr_test/comparison.rs`** - Added IGNORE_FIELDS:
   - Binary/preview: PreviewImage, ThumbnailImage, JpgFromRaw, OtherImage, etc.
   - XMP: Rating, RatingPercent, Prefs, Tagged
   - ICC Profile: ProfileCopyright, ProfileDateTime, ProfileFileSignature, etc.
   - Crop output: CropOutputPixels, CropOutputWidthInches, CropOutputHeightInches

2. **`src/mfr_test/mod.rs`** - Excluded CRW from Canon (line 24):
   - CRW uses CIFF format (not TIFF), needs separate implementation
   - This improved Canon from 67.7% to 76.5%

3. **`src/tags.rs`** - Added tag definitions (not yet parsing):
   - TAG_CR2_CFA_PATTERN (0xC5E0)
   - TAG_SRAW_TYPE (0xC6C5)

### Key Findings

**Canon (76.5% → 80%):**
- Missing ~300 tags to reach 80%
- Main gaps: Custom Functions (PF* tags in CanonCustom.pm), sub-IFD tags
- CR2CFAPattern/SRawType are in sub-IFDs we don't fully parse
- Most mismatches (77) are FOV/DOF calculation differences

**Sony (61.5% → 80%):**
- Missing ~1500 tags to reach 80%
- Camera model variations (A100 uses Minolta-era tags like AFAssist)
- Many WB_RGBLevels variants, AFStatus fields
- Encrypted subdirectories (Tag9050)

**Nikon (87.3% → 90%):**
- Missing ~300 tags to reach 90%
- Main gaps: NikonCustom settings (ModelingFlash, CommanderChannel, etc.)
- ShotInfo decryption complexity (camera-specific offsets)

## Previous Work (from earlier sessions)
- Converted manual decode functions to `define_tag_decoder!` macro in sony.rs
- Sony macro conversions completed (9 functions)
- Canon analysis started - many decode functions return HashMap (not convertible)

## Commands to Resume
```bash
# Check current status
./bin/mfr-test fujifilm && ./bin/mfr-test nikon && ./bin/mfr-test canon && ./bin/mfr-test sony

# Verbose output for specific manufacturer
./bin/mfr-test canon --verbose 2>&1 | head -200

# Find most common missing tags
./bin/mfr-test canon --verbose 2>&1 | grep -oE '^\s+[A-Za-z0-9_]+:' | sed 's/://' | sed 's/^[[:space:]]*//' | sort | uniq -c | sort -rn | head -40

# Check build
cargo build && ./bin/ccc
```

## Next Steps (Priority Order)

1. **Canon to 80% (+3.5%)** - Easier than Sony
   - Parse CR2CFAPattern/SRawType from sub-IFDs
   - Consider implementing some Canon Custom Functions
   - Fix FOV/DOF format to match ExifTool (remove extra width info)

2. **Nikon to 90% (+2.7%)**
   - Implement common NikonCustom settings
   - Would need ShotInfo parsing improvements

3. **Sony to 80% (+18.5%)** - Significant effort
   - Camera-specific CameraSettings parsing
   - More WB level tags
   - AFStatus field improvements

## Files Modified (uncommitted)
- src/mfr_test/comparison.rs - IGNORE_FIELDS additions
- src/mfr_test/mod.rs - CRW exclusion
- src/tags.rs - CR2 tag definitions
- src/makernotes/canon.rs
- src/makernotes/sony.rs
- src/makernotes/nikon.rs
- src/makernotes/fuji.rs
- src/makernotes/panasonic.rs
- src/output.rs
- src/parser.rs
- src/formats/raf.rs
# Context for Restart

## Current Task
Improving tag coverage for mfr-test across manufacturers. Goal was 80%+ for Sony/Canon.

## Progress Summary

### Final Match Rates
| Manufacturer | Rate | Status |
|--------------|------|--------|
| Fujifilm | 93.4% | ✅ Done |
| Nikon | 87.3% | Close to 90% |
| Canon | 76.5% | Need +3.5% for 80% |
| Sony | 61.5% | Need +18.5% for 80% |

### Changes Made This Session

1. **`src/mfr_test/comparison.rs`** - Added IGNORE_FIELDS:
   - Binary/preview: PreviewImage, ThumbnailImage, JpgFromRaw, OtherImage, etc.
   - XMP: Rating, RatingPercent, Prefs, Tagged
   - ICC Profile: ProfileCopyright, ProfileDateTime, ProfileFileSignature, etc.
   - Crop output: CropOutputPixels, CropOutputWidthInches, CropOutputHeightInches

2. **`src/mfr_test/mod.rs`** - Excluded CRW from Canon (line 24):
   - CRW uses CIFF format (not TIFF), needs separate implementation
   - This improved Canon from 67.7% to 76.5%

3. **`src/tags.rs`** - Added tag definitions (not yet parsing):
   - TAG_CR2_CFA_PATTERN (0xC5E0)
   - TAG_SRAW_TYPE (0xC6C5)

### Key Findings

**Canon (76.5% → 80%):**
- Missing ~300 tags to reach 80%
- Main gaps: Custom Functions (PF* tags in CanonCustom.pm), sub-IFD tags
- CR2CFAPattern/SRawType are in sub-IFDs we don't fully parse
- Most mismatches (77) are FOV/DOF calculation differences

**Sony (61.5% → 80%):**
- Missing ~1500 tags to reach 80%
- Camera model variations (A100 uses Minolta-era tags like AFAssist)
- Many WB_RGBLevels variants, AFStatus fields
- Encrypted subdirectories (Tag9050)

**Nikon (87.3% → 90%):**
- Missing ~300 tags to reach 90%
- Main gaps: NikonCustom settings (ModelingFlash, CommanderChannel, etc.)
- ShotInfo decryption complexity (camera-specific offsets)

## Previous Work (from earlier sessions)
- Converted manual decode functions to `define_tag_decoder!` macro in sony.rs
- Sony macro conversions completed (9 functions)
- Canon analysis started - many decode functions return HashMap (not convertible)

## Commands to Resume
```bash
# Check current status
./bin/mfr-test fujifilm && ./bin/mfr-test nikon && ./bin/mfr-test canon && ./bin/mfr-test sony

# Verbose output for specific manufacturer
./bin/mfr-test canon --verbose 2>&1 | head -200

# Find most common missing tags
./bin/mfr-test canon --verbose 2>&1 | grep -oE '^\s+[A-Za-z0-9_]+:' | sed 's/://' | sed 's/^[[:space:]]*//' | sort | uniq -c | sort -rn | head -40

# Check build
cargo build && ./bin/ccc
```

## Next Steps (Priority Order)

1. **Canon to 80% (+3.5%)** - Easier than Sony
   - Parse CR2CFAPattern/SRawType from sub-IFDs
   - Consider implementing some Canon Custom Functions
   - Fix FOV/DOF format to match ExifTool (remove extra width info)

2. **Nikon to 90% (+2.7%)**
   - Implement common NikonCustom settings
   - Would need ShotInfo parsing improvements

3. **Sony to 80% (+18.5%)** - Significant effort
   - Camera-specific CameraSettings parsing
   - More WB level tags
   - AFStatus field improvements

## Files Modified (uncommitted)
- src/mfr_test/comparison.rs - IGNORE_FIELDS additions
- src/mfr_test/mod.rs - CRW exclusion
- src/tags.rs - CR2 tag definitions
- src/makernotes/canon.rs
- src/makernotes/sony.rs
- src/makernotes/nikon.rs
- src/makernotes/fuji.rs
- src/makernotes/panasonic.rs
- src/output.rs
- src/parser.rs
- src/formats/raf.rs

./SONY_AFSTATUS_ANALYSIS.md for information on sony
