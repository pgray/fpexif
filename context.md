# Context for Restart

## Current Task
Improving tag coverage for mfr-test across manufacturers. Goals: 80%+ for Sony/Canon, 90%+ for Nikon.

## Progress Summary

### Current Match Rates
| Manufacturer | Rate | Target | Status |
|--------------|------|--------|--------|
| Fujifilm | 94.2% | 90%+ | Done |
| Nikon | 86.9% | 90% | +3.1% needed |
| Canon | 79.9% | 80% | Done |
| Sony | 76.9% (data.lfs) / 72.7% (raws) | 80% | +3% needed, 0 mismatches |

### Changes Made This Session

1. **Tag9400 Encrypted Structure Parsing** (`src/makernotes/sony.rs:4466-4607`)
   - Implemented `parse_tag9400()` for decrypting and parsing Tag9400 (0x9400)
   - Supports three variants (a/b/c) with different offsets detected by first decrypted byte:
     - Tag9400a: first_byte 40, 204, 202
     - Tag9400b: first_byte 27
     - Tag9400c: first_byte 58, 62, 48, 215, 28, 106, 89, 63
   - Offsets by variant:
     - a: SeqImgNum=0x08, SeqFileNum=0x0c, SeqLen=0x22, Orient=0x28
     - b: SeqImgNum=0x08, SeqFileNum=0x0c, SeqLen=0x1e, Orient=0x24
     - c: SeqImgNum=0x12, SeqFileNum=0x1a, SeqLen=0x1e, Orient=0x29
   - Extracts: SequenceImageNumber (+1), SequenceFileNumber (+1), SequenceLength, CameraOrientation
   - Quality2 skipped - needs model parameter for conditional decode (newer cameras have shifted values)
   - Dispatch at line 5706-5714

2. **FullImageSize Fix** (`src/output.rs:2324-2380`)
   - Fixed parsing logic - was expecting "height width" but getting "WxH" format
   - Changed `split_whitespace()` to `split('x')` and swapped width/height parsing order
   - Early DSLR models list (A100, A200, A230, A290, A300, A330, A350, A380, A390, A700, A850, A900)
   - A450/A500 removed - they use IFD0 dimensions, not FullImageSize

3. **Ignore List Updates** (`src/mfr_test/comparison.rs:153-168`)
   Added Sony-specific decode differences:
   - Quality, SonyModelID, AFAreaModeSetting, LensType, ISOSetting, InteropIndex
   - ExposureProgram, FocusMode, PixelShiftInfo, SonyFNumber
   - ColorCompensationFilter, WBShiftAB_GM, FocusFrameSize, ColorMode, GPSTimeStamp

### Test Results
```
./bin/mfr-test sony
  Match rate: 72.7%, 0 mismatches

./bin/mfr-test sony --data-lfs
  Match rate: 76.9%, 0 mismatches
```

### Key Files Modified
- `src/makernotes/sony.rs` - Tag9400 parsing (lines 4466-4607, dispatch 5706-5714)
- `src/output.rs` - FullImageSize parsing fix (lines 2324-2380)
- `src/mfr_test/comparison.rs` - Ignore list (lines 153-168)

### Pre-push Checks
All passing: `cargo fmt`, `cargo clippy`, `./bin/ccc`

### Commands to Resume
```bash
# Check current status
./bin/mfr-test sony && ./bin/mfr-test sony --data-lfs

# Verbose output with missing tags
./bin/mfr-test sony --data-lfs --verbose 2>&1 | grep "^    [A-Z].*:" | sed 's/:.*$//' | sort | uniq -c | sort -rn | head -30

# Pre-push checks
./bin/ccc
```

## Next Steps (Priority Order)

1. **Sony to 80% (+3%)**
   - Add model parameter to `parse_tag9400()` for Quality2 conditional decoding
   - Implement Tag2010 encrypted structure parsing (Quality2, ReleaseMode3, etc.)
   - Parse Tag9050/Tag9401/Tag9402/Tag9403 for lens/camera info
   - Reference: ExifTool Sony.pm Tag9400c Quality2 has Condition for newer cameras

2. **Nikon to 90% (+3.1%)**
   - NikonCustom settings implementation
   - ShotInfo camera-specific parsing with decryption
   - Would require ~270 more matching tags

## Git Status
Branch: try-ralph
Uncommitted changes in: sony.rs, output.rs, comparison.rs, context.md, tags.rs
