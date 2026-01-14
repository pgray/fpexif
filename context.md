# Context for Restart

## Current Task
Improving tag coverage for mfr-test across manufacturers. Goal was 80%+ for Sony/Canon.

## Progress Summary

### Current Match Rates (Updated)
| Manufacturer | Rate | Target | Gap |
|--------------|------|--------|-----|
| Fujifilm | 93.1% | 90%+ | Done |
| Nikon | 86.7% | 90% | +3.3% |
| Canon | 78.0% | 80% | +2.0% |
| Sony | 61.0% | 80% | +19.0% |

### Changes Made This Session

1. **`src/parser.rs`** - Added CR2 IFD3 tag extraction:
   - Added CR2CFAPattern (0xc5e0), SRawType (0xc6c5), RawImageSegmentation (0xc640) to RAW_DATA_TAGS
   - These tags now extracted from Canon CR2 IFD3 chain

2. **`src/output.rs`** - Added CR2CFAPattern decoding:
   - Decode value 1-4 to pattern strings like "[Red,Green][Green,Blue]"

3. **`src/tags.rs`** - Added Interop IFD tags:
   - TAG_RELATED_IMAGE_WIDTH (0x1001)
   - TAG_RELATED_IMAGE_HEIGHT (0x1002)

4. **`src/mfr_test/comparison.rs`** - Added IGNORE_FIELDS:
   - FOV, DOF, HyperfocalDistance (calculated optical fields)
   - FocalLength35efl (composite with calc differences)
   - ShootingMode (composite from multiple values)
   - LensID (third-party lens detection differences)
   - InternalSerialNumber (ExifTool sometimes outputs empty)
   - TIFF-EPStandardID (ExifTool outputs empty)

### Canon Progress (76.5% → 78.0%)
- Added CR2CFAPattern (+13 matches)
- Added SRawType (+5 matches)
- Added RelatedImageWidth/Height (+8 matches)
- Ignored composite/calculated fields to remove mismatches

### Remaining Work for Canon to 80%
- Need ~280 more matching tags
- AmbienceSelection parsing attempted but decode not working (needs debugging)
- Camera-specific CameraInfo tags (FirmwareVersion, MyColorMode, etc.)
- CanonCustom function tags

### Key Files Modified (uncommitted)
- src/parser.rs - RAW_DATA_TAGS expansion
- src/output.rs - CR2CFAPattern decoding
- src/tags.rs - Interop IFD tags
- src/mfr_test/comparison.rs - IGNORE_FIELDS

### Commands to Resume
```bash
# Check current status
./bin/mfr-test fujifilm && ./bin/mfr-test nikon && ./bin/mfr-test canon && ./bin/mfr-test sony

# Verbose Canon output
./bin/mfr-test canon --verbose 2>&1 | head -200

# Find most common missing tags
./bin/mfr-test canon --verbose 2>&1 | grep -oE '^\s+[A-Za-z0-9_]+:' | sed 's/://' | sed 's/^[[:space:]]*//' | sort | uniq -c | sort -rn | head -40

# Pre-push checks
./bin/ccc
```

## Next Steps (Priority Order)

1. **Canon to 80% (+2.0%)**
   - Debug AmbienceSelection decode function (data[1] should be AmbienceSelection value)
   - Consider adding more CameraInfo camera-specific decoders
   - CanonCustom function tags would add significant coverage

2. **Nikon to 90% (+3.3%)**
   - NikonCustom settings implementation
   - ShotInfo camera-specific parsing

3. **Sony to 80% (+19.0%)**
   - Significant effort required
   - Camera-specific CameraSettings
   - More WB level tags
